use super::carrier::Readiness;
use super::*;
use kinekino_player::{
    application::api::{ApiError, ApplicationClient, ErrorCode, PlayerSdk, Request, Response},
    infrastructure::local_host::{EndpointClient, LocalHost, TransportOptions, WorkerJoin},
};
use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

struct Dependency(Arc<AtomicUsize>);
impl Drop for Dependency {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
struct Release(Arc<(Mutex<bool>, Condvar)>);
impl Release {
    fn open(&self) {
        let (lock, changed) = &*self.0;
        *lock.lock().unwrap() = true;
        changed.notify_all();
    }
}
impl Drop for Release {
    fn drop(&mut self) {
        self.open();
    }
}
struct Blocked {
    gate: Arc<(Mutex<bool>, Condvar)>,
    started: mpsc::Sender<()>,
}
impl ApplicationClient for Blocked {
    fn execute(&self, request: Request) -> Response {
        self.started.send(()).unwrap();
        let (lock, changed) = &*self.gate;
        drop(
            changed
                .wait_while(lock.lock().unwrap(), |released| !*released)
                .unwrap(),
        );
        PlayerSdk::default().execute(request)
    }
}
fn finish<D>(mut owner: ClosingHost<D>) -> ReadyForDependencies<D> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match advance(owner, deadline) {
            DriveOutcome::Ready(ready) => return ready,
            DriveOutcome::Pending(retained) | DriveOutcome::Partial(retained) => {
                assert!(!retained.budget_expired, "transport join watchdog");
                owner = retained.owner;
                std::thread::yield_now();
            }
            DriveOutcome::Panicked(_) => panic!("unexpected transport worker panic"),
        }
    }
}

#[test]
fn expired_budget_retains_blocked_owner_and_original_error_until_both_joins() {
    kinekino_platform::install();
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let release = Release(gate.clone());
    let (started, receiver) = mpsc::channel();
    let local = LocalHost::start(
        Arc::new(Blocked { gate, started }),
        TransportOptions::default(),
    )
    .unwrap();
    let client = EndpointClient::new(local.endpoint(), TransportOptions::default()).unwrap();
    let request = std::thread::spawn(move || client.execute(Request::new("version")));
    receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let mut error = ApiError::new(ErrorCode::EngineFailed, "original stdin read failure");
    error.retryable = true;
    error.details = serde_json::json!({"original":"kept"});
    error.remediation = "original remediation".into();
    error.request_id = "original-request".into();
    let expected = serde_json::to_value(&error).unwrap();
    let owner = ClosingHost::begin(local, Dependency(drops.clone()), Some(error));
    let expired = Instant::now();
    let retained = match advance(owner, expired) {
        DriveOutcome::Pending(retained) | DriveOutcome::Partial(retained) => retained,
        _ => panic!("blocked SDK cannot be ready or panicked"),
    };
    assert!(retained.budget_expired);
    assert_eq!(retained.transport.sdk, WorkerJoin::Pending);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(
        serde_json::to_value(retained.owner.first_error()).unwrap(),
        expected
    );
    let retained = match advance(retained.owner, expired) {
        DriveOutcome::Pending(retained) | DriveOutcome::Partial(retained) => retained,
        _ => panic!("repeated poll must keep original owner"),
    };
    assert!(retained.budget_expired);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    release.open();
    let (dependency, error, status) = finish(retained.owner).into_parts();
    assert!(status.is_quiescent());
    assert_eq!(serde_json::to_value(error).unwrap(), expected);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(dependency);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !request.is_finished() {
        assert!(Instant::now() < deadline, "client watchdog");
        std::thread::yield_now();
    }
    let _ = request.join().unwrap();
}

#[test]
fn eof_recovers_original_dependency_without_running_its_cleanup() {
    kinekino_platform::install();
    let local =
        LocalHost::start(Arc::new(PlayerSdk::default()), TransportOptions::default()).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let (dependency, error, status) =
        finish(ClosingHost::begin(local, Dependency(drops.clone()), None)).into_parts();
    assert!(error.is_none());
    assert!(status.is_quiescent());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(dependency);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn sdk_worker_panic_keeps_dependency_and_first_error_in_retained_outcome() {
    kinekino_platform::install();
    struct DropPanics;
    impl ApplicationClient for DropPanics {
        fn execute(&self, request: Request) -> Response {
            PlayerSdk::default().execute(request)
        }
    }
    impl Drop for DropPanics {
        fn drop(&mut self) {
            panic!("SDK owner-drop fixture");
        }
    }
    let local = LocalHost::start(Arc::new(DropPanics), TransportOptions::default()).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let error = ApiError::new(ErrorCode::PermissionDenied, "original ready flush failure");
    let mut owner = ClosingHost::begin(local, Dependency(drops.clone()), Some(error));
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match advance(owner, deadline) {
            DriveOutcome::Ready(_) => panic!("panic must not become ready"),
            DriveOutcome::Pending(retained) | DriveOutcome::Partial(retained) => {
                assert!(!retained.budget_expired);
                owner = retained.owner;
            }
            DriveOutcome::Panicked(retained) => {
                assert!(!retained.budget_expired, "panic join watchdog");
                assert_eq!(retained.transport.sdk, WorkerJoin::Panicked);
                assert_eq!(drops.load(Ordering::SeqCst), 0);
                assert_eq!(
                    retained.owner.first_error().unwrap().code,
                    ErrorCode::PermissionDenied
                );
                if retained.transport.pipe == WorkerJoin::Pending {
                    owner = retained.owner;
                    continue;
                }
                assert_eq!(retained.transport.pipe, WorkerJoin::Joined);
                // Explicit fixture disposal after both joins, including a failed join.
                // Production recovery/exit policy is not implemented here.
                drop(retained);
                assert_eq!(drops.load(Ordering::SeqCst), 1);
                break;
            }
        }
        std::thread::yield_now();
    }
}

#[test]
fn classification_requires_both_normal_joins_and_keeps_panic_distinct() {
    kinekino_platform::install();
    use super::driver::{Phase, phase};
    use WorkerJoin::{Joined, Panicked, Pending};
    use kinekino_player::infrastructure::local_host::LocalHostShutdownStatus;
    for (pipe, sdk, expected) in [
        (Pending, Pending, Phase::Pending),
        (Joined, Pending, Phase::Partial),
        (Pending, Joined, Phase::Partial),
        (Joined, Joined, Phase::Ready),
        (Panicked, Pending, Phase::Panicked),
        (Pending, Panicked, Phase::Panicked),
        (Joined, Panicked, Phase::Panicked),
        (Panicked, Joined, Phase::Panicked),
        (Panicked, Panicked, Phase::Panicked),
    ] {
        assert_eq!(
            phase(LocalHostShutdownStatus {
                pipe,
                sdk,
                dispatch: None
            }),
            expected
        );
    }
}

#[test]
fn forged_public_join_status_cannot_extract_dependency_before_original_sdk_join() {
    kinekino_platform::install();
    use kinekino_player::infrastructure::local_host::LocalHostShutdownStatus;
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let release = Release(gate.clone());
    let (started, receiver) = mpsc::channel();
    let local = LocalHost::start(
        Arc::new(Blocked { gate, started }),
        TransportOptions::default(),
    )
    .unwrap();
    let client = EndpointClient::new(local.endpoint(), TransportOptions::default()).unwrap();
    let request = std::thread::spawn(move || client.execute(Request::new("version")));
    receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let owner = ClosingHost::begin(local, Dependency(drops.clone()), None);
    let forged = LocalHostShutdownStatus {
        pipe: WorkerJoin::Joined,
        sdk: WorkerJoin::Joined,
        dispatch: None,
    };
    assert!(forged.is_quiescent());
    // Public status is forgeable; extraction accepts no caller-supplied status at all.
    let (owner, observed) = match owner.try_ready() {
        Readiness::Incomplete(owner, observed) => (owner, observed),
        Readiness::Ready(_) => panic!("forged status must not replace the original Pending join"),
    };
    assert_eq!(observed.sdk, WorkerJoin::Pending);
    assert_ne!(observed, forged);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    let (owner, observed) = match owner.try_ready() {
        Readiness::Incomplete(owner, observed) => (owner, observed),
        Readiness::Ready(_) => panic!("repeated extraction must return the same incomplete owner"),
    };
    assert_eq!(observed.sdk, WorkerJoin::Pending);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    release.open();
    let (dependency, _, status) = finish(owner).into_parts();
    assert!(status.is_quiescent());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(dependency);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !request.is_finished() {
        assert!(Instant::now() < deadline, "client watchdog");
        std::thread::yield_now();
    }
    let _ = request.join().unwrap();
}

struct InputProbe {
    fail: bool,
    reads: usize,
}
impl std::io::Read for InputProbe {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        self.reads += 1;
        if self.fail {
            Err(std::io::Error::other("original read error"))
        } else {
            Ok(0)
        }
    }
}
struct OutputProbe {
    fail_write: bool,
    fail_flush: bool,
    bytes: Vec<u8>,
}
impl std::io::Write for OutputProbe {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.fail_write {
            return Err(std::io::Error::other("original write error"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if self.fail_flush {
            Err(std::io::Error::other("original flush error"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn injected_eof_ready_write_flush_and_read_errors_recover_owner_only_after_joins() {
    kinekino_platform::install();
    for (fail_write, fail_flush, fail_read, expected_phase) in [
        (false, false, false, None),
        (true, false, false, Some(IoPhase::ReadyWrite)),
        (false, true, false, Some(IoPhase::ReadyFlush)),
        (false, false, true, Some(IoPhase::Read)),
    ] {
        let local =
            LocalHost::start(Arc::new(PlayerSdk::default()), TransportOptions::default()).unwrap();
        let drops = Arc::new(AtomicUsize::new(0));
        let closed = Arc::new(AtomicUsize::new(0));
        let close = closed.clone();
        let mut input = InputProbe {
            fail: fail_read,
            reads: 0,
        };
        let mut output = OutputProbe {
            fail_write,
            fail_flush,
            bytes: Vec::new(),
        };
        let result = serve_transport(
            local,
            Dependency(drops.clone()),
            &mut input,
            &mut output,
            &serde_json::json!({"event":"test_ready"}),
            |phase, error| {
                let mut original = ApiError::new(ErrorCode::PermissionDenied, error.to_string());
                original.retryable = true;
                original.details =
                    serde_json::json!({"phase":format!("{phase:?}"),"source":"original"});
                original.request_id = "original-io".into();
                original.remediation = "original remediation".into();
                original
            },
            move || {
                close.fetch_add(1, Ordering::SeqCst);
            },
        );
        let ready = match result {
            TransportOutcome::Ready(ready) => ready,
            TransportOutcome::Incomplete(_) => panic!("idle transport must join normally"),
        };
        let (dependency, error, status) = ready.into_parts();
        assert!(status.is_quiescent());
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert_eq!(closed.load(Ordering::SeqCst), 1);
        assert_eq!(input.reads, if fail_write || fail_flush { 0 } else { 1 });
        match expected_phase {
            None => assert!(error.is_none()),
            Some(phase) => {
                let error = error.unwrap();
                assert_eq!(error.code, ErrorCode::PermissionDenied);
                assert!(error.retryable);
                assert_eq!(
                    error.details,
                    serde_json::json!({"phase":format!("{phase:?}"),"source":"original"})
                );
                assert_eq!(error.request_id, "original-io");
                assert_eq!(error.remediation, "original remediation");
                assert!(error.message.starts_with("original"));
            }
        }
        assert!(!String::from_utf8(output.bytes).unwrap().contains("stopped"));
        drop(dependency);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn timeout_outcome_keeps_dependency_and_original_exit_code_without_success_event() {
    kinekino_platform::install();
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let release = Release(gate.clone());
    let (started, receiver) = mpsc::channel();
    let local = LocalHost::start(
        Arc::new(Blocked { gate, started }),
        TransportOptions::default(),
    )
    .unwrap();
    let client = EndpointClient::new(local.endpoint(), TransportOptions::default()).unwrap();
    let request = std::thread::spawn(move || client.execute(Request::new("version")));
    receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let error = ApiError::new(ErrorCode::PermissionDenied, "original ready failure");
    let outcome = super::driver::observe(
        ClosingHost::begin(local, Dependency(drops.clone()), Some(error)),
        Instant::now(),
    );
    let owner = match outcome {
        TransportOutcome::Incomplete(owner) => owner,
        TransportOutcome::Ready(_) => panic!("blocked SDK cannot be ready"),
    };
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(owner.exit_code(), 6);
    let diagnostic = owner.diagnostic("test-host");
    assert_eq!(diagnostic["reason"], "ObservationBudgetExpired");
    assert_eq!(diagnostic["cleanup_complete"], false);
    assert_eq!(diagnostic["sdk_cancellation_confirmed"], false);
    assert!(diagnostic.get("host_lifetime_ended").is_none());
    assert!(!diagnostic.to_string().contains("stopped"));
    release.open();
    let (dependency, error, status) = finish(owner.into_owner()).into_parts();
    assert!(status.is_quiescent());
    assert_eq!(error.unwrap().message, "original ready failure");
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(dependency);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !request.is_finished() {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    let _ = request.join().unwrap();
}

#[test]
fn panic_outcome_has_engine_failed_fallback_and_never_reports_stopped() {
    kinekino_platform::install();
    struct DropPanics;
    impl ApplicationClient for DropPanics {
        fn execute(&self, request: Request) -> Response {
            PlayerSdk::default().execute(request)
        }
    }
    impl Drop for DropPanics {
        fn drop(&mut self) {
            panic!("SDK drop fixture");
        }
    }
    let local = LocalHost::start(Arc::new(DropPanics), TransportOptions::default()).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let outcome = super::driver::observe(
        ClosingHost::begin(local, Dependency(drops.clone()), None),
        Instant::now() + Duration::from_secs(5),
    );
    let owner = match outcome {
        TransportOutcome::Incomplete(owner) => owner,
        TransportOutcome::Ready(_) => panic!("panic is not normal completion"),
    };
    assert_eq!(
        owner.exit_code(),
        ApiError::new(ErrorCode::EngineFailed, "fallback").exit_code()
    );
    let diagnostic = owner.diagnostic("test-host");
    assert_eq!(diagnostic["reason"], "WorkerPanicked");
    assert_eq!(diagnostic["cleanup_complete"], false);
    assert!(!diagnostic.to_string().contains("stopped"));
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    let mut closing = owner.into_owner();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match advance(closing, deadline) {
            DriveOutcome::Panicked(retained) if retained.transport.pipe == WorkerJoin::Joined => {
                // Explicit fixture disposal after both terminal joins, including failure.
                drop(retained);
                break;
            }
            DriveOutcome::Panicked(retained)
            | DriveOutcome::Partial(retained)
            | DriveOutcome::Pending(retained) => {
                assert!(!retained.budget_expired);
                closing = retained.owner;
            }
            DriveOutcome::Ready(_) => panic!("panic cannot become ready"),
        }
    }
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn dependency_failure_does_not_replace_first_error_details() {
    kinekino_platform::install();
    let mut first = ApiError::new(ErrorCode::PermissionDenied, "first");
    first.details = serde_json::json!({"original":true});
    let expected = serde_json::to_value(&first).unwrap();
    let result: Result<(), ApiError> = preserve_first_error(
        Some(first),
        Err(ApiError::new(ErrorCode::EngineFailed, "cleanup")),
    );
    assert_eq!(serde_json::to_value(result.unwrap_err()).unwrap(), expected);
}

#[test]
fn shutdown_dto_keeps_each_original_join_and_explicit_dispatch_poison() {
    kinekino_platform::install();
    use kinekino_player::infrastructure::local_host::{DispatchState, LocalHostShutdownStatus};
    let mixed = LocalHostShutdownStatus {
        pipe: WorkerJoin::Joined,
        sdk: WorkerJoin::Panicked,
        dispatch: None,
    };
    assert_eq!(
        serde_json::to_value(LocalHostShutdownView::from(mixed)).unwrap(),
        serde_json::json!({
            "pipe":"joined", "sdk":"panicked", "dispatch":null, "dispatch_poisoned":true
        })
    );
    let pending = LocalHostShutdownStatus {
        pipe: WorkerJoin::Pending,
        sdk: WorkerJoin::Pending,
        dispatch: Some(DispatchState::Running),
    };
    assert_eq!(
        serde_json::to_value(LocalHostShutdownView::from(pending)).unwrap(),
        serde_json::json!({
            "pipe":"pending", "sdk":"pending", "dispatch":"running", "dispatch_poisoned":false
        })
    );
    let returned = LocalHostShutdownStatus {
        pipe: WorkerJoin::Joined,
        sdk: WorkerJoin::Joined,
        dispatch: Some(DispatchState::Completed),
    };
    assert_eq!(
        serde_json::to_value(LocalHostShutdownView::from(returned)).unwrap(),
        serde_json::json!({
            "pipe":"joined", "sdk":"joined", "dispatch":"completed", "dispatch_poisoned":false
        })
    );
}

#[test]
fn broken_additional_diagnostic_keeps_first_error_fields_and_exit_code() {
    kinekino_platform::install();
    struct Broken {
        attempts: usize,
        offered: Vec<u8>,
    }
    impl std::io::Write for Broken {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.attempts += 1;
            self.offered.extend_from_slice(bytes);
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "fixture stderr failure",
            ))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut original = ApiError::new(ErrorCode::PermissionDenied, "original input error");
    original.retryable = true;
    original.details = serde_json::json!({"original":true});
    original.request_id = "original-request".into();
    original.remediation = "original remediation".into();
    let expected = serde_json::to_value(&original).unwrap();
    let expected_code = original.exit_code();
    let mut broken = Broken {
        attempts: 0,
        offered: Vec::new(),
    };
    let result: Result<(), ApiError> = super::driver::preserve_first_error_to(
        Some(original),
        Err(ApiError::new(
            ErrorCode::EngineFailed,
            "additional cleanup error",
        )),
        &mut broken,
    );
    let preserved = result.unwrap_err();
    assert!(broken.attempts > 0);
    assert_eq!(serde_json::to_value(&preserved).unwrap(), expected);
    assert_eq!(preserved.exit_code(), expected_code);
    assert_eq!(expected_code, 6);
    let offered = String::from_utf8(broken.offered).unwrap();
    assert!(!offered.contains("stopped"));
    assert!(!offered.contains("host_lifetime_ended"));
}
