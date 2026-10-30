use super::*;
use crate::application::api::{ErrorCode, PlayerSdk};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

fn budget() -> ExecutionBudget {
    ExecutionBudget {
        max_operations: 8,
        max_request_bytes: 262144,
        max_total_request_bytes: 8 * 262144,
        max_terminal_bytes: 262144,
        max_total_terminal_bytes: 8 * 262144,
    }
}
struct CountNotifier(AtomicUsize);
impl CompletionNotifier for CountNotifier {
    fn notify(&self) -> Result<(), ApiError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn legacy_public_entry_is_the_same_function_and_failure_type() {
    use super::super::proposal_window_startup as legacy;
    type Start = fn(
        ApplicationClientHandle,
        ExecutionBudget,
        HostScheduleLimits,
    ) -> Result<PreparedNativeExecution, Box<NativeExecutionFailure>>;
    let common: Start = prepare_native_execution;
    let compatible: Start = legacy::prepare_proposal_execution;
    assert!(std::ptr::fn_addr_eq(common, compatible));
    fn retain_type(failure: Box<legacy::ProposalExecutionFailure>) -> Box<NativeExecutionFailure> {
        failure
    }
    let _ = retain_type;
}

#[test]
fn native_failure_retry_preserves_the_original_box_client_and_diagnostic() {
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let failure = Box::new(NativeExecutionFailure {
        cause: NativeExecutionStartError::Native(NativeStartupError::AlreadyIssued),
        loop_owner: None,
        startup: None,
        client: client.clone(),
        notifier: None,
    });
    let original = std::ptr::from_ref(failure.as_ref());
    let retained = match failure.retry_execution(budget()) {
        Err(retained) => retained,
        Ok(_) => panic!("native failure cannot restart execution"),
    };
    assert_eq!(original, std::ptr::from_ref(retained.as_ref()));
    assert!(Arc::ptr_eq(&client, &retained.client));
    assert!(matches!(
        retained.cause,
        NativeExecutionStartError::Native(NativeStartupError::AlreadyIssued)
    ));
}

#[test]
fn incomplete_sdk_failure_cannot_start_a_worker_or_replace_its_notifier() {
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let counted = Arc::new(CountNotifier(AtomicUsize::new(0)));
    let notifier: Arc<dyn CompletionNotifier> = counted.clone();
    let failure = Box::new(NativeExecutionFailure {
        cause: NativeExecutionStartError::Sdk(ApiError::new(
            ErrorCode::InvalidArgument,
            "original budget failure",
        )),
        loop_owner: None,
        startup: None,
        client: client.clone(),
        notifier: Some(notifier.clone()),
    });
    let original = std::ptr::from_ref(failure.as_ref());
    let retained = match failure.retry_execution(budget()) {
        Err(retained) => retained,
        Ok(_) => panic!("missing startup must not be reconstructed"),
    };
    assert_eq!(original, std::ptr::from_ref(retained.as_ref()));
    assert!(Arc::ptr_eq(&client, &retained.client));
    assert!(Arc::ptr_eq(&notifier, retained.notifier.as_ref().unwrap()));
    assert_eq!(counted.0.load(Ordering::SeqCst), 0);
}

#[test]
#[ignore = "Requires an explicit Native-loop authority slot; creates no OS view"]
fn sdk_budget_retry_reuses_issued_startup_and_wraps_notifier_only_once() {
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let limits =
        HostScheduleLimits::new(Duration::from_millis(2), Duration::from_millis(250), 100).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let wrapped = calls.clone();
    let mut invalid = budget();
    invalid.max_operations = 0;
    let failure = prepare_native_execution_with_notifier_wrapper(
        client.clone(),
        invalid,
        limits,
        move |notifier| {
            wrapped.fetch_add(1, Ordering::SeqCst);
            notifier
        },
    )
    .err()
    .expect("SDK budget must fail");
    let stamp = failure.startup.as_ref().unwrap().stamp();
    let notifier = failure.notifier.as_ref().unwrap().clone();
    let retained = failure
        .retry_execution(invalid)
        .err()
        .expect("same budget remains invalid");
    assert_eq!(retained.startup.as_ref().unwrap().stamp(), stamp);
    assert!(Arc::ptr_eq(&notifier, retained.notifier.as_ref().unwrap()));
    assert!(Arc::ptr_eq(&client, &retained.client));
    let (owner, startup, execution) = retained.retry_execution(budget()).unwrap().into_parts();
    assert_eq!(startup.stamp(), stamp);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    let closed = loop {
        match execution.poll_close().unwrap() {
            crate::application::sdk_execution::ExecutionCloseProgress::Closed(receipt) => {
                break receipt;
            }
            pending => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "worker close remained unobserved: {pending:?}"
                );
                std::thread::sleep(Duration::from_millis(2));
            }
        }
    };
    assert!(closed.worker_stopped);
    assert_eq!(closed.unacknowledged, 0);
    drop((owner, startup, execution));
}
