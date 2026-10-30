use super::super::super::{product_window::product_intent_pins, scoped_panel};
use super::*;
use kinekino_player::assembly::meari::proposal_registration::ProposalSdkRegistration;
use meari::{
    application::host_callbacks::HostCallbacks,
    domain::{
        build::{ContentHash, TargetKind},
        host_callbacks::{CallbackReference, CallbackSignature},
    },
};
use meari_native::{domain::Viewport, infrastructure::document::InteractiveDocument};
use std::cell::Cell;

struct Lease(ProposalSdkRegistration);
impl ParentProposalLease for Lease {
    fn registration(&self) -> Result<&RegistrationHandle, PluginError> {
        self.0.handle()
    }
    fn close(&mut self) -> Result<(), PluginError> {
        self.0.close()
    }
}

fn mount_input(calls: Rc<Cell<usize>>) -> ScopedBridgeMount<Lease> {
    input_with_commit(calls, false)
}
fn input_with_commit(calls: Rc<Cell<usize>>, commit_succeeds: bool) -> ScopedBridgeMount<Lease> {
    let (_, mut input, stored) = scoped_panel::tests::fixture();
    let entry: BuildPath = "frame.tsx".to_owned().try_into().unwrap();
    let inventory = BTreeMap::from([(
        entry.clone(),
        "type Props={onReturnToViewing:()=>void};export function Frame(props:Props){return <main><button onClick={() => props.onReturnToViewing()}>Return</button></main>;}".into(),
    )]);
    let graph = meari::parse_module_graph(&entry, &inventory).unwrap();
    let program = meari::application::source_modules::ModuleProgram::mount(
        graph,
        navigation_props(),
        "scoped-bridge".into(),
        1,
    )
    .unwrap();
    let initial_callbacks = Rc::new(RefCell::new(None));
    let exporting = initial_callbacks.clone();
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
        move |scope| {
            let mut callbacks = HostCallbacks::new(scope);
            callbacks
                .bind_local("onReturnToViewing".into(), CallbackSignature::Action)
                .unwrap();
            let export = callbacks.export_bindings().map_err(|error| error.message)?;
            *exporting.borrow_mut() = Some(callbacks);
            Ok(export)
        },
    )
    .unwrap();
    input.scope = document.callback_scope().unwrap().clone();
    input
        .local_signatures
        .insert("onReturnToViewing".into(), CallbackSignature::Action);
    input.registry = Rc::new(RefCell::new(
        PluginRegistry::new(
            ContentHash::of(b"scoped-bridge"),
            TargetKind::Native,
            input.scope.clone(),
            8,
        )
        .unwrap(),
    ));
    let labels = input.labels.panel.clone();
    let panel = scoped_panel::bind_card_panel(input, stored, |client, registry, scope, context| {
        ProposalSdkRegistration::register(client, registry, scope, context).map(Lease)
    })
    .unwrap_or_else(|_| panic!("Canonical Card must bind"));
    let handle = NativeDocumentHandle::new(document, 1).unwrap();
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
        pins: product_intent_pins(),
        commit: Box::new(move |document, _, export| {
            assert!(initial_callbacks.borrow().is_some());
            calls.set(calls.get() + 1);
            if commit_succeeds {
                document.update_module_props_with_callbacks(navigation_props(), export)
            } else {
                Err("original commit rejection".into())
            }
        }),
    }
}

fn navigation_props() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "onReturnToViewing".into(),
        Value::CallbackReference(CallbackReference {
            binding: "onReturnToViewing".into(),
            signature: CallbackSignature::Action,
        }),
    )])
}

#[test]
fn real_renderer_local_callback_uses_current_opaque_key_and_capacity_resume() {
    let input = input_with_commit(Rc::new(Cell::new(0)), true);
    let handle = input.handle.clone();
    let before = handle.stamp().unwrap();
    let (bridge, receipt) = ProposalWindowBridge::mount_scoped(input)
        .unwrap_or_else(|_| panic!("Actual fixture props commit must succeed"));
    assert_eq!(receipt.before(), &before);
    assert_eq!(receipt.after().host_id, before.host_id);
    assert_eq!(
        receipt.after().document_revision,
        before.document_revision + 1
    );
    let attempts = Rc::new(Cell::new(0));
    let accepted = Rc::new(Cell::new(0));
    let blocked = Rc::new(Cell::new(true));
    let (count_attempts, count_accepted, is_blocked) =
        (attempts.clone(), accepted.clone(), blocked.clone());
    let bridge = bridge.with_local_intents(Box::new(move |session, invocation| {
        assert!(invocation.matches_key(session.local_action_key("onReturnToViewing").unwrap()));
        count_attempts.set(count_attempts.get() + 1);
        if is_blocked.get() {
            return IntentAdmission::Capacity;
        }
        count_accepted.set(count_accepted.get() + 1);
        IntentAdmission::Accepted
    }));
    let button = handle
        .try_with(|document| {
            document
                .elements()
                .iter()
                .find(|element| element.dom_tag == "button")
                .unwrap()
                .path
                .clone()
        })
        .unwrap();
    handle
        .dispatch_action(meari_native::domain::widgets::Action::Activate { path: button })
        .unwrap();
    let mut runtime = NativeHostRuntime::new(handle, bridge);
    let first = runtime.after_dispatch(HostTurnReason::Input).unwrap();
    assert_eq!(
        (first.accepted_callbacks, first.remaining_callbacks),
        (0, 1)
    );
    assert_eq!(attempts.get(), 1);
    assert_eq!(accepted.get(), 0);
    assert!(first.committed.is_none());
    blocked.set(false);
    let resumed = runtime
        .after_dispatch(HostTurnReason::ResumePending)
        .unwrap();
    assert_eq!(
        (resumed.accepted_callbacks, resumed.remaining_callbacks),
        (1, 0)
    );
    assert_eq!(accepted.get(), 1);
    assert_eq!(attempts.get(), 2);
    let quiet = runtime.after_dispatch(HostTurnReason::Completion).unwrap();
    assert_eq!(quiet.accepted_callbacks, 0);
    assert_eq!(accepted.get(), 1);
    runtime.host_mut().unbind_scope().unwrap();
}

#[test]
fn unhandled_real_local_callback_is_rejected_without_plugin_fallback() {
    let input = input_with_commit(Rc::new(Cell::new(0)), true);
    let handle = input.handle.clone();
    let (bridge, _) = ProposalWindowBridge::mount_scoped(input)
        .unwrap_or_else(|_| panic!("Actual fixture props commit must succeed"));
    let revision = bridge.review().stored().document.draft_revision;
    let button = handle
        .try_with(|document| {
            document
                .elements()
                .iter()
                .find(|element| element.dom_tag == "button")
                .unwrap()
                .path
                .clone()
        })
        .unwrap();
    handle
        .dispatch_action(meari_native::domain::widgets::Action::Activate { path: button })
        .unwrap();
    let mut runtime = NativeHostRuntime::new(handle, bridge);
    let turn = runtime.after_dispatch(HostTurnReason::Input).unwrap();
    assert_eq!(
        (
            turn.accepted_callbacks,
            turn.rejected_callbacks,
            turn.remaining_callbacks
        ),
        (0, 1, 0)
    );
    assert!(matches!(
        runtime.last_host_error(),
        Some(PanelBridgeError::Admission(
            "Frame local intent consumer is absent"
        ))
    ));
    assert_eq!(
        runtime.host().review().stored().document.draft_revision,
        revision
    );
    assert!(!runtime.host().operation_pending());
    runtime.host_mut().unbind_scope().unwrap();
}

#[test]
fn rejected_commit_retains_original_document_and_live_scoped_owner_for_retry() {
    let calls = Rc::new(Cell::new(0));
    let input = mount_input(calls.clone());
    let before = input.handle.stamp().unwrap();
    let mut failure = match ProposalWindowBridge::mount_scoped(input) {
        Err(failure) => failure,
        Ok(_) => panic!("Commit must reject"),
    };
    assert_eq!(calls.get(), 1);
    assert!(
        matches!(&failure.error, PanelBridgeError::Commit(message) if message == "original commit rejection")
    );
    assert_eq!(failure.input.handle.stamp().unwrap(), before);
    assert!(failure.input.panel.parent.registration().is_ok());
    failure.input.panel.unbind().unwrap();
    assert!(failure.input.panel.parent.registration().is_err());
}

#[test]
fn remaining_callback_suffix_rejects_before_renderer_or_scoped_unbind() {
    let calls = Rc::new(Cell::new(0));
    let mut input = mount_input(calls.clone());
    input.turn.remaining_callbacks = 1;
    let mut failure = match ProposalWindowBridge::mount_scoped(input) {
        Err(failure) => failure,
        Ok(_) => panic!("Callback suffix must reject"),
    };
    assert_eq!(calls.get(), 0);
    assert!(matches!(
        failure.error,
        PanelBridgeError::Admission("Frame callback suffix remains")
    ));
    assert!(failure.input.panel.parent.registration().is_ok());
    failure.input.panel.unbind().unwrap();
}
