use super::*;
use crate::application::proposals::InspectProposal;

#[test]
fn initial_queued_cancel_requires_typed_acceptance_and_preserves_evidence_after_ack() {
    let (owner, client, entered) = setup(0, false);
    let handle = owner.handle();
    let running = handle.submit(Request::new("proposals.inspect")).unwrap();
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    let mut initial = InitialProposalInspection::start(
        handle.clone(),
        crate::application::api::RequestContext::default(),
        InspectProposal {
            proposal_id: "explicit-initial".into(),
        },
        4096,
    )
    .unwrap();
    assert_eq!(
        initial.acknowledge().unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        initial.cancel().unwrap(),
        CancelDisposition::QueuedCancelled
    );
    let raw = initial.retained_terminal().unwrap().unwrap();
    let typed = initial.peek_terminal().unwrap().unwrap();
    let error = typed.as_ref().as_ref().unwrap_err();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(error.details["execution_started"], false);
    assert!(!initial.acknowledge().unwrap().already_acknowledged);
    assert!(initial.acknowledge().unwrap().already_acknowledged);
    assert_eq!(
        initial.cancel().unwrap(),
        CancelDisposition::AlreadyAcknowledged
    );
    assert!(Arc::ptr_eq(
        &typed,
        &initial.peek_terminal().unwrap().unwrap()
    ));
    assert!(Arc::ptr_eq(
        &raw,
        &initial.retained_terminal().unwrap().unwrap()
    ));
    release(&client);
    terminal(&handle, &running);
    handle.acknowledge(&running).unwrap();
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    close(&handle);
}

struct InitialFailure {
    wrong_correlation: bool,
}
impl ApplicationClient for InitialFailure {
    fn execute(&self, request: Request) -> Response {
        Response::failure(
            if self.wrong_correlation {
                "foreign".into()
            } else {
                request.context.request_id
            },
            ApiError::new(
                ErrorCode::NotFound,
                "Explicit initial proposal is unavailable",
            ),
        )
    }
}

#[test]
fn initial_business_failure_is_accepted_but_wrong_envelope_never_becomes_ackable() {
    for wrong_correlation in [false, true] {
        let owner = ExecutionOwner::start(
            Arc::new(InitialFailure { wrong_correlation }),
            budget(),
            Arc::new(Notify {
                signals: AtomicUsize::new(0),
                fail: false,
            }),
        )
        .unwrap();
        let handle = owner.handle();
        let mut initial = InitialProposalInspection::start(
            handle.clone(),
            crate::application::api::RequestContext::default(),
            InspectProposal {
                proposal_id: "explicit-initial".into(),
            },
            4096,
        )
        .unwrap();
        let until = Instant::now() + Duration::from_secs(3);
        while initial.retained_terminal().unwrap().is_none() {
            assert!(Instant::now() < until);
            std::thread::sleep(Duration::from_millis(1));
        }
        let raw = initial.retained_terminal().unwrap().unwrap();
        if wrong_correlation {
            assert!(initial.peek_terminal().is_err());
            assert_eq!(
                initial.acknowledge().unwrap_err().code,
                ErrorCode::NeedsInput
            );
            assert_eq!(handle.retained_operations().unwrap().len(), 1);
            // Explicit trusted test cleanup is not an initial acceptance or normal host close.
            let operation = handle.retained_operations().unwrap().pop().unwrap();
            handle.acknowledge(&operation).unwrap();
        } else {
            let typed = initial.peek_terminal().unwrap().unwrap();
            assert_eq!(
                typed.as_ref().as_ref().unwrap_err().code,
                ErrorCode::NotFound
            );
            initial.acknowledge().unwrap();
            assert!(Arc::ptr_eq(
                &typed,
                &initial.peek_terminal().unwrap().unwrap()
            ));
        }
        assert!(Arc::ptr_eq(
            &raw,
            &initial.retained_terminal().unwrap().unwrap()
        ));
        close(&handle);
    }
}
