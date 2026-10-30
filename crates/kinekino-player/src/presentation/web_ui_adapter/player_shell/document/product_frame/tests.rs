//! Real parser/ModuleProgram/Native document slot contract; child fixtures are not product routes.
use super::*;
use meari::{
    application::{host_callbacks::HostCallbacks, plugins::PluginScope},
    domain::host_callbacks::{CallbackArguments, CallbackReference, CallbackSignature},
};
use meari_native::{
    domain::{Viewport, widgets::Action},
    infrastructure::native_host::NativeDocumentHandle,
};
use std::{cell::RefCell, rc::Rc};

fn child(path: &str, source: &str) -> (SourceModuleGraph, BTreeMap<BuildPath, String>) {
    let path: BuildPath = path.to_owned().try_into().unwrap();
    let inventory = BTreeMap::from([(path.clone(), source.into())]);
    (
        meari::parse_module_graph(&path, &inventory).unwrap(),
        inventory,
    )
}
fn blueprint() -> ProductFrameBlueprint {
    let (viewing, view_sources) = child(
        "viewing.tsx",
        "type Props={label:string;onTap:()=>void};export function Viewing(props:Props){return <section><button onClick={() => props.onTap()}>{props.label}</button></section>;}",
    );
    let (cards, card_sources) = child(
        "cards.tsx",
        "type Props={label:string;onTap:()=>void};export function Cards(props:Props){return <article><button onClick={() => props.onTap()}>{props.label}</button></article>;}",
    );
    ProductFrameBlueprint::prepare(viewing, view_sources, cards, card_sources).unwrap()
}

#[test]
fn all_live_task_sources_share_the_original_closed_studio_frame() {
    use crate::presentation::web_ui_adapter::proposals::{
        bootstrap::BootstrapPanel, frame_composition::FrameComposition,
        product_window::product_modules_for_panel,
    };
    let blueprint = blueprint();
    for panel in [BootstrapPanel::Correction, BootstrapPanel::Highlight] {
        let (entry, mut inventory) = product_modules_for_panel(panel);
        let provider = meari::parse_module_graph(&entry, &inventory).unwrap();
        let (graph, _) = FrameComposition::new(
            crate::domain::product::PRODUCT.display_name.into(),
            String::new(),
        )
        .prepare(provider, &mut inventory)
        .unwrap();
        assert_eq!(blueprint.studio, graph.entry);
        for (path, module) in graph.modules {
            assert_eq!(blueprint.graph.modules.get(&path), Some(&module));
        }
        for (path, source) in inventory {
            assert_eq!(blueprint.inventory.get(&path), Some(&source));
        }
    }
}

#[test]
fn typed_task_keys_and_routes_do_not_alias_existing_viewing_or_jobs() {
    let generation = NonZeroU64::new(17).unwrap();
    let tasks = [
        StudioTask::Card,
        StudioTask::Highlight,
        StudioTask::Correction,
    ];
    let keys: std::collections::BTreeSet<_> = tasks
        .iter()
        .map(|task| task.workspace().key(generation))
        .collect();
    assert_eq!(keys.len(), 3);
    for task in tasks {
        assert_ne!(
            task.workspace().key(generation),
            WorkspaceRoute::Viewing.key(generation)
        );
        assert_ne!(
            task.workspace().key(generation),
            WorkspaceRoute::Jobs.key(generation)
        );
    }
    assert_eq!(StudioTask::Card.route_id(), "studio.cards");
    assert_eq!(
        StudioTask::Highlight.route_id(),
        "studio.shorts.recommendations"
    );
    assert_eq!(
        StudioTask::Correction.route_id(),
        "studio.subtitles.correction"
    );
}

#[test]
fn a_foreign_studio_source_cannot_replace_the_registered_shared_frame() {
    let (viewing, sources) = child(
        "studio/frame.tsx",
        "export function Foreign(){return <main/>;}",
    );
    let (cards, card_sources) = child("card.tsx", "export function Card(){return <main/>;}");
    assert!(ProductFrameBlueprint::prepare(viewing, sources, cards, card_sources).is_err());
}
fn props(label: &str) -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("label".into(), Value::String(label.into())),
        (
            "onTap".into(),
            Value::CallbackReference(CallbackReference {
                binding: "tap".into(),
                signature: CallbackSignature::Action,
            }),
        ),
    ])
}
fn callbacks(scope: PluginScope) -> HostCallbacks {
    let mut owner = HostCallbacks::new(scope);
    owner
        .bind_local("tap".into(), CallbackSignature::Action)
        .unwrap();
    owner
}

#[test]
fn direct_keyed_workspace_switch_keeps_original_host_and_revokes_old_callback_export() {
    let blueprint = blueprint();
    let program = blueprint
        .mount_viewing(
            props("Viewing"),
            "product-frame-contract".into(),
            1,
            NonZeroU64::new(1).unwrap(),
        )
        .unwrap();
    let owner = Rc::new(RefCell::new(None));
    let exporting = owner.clone();
    let mut document = InteractiveDocument::from_module_program_with_callbacks(
        program,
        "",
        Viewport {
            width: 320,
            height: 200,
            dark: true,
        },
        None,
        1,
        move |scope| {
            let callbacks = callbacks(scope);
            let export = callbacks.export_bindings().map_err(|error| error.message)?;
            *exporting.borrow_mut() = Some(callbacks);
            Ok(export)
        },
    )
    .unwrap();
    let scope = document.callback_scope().unwrap().clone();
    let button = document
        .elements()
        .iter()
        .find(|element| element.dom_tag == "button")
        .unwrap()
        .path
        .clone();
    document
        .dispatch(Action::Activate { path: button })
        .unwrap();
    let stale = document.drain_callbacks().pop().unwrap();
    let handle = NativeDocumentHandle::new(document, 1).unwrap();
    let before = handle.stamp().unwrap();
    let next_owner = callbacks(scope.clone());
    handle
        .try_reproject(&before, |document| {
            let props = blueprint.workspace_props(
                document,
                WorkspaceRoute::Cards,
                NonZeroU64::new(2).unwrap(),
                props("Cards"),
            )?;
            document.update_module_props_with_callbacks(
                props,
                next_owner
                    .export_bindings()
                    .map_err(|error| error.message)?,
            )
        })
        .unwrap();
    let after = handle.stamp().unwrap();
    assert_eq!(after.host_id, before.host_id);
    assert_eq!(after.owner, before.owner);
    assert_eq!(after.document_revision, before.document_revision + 1);
    assert!(
        handle
            .try_with(|document| document
                .elements()
                .iter()
                .any(|element| element.dom_tag == "article"))
            .unwrap()
    );
    assert!(
        owner
            .borrow()
            .as_ref()
            .unwrap()
            .dispatch_local(&stale.handle, CallbackArguments::Action, &scope)
            .is_err()
    );
    assert!(
        handle
            .try_reproject(&before, |_| Ok::<(), String>(()))
            .is_err()
    );
    assert_eq!(handle.stamp().unwrap(), after);
}

#[test]
fn child_graph_must_match_registered_source_inventory() {
    let (viewing, mut view_sources) = child(
        "viewing.tsx",
        "type Props={};export function Viewing(props:Props){return <section/>;}",
    );
    let (cards, card_sources) = child(
        "cards.tsx",
        "type Props={};export function Cards(props:Props){return <article/>;}",
    );
    *view_sources.values_mut().next().unwrap() = "different source bytes".into();
    assert!(ProductFrameBlueprint::prepare(viewing, view_sources, cards, card_sources).is_err());
}

#[test]
fn callback_preflight_rejection_keeps_program_export_and_original_sdk_attachment() {
    use kinekino_player::{
        application::{
            api::{ApplicationClientHandle, PlayerSdk, RequestContext},
            sdk_execution::{
                CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner,
            },
        },
        assembly::meari::viewer_callbacks::ViewerCallbacks,
    };
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };
    struct Notify;
    impl CompletionNotifier for Notify {
        fn notify(&self) -> Result<(), kinekino_player::application::api::ApiError> {
            Ok(())
        }
    }
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let execution = ExecutionOwner::start(
        client,
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 262144,
            max_total_request_bytes: 8 * 262144,
            max_terminal_bytes: 262144,
            max_total_terminal_bytes: 8 * 262144,
        },
        Arc::new(Notify),
    )
    .unwrap();
    let owner = Rc::new(RefCell::new(ViewerCallbacks::new(
        execution.handle(),
        RequestContext::default(),
    )));
    let blueprint = blueprint();
    let mut viewing_props = props("Viewing");
    viewing_props.insert(
        "onTap".into(),
        Value::CallbackReference(CallbackReference {
            binding: "onReadCapabilities".into(),
            signature: CallbackSignature::Action,
        }),
    );
    let program = blueprint
        .mount_viewing(
            viewing_props,
            "product-frame-rejection".into(),
            1,
            NonZeroU64::new(1).unwrap(),
        )
        .unwrap();
    let exporting = owner.clone();
    let mut document = InteractiveDocument::from_module_program_with_callbacks(
        program,
        "",
        Viewport {
            width: 320,
            height: 200,
            dark: true,
        },
        None,
        1,
        move |scope| {
            exporting
                .borrow_mut()
                .export(scope)
                .map_err(|error| error.message)
        },
    )
    .unwrap();
    let scope = document.callback_scope().unwrap().clone();
    let before_component = document.module_projection().unwrap().component.clone();
    let button = document
        .elements()
        .iter()
        .find(|element| element.dom_tag == "button")
        .unwrap()
        .path
        .clone();
    document
        .dispatch(Action::Activate { path: button })
        .unwrap();
    let pending = document.drain_callbacks().pop().unwrap();
    let handle = NativeDocumentHandle::new(document, 1).unwrap();
    let before = handle.stamp().unwrap();
    let export = owner.borrow().bindings().unwrap();
    let rejected = handle.try_reproject(&before, |document| {
        // The Cards fixture references "tap", absent from the real SDK owner's export.
        let props = blueprint.workspace_props(
            document,
            WorkspaceRoute::Cards,
            NonZeroU64::new(2).unwrap(),
            props("Cards"),
        )?;
        document.update_module_props_with_callbacks(props, export)
    });
    let error = match rejected {
        Err(error) => error,
        Ok(_) => panic!("The actual callback preflight must reject the missing binding"),
    };
    assert!(format!("{error:?}").contains("callback reference is not exported"));
    assert_eq!(handle.stamp().unwrap(), before);
    assert!(
        handle
            .try_with(|document| document
                .module_projection()
                .unwrap()
                .component
                .semantic_eq(&before_component))
            .unwrap()
    );
    assert!(execution.handle().attach().is_err());
    let ticket = owner.borrow().dispatch(&scope, &pending).unwrap().ticket;
    let deadline = Instant::now() + Duration::from_secs(2);
    let terminal = loop {
        if let Some(terminal) = owner.borrow().accept_terminal(&ticket).unwrap() {
            break terminal;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    };
    terminal.owner_ack.unwrap();
    assert!(
        owner
            .borrow_mut()
            .prepare_route_release(&scope)
            .unwrap()
            .is_none()
    );
    let next = execution.handle().attach().unwrap();
    next.release().unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let ExecutionCloseProgress::Closed(receipt) = execution.poll_close().unwrap() {
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
