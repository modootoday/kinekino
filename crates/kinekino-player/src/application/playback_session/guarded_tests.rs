use super::*;
struct UnsupportedEngine;

#[test]
fn original_decorated_and_cloned_ticket_share_admission_receipt_and_one_read_context() {
    let submission =
        guarded::test_submission(crate::application::playback_control::ControlReceipt::enqueued(7));
    let receipt = submission.receipt().clone();
    let original = submission.ticket;
    let decorated = original.clone();
    let before = original.clone();
    assert!(original.read_context().is_none());
    decorated.bind_read_context(Arc::new(42u64)).unwrap();
    let cloned = decorated.clone();
    for ticket in [&original, &decorated, &before, &cloned] {
        assert!(original.same_request(ticket));
        assert_eq!(ticket.identity(), original.identity());
        assert_eq!(
            ticket.read_context().unwrap().downcast_ref::<u64>(),
            Some(&42)
        );
        let reread = GuardedSeekSubmission {
            ticket: ticket.clone(),
        };
        assert_eq!(
            serde_json::to_value(reread.receipt()).unwrap(),
            serde_json::to_value(&receipt).unwrap()
        );
    }
    assert_eq!(
        before
            .bind_read_context(Arc::new("foreign"))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        cloned.bind_read_context(Arc::new(42u64)).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        original.read_context().unwrap().downcast_ref::<u64>(),
        Some(&42)
    );
    assert_eq!(
        original.read_original(false).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        original.read_in_context(&7u64, false).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
}
impl EpisodePlaybackEngine for UnsupportedEngine {
    fn open(
        &self,
        _: EpisodePlaybackSource,
        _: bool,
        _: f64,
    ) -> Result<OpenedEpisodePlayback, ApiError> {
        panic!("legacy open fallback")
    }
    fn close(&self, _: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
        panic!("legacy close fallback")
    }
    fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
        panic!("legacy observation fallback")
    }
    fn control(
        &self,
        _: ControlEpisodePlayback,
    ) -> Result<crate::application::playback_control::ControlReceipt, ApiError> {
        panic!("legacy writer fallback")
    }
}
#[test]
fn default_guarded_methods_are_object_safe_unsupported_without_legacy_fallback() {
    let engine: &dyn EpisodePlaybackEngine = &UnsupportedEngine;
    let (seek, query) = guarded::test_inputs();
    assert_eq!(
        engine.observe_native_guard().unwrap_err().code,
        ErrorCode::Unsupported
    );
    assert_eq!(
        engine.submit_guarded_seek(seek).unwrap_err().code,
        ErrorCode::Unsupported
    );
    assert_eq!(
        engine.guarded_seek_result(query).unwrap_err().code,
        ErrorCode::Unsupported
    );
}

#[test]
fn public_diagnostics_cannot_mint_guarded_parts() {
    let mut observation = guarded::test_observation(Default::default());
    observation.guard = None;
    observation.observation_authorized = true;
    observation.native.connected = true;
    observation.native.seekable.value = Some(true);
    assert_eq!(
        observation.into_guarded_parts().unwrap_err().code,
        ErrorCode::NeedsInput
    );
}

#[test]
fn splitting_consumes_only_private_same_view_despite_mutated_public_binding() {
    use crate::application::native_playback::NativePlayback;
    let mut observation = guarded::test_observation(NativePlayback {
        time: Some(3.75),
        duration: Some(10.),
        connected: true,
        pause_observed: true,
        idle_observed: true,
        ..Default::default()
    });
    observation.engine_instance_id = crate::domain::api_values::DecimalU64(999);
    observation.resolution_artifact_id = "foreign".into();
    observation.observation_authorized = false;
    observation.native.position_seconds.value = Some(9999.);
    observation.native.connection_generation = u64::MAX;
    let (view, guard) = observation.into_guarded_parts().unwrap();
    assert_eq!(view.identity().engine_instance_id().0, 7);
    assert_eq!(view.identity().resolution_artifact_id(), "a".repeat(64));
    assert_eq!(view.playback().position_us.unwrap().0, 3_750_000);
    assert_eq!(
        view.seekability(),
        crate::domain::skip_segments::Seekability::Yes
    );
    assert!(view.observation_authorized());
    let _move_only_request = GuardedSeek {
        guard,
        position_us: crate::domain::api_values::DecimalU64(8_000_000),
    };
}

#[test]
fn submission_borrows_original_shared_receipt_and_returns_it_without_completion_promotion() {
    use crate::application::playback_control::{ControlCompletion, ControlReceipt};
    let original = ControlReceipt::enqueued(7);
    let expected = serde_json::to_value(&original).unwrap();
    let submission = guarded::test_submission(original);
    let cloned = GuardedSeekSubmission {
        ticket: submission.ticket.clone(),
    };
    assert!(std::ptr::eq(submission.receipt(), cloned.receipt()));
    assert!(submission.ticket.same_request(&cloned.ticket));
    let (receipt, ticket) = submission.into_parts();
    assert_eq!(serde_json::to_value(&receipt).unwrap(), expected);
    assert_eq!(receipt.completion, ControlCompletion::Unobserved);
    assert!(ticket.same_request(&cloned.ticket));
    assert_eq!(
        ticket.identity().engine_instance_id(),
        receipt.engine_instance_id
    );
}

#[test]
fn read_ticket_clone_keeps_original_query_right_after_default_rejection() {
    use crate::application::playback_control::ControlReceipt;
    let ticket = guarded::test_submission(ControlReceipt::enqueued(7)).ticket;
    let query = GuardedSeekResultQuery {
        ticket: ticket.clone(),
        consume_terminal: true,
    };
    assert_eq!(
        UnsupportedEngine
            .guarded_seek_result(query)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    let next = GuardedSeekResultQuery {
        ticket: ticket.clone(),
        consume_terminal: false,
    };
    assert!(ticket.same_request(&next.ticket));
    let other = guarded::test_submission(ControlReceipt::enqueued(7)).ticket;
    assert!(!ticket.same_request(&other));
    // No registry/pending-consume semantics are simulated here; those require stage B.
}

#[test]
fn result_views_keep_queue_write_ack_failure_and_original_identity_distinct() {
    for state in [
        GuardedSeekState::Queued,
        GuardedSeekState::WriteCommitted,
        GuardedSeekState::WrittenAwaitingAck,
    ] {
        assert!(!state.is_terminal());
    }
    let accepted = guarded::test_result(GuardedSeekState::CommandAccepted, true, None);
    assert!(accepted.state().is_terminal());
    assert!(accepted.write_commit());
    assert!(accepted.failure().is_none());
    let mut error = ApiError::new(ErrorCode::EngineFailed, "synthetic original pipe failure");
    error.details = serde_json::json!({"raw_os_error":109});
    error.request_id = "original-request".into();
    let failed = guarded::test_result(GuardedSeekState::Failed, true, Some(error));
    assert_eq!(failed.state(), GuardedSeekState::Failed);
    assert_eq!(failed.original_identity().engine_instance_id().0, 7);
    let failure = failed.failure().unwrap();
    assert_eq!(failure.stage(), GuardedSeekFailureStage::Write);
    assert_eq!(failure.api_error().code, ErrorCode::EngineFailed);
    assert_eq!(failure.api_error().details["raw_os_error"], 109);
    assert_eq!(failure.api_error().request_id, "original-request");
    let unobserved = guarded::test_result(GuardedSeekState::OutcomeUnobserved, true, None);
    assert_eq!(unobserved.state(), GuardedSeekState::OutcomeUnobserved);
}
