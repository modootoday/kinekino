use super::*;
use crate::presentation::web_ui_adapter::proposals::pending_host::PendingOutput;
use kinekino_player::{
    application::sdk_execution::{
        CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner,
    },
    assembly::meari::{
        card_current_inspection::{CardCurrentInspection, CardInspectionInput},
        scoped_card_session::bind_scoped_card,
    },
};
use std::time::{Duration, Instant};

struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), kinekino_player::application::api::ApiError> {
        Ok(())
    }
}
fn execution(client: ApplicationClientHandle) -> ExecutionOwner {
    ExecutionOwner::start(
        client,
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 262144,
            max_total_request_bytes: 8 * 262144,
            max_terminal_bytes: 262144,
            max_total_terminal_bytes: 8 * 262144,
        },
        Arc::new(Notify),
    )
    .unwrap()
}
fn selection(input: &ScopedCardInput) -> BootstrapSelection {
    BootstrapSelection {
        proposal_id: input.selection.proposal_id.clone(),
        document_revision: input.selection.document_revision,
        source_id: input.selection.source_id.clone(),
        source_revision: input.selection.source_revision,
        source_sha256: input.selection.source_sha256.clone(),
        target: BootstrapTarget::Card { index: 1 },
    }
}
fn inspection_input(input: &ScopedCardInput, owner: &ExecutionOwner) -> CardInspectionInput {
    CardInspectionInput {
        execution: owner.handle(),
        scope: input.scope.clone(),
        registry: input.registry.clone(),
        context: Rc::new(RefCell::new(RequestContext::default())),
        namespace: "current-card".into(),
        limit: 8,
        codec: super::super::super::product_window::product_codec_budget(),
    }
}
fn poll(inspection: &mut CardCurrentInspection) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !inspection.poll().unwrap() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn join(owner: ExecutionOwner) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let ExecutionCloseProgress::Closed(receipt) = owner.poll_close().unwrap() {
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn original_execution_current_inspect_releases_before_scoped_card_binding() {
    let (_frame, input, _) = fixture();
    let owner = execution(input.client.clone());
    let mut inspection = CardCurrentInspection::new(inspection_input(&input, &owner))
        .unwrap_or_else(|_| panic!("Original execution must attach"));
    inspection.begin(selection(&input)).unwrap();
    let ticket = inspection.pending_ticket().unwrap().clone();
    assert!(inspection.begin(selection(&input)).is_err());
    assert_eq!(inspection.pending_ticket(), Some(&ticket));
    assert!(inspection.release().is_err());
    assert!(owner.handle().attach().is_err());
    poll(&mut inspection);
    let terminal = inspection.terminal().unwrap();
    assert_eq!(terminal.ticket, ticket);
    assert!(!terminal.cancellation_requested);
    let Ok(PendingOutput::Inspected(stored)) = &terminal.outcome else {
        panic!("Current typed inspect required")
    };
    let stored = stored.clone();
    inspection.release().unwrap();
    assert!(inspection.is_released());
    assert!(inspection.terminal().is_some());
    let next = owner.handle().attach().unwrap();
    next.release().unwrap();
    let mut panel = bind_scoped_card(
        input,
        stored,
        owner.handle(),
        super::super::super::product_window::product_codec_budget(),
    )
    .unwrap_or_else(|_| panic!("Same original execution must bind the inspected Card"));
    assert_eq!(
        panel.initial_props()["front"],
        Value::String("second".into())
    );
    assert!(owner.handle().attach().is_err());
    panel.unbind().unwrap();
    let next = owner.handle().attach().unwrap();
    next.release().unwrap();
    join(owner);
}

#[test]
fn unavailable_live_proposal_retains_canonical_error_without_capture_substitution() {
    let (_frame, input, _) = fixture();
    let owner = execution(input.client.clone());
    let mut inspection = CardCurrentInspection::new(inspection_input(&input, &owner))
        .unwrap_or_else(|_| panic!("Original execution must attach"));
    let mut selected = selection(&input);
    selected.proposal_id = "missing-live-proposal".into();
    inspection.begin(selected).unwrap();
    poll(&mut inspection);
    assert!(inspection.terminal().unwrap().outcome.is_err());
    assert_eq!(
        inspection.selection().unwrap().proposal_id,
        "missing-live-proposal"
    );
    inspection.release().unwrap();
    assert!(inspection.terminal().unwrap().outcome.is_err());
    join(owner);
}

#[test]
fn rejected_pending_consumer_retains_registration_until_explicit_unbind() {
    let (_frame, input, _) = fixture();
    let owner = execution(input.client.clone());
    let mut config = inspection_input(&input, &owner);
    config.namespace.clear();
    let mut failure = match CardCurrentInspection::new(config) {
        Err(failure) => failure,
        Ok(_) => panic!("Empty namespace must reject"),
    };
    assert!(failure.registration.is_some());
    assert!(owner.handle().attach().is_err());
    failure.retry_unbind().unwrap();
    let next = owner.handle().attach().unwrap();
    next.release().unwrap();
    join(owner);
}
