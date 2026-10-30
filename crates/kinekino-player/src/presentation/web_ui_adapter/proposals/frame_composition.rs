//! Original StudioFrame with one SDK editor and optional controlled InspectorDock.
mod raw_review;
use super::{
    inspector_intents::InspectorIntents,
    inspector_state::{InspectorLayout, InspectorViewState},
};
use crate::presentation::web_ui_adapter::player_shell::window_caption::{
    CaptionState, CaptionView,
};
use meari::domain::host_callbacks::{CallbackReference, CallbackSignature};
use meari::{
    application::source_modules::{ModuleDiagnostic, ModuleProgram},
    domain::{
        Diagnostic, Span, Value,
        build::BuildPath,
        node_slots::NodeSlot,
        source_modules::{ComponentReference, SourceModuleGraph},
    },
};
use std::collections::BTreeMap;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub struct FrameComposition {
    product_name: String,
    context_label: String,
    workspace_key: String,
    inspector: Option<(String, InspectorViewState)>,
}
#[derive(Clone)]
pub struct FrameProjection {
    caption_source: Option<Weak<RefCell<CaptionState>>>,
    caption_committed: CaptionView,
    raw_review: bool,
    composition: FrameComposition,
    editor: ComponentReference,
    inspector: Option<InspectorFrame>,
    navigation: ComponentReference,
}
#[derive(Clone)]
struct InspectorFrame {
    title: String,
    state: InspectorViewState,
    reference: ComponentReference,
    mailbox: Rc<RefCell<InspectorIntents>>,
}
fn path(value: &str) -> BuildPath {
    value
        .to_owned()
        .try_into()
        .expect("fixed product source path")
}
fn diagnostic(message: String) -> ModuleDiagnostic {
    ModuleDiagnostic {
        file: path("studio/frame.tsx"),
        diagnostic: Diagnostic::unsupported(Span::default(), message),
    }
}
impl FrameComposition {
    pub fn new(product_name: String, context_label: String) -> Self {
        Self {
            product_name,
            context_label,
            workspace_key: "proposal-editor".into(),
            inspector: None,
        }
    }
    /// Trusted presentation identity; factory validation remains authoritative for key limits.
    pub fn with_workspace_key(mut self, key: String) -> Self {
        self.workspace_key = key;
        self
    }
    pub fn with_inspector(mut self, title: String, state: InspectorViewState) -> Self {
        self.inspector = Some((title, state));
        self
    }
    pub fn prepare(
        self,
        provider: SourceModuleGraph,
        inventory: &mut BTreeMap<BuildPath, String>,
    ) -> Result<(SourceModuleGraph, FrameProjection), ModuleDiagnostic> {
        let mut sources = crate::presentation::web_ui_adapter::window_chrome_sources::inventory();
        sources.extend([
            (
                path("components/window-caption-controls.tsx"),
                include_str!("../../web_ui/components/window-caption-controls.tsx").into(),
            ),
            (
                path("studio/frame.tsx"),
                include_str!("../../web_ui/studio/frame.tsx").into(),
            ),
        ]);
        let mut graph = meari::parse_module_graph(&path("studio/frame.tsx"), &sources)?;
        let raw_review = inventory.contains_key(&raw_review::raw_reference().module)
            && provider.entry.module != path("studio/manual-repair.tsx");
        if raw_review {
            sources.extend(raw_review::sources());
            for reference in [
                raw_review::raw_reference(),
                raw_review::workspace_reference(),
            ] {
                let extra = meari::parse_module_graph(&reference.module, &sources)?;
                graph.modules.extend(extra.modules);
            }
        }
        let navigation_sources = BTreeMap::from([
            (
                path("studio/navigation.tsx"),
                include_str!("../../web_ui/studio/navigation.tsx").into(),
            ),
            (
                path("studio/navigation-item.tsx"),
                include_str!("../../web_ui/studio/navigation-item.tsx").into(),
            ),
        ]);
        let navigation =
            meari::parse_module_graph(&path("studio/navigation.tsx"), &navigation_sources)?;
        let navigation_reference = navigation.entry.clone();
        for (key, module) in navigation.modules {
            if graph.modules.insert(key, module).is_some() {
                return Err(diagnostic("Navigation graph collides with frame".into()));
            }
        }
        sources.extend(navigation_sources);
        let inspector = if let Some((title, state)) = &self.inspector {
            sources.insert(
                path("studio/inspector-dock.tsx"),
                include_str!("../../web_ui/studio/inspector-dock.tsx").into(),
            );
            let dock_sources = BTreeMap::from([(
                path("studio/inspector-dock.tsx"),
                sources[&path("studio/inspector-dock.tsx")].clone(),
            )]);
            let dock =
                meari::parse_module_graph(&path("studio/inspector-dock.tsx"), &dock_sources)?;
            let reference = dock.entry.clone();
            for (key, module) in dock.modules {
                if graph.modules.insert(key, module).is_some() {
                    return Err(diagnostic("Inspector graph collides with frame".into()));
                }
            }
            Some(InspectorFrame {
                title: title.clone(),
                state: *state,
                reference,
                mailbox: Rc::new(RefCell::new(InspectorIntents::default())),
            })
        } else {
            None
        };
        let projection = FrameProjection {
            caption_source: None,
            caption_committed: CaptionView::default(),
            raw_review,
            composition: self,
            editor: provider.entry.clone(),
            inspector,
            navigation: navigation_reference,
        };
        for (key, module) in provider.modules {
            if let Some(existing) = graph.modules.get(&key) {
                // The parser includes the full input inventory, including these fixed siblings.
                // Share only their exact declarations; all other editor/frame collisions fail.
                if raw_review
                    && (key == raw_review::raw_reference().module
                        || key == raw_review::workspace_reference().module)
                    && existing == &module
                {
                    continue;
                }
                return Err(diagnostic(
                    "Editor graph collides with the fixed frame inventory".into(),
                ));
            }
            graph.modules.insert(key, module);
        }
        graph.validate().map_err(diagnostic)?;
        for (key, source) in sources {
            if inventory
                .get(&key)
                .is_some_and(|existing| existing != &source)
            {
                return Err(diagnostic(
                    "Frame source differs from the fixed product inventory".into(),
                ));
            }
            inventory.insert(key, source);
        }
        Ok((graph, projection))
    }
}
impl FrameProjection {
    #[cfg(test)]
    fn assert_caption_factory_fields(&self, expected: &CaptionView) {
        let props = self.root_props(Value::NodeSlot(NodeSlot::Empty));
        for (key, value) in expected.props() {
            assert_eq!(props.get(&key), Some(&value));
        }
    }
    pub fn raw_review_enabled(&self) -> bool {
        self.raw_review
    }
    pub fn compose_editor_slot(
        &self,
        mut editor_props: BTreeMap<String, Value>,
        mut make: impl FnMut(&str, ComponentReference, BTreeMap<String, Value>) -> Result<Value, String>,
    ) -> Result<Value, String> {
        if !self.raw_review {
            return make(self.workspace_key(), self.editor.clone(), editor_props);
        }
        let raw_props = raw_review::split(&mut editor_props);
        let editor = make(self.workspace_key(), self.editor.clone(), editor_props)?;
        let raw = make(
            "proposal-raw-review",
            raw_review::raw_reference(),
            raw_props,
        )?;
        make(
            "proposal-review-workspace",
            raw_review::workspace_reference(),
            BTreeMap::from([("editor".into(), editor), ("rawReview".into(), raw)]),
        )
    }
    pub fn navigation_reference(&self) -> &ComponentReference {
        &self.navigation
    }
    pub fn navigation_props(
        &self,
        selected_id: &str,
        callback: CallbackReference,
    ) -> BTreeMap<String, Value> {
        BTreeMap::from([
            ("selectedId".into(), Value::String(selected_id.into())),
            ("onNavigate".into(), Value::CallbackReference(callback)),
        ])
    }
    pub fn inspector_reference(&self) -> Option<&ComponentReference> {
        self.inspector
            .as_ref()
            .map(|inspector| &inspector.reference)
    }
    pub fn inspector_state(&self) -> Option<InspectorViewState> {
        self.inspector.as_ref().map(|inspector| inspector.state)
    }
    pub fn inspector_pending(&self) -> bool {
        self.inspector
            .as_ref()
            .is_some_and(|inspector| inspector.mailbox.borrow().pending())
    }
    pub fn observe_inspector_layout(&self, layout: InspectorLayout) -> Result<(), String> {
        let inspector = self
            .inspector
            .as_ref()
            .ok_or("Inspector composition is absent")?;
        inspector
            .mailbox
            .try_borrow_mut()
            .map_err(|_| "Inspector queue is borrowed")?
            .enqueue_layout(layout)
            .map_err(str::to_owned)
    }
    pub fn stage_inspector(&self) -> Result<Self, String> {
        let mut staged = self.clone();
        if let Some(inspector) = &mut staged.inspector {
            inspector.state = inspector
                .mailbox
                .try_borrow_mut()
                .map_err(|_| "Inspector queue is borrowed")?
                .stage(inspector.state);
        }
        Ok(staged)
    }
    pub fn bind_inspector(
        &self,
        bindings: &mut super::ReviewBindings,
    ) -> Result<(), super::IntentError> {
        if let Some(inspector) = &self.inspector {
            let node = bindings.queue.borrow_mut().bind_node()?;
            bindings.bindings.insert(
                "onInspectorOpenChange".into(),
                super::QueueBinding::InspectorOpen {
                    node,
                    mailbox: inspector.mailbox.clone(),
                },
            );
        }
        Ok(())
    }
    pub fn inspector_props(&self, children: Value) -> Option<BTreeMap<String, Value>> {
        self.inspector.as_ref().map(|inspector| {
            let state = inspector.state.projection();
            BTreeMap::from([
                ("narrow".into(), Value::Bool(state.modal)),
                ("open".into(), Value::Bool(state.drawer_open)),
                ("title".into(), Value::String(inspector.title.clone())),
                ("children".into(), children),
                (
                    "onOpenChange".into(),
                    Value::CallbackReference(CallbackReference {
                        binding: "onInspectorOpenChange".into(),
                        signature: CallbackSignature::Boolean,
                    }),
                ),
            ])
        })
    }
    pub fn inspector_root_props(&self, inspector: Value) -> BTreeMap<String, Value> {
        let mut props = self.root_props(Value::NodeSlot(NodeSlot::Empty));
        props.insert("inspector".into(), inspector);
        props
    }
    pub fn editor(&self) -> &ComponentReference {
        &self.editor
    }
    pub fn workspace_key(&self) -> &str {
        &self.composition.workspace_key
    }
    pub fn replace_key(&mut self, key: String) {
        self.composition.workspace_key = key;
    }
    pub fn root_props(&self, workspace: Value) -> BTreeMap<String, Value> {
        let mut props = BTreeMap::from([
            (
                "productName".into(),
                Value::String(self.composition.product_name.clone()),
            ),
            (
                "contextLabel".into(),
                Value::String(self.composition.context_label.clone()),
            ),
            ("workspace".into(), workspace),
            ("navigation".into(), Value::NodeSlot(NodeSlot::Empty)),
            ("inspector".into(), Value::NodeSlot(NodeSlot::Empty)),
            ("queue".into(), Value::NodeSlot(NodeSlot::Empty)),
        ]);
        props.extend(self.caption_view().props());
        props
    }
    pub(crate) fn set_caption_source(&mut self, source: Weak<RefCell<CaptionState>>) {
        self.caption_source = Some(source);
    }
    fn caption_view(&self) -> CaptionView {
        self.caption_source
            .as_ref()
            .and_then(Weak::upgrade)
            .and_then(|source| source.try_borrow().ok().map(|source| source.view.clone()))
            .unwrap_or_default()
    }
    pub(crate) fn caption_pending(&self) -> bool {
        self.caption_view() != self.caption_committed
    }
    pub(crate) fn caption_committed(&mut self) {
        self.caption_committed = self.caption_view();
    }
    pub fn mount(
        &self,
        graph: SourceModuleGraph,
        editor_props: BTreeMap<String, Value>,
        document: String,
        epoch: u64,
    ) -> Result<ModuleProgram, ModuleDiagnostic> {
        ModuleProgram::mount_with_node_slots(graph, document, epoch, |factory| {
            let editor = self.compose_editor_slot(editor_props, |key, reference, props| {
                factory.component(key, reference, props)
            })?;
            if let Some(reference) = self.inspector_reference() {
                let props = self
                    .inspector_props(editor)
                    .expect("inspector reference has props");
                Ok(self.inspector_root_props(factory.component(
                    "proposal-inspector",
                    reference.clone(),
                    props,
                )?))
            } else {
                Ok(self.root_props(editor))
            }
        })
    }
}

#[cfg(test)]
mod caption_tests {
    use super::*;

    #[test]
    fn studio_caption_factory_uses_single_view_and_closed_source_fails_disabled() {
        let entry = path("caption-editor.tsx");
        let mut inventory = BTreeMap::from([(
            entry.clone(),
            "export function Editor(){return <article>Original editor</article>;}".into(),
        )]);
        let editor = meari::parse_module_graph(&entry, &inventory).unwrap();
        let (graph, mut frame) = FrameComposition::new("Product".into(), "Original route".into())
            .prepare(editor, &mut inventory)
            .unwrap();
        let state = Rc::new(RefCell::new(CaptionState::default()));
        state.borrow_mut().view = CaptionView {
            minimize_disabled: false,
            maximize_disabled: false,
            close_disabled: false,
            maximize_label: "복원".into(),
            maximize_icon: "aw-icon aw-icon-restore".into(),
            status: "원 창 최대화 관측".into(),
            error: false,
        };
        frame.set_caption_source(Rc::downgrade(&state));
        frame.assert_caption_factory_fields(&state.borrow().view);
        let original = frame.root_props(Value::NodeSlot(NodeSlot::Empty));
        crate::presentation::web_ui_adapter::player_shell::window_caption::contract_fixture::assert_required_caption_fields(
            &original, |invalid| ModuleProgram::mount_with_node_slots(graph.clone(), "caption-studio-negative".into(), 1,
                |_| Ok(invalid)).is_ok());
        let program = frame
            .mount(graph, BTreeMap::new(), "original-caption-factory".into(), 1)
            .unwrap();
        let projection = program.project().unwrap();
        let text = serde_json::to_string(&projection.component).unwrap();
        assert!(text.contains("Original editor"));
        assert!(text.contains("aw-icon-restore"));
        assert!(text.contains("원 창 최대화 관측"));
        let handlers = projection.callback_bindings();
        for (name, _) in crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS
        {
            assert!(
                handlers
                    .values()
                    .any(|handler| handler.reference.binding == name
                        && handler.reference.signature == CallbackSignature::Action)
            );
        }
        frame.caption_committed();
        assert!(!frame.caption_pending());
        state.borrow_mut().close();
        assert!(frame.caption_pending());
        frame.assert_caption_factory_fields(&CaptionView::default());
        drop(state);
        frame.assert_caption_factory_fields(&CaptionView::default());
    }
}
