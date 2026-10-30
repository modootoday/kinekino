use super::*;
use crate::{
    application::{
        api::{ApplicationClient, ErrorCode, Request, RequestContext, Response},
        sdk_execution::*,
    },
    presentation::web_ui_adapter::proposals::product_window::*,
};
use meari_native::{domain::Viewport, infrastructure::native_host::HostScheduleLimits};
use std::sync::{Arc, Barrier};

struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct Blocked {
    entered: std::sync::mpsc::Sender<()>,
    release: Arc<Barrier>,
}
impl ApplicationClient for Blocked {
    fn execute(&self, request: Request) -> Response {
        self.entered.send(()).unwrap();
        self.release.wait();
        Response::failure(
            request.context.request_id,
            ApiError::new(ErrorCode::NeedsInput, "retained technical terminal"),
        )
    }
}

#[test]
fn bounded_failure_retains_same_owner_raw_terminal_and_original_cause_without_auto_ack() {
    let (entered, observed) = std::sync::mpsc::channel();
    let release = Arc::new(Barrier::new(2));
    let owner = ExecutionOwner::start(
        Arc::new(Blocked {
            entered,
            release: release.clone(),
        }),
        ExecutionBudget {
            max_operations: 2,
            max_request_bytes: 4096,
            max_total_request_bytes: 8192,
            max_terminal_bytes: 4096,
            max_total_terminal_bytes: 8192,
        },
        Arc::new(Notify),
    )
    .unwrap();
    let handle = owner.handle();
    let operation = handle
        .submit(Request {
            context: RequestContext::default(),
            command: "capabilities".into(),
            payload: serde_json::json!({}),
        })
        .unwrap();
    observed.recv_timeout(Duration::from_secs(2)).unwrap();
    let request = serde_json::from_value(serde_json::json!({"schema_version":"1","context":RequestContext::default(),"selection":{"proposal_id":"p","document_revision":"1","source_id":"s","source_revision":"1","source_sha256":"a".repeat(64),"target":{"kind":"card","index":1}}})).unwrap();
    let config = ProductWindowConfig {
        labels: ProductWindowLabels::korean(),
        visuals: ProductWindowVisuals {
            css: String::new(),
            assets: None,
        },
        viewport: Viewport {
            width: 1180,
            height: 760,
            dark: true,
        },
        limits: HostScheduleLimits::new(Duration::from_millis(1), Duration::from_millis(10), 10)
            .unwrap(),
        worker_exit_delay: Duration::from_millis(2),
        observer: Box::new(|_| Ok(())),
    };
    let owners = ProductWindowOwners::BuildFailure(Box::new(ProductWindowBuildFailure::Input(
        Box::new(ProductInputFailure {
            error: ApiError::new(ErrorCode::InvalidArgument, "original input failure"),
            owner,
            request,
            config,
        }),
    )));
    let mut bounded = OwnedProductCleanup::new(
        owners,
        CleanupLimits::new(Duration::from_secs(1), 1, Duration::from_millis(2)).unwrap(),
    );
    assert_eq!(bounded.poll(), CleanupStatus::NeedsTerminalAcceptance);
    assert_eq!(bounded.poll(), CleanupStatus::BudgetExhausted);
    let (owners, evidence) = bounded.into_parts();
    assert_eq!(evidence.status, Some(CleanupStatus::BudgetExhausted));
    assert!(matches!(
        handle.peek(&operation).unwrap(),
        ExecutionSnapshot::Running { .. }
    ));
    release.wait();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    let terminal = loop {
        if let ExecutionSnapshot::Terminal(terminal) = handle.peek(&operation).unwrap() {
            break terminal;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    };
    let outcome = finish_owned_product(owners);
    assert_eq!(outcome.status, CleanupStatus::Faulted);
    assert_eq!(
        outcome.acceptance_error.as_ref().unwrap().code,
        ErrorCode::NeedsInput
    );
    let ExecutionSnapshot::Terminal(after) = handle.peek(&operation).unwrap() else {
        panic!("no automatic ACK");
    };
    assert!(Arc::ptr_eq(&terminal, &after));
    let ProductWindowOwners::BuildFailure(failure) = &outcome.owners else {
        panic!("same carrier");
    };
    let ProductWindowBuildFailure::Input(failure) = failure.as_ref() else {
        panic!("same input owner");
    };
    assert_eq!(failure.error.message, "original input failure");
    handle.acknowledge(&operation).unwrap();
    let outcome = finish_owned_product(outcome.owners);
    assert_eq!(
        outcome.status,
        CleanupStatus::SdkQuiescent {
            native: NativeCleanupState::NoDocument
        }
    );
}
