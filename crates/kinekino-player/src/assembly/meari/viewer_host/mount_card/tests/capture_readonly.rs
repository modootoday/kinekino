//! Original host negative boundaries. Real LocalHost needs a separate execution slot.
#[path = "capture_readonly/execution_owner.rs"]
mod execution_owner;
#[path = "capture_readonly/guarded_ipc.rs"]
mod guarded_ipc;
use super::*;
use crate::assembly::meari::viewer_host::{
    ViewerCloseProgress, ViewerHost, ViewerHostError,
    close::{LocalEndpoint, OrderedClose, ViewerExecution},
};
use crate::presentation::web_ui_adapter::player_shell::{
    NavigationGuards, RouteCatalog, ShellState,
};
use meari::{
    application::host_callbacks::HostCallbacks,
    domain::host_callbacks::{CallbackArguments, CallbackReference},
};

fn host(
    input: &ProductCardMountInput,
    owner: ExecutionOwner,
    callbacks: Rc<RefCell<ViewerCallbacks>>,
) -> ViewerHost {
    let endpoint = crate::infrastructure::local_host::LocalHost::start(
        input.client.clone(),
        Default::default(),
    )
    .unwrap();
    let cards = Rc::new(RefCell::new(
        super::super::super::cards::CardRoute::default(),
    ));
    let jobs = Rc::new(RefCell::new(
        super::super::super::jobs::JobsLedger::default(),
    ));
    ViewerHost {
        native_route: {
            let mut gate =
                crate::assembly::meari::viewer_host::route_media::NativeRouteGate::default();
            gate.declare_headless().unwrap();
            gate
        },
        window_ui: Default::default(),
        automatic: Default::default(),
        registration_redraw: Default::default(),
        transition_slot: None,
        frame: Default::default(),
        document: Some(input.document.clone()),
        base_props: read_props(&StartLabels {
            connection: "Not observed".into(),
            time: "Not observed".into(),
        }),
        product_frame: Some(input.frame.clone()),
        read_retries: 3,
        read_admitted: false,
        viewing_chooser: Default::default(),
        viewing_generation: NonZeroU64::new(1).unwrap(),
        shutdown_started: false,
        completion: None,
        client: input.client.clone(),
        closing: OrderedClose::new(
            LocalEndpoint::new(endpoint),
            ViewerExecution {
                owner,
                callbacks: callbacks.clone(),
                cards: cards.clone(),
                jobs: jobs.clone(),
            },
        ),
        callbacks,
        cards,
        jobs,
        route_observation: None,
        identity: Rc::new(()),
        last_error: None,
    }
}

fn enqueue_legacy_capture(
    host: &ViewerHost,
    scope: &PluginScope,
) -> Rc<RefCell<CardSelectionQueue>> {
    let queue = host.card_selection_queue().unwrap();
    let mut authority = HostCallbacks::new(scope.clone());
    let key = authority
        .bind_local(CARD_SELECTION_BINDING.into(), CallbackSignature::Text)
        .unwrap();
    let export = authority.export_bindings().unwrap();
    let handle = export
        .resolve(&CallbackReference {
            binding: CARD_SELECTION_BINDING.into(),
            signature: CallbackSignature::Text,
        })
        .unwrap();
    let invocation = authority
        .dispatch_local(
            &handle,
            CallbackArguments::Text("project-card-1-0-1".into()),
            scope,
        )
        .unwrap();
    // Inject a pre-restriction queued target ONLY to exercise the direct public caller.
    // This is not evidence for product callback admission; source5 has that separate test.
    assert!(matches!(
        queue.borrow_mut().admit(invocation, &key),
        IntentAdmission::Accepted
    ));
    queue
}

type CasePanic = Box<dyn std::any::Any + Send>;
struct RetainedCaseFailure {
    host: Box<ViewerHost>,
    primary: Option<CasePanic>,
    close_panic: Option<CasePanic>,
    close_errors: Vec<ViewerHostError>,
    complete: bool,
}
struct PanicDescription<'a>(&'a (dyn std::any::Any + Send));
impl std::fmt::Debug for PanicDescription<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(message) = self.0.downcast_ref::<String>() {
            std::fmt::Debug::fmt(message, f)
        } else if let Some(message) = self.0.downcast_ref::<&str>() {
            std::fmt::Debug::fmt(message, f)
        } else {
            f.debug_tuple("OpaquePanic")
                .field(&self.0.type_id())
                .finish()
        }
    }
}
impl std::fmt::Debug for RetainedCaseFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginalViewerCaseFailed")
            .field(
                "primary",
                &self.primary.as_ref().map(|p| PanicDescription(p.as_ref())),
            )
            .field(
                "close_panic",
                &self
                    .close_panic
                    .as_ref()
                    .map(|p| PanicDescription(p.as_ref())),
            )
            .field("close_errors", &self.close_errors)
            .field("complete", &self.complete)
            .field("original_progress", &self.host.last_close_progress())
            .field("original_error", &self.host.last_close_error())
            .finish()
    }
}

// Test-only: keep the exact owner in a returned failure carrier through failure reporting.
// Neither unwinding, Drop nor a deadline is recorded as completed shutdown.
fn run_retaining_case(
    mut host: ViewerHost,
    body: impl FnOnce(&mut ViewerHost),
) -> Result<(), Box<RetainedCaseFailure>> {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let primary = catch_unwind(AssertUnwindSafe(|| body(&mut host))).err();
    let mut close_errors = Vec::new();
    let close = catch_unwind(AssertUnwindSafe(|| {
        close_unrun(&mut host, &mut close_errors)
    }));
    let (complete, close_panic) = match close {
        Ok(complete) => (complete, None),
        Err(payload) => (false, Some(payload)),
    };
    eprintln!(
        "original_viewer_case_closeout complete={complete} primary_panic={} close_panic={} close_errors={} progress={:?}",
        primary.is_some(),
        close_panic.is_some(),
        close_errors.len(),
        host.last_close_progress()
    );
    if complete && primary.is_none() && close_panic.is_none() && close_errors.is_empty() {
        Ok(())
    } else {
        Err(Box::new(RetainedCaseFailure {
            host: Box::new(host),
            primary,
            close_panic,
            close_errors,
            complete,
        }))
    }
}

fn close_unrun(host: &mut ViewerHost, errors: &mut Vec<ViewerHostError>) -> bool {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match host.poll_unrun_shutdown() {
            Ok(ViewerCloseProgress::Complete {
                endpoint,
                execution,
            }) => {
                return endpoint.is_quiescent()
                    && execution.worker_stopped
                    && execution.unacknowledged == 0;
            }
            Err(error) => errors.push(error),
            Ok(_) => {}
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
#[ignore = "Requires an explicit LocalHost named-pipe slot; no window or renderer is started"]
fn public_capture_route_rejects_without_consuming_target_state_or_live_read_attachment()
-> Result<(), Box<RetainedCaseFailure>> {
    let ((input, owner, gate), callbacks) = fixture_with_capture_choices(true);
    let scope = input.document.stamp().unwrap().owner;
    let execution = owner.handle();
    let host = host(&input, owner, callbacks.clone());
    run_retaining_case(host, |host| {
        let bindings = callbacks
            .borrow_mut()
            .resume_released_route(&scope)
            .unwrap();
        input
            .document
            .try_reproject(&input.document.stamp().unwrap(), |document| {
                let props = input.frame.workspace_props(
                    document,
                    WorkspaceRoute::Viewing,
                    NonZeroU64::new(1).unwrap(),
                    read_props(&StartLabels {
                        connection: "Not observed".into(),
                        time: "Not observed".into(),
                    }),
                )?;
                document.update_module_props_with_callbacks(props, bindings)
            })
            .unwrap();
        let queue = enqueue_legacy_capture(host, &scope);
        let target = queue.borrow().front().unwrap().clone();
        let captured = serde_json::to_vec(target.project.as_ref()).unwrap();
        let before = input.document.stamp().unwrap();
        let commands = gate.all_calls.lock().unwrap().clone();
        let mut state = ShellState::new();
        let catalog = RouteCatalog::bundled().unwrap();
        assert!(
            matches!(host.request_selected_card_route(&mut state, &catalog, NavigationGuards::default()),
        Err(ViewerHostError::Sdk(error)) if error.code == crate::application::api::ErrorCode::Unsupported)
        );
        assert_eq!(
            state.phase(),
            crate::presentation::web_ui_adapter::player_shell::ShellPhase::NoSelection
        );
        assert!(state.observed_route().is_none());
        assert!(Arc::ptr_eq(queue.borrow().front().unwrap(), &target));
        assert_eq!(
            serde_json::to_vec(target.project.as_ref()).unwrap(),
            captured
        );
        assert_eq!(input.document.stamp().unwrap(), before);
        let observed_commands = gate.all_calls.lock().unwrap().clone();
        assert_eq!(observed_commands, commands);
        assert!(!callbacks.borrow().is_released());
        assert!(execution.attach().is_err());
    })
}

#[test]
#[ignore = "Requires an explicit LocalHost named-pipe slot; no window or renderer is started"]
fn retained_capture_inspection_cannot_bind_editor_and_keeps_original_registration_for_close()
-> Result<(), Box<RetainedCaseFailure>> {
    let ((input, owner, gate), callbacks) = fixture_with_capture_choices(true);
    let scope = input.document.stamp().unwrap().owner;
    let execution = owner.handle();
    let host = host(&input, owner, callbacks);
    run_retaining_case(host, |host| {
        let queue = enqueue_legacy_capture(host, &scope);
        let mut state = ShellState::new();
        let request = queue
            .borrow_mut()
            .request_route(
                &mut state,
                &RouteCatalog::bundled().unwrap(),
                NavigationGuards::default(),
            )
            .unwrap()
            .unwrap();
        let target = request.target.clone();
        host.cards.borrow_mut().opening = Some(super::super::super::cards::CardOpening {
            request,
            attempted: true,
            panel_attempted: false,
            retained_inspection: None,
            receipt: None,
            route_error: None,
            start_failure: None,
            api_error: None,
        });
        let registry = Rc::new(RefCell::new(
            PluginRegistry::new(
                input.frame.graph.fingerprint().unwrap(),
                TargetKind::Native,
                scope.clone(),
                8,
            )
            .unwrap(),
        ));
        let inspection = match CardCurrentInspection::new(CardInspectionInput {
            execution: execution.clone(),
            scope,
            registry,
            context: Rc::new(RefCell::new(RequestContext::default())),
            namespace: "retained-capture-boundary".into(),
            limit: 8,
            codec: product_codec_budget(),
        }) {
            Ok(inspection) => inspection,
            Err(failure) => {
                host.cards
                    .borrow_mut()
                    .opening
                    .as_mut()
                    .unwrap()
                    .start_failure = Some(failure);
                panic!(
                    "Original inspect owner must attach; failed registration retained for close"
                );
            }
        };
        host.cards.borrow_mut().inspection = Some(inspection);
        host.cards
            .borrow_mut()
            .inspection
            .as_mut()
            .unwrap()
            .begin(BootstrapSelection::try_from(target.selection.clone()).unwrap())
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while !host
            .cards
            .borrow_mut()
            .inspection
            .as_mut()
            .unwrap()
            .poll()
            .unwrap()
        {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        let terminal = host
            .cards
            .borrow()
            .inspection
            .as_ref()
            .unwrap()
            .terminal()
            .unwrap() as *const _ as usize;
        let before = input.document.stamp().unwrap();
        let commands = gate.all_calls.lock().unwrap().clone();
        assert!(matches!(host.finish_inspected_card(input.turn.clone()),
        Err(ViewerHostError::Sdk(error)) if error.code == crate::application::api::ErrorCode::Unsupported));
        {
            let cards = host.cards.borrow();
            assert!(cards.active.is_none());
            assert!(cards.opening.as_ref().unwrap().panel_attempted);
            assert!(Arc::ptr_eq(
                &cards.opening.as_ref().unwrap().request.target,
                &target
            ));
            let retained = cards.inspection.as_ref().unwrap();
            assert_eq!(retained.terminal().unwrap() as *const _ as usize, terminal);
            assert!(!retained.is_released());
        }
        assert_eq!(input.document.stamp().unwrap(), before);
        let observed_commands = gate.all_calls.lock().unwrap().clone();
        assert_eq!(observed_commands, commands);
        assert!(execution.attach().is_err());
    })
}

#[test]
#[ignore = "Requires an explicit LocalHost named-pipe slot; injects a body panic, no window"]
fn assertion_failure_still_closes_original_viewer_and_retains_primary_payload() {
    let ((input, owner, _gate), callbacks) = fixture_with_capture_choices(true);
    let host = host(&input, owner, callbacks);
    let failed = run_retaining_case(host, |_| {
        panic!("retaining viewer case assertion injection")
    })
    .unwrap_err();
    assert!(failed.primary.is_some());
    assert_eq!(
        failed.primary.as_ref().and_then(|payload| payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())),
        Some("retaining viewer case assertion injection")
    );
    assert!(failed.complete);
    assert!(failed.close_errors.is_empty());
    assert!(failed.close_panic.is_none());
    assert!(matches!(
        failed.host.last_close_progress(),
        Some(ViewerCloseProgress::Complete { .. })
    ));
}
