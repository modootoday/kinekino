use super::*;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_control::{ControlAcceptance, ControlCompletion, ControlReceipt},
        playback_session::ControlEpisodePlayback,
        playback_status::{PlaybackStatus, PlaybackStatusOrigin},
    },
    domain::{
        api_values::DecimalU64,
        playback_control::PlaybackControl,
        skip_segments::{
            BlockReason, Decision, Intent, Seekability, SegmentKind, SegmentOrigin, SkipMode,
            SkipPolicy,
        },
    },
};
use std::cell::{Cell, RefCell};

struct Port {
    observation: RefCell<SkipObservation>,
    observations: Cell<usize>,
    submitted: RefCell<Vec<ControlEpisodePlayback>>,
    fail: Cell<bool>,
    deferred: Cell<bool>,
    wrong_engine: Cell<bool>,
}
fn binding() -> SkipBinding {
    SkipBinding {
        engine_instance_id: DecimalU64(7),
        resolution_artifact_id: "resolved-episode".into(),
    }
}
fn port() -> Port {
    Port {
        observation: RefCell::new(SkipObservation {
            binding: binding(),
            observation_authorized: true,
            seekability: Seekability::Yes,
            playback: PlaybackStatus {
                origin: PlaybackStatusOrigin::NativeObservation,
                published_age_ms: DecimalU64(0),
                connected: true,
                paused: Some(false),
                idle: Some(false),
                position_us: Some(DecimalU64(15_000_000)),
                duration_us: Some(DecimalU64(100_000_000)),
                volume_percent: None,
                muted: None,
                video_size: None,
                audio_channels: None,
                has_error: false,
            },
        }),
        observations: Cell::new(0),
        submitted: RefCell::new(Vec::new()),
        fail: Cell::new(false),
        deferred: Cell::new(false),
        wrong_engine: Cell::new(false),
    }
}
impl SkipPlaybackPort for Port {
    fn observe(&self) -> Result<SkipObservation, ApiError> {
        self.observations.set(self.observations.get() + 1);
        let current = self.observation.borrow();
        Ok(SkipObservation {
            binding: current.binding.clone(),
            playback: current.playback.clone(),
            seekability: current.seekability,
            observation_authorized: current.observation_authorized,
        })
    }
    fn submit(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        self.submitted.borrow_mut().push(input);
        if self.fail.get() {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Original native admission error",
            ));
        }
        let id = if self.wrong_engine.get() { 8 } else { 7 };
        Ok(if self.deferred.get() {
            ControlReceipt::deferred_until_handoff(id)
        } else {
            ControlReceipt::enqueued(id)
        })
    }
}
fn revision(session: &SkipSession) -> DecimalU64 {
    session.inspect().revision
}
fn request(session: &SkipSession, intent: Intent) -> SkipRequest {
    SkipRequest {
        expected_revision: revision(session),
        segment_id: "op".into(),
        intent,
        cancelled: false,
    }
}
fn session() -> SkipSession {
    let mut session = SkipSession::new(binding()).unwrap();
    session
        .edit(EditSegment {
            expected_revision: revision(&session),
            segment_id: "op".into(),
            kind: SegmentKind::Opening,
            in_us: DecimalU64(10_000_000),
            out_us: DecimalU64(20_000_000),
        })
        .unwrap();
    session
        .verify(VerifySegment {
            expected_revision: revision(&session),
            segment_id: "op".into(),
            annotation_revision: DecimalU64(1),
        })
        .unwrap();
    session
        .set_policy(SetSkipPolicy {
            expected_revision: revision(&session),
            policy: SkipPolicy {
                opening: SkipMode::Auto,
                ending: SkipMode::Manual,
            },
        })
        .unwrap();
    session
}
fn blocked(result: SkipResult, expected: BlockReason) {
    assert!(
        matches!(result,SkipResult::NotSubmitted { decision: Decision::Blocked { reason } } if reason == expected)
    );
}

#[test]
fn one_admission_uses_existing_seek_and_never_claims_landing_or_detection() {
    let mut session = session();
    let port = port();
    let result = session
        .skip(request(&session, Intent::Automatic), &port)
        .unwrap();
    assert!(matches!(
        result,
        SkipResult::Admitted {
            receipt: ControlReceipt {
                acceptance: ControlAcceptance::Accepted,
                completion: ControlCompletion::Unobserved,
                ..
            },
            ..
        }
    ));
    blocked(
        session
            .skip(request(&session, Intent::Automatic), &port)
            .unwrap(),
        BlockReason::AlreadyAttempted,
    );
    assert_eq!(port.submitted.borrow().len(), 1);
    let calls = port.submitted.borrow();
    assert_eq!(calls[0].engine_instance_id, DecimalU64(7));
    assert_eq!(
        calls[0].control,
        PlaybackControl::Seek {
            position_us: DecimalU64(20_000_000)
        }
    );
    assert_eq!(session.inspect().segments[0].origin, SegmentOrigin::Manual);
    assert!(!session.inspect().durable && !session.inspect().detection_supported);
}

#[test]
fn cancelled_before_submission_and_late_cancel_are_distinct() {
    let mut session = session();
    let port = port();
    let mut input = request(&session, Intent::Automatic);
    input.cancelled = true;
    assert!(matches!(
        session.skip(input, &port).unwrap(),
        SkipResult::Cancelled {
            status: CancelStatus::BeforeAdmission,
            ..
        }
    ));
    assert_eq!(port.observations.get(), 0);
    assert!(port.submitted.borrow().is_empty());
    blocked(
        session
            .skip(request(&session, Intent::Automatic), &port)
            .unwrap(),
        BlockReason::Overridden,
    );
    // Explicit manual override can still submit after watch-this-once.
    session
        .skip(request(&session, Intent::Manual), &port)
        .unwrap();
    assert_eq!(
        session.cancel(revision(&session), "op").unwrap(),
        CancelStatus::TooLateOrUnobserved
    );
    assert_eq!(port.submitted.borrow().len(), 1);
}

#[test]
fn unknown_duration_seekability_and_pause_submit_zero() {
    let mut session = session();
    let port = port();
    port.observation.borrow_mut().seekability = Seekability::Unknown;
    blocked(
        session
            .skip(request(&session, Intent::Manual), &port)
            .unwrap(),
        BlockReason::SeekUnknown,
    );
    port.observation.borrow_mut().seekability = Seekability::Yes;
    port.observation.borrow_mut().playback.duration_us = None;
    blocked(
        session
            .skip(request(&session, Intent::Manual), &port)
            .unwrap(),
        BlockReason::DurationUnknown,
    );
    port.observation.borrow_mut().playback.duration_us = Some(DecimalU64(100_000_000));
    port.observation.borrow_mut().playback.paused = Some(true);
    blocked(
        session
            .skip(request(&session, Intent::Automatic), &port)
            .unwrap(),
        BlockReason::PausedOrUnknown,
    );
    assert!(port.submitted.borrow().is_empty());
}

#[test]
fn stale_revision_and_media_instance_never_reach_seek() {
    let mut session = session();
    let port = port();
    let mut input = request(&session, Intent::Automatic);
    input.expected_revision = DecimalU64(1);
    assert_eq!(
        session.skip(input, &port).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(port.observations.get(), 0);
    port.observation.borrow_mut().binding.engine_instance_id = DecimalU64(8);
    assert_eq!(
        session
            .skip(request(&session, Intent::Automatic), &port)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert!(port.submitted.borrow().is_empty());
}

#[test]
fn backward_seek_stays_overridden_until_explicit_new_pass() {
    let mut session = session();
    let port = port();
    session.evaluate("op", Intent::Automatic, &port).unwrap();
    port.observation.borrow_mut().playback.position_us = Some(DecimalU64(5_000_000));
    assert_eq!(
        session.evaluate("op", Intent::Automatic, &port).unwrap(),
        Decision::Blocked {
            reason: BlockReason::Overridden
        }
    );
    port.observation.borrow_mut().playback.position_us = Some(DecimalU64(15_000_000));
    blocked(
        session
            .skip(request(&session, Intent::Automatic), &port)
            .unwrap(),
        BlockReason::Overridden,
    );
    assert!(port.submitted.borrow().is_empty());
    let previous = session.inspect().pass_id;
    session.begin_new_pass(revision(&session)).unwrap();
    assert!(session.inspect().pass_id > previous);
    session
        .skip(request(&session, Intent::Automatic), &port)
        .unwrap();
    assert_eq!(port.submitted.borrow().len(), 1);
}

#[test]
fn admission_error_preserved_no_automatic_retry_but_explicit_manual_retry_possible() {
    let mut session = session();
    let port = port();
    port.fail.set(true);
    let error = session
        .skip(request(&session, Intent::Automatic), &port)
        .unwrap_err();
    assert_eq!(error.message, "Original native admission error");
    port.fail.set(false);
    blocked(
        session
            .skip(request(&session, Intent::Automatic), &port)
            .unwrap(),
        BlockReason::AlreadyAttempted,
    );
    assert_eq!(port.submitted.borrow().len(), 1);
    session
        .skip(request(&session, Intent::Manual), &port)
        .unwrap();
    assert_eq!(port.submitted.borrow().len(), 2);
}

#[test]
fn edit_invalidates_verification_and_notice_without_resetting_pass_guard() {
    let mut session = session();
    let port = port();
    port.observation.borrow_mut().playback.position_us = Some(DecimalU64(7_000_000));
    assert!(matches!(
        session.evaluate("op", Intent::Automatic, &port).unwrap(),
        Decision::Notice { .. }
    ));
    session
        .edit(EditSegment {
            expected_revision: revision(&session),
            segment_id: "op".into(),
            kind: SegmentKind::Opening,
            in_us: DecimalU64(11_000_000),
            out_us: DecimalU64(21_000_000),
        })
        .unwrap();
    blocked(
        session
            .skip(request(&session, Intent::Automatic), &port)
            .unwrap(),
        BlockReason::Unverified,
    );
    assert_eq!(
        session
            .verify(VerifySegment {
                expected_revision: revision(&session),
                segment_id: "op".into(),
                annotation_revision: DecimalU64(1)
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert!(port.submitted.borrow().is_empty());
}

#[test]
fn deferred_admission_is_unobserved_and_mismatched_receipt_is_not_replayed() {
    let mut first = session();
    let port = port();
    port.deferred.set(true);
    assert!(matches!(
        first
            .skip(request(&first, Intent::Automatic), &port)
            .unwrap(),
        SkipResult::Admitted {
            receipt: ControlReceipt {
                acceptance: ControlAcceptance::DeferredUntilHandoff,
                completion: ControlCompletion::Unobserved,
                ..
            },
            ..
        }
    ));
    blocked(
        first
            .skip(request(&first, Intent::Automatic), &port)
            .unwrap(),
        BlockReason::AlreadyAttempted,
    );
    let mut second = session();
    port.wrong_engine.set(true);
    assert_eq!(
        second
            .skip(request(&second, Intent::Automatic), &port)
            .unwrap_err()
            .code,
        ErrorCode::EngineFailed
    );
    blocked(
        second
            .skip(request(&second, Intent::Automatic), &port)
            .unwrap(),
        BlockReason::AlreadyAttempted,
    );
    assert_eq!(port.submitted.borrow().len(), 2);
}

#[test]
fn off_and_overlapping_segments_submit_zero_then_adjacent_boundary_allows_one() {
    let mut session = session();
    let port = port();
    session
        .set_policy(SetSkipPolicy {
            expected_revision: revision(&session),
            policy: SkipPolicy {
                opening: SkipMode::Off,
                ending: SkipMode::Manual,
            },
        })
        .unwrap();
    blocked(
        session
            .skip(request(&session, Intent::Manual), &port)
            .unwrap(),
        BlockReason::Off,
    );
    session
        .set_policy(SetSkipPolicy {
            expected_revision: revision(&session),
            policy: SkipPolicy {
                opening: SkipMode::Auto,
                ending: SkipMode::Manual,
            },
        })
        .unwrap();
    session
        .edit(EditSegment {
            expected_revision: revision(&session),
            segment_id: "ed".into(),
            kind: SegmentKind::Ending,
            in_us: DecimalU64(19_000_000),
            out_us: DecimalU64(30_000_000),
        })
        .unwrap();
    blocked(
        session
            .skip(request(&session, Intent::Automatic), &port)
            .unwrap(),
        BlockReason::Overlap,
    );
    assert!(port.submitted.borrow().is_empty());
    session
        .edit(EditSegment {
            expected_revision: revision(&session),
            segment_id: "ed".into(),
            kind: SegmentKind::Ending,
            in_us: DecimalU64(20_000_000),
            out_us: DecimalU64(30_000_000),
        })
        .unwrap();
    session
        .skip(request(&session, Intent::Automatic), &port)
        .unwrap();
    assert_eq!(port.submitted.borrow().len(), 1);
}

#[test]
fn explicit_manual_override_survives_policy_changes_and_manual_action_remains_explicit() {
    let mut session = session();
    let port = port();
    session.manual_seek_override(revision(&session)).unwrap();
    session
        .set_policy(SetSkipPolicy {
            expected_revision: revision(&session),
            policy: SkipPolicy {
                opening: SkipMode::Auto,
                ending: SkipMode::Auto,
            },
        })
        .unwrap();
    blocked(
        session
            .skip(request(&session, Intent::Automatic), &port)
            .unwrap(),
        BlockReason::Overridden,
    );
    assert_eq!(
        session.cancel(revision(&session), "op").unwrap(),
        CancelStatus::BeforeAdmission
    );
    assert!(port.submitted.borrow().is_empty());
    session
        .skip(request(&session, Intent::Manual), &port)
        .unwrap();
    assert_eq!(port.submitted.borrow().len(), 1);
}

#[test]
fn cancelled_request_after_accepted_deferred_or_error_is_too_late_or_unobserved() {
    for outcome in ["accepted", "deferred", "error"] {
        let mut session = session();
        let port = port();
        port.deferred.set(outcome == "deferred");
        port.fail.set(outcome == "error");
        let initial = session.skip(request(&session, Intent::Automatic), &port);
        assert_eq!(initial.is_err(), outcome == "error");
        let observations_before = port.observations.get();
        let mut cancel = request(&session, Intent::Automatic);
        cancel.cancelled = true;
        assert!(
            matches!(
                session.skip(cancel, &port).unwrap(),
                SkipResult::Cancelled {
                    status: CancelStatus::TooLateOrUnobserved,
                    ..
                }
            ),
            "{outcome}"
        );
        assert_eq!(port.submitted.borrow().len(), 1, "{outcome}");
        assert_eq!(port.observations.get(), observations_before, "{outcome}");
    }
}
