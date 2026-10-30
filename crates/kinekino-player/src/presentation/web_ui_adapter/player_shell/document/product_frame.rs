//! One fixed root graph with an exclusively keyed workspace child; no runtime or SDK ownership.
use meari::{
    application::source_modules::ModuleProgram,
    domain::{
        Value,
        build::{BuildPath, ContentHash},
        source_modules::{ComponentReference, SourceModuleGraph},
    },
};
use meari_native::infrastructure::document::InteractiveDocument;
use std::{collections::BTreeMap, num::NonZeroU64};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceRoute {
    Viewing,
    Cards,
    Jobs,
    Correction,
    Highlights,
}
impl WorkspaceRoute {
    fn key(self, generation: NonZeroU64) -> String {
        let route = match self {
            Self::Viewing => "viewing",
            Self::Cards => "cards",
            Self::Jobs => "jobs",
            Self::Correction => "correction",
            Self::Highlights => "highlights",
        };
        format!("product-workspace-{route}-{generation}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StudioTask {
    Card,
    Highlight,
    Correction,
}
impl StudioTask {
    pub fn route_id(self) -> &'static str {
        match self {
            Self::Card => "studio.cards",
            Self::Highlight => "studio.shorts.recommendations",
            Self::Correction => "studio.subtitles.correction",
        }
    }
    pub fn workspace(self) -> WorkspaceRoute {
        match self {
            Self::Card => WorkspaceRoute::Cards,
            Self::Highlight => WorkspaceRoute::Highlights,
            Self::Correction => WorkspaceRoute::Correction,
        }
    }
}

/// Both child graphs/inventories must be the product's closed source preparation inputs.
/// The Cards entry is the existing composed StudioFrame, including its original chrome.
#[derive(Clone)]
pub struct ProductFrameBlueprint {
    pub graph: SourceModuleGraph,
    pub inventory: BTreeMap<BuildPath, String>,
    viewing: ComponentReference,
    cards: ComponentReference,
    jobs: ComponentReference,
    studio: ComponentReference,
}
impl ProductFrameBlueprint {
    pub fn prepare(
        viewing: SourceModuleGraph,
        viewing_inventory: BTreeMap<BuildPath, String>,
        cards: SourceModuleGraph,
        cards_inventory: BTreeMap<BuildPath, String>,
    ) -> Result<Self, String> {
        let (jobs, jobs_inventory) = crate::presentation::web_ui_adapter::studio::jobs::prepare()?;
        let mut children = vec![
            (viewing, viewing_inventory),
            (cards, cards_inventory),
            (jobs, jobs_inventory),
        ];
        for panel in [
            crate::presentation::web_ui_adapter::proposals::bootstrap::BootstrapPanel::Correction,
            crate::presentation::web_ui_adapter::proposals::bootstrap::BootstrapPanel::Highlight,
            crate::presentation::web_ui_adapter::proposals::bootstrap::BootstrapPanel::ManualRepair,
        ] {
            let (entry, mut sources) = crate::presentation::web_ui_adapter::proposals::product_window::product_modules_for_panel(panel);
            let provider = meari::parse_module_graph(&entry, &sources)
                .map_err(|error| format!("{error:?}"))?;
            let (graph, _) = crate::presentation::web_ui_adapter::proposals::frame_composition::FrameComposition::new(crate::domain::product::PRODUCT.display_name.into(), String::new())
                .prepare(provider, &mut sources).map_err(|error| format!("{error:?}"))?;
            children.push((graph, sources));
        }
        let entry: BuildPath = "product/frame.tsx"
            .to_owned()
            .try_into()
            .map_err(|error| format!("{error:?}"))?;
        let mut inventory = BTreeMap::from([(
            entry.clone(),
            include_str!("../../../web_ui/product/frame.tsx").into(),
        )]);
        let mut graph =
            meari::parse_module_graph(&entry, &inventory).map_err(|error| format!("{error:?}"))?;
        for (_, sources) in &children {
            for (path, text) in sources {
                if inventory.get(path).is_some_and(|previous| previous != text) {
                    return Err("Product frame source inventory collision".into());
                }
                inventory.insert(path.clone(), text.clone());
            }
        }
        let viewing_reference = children[0].0.entry.clone();
        let cards_reference = children[1].0.entry.clone();
        let jobs_reference = children[2].0.entry.clone();
        let studio_reference = children[3].0.entry.clone();
        if children[4].0.entry != studio_reference {
            return Err("Studio task frames differ".into());
        }
        for (child, _) in children {
            child.validate()?;
            for (path, module) in child.modules {
                let text = inventory
                    .get(&path)
                    .ok_or("Product child source is missing")?;
                if module.content != ContentHash::of(text.as_bytes())
                    || module.source_bytes != text.len() as u64
                {
                    return Err("Product child graph differs from its source inventory".into());
                }
                if graph
                    .modules
                    .get(&path)
                    .is_some_and(|previous| previous != &module)
                {
                    return Err("Product frame component definition collision".into());
                }
                graph.modules.insert(path, module);
            }
        }
        graph.validate()?;
        Ok(Self {
            graph,
            inventory,
            viewing: viewing_reference,
            cards: cards_reference,
            jobs: jobs_reference,
            studio: studio_reference,
        })
    }
    /// Called once by the original startup. Later route changes use workspace_props instead.
    pub fn mount_viewing(
        &self,
        props: BTreeMap<String, Value>,
        namespace: String,
        epoch: u64,
        generation: NonZeroU64,
    ) -> Result<ModuleProgram, String> {
        ModuleProgram::mount_with_node_slots(self.graph.clone(), namespace, epoch, |factory| {
            let mut props = props;
            super::super::start_screen::compose_slot(&mut props, |reference, props| {
                factory.component("product-viewing-start", reference, props)
            })?;
            if props.contains_key("jobsEntryDisabled") {
                props.insert("jobsEntryDisabled".into(), Value::Bool(false));
            }
            crate::presentation::web_ui_adapter::proposals::capture_choices::compose_slot(
                &mut props,
                |reference, props| factory.component("product-capture-choices", reference, props),
            )?;
            super::proposal_chooser::compose_slot(&mut props, |reference, props| {
                factory.component("product-proposal-chooser", reference, props)
            })?;
            let workspace = factory.component(
                &WorkspaceRoute::Viewing.key(generation),
                self.viewing.clone(),
                props,
            )?;
            Ok(Self::root_props(workspace))
        })
        .map_err(|error| format!("{error:?}"))
    }
    /// Prepare a fresh owned slot on the same live document. Caller commits these props and the
    /// matching revocable export in one original try_reproject; failures retain original owners.
    pub fn workspace_props(
        &self,
        document: &InteractiveDocument,
        route: WorkspaceRoute,
        generation: NonZeroU64,
        props: BTreeMap<String, Value>,
    ) -> Result<BTreeMap<String, Value>, String> {
        let mut props = props;
        if matches!(route, WorkspaceRoute::Viewing) {
            super::super::start_screen::compose_slot(&mut props, |reference, props| {
                document.node_slot("product-viewing-start", reference, props)
            })?;
            if props.contains_key("jobsEntryDisabled") {
                props.insert("jobsEntryDisabled".into(), Value::Bool(false));
            }
            crate::presentation::web_ui_adapter::proposals::capture_choices::compose_slot(
                &mut props,
                |reference, props| document.node_slot("product-capture-choices", reference, props),
            )?;
            super::proposal_chooser::compose_slot(&mut props, |reference, props| {
                document.node_slot("product-proposal-chooser", reference, props)
            })?;
        }
        let reference = match route {
            WorkspaceRoute::Viewing => &self.viewing,
            WorkspaceRoute::Cards => &self.cards,
            WorkspaceRoute::Jobs => &self.jobs,
            WorkspaceRoute::Correction | WorkspaceRoute::Highlights => &self.studio,
        };
        let workspace = document.node_slot(&route.key(generation), reference.clone(), props)?;
        Ok(Self::root_props(workspace))
    }
    fn root_props(workspace: Value) -> BTreeMap<String, Value> {
        BTreeMap::from([("workspace".into(), workspace)])
    }
}

#[cfg(test)]
#[path = "product_frame/tests.rs"]
mod tests;
