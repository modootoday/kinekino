use super::*;
use crate::application::sdk_execution::host::{HostExecution, SubmissionIdentity};
struct Counting {
    calls: AtomicUsize,
}
impl ApplicationClient for Counting {
    fn execute(&self, request: Request) -> Response {
        self.calls.fetch_add(1, Ordering::SeqCst);
        crate::application::api::PlayerSdk::default().execute(request)
    }
}
fn terminal(
    host: &HostExecution,
    id: &crate::application::sdk_execution::host::HostOperationReference,
) {
    let cutoff = Instant::now() + Duration::from_secs(5);
    loop {
        if matches!(host.peek(id).unwrap(), ExecutionSnapshot::Terminal(_)) {
            break;
        }
        assert!(Instant::now() < cutoff);
        std::thread::yield_now();
    }
}
fn close(host: &HostExecution) {
    let cutoff = Instant::now() + Duration::from_secs(5);
    while !matches!(
        host.poll_close().unwrap(),
        ExecutionCloseProgress::Closed(_)
    ) {
        assert!(Instant::now() < cutoff);
        std::thread::yield_now();
    }
}
#[test]
fn submission_lookup_recovers_discarded_admission_without_reexecution_or_ack_reuse() {
    let client = Arc::new(Counting {
        calls: AtomicUsize::new(0),
    });
    let host = HostExecution::start(
        "f".repeat(32),
        client.clone(),
        budget(),
        Arc::new(Notify {
            signals: AtomicUsize::new(0),
            fail: false,
        }),
    )
    .unwrap();
    let identity = SubmissionIdentity {
        host_scope: host.host_scope().into(),
        submission_id: "original-client-request".into(),
    };
    assert_eq!(
        host.inspect_submission(&identity).unwrap_err().code,
        ErrorCode::NotFound
    );
    // Discard the first admission receipt, as a caller with a lost response would.
    host.submit_correlated(&identity, Request::new("version"))
        .unwrap();
    let found = host.inspect_submission(&identity).unwrap();
    assert!(!found.acknowledged);
    let retried = host
        .submit_correlated(&identity, Request::new("version"))
        .unwrap();
    assert!(retried.replayed);
    assert_eq!(retried.operation, found.operation);
    assert_eq!(
        host.submit_correlated(&identity, Request::new("capabilities"))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let mut foreign = identity.clone();
    foreign.host_scope = "0".repeat(32);
    assert_eq!(
        host.inspect_submission(&foreign).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    terminal(&host, &found.operation);
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    host.acknowledge(&found.operation).unwrap();
    assert!(host.inspect_submission(&identity).unwrap().acknowledged);
    let old = host
        .submit_correlated(&identity, Request::new("version"))
        .unwrap();
    assert!(old.replayed && old.acknowledged);
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        host.peek(&found.operation).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    close(&host);
}
#[test]
fn submission_record_budget_never_evicts_acknowledged_ids_into_new_execution() {
    let client = Arc::new(Counting {
        calls: AtomicUsize::new(0),
    });
    let host = HostExecution::start(
        "a".repeat(32),
        client.clone(),
        budget(),
        Arc::new(Notify {
            signals: AtomicUsize::new(0),
            fail: false,
        }),
    )
    .unwrap();
    for index in 0..128 {
        let identity = SubmissionIdentity {
            host_scope: host.host_scope().into(),
            submission_id: format!("bounded-{index}"),
        };
        let receipt = host
            .submit_correlated(&identity, Request::new("version"))
            .unwrap();
        terminal(&host, &receipt.operation);
        host.acknowledge(&receipt.operation).unwrap();
    }
    let extra = SubmissionIdentity {
        host_scope: host.host_scope().into(),
        submission_id: "beyond-budget".into(),
    };
    assert_eq!(
        host.submit_correlated(&extra, Request::new("version"))
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    let original = SubmissionIdentity {
        host_scope: host.host_scope().into(),
        submission_id: "bounded-0".into(),
    };
    assert!(
        host.submit_correlated(&original, Request::new("version"))
            .unwrap()
            .acknowledged
    );
    assert_eq!(client.calls.load(Ordering::SeqCst), 128);
    close(&host);
}
