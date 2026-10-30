use super::*;

struct NoWork;
impl ApplicationClient for NoWork {
    fn execute(&self, _: Request) -> Response {
        panic!("idle shutdown must not execute SDK work")
    }
}
struct WakeBarrier {
    entered: mpsc::Sender<()>,
    released: (Mutex<bool>, Condvar),
    calls: AtomicUsize,
}
impl CompletionNotifier for WakeBarrier {
    fn notify(&self) -> Result<(), ApiError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.entered.send(()).unwrap();
        let mut released = self.released.0.lock().unwrap();
        while !*released {
            released = self.released.1.wait(released).unwrap();
        }
        Ok(())
    }
}
impl WakeBarrier {
    fn release(&self) {
        *self.released.0.lock().unwrap() = true;
        self.released.1.notify_all();
    }
}
struct ReleaseOnDrop(Arc<WakeBarrier>);
impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        self.0.release();
    }
}
fn idle_barrier() -> (
    ExecutionOwner,
    Arc<WakeBarrier>,
    mpsc::Receiver<()>,
    ReleaseOnDrop,
) {
    let (entered, receiver) = mpsc::channel();
    let barrier = Arc::new(WakeBarrier {
        entered,
        released: (Mutex::new(false), Condvar::new()),
        calls: AtomicUsize::new(0),
    });
    let owner = ExecutionOwner::start(Arc::new(NoWork), budget(), barrier.clone()).unwrap();
    (owner, barrier.clone(), receiver, ReleaseOnDrop(barrier))
}

#[test]
fn final_wake_before_thread_return_requires_another_close_poll() {
    let (owner, barrier, entered, _release) = idle_barrier();
    assert_eq!(owner.close().unwrap_err().code, ErrorCode::NeedsInput);
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    let before_return = owner.close().unwrap_err();
    assert_eq!(
        before_return.details,
        serde_json::json!({
            "worker_returned":true,"worker_stopped":false,"unacknowledged":0,"closing":true
        })
    );
    assert_eq!(barrier.calls.load(Ordering::SeqCst), 1);
    barrier.release();
    close(&owner.handle());
    assert_eq!(barrier.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn typed_close_reports_final_notifier_as_exit_pending_without_extra_wakes() {
    let (owner, barrier, entered, _release) = idle_barrier();
    assert!(matches!(
        owner.poll_close().unwrap(),
        ExecutionCloseProgress::WorkerExitPending { .. }
    ));
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(matches!(
        owner.poll_close().unwrap(),
        ExecutionCloseProgress::WorkerExitPending {
            loop_finished: true
        }
    ));
    assert_eq!(barrier.calls.load(Ordering::SeqCst), 1);
    barrier.release();
    close(&owner.handle());
    assert!(matches!(
        owner.poll_close().unwrap(),
        ExecutionCloseProgress::Closed(ExecutionCloseReceipt {
            worker_stopped: true,
            unacknowledged: 0
        })
    ));
    assert_eq!(barrier.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        owner.handle().shared.join_attempts.load(Ordering::SeqCst),
        1
    );
}

#[test]
fn joined_worker_does_not_ack_a_retained_terminal_and_legacy_details_stay_stable() {
    let (owner, client, entered) = setup(0, false);
    let handle = owner.handle();
    let id = handle.submit(Request::new("proposals.inspect")).unwrap();
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    release(&client);
    let raw = terminal(&handle, &id);
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        match handle.poll_close().unwrap() {
            ExecutionCloseProgress::TerminalAckPending {
                unacknowledged: 1,
                worker: WorkerCloseState::Joined,
            } => break,
            ExecutionCloseProgress::TerminalAckPending {
                unacknowledged: 1, ..
            } => {}
            other => panic!("expected retained terminal, got {other:?}"),
        }
        assert!(Instant::now() < until);
        std::thread::yield_now();
    }
    let legacy = handle.close().unwrap_err();
    assert_eq!(legacy.code, ErrorCode::NeedsInput);
    assert_eq!(
        legacy.details,
        serde_json::json!({"worker_stopped":true,"worker_returned":true,"unacknowledged":1,"closing":true})
    );
    assert!(Arc::ptr_eq(&raw, &terminal(&handle, &id)));
    handle.acknowledge(&id).unwrap();
    assert!(matches!(
        handle.poll_close().unwrap(),
        ExecutionCloseProgress::Closed(_)
    ));
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn fault_progress_repeats_original_diagnostic_and_preserves_raw_until_ack() {
    let (owner, client, entered) = setup(8192, false);
    let handle = owner.handle();
    let id = handle.submit(Request::new("proposals.inspect")).unwrap();
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    release(&client);
    let raw = terminal(&handle, &id);
    let ExecutionCloseProgress::Faulted {
        error,
        unacknowledged: 1,
        ..
    } = handle.poll_close().unwrap()
    else {
        panic!("fault must be typed")
    };
    let ExecutionCloseProgress::Faulted {
        error: repeated, ..
    } = handle.poll_close().unwrap()
    else {
        panic!("fault must remain observable")
    };
    assert!(Arc::ptr_eq(&error, &repeated));
    assert!(error.message.contains("encoding"));
    assert!(Arc::ptr_eq(&raw.response, &terminal(&handle, &id).response));
    assert_eq!(handle.close().unwrap_err().code, ErrorCode::NeedsInput);
    handle.acknowledge(&id).unwrap();
    close(&handle);
    assert!(matches!(
        handle.poll_close().unwrap(),
        ExecutionCloseProgress::Closed(_)
    ));
}

#[test]
fn concurrent_typed_close_joins_once_and_active_attachment_denies_close() {
    let (owner, barrier, entered, _release) = idle_barrier();
    let handle = owner.handle();
    let attachment = handle.attach().unwrap();
    assert_eq!(
        handle.poll_close().unwrap_err().code,
        ErrorCode::PermissionDenied
    );
    assert!(!handle.shared.lock().unwrap().closing);
    attachment.release().unwrap();
    owner.poll_close().unwrap();
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    barrier.release();
    let until = Instant::now() + Duration::from_secs(3);
    while !handle
        .shared
        .worker
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .is_finished()
    {
        assert!(Instant::now() < until);
        std::thread::yield_now();
    }
    let ready = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        let mut callers = Vec::new();
        for _ in 0..8 {
            let handle = &handle;
            let ready = &ready;
            callers.push(scope.spawn(move || {
                ready.wait();
                assert!(matches!(
                    handle.poll_close().unwrap(),
                    ExecutionCloseProgress::Closed(_)
                ));
            }));
        }
        for caller in callers {
            caller.join().unwrap();
        }
    });
    assert_eq!(handle.shared.join_attempts.load(Ordering::SeqCst), 1);
    assert!(handle.shared.worker.lock().unwrap().is_none());
}

#[test]
fn actual_join_panic_stays_faulted_after_handle_is_taken() {
    let (owner, _client, _entered) = setup(0, false);
    let handle = owner.handle();
    close(&handle);
    // Replace only an already closed test owner's empty worker slot with an owned failing thread.
    let failing = std::thread::spawn(|| panic!("controlled worker join failure"));
    let until = Instant::now() + Duration::from_secs(3);
    while !failing.is_finished() {
        assert!(Instant::now() < until);
        std::thread::yield_now();
    }
    *handle.shared.worker.lock().unwrap() = Some(failing);
    handle.shared.lock().unwrap().worker_joined = false;
    let ExecutionCloseProgress::Faulted { error, .. } = handle.poll_close().unwrap() else {
        panic!("join panic must remain a fault")
    };
    for _ in 0..3 {
        let ExecutionCloseProgress::Faulted {
            error: repeated, ..
        } = handle.poll_close().unwrap()
        else {
            panic!("taken handle must not erase failure")
        };
        assert!(Arc::ptr_eq(&error, &repeated));
        let legacy = handle.close().unwrap_err();
        assert_eq!(
            serde_json::to_value(legacy).unwrap(),
            serde_json::to_value(error.as_ref()).unwrap()
        );
    }
    assert_eq!(handle.shared.join_attempts.load(Ordering::SeqCst), 2);
    assert!(handle.shared.worker.lock().unwrap().is_none());
}
