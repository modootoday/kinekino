use super::*;
struct Recovering {
    rounds: u8,
    exhaust: bool,
    entered: mpsc::Sender<u8>,
    advance: Mutex<mpsc::Receiver<()>>,
}
impl ApplicationClient for Recovering {
    fn execute(&self, _: Request) -> Response {
        panic!("recovery hook required")
    }
    fn execute_recoverable(
        &self,
        request: Request,
        _: &(dyn crate::application::review_source::MutationControl + Sync),
        recovery: &dyn ReceiptRecoveryControl,
    ) -> Response {
        for expected in 1..=self.rounds {
            let generation = recovery.receipt_failed().unwrap();
            assert_eq!(generation, expected);
            assert!(
                !recovery.take_retry(generation).unwrap(),
                "no implicit recovery"
            );
            self.entered.send(generation).unwrap();
            self.advance
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(3))
                .unwrap();
            assert!(recovery.take_retry(generation).unwrap());
            assert!(
                !recovery.take_retry(generation).unwrap(),
                "one consumption per request"
            );
        }
        if self.exhaust {
            assert_eq!(
                recovery.receipt_failed().unwrap_err().code,
                ErrorCode::ResourceExhausted
            );
            assert_eq!(
                recovery.receipt_failed().unwrap_err().code,
                ErrorCode::ResourceExhausted
            );
            self.entered.send(0).unwrap();
            self.advance
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(3))
                .unwrap();
            assert!(!recovery.take_retry(self.rounds).unwrap());
        }
        Response::failure(
            request.context.request_id,
            ApiError::new(ErrorCode::EngineFailed, "original receipt failure"),
        )
    }
}
#[test]
fn original_receipt_recovery_rejects_foreign_stale_duplicate_and_unattached_authority() {
    let (entered, receive) = mpsc::channel();
    let (advance, gate) = mpsc::channel();
    let owner = ExecutionOwner::start(
        Arc::new(Recovering {
            rounds: 2,
            exhaust: false,
            entered,
            advance: Mutex::new(gate),
        }),
        budget(),
        Arc::new(Notify {
            signals: AtomicUsize::new(0),
            fail: false,
        }),
    )
    .unwrap();
    let handle = owner.handle();
    let attached = handle.attach().unwrap();
    let id = attached
        .submit(Request::new("production.capture_frame"))
        .unwrap();
    assert_eq!(receive.recv_timeout(Duration::from_secs(3)).unwrap(), 1);
    let first = attached.receipt_recovery(&id).unwrap().unwrap();
    assert_eq!(first.generation(), 1);
    assert!(!first.retry_requested);
    assert!(!first.retry_consumed);
    assert_eq!(
        handle.receipt_recovery(&id).unwrap_err().code,
        ErrorCode::PermissionDenied
    );
    assert_eq!(
        handle.retry_receipt(&id, &first.token).unwrap_err().code,
        ErrorCode::PermissionDenied
    );
    let foreign = OperationId {
        owner: Arc::new(()),
        sequence: id.sequence,
    };
    assert_eq!(
        attached
            .retry_receipt(&foreign, &first.token)
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
    let queued = attached.submit(Request::new("version")).unwrap();
    assert_eq!(
        attached
            .retry_receipt(&queued, &first.token)
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        attached.submit(Request::new("version")).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    assert_eq!(
        attached.retry_receipt(&id, &first.token).unwrap(),
        CleanupRetryDisposition::Requested
    );
    assert_eq!(
        attached.retry_receipt(&id, &first.token).unwrap(),
        CleanupRetryDisposition::AlreadyRequested
    );
    advance.send(()).unwrap();
    assert_eq!(receive.recv_timeout(Duration::from_secs(3)).unwrap(), 2);
    assert_eq!(
        attached.retry_receipt(&id, &first.token).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    let second = attached.receipt_recovery(&id).unwrap().unwrap();
    assert_eq!(second.generation(), 2);
    assert!(!second.retry_requested);
    assert_eq!(
        attached.cancel(&queued).unwrap(),
        CancelDisposition::QueuedCancelled
    );
    attached.acknowledge(&queued).unwrap();
    assert!(
        matches!(
            attached.poll_close().unwrap(),
            ExecutionCloseProgress::TerminalAckPending {
                worker: WorkerCloseState::ExitPending { .. },
                ..
            }
        ),
        "pending recovery keeps original worker alive"
    );
    assert_eq!(
        attached.retry_receipt(&id, &second.token).unwrap(),
        CleanupRetryDisposition::Requested
    );
    advance.send(()).unwrap();
    let cutoff = Instant::now() + Duration::from_secs(3);
    let done = loop {
        if let ExecutionSnapshot::Terminal(done) = attached.peek(&id).unwrap() {
            break done;
        }
        assert!(Instant::now() < cutoff);
        std::thread::sleep(Duration::from_millis(1));
    };
    assert_eq!(
        done.response.error.as_ref().unwrap().message,
        "original receipt failure"
    );
    assert!(attached.receipt_recovery(&id).unwrap().is_none());
    assert_eq!(
        attached.retry_receipt(&id, &second.token).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    attached.acknowledge(&id).unwrap();
    assert_eq!(
        attached.retry_receipt(&id, &second.token).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    attached.release().unwrap();
    close(&handle);
}

#[test]
fn receipt_recovery_sixteen_generations_exhaust_without_implicit_retry_or_false_terminal() {
    let (entered, receive) = mpsc::channel();
    let (advance, gate) = mpsc::channel();
    let owner = ExecutionOwner::start(
        Arc::new(Recovering {
            rounds: 16,
            exhaust: true,
            entered,
            advance: Mutex::new(gate),
        }),
        budget(),
        Arc::new(Notify {
            signals: AtomicUsize::new(0),
            fail: false,
        }),
    )
    .unwrap();
    let handle = owner.handle();
    let id = handle
        .submit(Request::new("production.capture_frame"))
        .unwrap();
    let mut previous = None;
    for expected in 1..=16 {
        assert_eq!(
            receive.recv_timeout(Duration::from_secs(3)).unwrap(),
            expected
        );
        let pending = handle.receipt_recovery(&id).unwrap().unwrap();
        assert_eq!(pending.generation(), expected);
        assert!(!pending.attempt_budget_exhausted);
        assert!(!pending.retry_requested);
        if let Some(token) = previous {
            assert_eq!(
                handle.retry_receipt(&id, &token).unwrap_err().code,
                ErrorCode::SourceExpired
            );
        }
        assert_eq!(
            handle.retry_receipt(&id, &pending.token).unwrap(),
            CleanupRetryDisposition::Requested
        );
        assert_eq!(
            handle.retry_receipt(&id, &pending.token).unwrap(),
            CleanupRetryDisposition::AlreadyRequested
        );
        previous = Some(pending.token);
        advance.send(()).unwrap();
    }
    assert_eq!(receive.recv_timeout(Duration::from_secs(3)).unwrap(), 0);
    let exhausted = handle.receipt_recovery(&id).unwrap().unwrap();
    assert_eq!(exhausted.generation(), 16);
    assert!(exhausted.attempt_budget_exhausted);
    assert!(exhausted.retry_consumed);
    for _ in 0..20 {
        assert_eq!(
            handle
                .retry_receipt(&id, &exhausted.token)
                .unwrap_err()
                .code,
            ErrorCode::ResourceExhausted
        );
        assert!(matches!(
            handle.peek(&id).unwrap(),
            ExecutionSnapshot::Running { .. }
        ));
    }
    assert!(matches!(
        owner.poll_close().unwrap(),
        ExecutionCloseProgress::TerminalAckPending {
            worker: WorkerCloseState::ExitPending { .. },
            ..
        }
    ));
    // Only the synthetic client is released; this is not a native owner drop or
    // a product recovery after exhaustion. The real adapter remains pending.
    advance.send(()).unwrap();
    assert_eq!(
        terminal(&handle, &id)
            .response
            .error
            .as_ref()
            .unwrap()
            .message,
        "original receipt failure"
    );
    handle.acknowledge(&id).unwrap();
    close(&handle);
}
