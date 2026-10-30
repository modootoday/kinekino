use super::*;

#[test]
fn empty_callback_snapshot_cannot_claim_ack_requirement_or_worker_completion() {
    let progress = ViewerCloseProgress::CallbackPending {
        tickets: Vec::new(),
    };
    assert!(matches!(
        project_progress(
            Some(progress),
            ViewerNativeState::Unconfirmed,
            Duration::from_millis(1)
        ),
        CleanupPoll::Pending(_)
    ));
}
use crate::{
    application::sdk_execution::{ExecutionCloseReceipt, WorkerCloseState},
    infrastructure::local_host::{LocalHostShutdownStatus, WorkerJoin},
};

#[test]
fn only_open_requests_and_only_closing_observes_the_original_runtime() {
    assert_eq!(close_action(HostLifecycle::Open), CloseAction::Request);
    assert_eq!(close_action(HostLifecycle::Closing), CloseAction::Observe);
    for state in [
        HostLifecycle::CleanupFailed,
        HostLifecycle::ForcedIncomplete,
    ] {
        assert_eq!(close_action(state), CloseAction::ExplicitRecovery);
    }
    for state in [HostLifecycle::HostCleaned, HostLifecycle::Closed] {
        assert_eq!(close_action(state), CloseAction::Retain);
    }
}

#[test]
#[ignore = "Requires explicit local pipe/client worker authority; no OS window, parent or event loop is created"]
fn actual_open_returned_runtime_starts_drain_before_execution_close() {
    use crate::{
        application::{
            api::{
                ApplicationClient, ApplicationClientHandle, PlayerSdk, Request, RequestContext,
                Response,
            },
            sdk_execution::{
                CompletionNotifier, ExecutionBudget, ExecutionOwner, ExecutionSnapshot,
            },
        },
        assembly::meari::{
            native_cleanup_driver::drive_cleanup,
            player_shell_document::{DocumentOptions, mount_prepared_shell},
            player_shell_host::{ShellHostConfig, ShellHostController, ShellHostError},
            player_shell_startup::{ShellInputs, prepare_shell},
            viewer_host::ViewerHost,
        },
        infrastructure::local_host::{
            EndpointClient, LocalHost, TransportDiagnostics, TransportOptions,
        },
        presentation::web_ui_adapter::player_shell::document::StartLabels,
    };
    use meari_native::{
        application::{
            WindowOptions,
            window_lifecycle::{MonotonicClock, WindowLifecycle},
        },
        domain::Viewport,
        infrastructure::{native_host::HostScheduleLimits, window::NativeRunError},
    };
    use std::{
        sync::{Arc, Mutex, mpsc},
        task::Waker,
    };
    struct Notify;
    impl CompletionNotifier for Notify {
        fn notify(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    struct Clock;
    impl MonotonicClock for Clock {
        fn now(&self) -> Duration {
            Duration::ZERO
        }
    }
    struct Gate {
        entered: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
        sdk: PlayerSdk,
    }
    impl ApplicationClient for Gate {
        fn execute(&self, request: Request) -> Response {
            if request.context.request_id == "endpoint-block" {
                let _ = self.entered.send(());
                let _ = self
                    .release
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(30));
            }
            self.sdk.execute(request)
        }
    }
    let (entered, observed) = mpsc::channel();
    let (release, waiting) = mpsc::channel();
    struct Release(Option<mpsc::Sender<()>>);
    impl Drop for Release {
        fn drop(&mut self) {
            if let Some(release) = self.0.take() {
                let _ = release.send(());
            }
        }
    }
    let mut release = Release(Some(release));
    let client: ApplicationClientHandle = Arc::new(Gate {
        entered,
        release: Mutex::new(waiting),
        sdk: PlayerSdk::default(),
    });
    let trace = TransportDiagnostics::default();
    let transport = TransportOptions {
        timeout: Duration::from_secs(2),
    };
    let endpoint = LocalHost::start_observed(client.clone(), transport, trace.clone()).unwrap();
    let endpoint_name = endpoint.endpoint().to_owned();
    let requester = std::thread::spawn(move || {
        let endpoint = EndpointClient::new(&endpoint_name, transport).unwrap();
        endpoint.execute(Request {
            context: RequestContext {
                request_id: "endpoint-block".into(),
                ..Default::default()
            },
            command: "capabilities".into(),
            payload: serde_json::json!({}),
        })
    });
    observed.recv_timeout(Duration::from_secs(2)).unwrap();
    let execution = ExecutionOwner::start(
        client.clone(),
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
    let execution_handle = execution.handle();
    let host = ViewerHost::new(client.clone(), execution, endpoint);
    let prepared = match prepare_shell(
        ShellInputs {
            client: client.clone(),
            host_owner: host,
        },
        StartLabels {
            connection: "관측 없음".into(),
            time: "관측 없음".into(),
        },
    ) {
        Ok(prepared) => prepared,
        Err(_) => panic!("fixed shell source must prepare"),
    };
    let document = match mount_prepared_shell(
        prepared,
        DocumentOptions {
            namespace: "returned-open-test".into(),
            epoch: 1,
            source_generation: 1,
            viewport: Viewport {
                width: 320,
                height: 180,
                dark: true,
            },
        },
    ) {
        Ok(document) => document,
        Err(_) => panic!("CPU document must prepare"),
    };
    let limits =
        HostScheduleLimits::new(Duration::from_millis(2), Duration::from_millis(250), 100).unwrap();
    let mut controller: ViewerController = ShellHostController::new(
        document,
        ShellHostConfig {
            options: WindowOptions::default(),
            children: LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
            schedule: limits,
        },
    );
    let children = Rc::new(
        WindowLifecycle::<ViewerNoChild>::new(
            LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
            Clock,
            Waker::noop().clone(),
        )
        .unwrap(),
    );
    // The original production binding function moves the same H/document into this real runtime.
    let runtime = controller.bind_unrun_runtime_for_test(children).unwrap();
    assert_eq!(runtime.borrow().document().lifecycle(), HostLifecycle::Open);
    assert!(Rc::ptr_eq(controller.runtime.as_ref().unwrap(), &runtime));
    let owners = ViewerOwners::Returned(Box::new(NativeRunOutcome {
        controller,
        startup: None,
        runtime: Some(runtime.clone()),
        recovery: None,
        loop_owner: None,
        result: Err(NativeRunError::Consumer(ShellHostError::Attach)),
    }));
    let mut cleanup = ViewerCleanup::new(
        owners,
        ViewerCleanupLimits::new(Duration::from_secs(2), 500, Duration::from_millis(2)).unwrap(),
    );
    let first = cleanup.poll(); // Actual Returned branch, not project_progress alone.
    let started = matches!(first, CleanupPoll::Pending(_))
        && matches!(
            cleanup.evidence.last_workers,
            Some(ViewerCloseProgress::EndpointPending(_))
        );
    let probe = execution_handle.submit(Request {
        context: RequestContext {
            request_id: "execution-probe".into(),
            ..Default::default()
        },
        command: "capabilities".into(),
        payload: serde_json::json!({}),
    });
    let execution_was_open = probe.is_ok();
    if let Ok(operation) = probe {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if matches!(
                execution_handle.peek(&operation).unwrap(),
                ExecutionSnapshot::Terminal(_)
            ) {
                execution_handle.acknowledge(&operation).unwrap();
                break;
            }
            assert!(Instant::now() < deadline, "probe response missing");
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    let _ = release.0.take().unwrap().send(());
    let _original_request_result = requester.join().unwrap();
    // On a regressed build, recover the retained original runtime for cleanup before failing assertions.
    if !started {
        let _ = runtime.borrow_mut().request_close(CloseCause::Host);
    }
    let (owners, _) = cleanup.into_parts();
    let result = drive_cleanup(
        ViewerCleanup::new(
            owners,
            ViewerCleanupLimits::new(Duration::from_secs(2), 500, Duration::from_millis(2))
                .unwrap(),
        ),
        Duration::from_millis(2),
        std::thread::sleep,
    );
    assert!(
        started,
        "Open Returned runtime never started endpoint drain"
    );
    assert!(
        execution_was_open,
        "ExecutionOwner was closed before accepted endpoint work drained"
    );
    assert_eq!(
        result.status,
        ViewerCleanupStatus::WorkersQuiescent(ViewerNativeState::Unconfirmed)
    );
    match result.evidence.last_workers.unwrap() {
        ViewerCloseProgress::Complete {
            endpoint,
            execution,
        } => {
            assert!(endpoint.is_quiescent());
            assert!(execution.worker_stopped);
            assert_eq!(execution.unacknowledged, 0);
        }
        _ => panic!("actual join receipts required"),
    }
    assert_eq!(
        trace
            .snapshot()
            .iter()
            .filter(|event| event.phase == "host" && event.operation == "stopped")
            .count(),
        1
    );
}

#[test]
fn terminal_ack_is_a_returned_boundary_not_an_automatic_accept_request() {
    let progress =
        ViewerCloseProgress::ExecutionPending(ExecutionCloseProgress::TerminalAckPending {
            unacknowledged: 1,
            worker: WorkerCloseState::Joined,
        });
    assert!(matches!(
        project_progress(
            Some(progress),
            ViewerNativeState::Unconfirmed,
            Duration::from_millis(5)
        ),
        CleanupPoll::Terminal(ViewerCleanupStatus::NeedsTerminalAck)
    ));
}
#[test]
fn worker_receipts_do_not_promote_unconfirmed_or_forced_native_return() {
    for native in [
        ViewerNativeState::NotRun,
        ViewerNativeState::Unconfirmed,
        ViewerNativeState::ForcedIncomplete,
    ] {
        let progress = ViewerCloseProgress::Complete {
            endpoint: LocalHostShutdownStatus {
                pipe: WorkerJoin::Joined,
                sdk: WorkerJoin::Joined,
                dispatch: None,
            },
            execution: ExecutionCloseReceipt {
                worker_stopped: true,
                unacknowledged: 0,
            },
        };
        assert!(
            matches!(project_progress(Some(progress), native, Duration::from_millis(5)),
            CleanupPoll::Terminal(ViewerCleanupStatus::WorkersQuiescent(observed)) if observed == native)
        );
    }
}
#[test]
fn missing_worker_observation_is_pending_and_invalid_budget_is_rejected() {
    assert!(
        matches!(project_progress(None, ViewerNativeState::Unconfirmed, Duration::from_millis(7)),
        CleanupPoll::Pending(HostRetry::After(delay)) if delay == Duration::from_millis(7))
    );
    assert!(ViewerCleanupLimits::new(Duration::ZERO, 1, Duration::from_millis(1)).is_err());
    assert!(ViewerCleanupLimits::new(Duration::from_secs(1), 0, Duration::from_millis(1)).is_err());
}
