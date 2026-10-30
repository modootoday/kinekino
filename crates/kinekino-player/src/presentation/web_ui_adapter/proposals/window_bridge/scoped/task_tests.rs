//! Headless mount/owner policy, not rendered correction or checked-frame proof.
use super::*;
use crate::presentation::web_ui_adapter::proposals::{
    correction_panel::CorrectionLabels,
    scoped_task_panel::{
        bind_task_panel,
        tests::{Kind, Lease, correction_labels, fixture, register},
    },
};
use meari::{application::source_modules::ModuleProgram, domain::build::ContentHash};
use meari_native::{domain::Viewport, infrastructure::document::InteractiveDocument};
use std::cell::Cell;

fn input(
    kind: Kind,
    reject: bool,
) -> (ScopedBridgeMount<Lease>, CorrectionLabels, Rc<Cell<usize>>) {
    let (_, mut input, stored) = fixture(kind);
    let entry: BuildPath = "scope-host.tsx".to_owned().try_into().unwrap();
    let inventory = BTreeMap::from([(
        entry.clone(),
        "type Props={};export function Host(props:Props){return <main/>;}".into(),
    )]);
    let graph = meari::parse_module_graph(&entry, &inventory).unwrap();
    let program = ModuleProgram::mount(graph, BTreeMap::new(), "task-frame".into(), 1).unwrap();
    let document = InteractiveDocument::from_module_program_with_callbacks(
        program,
        "",
        Viewport {
            width: 320,
            height: 200,
            dark: true,
        },
        None,
        1,
        |scope| {
            meari::application::host_callbacks::HostCallbacks::new(scope)
                .export_bindings()
                .map_err(|error| error.message)
        },
    )
    .unwrap();
    input.scope = document.callback_scope().unwrap().clone();
    input.registry = Rc::new(RefCell::new(
        PluginRegistry::new(
            ContentHash::of(b"task-host"),
            meari::domain::build::TargetKind::Native,
            input.scope.clone(),
            8,
        )
        .unwrap(),
    ));
    let correction = correction_labels(&input);
    let labels = input.labels.panel.clone();
    let panel = bind_task_panel(
        input,
        stored,
        if matches!(kind, Kind::Highlight) {
            None
        } else {
            Some(correction.clone())
        },
        register,
    )
    .unwrap_or_else(|failure| panic!("{:?}", failure.cause));
    let handle = NativeDocumentHandle::new(document, 1).unwrap();
    let calls = Rc::new(Cell::new(0));
    let called = calls.clone();
    (
        ScopedBridgeMount {
            turn: HostTurn {
                stamp: handle.stamp().unwrap(),
                reason: HostTurnReason::Input,
                accepted_callbacks: 0,
                rejected_callbacks: 0,
                remaining_callbacks: 0,
            },
            handle,
            panel,
            frame: None,
            inventory,
            labels: Box::new(move |_, _, _| labels.clone()),
            close_policy: Box::new(|_, _| Ok(true)),
            pins:
                crate::presentation::web_ui_adapter::proposals::product_window::product_intent_pins(
                ),
            commit: Box::new(move |document, _, export| {
                called.set(called.get() + 1);
                if reject {
                    Err("original task commit fault".into())
                } else {
                    document.update_module_props_with_callbacks(BTreeMap::new(), export)
                }
            }),
        },
        correction,
        calls,
    )
}

#[test]
fn correction_provider_is_required_before_any_original_commit() {
    let (input, _, calls) = input(Kind::Cue, false);
    let before = input.handle.stamp().unwrap();
    let mut failure = match ProposalWindowBridge::mount_scoped(input) {
        Err(failure) => failure,
        Ok(_) => panic!("Missing correction provider"),
    };
    assert_eq!(calls.get(), 0);
    assert!(failure.correction_labels.is_none());
    assert_eq!(failure.input.handle.stamp().unwrap(), before);
    assert!(failure.input.panel.parent.registration().is_ok());
    failure.input.panel.unbind().unwrap();
}

#[test]
fn callback_suffix_and_foreign_scope_reject_before_commit_without_unbinding() {
    for foreign in [false, true] {
        let (mut input, labels, calls) = input(Kind::Document, false);
        if foreign {
            input.turn.stamp.owner.source_generation += 1;
        } else {
            input.turn.remaining_callbacks = 1;
        }
        let mut failure = match ProposalWindowBridge::mount_scoped_task(
            input,
            Some(Box::new(move |_, _, _| labels.clone())),
        ) {
            Err(failure) => failure,
            Ok(_) => panic!("Invalid original turn"),
        };
        assert_eq!(calls.get(), 0);
        assert!(failure.correction_labels.is_some());
        assert!(failure.input.panel.parent.registration().is_ok());
        failure.input.panel.unbind().unwrap();
    }
}

#[test]
fn failed_original_commit_keeps_same_handle_and_complete_scoped_owner() {
    let (input, labels, calls) = input(Kind::Document, true);
    let before = input.handle.stamp().unwrap();
    let mut failure = match ProposalWindowBridge::mount_scoped_task(
        input,
        Some(Box::new(move |_, _, _| labels.clone())),
    ) {
        Err(failure) => failure,
        Ok(_) => panic!("Original commit fault"),
    };
    assert!(
        matches!(&failure.error, PanelBridgeError::Commit(message) if message == "original task commit fault")
    );
    assert_eq!(calls.get(), 1);
    assert!(failure.correction_labels.is_some());
    assert_eq!(failure.input.handle.stamp().unwrap(), before);
    assert!(failure.input.panel.parent.registration().is_ok());
    failure.input.panel.unbind().unwrap();
}

#[test]
fn typed_task_mount_installs_provider_and_uses_original_native_runtime() {
    for kind in [Kind::Highlight, Kind::Cue, Kind::Document] {
        let (input, labels, calls) = input(kind, false);
        let handle = input.handle.clone();
        let before = handle.stamp().unwrap();
        let provider: Option<CorrectionLabelProvider> = if matches!(kind, Kind::Highlight) {
            None
        } else {
            Some(Box::new(move |_, _, _| labels.clone()))
        };
        let (bridge, receipt) = ProposalWindowBridge::mount_scoped_task(input, provider)
            .unwrap_or_else(|_| panic!("Original task mount"));
        assert_eq!(receipt.before(), &before);
        assert_eq!(receipt.after().host_id, before.host_id);
        assert_eq!(calls.get(), 1);
        assert_eq!(
            bridge.correction_labels.is_some(),
            !matches!(kind, Kind::Highlight)
        );
        let mut runtime = NativeHostRuntime::new(handle, bridge);
        runtime.after_dispatch(HostTurnReason::Completion).unwrap();
        assert!(runtime.last_host_error().is_none());
        runtime.host_mut().unbind_scope().unwrap();
    }
}

#[test]
fn local_feedback_reprojects_on_original_task_owner_without_changing_callback_guards() {
    for kind in [Kind::Highlight, Kind::Cue, Kind::Document] {
        let (mut input, labels, _) = input(kind, false);
        let captured = Rc::new(RefCell::new(Vec::<BTreeMap<String, Value>>::new()));
        let sink = captured.clone();
        input.commit = Box::new(move |document, props, export| {
            sink.borrow_mut().push(props);
            document.update_module_props_with_callbacks(BTreeMap::new(), export)
        });
        let handle = input.handle.clone();
        let scope = input.panel.session.review().scope().clone();
        let provider: Option<CorrectionLabelProvider> = if matches!(kind, Kind::Highlight) {
            None
        } else {
            Some(Box::new(move |_, _, _| labels.clone()))
        };
        let (mut bridge, _) = ProposalWindowBridge::mount_scoped_task(input, provider)
            .unwrap_or_else(|_| panic!("Original task mount"));
        let registry = bridge.owners.registry.clone();
        let context = bridge.owners.context.clone();
        let selected = bridge.owners.selection.proposal.proposal_id.clone();
        let initial = captured.borrow()[0].clone();
        bridge.last_view_error = Some(ViewError::InvalidSelection);
        bridge.needs_projection = true;
        let mut runtime = NativeHostRuntime::new(handle.clone(), bridge);
        runtime.after_dispatch(HostTurnReason::Input).unwrap();
        assert!(runtime.last_host_error().is_none());
        let projected = captured.borrow().last().unwrap().clone();
        assert_ne!(projected["resultLabel"], initial["resultLabel"]);
        assert_eq!(projected["revisionLabel"], initial["revisionLabel"]);
        for name in ["canSave", "canAccept", "onSave", "onAccept", "onRefresh"] {
            assert_eq!(projected[name], initial[name]);
        }
        assert_eq!(runtime.host().review().scope(), &scope);
        assert_eq!(
            runtime.host().owners.selection.proposal.proposal_id,
            selected
        );
        assert!(Rc::ptr_eq(&runtime.host().owners.registry, &registry));
        assert!(Rc::ptr_eq(&runtime.host().owners.context, &context));
        assert_eq!(registry.borrow().pending_count(), 0);
        assert_eq!(
            runtime.host().document_handle().stamp().unwrap(),
            handle.stamp().unwrap()
        );
        runtime.host_mut().last_view_error = None;
        runtime.host_mut().needs_projection = true;
        runtime.after_dispatch(HostTurnReason::Input).unwrap();
        assert_eq!(
            captured.borrow().last().unwrap()["resultLabel"],
            initial["resultLabel"]
        );
        runtime.host_mut().unbind_scope().unwrap();
    }
}
