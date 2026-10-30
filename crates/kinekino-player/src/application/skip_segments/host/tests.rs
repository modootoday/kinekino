use super::*;
use crate::{
    application::{
        api::{PlayerSdk, Request},
        playback_status::{PlaybackStatus, PlaybackStatusOrigin},
        skip_segments::SkipBinding,
    },
    domain::{
        api_values::DecimalU64,
        skip_segments::{Intent, Seekability, SegmentKind, SkipMode, SkipPolicy},
    },
};
use std::sync::atomic::AtomicUsize;

#[test]
fn native_busy_stale_and_cancel_keep_existing_lock_and_preobserve_order() {
    use crate::application::skip_segments::native_port_tests::native_sessions;
    let (sessions, state) = native_sessions();
    let mut registry =
        SkipHostRegistry::new_native_owner("native-preobserve".into(), sessions, 1).unwrap();
    let reference = prepared(&registry);
    registry.seek_submission = true;
    let revision = registry.inspect(reference.clone()).unwrap().revision;
    let mut input = SessionRequest {
        session: reference,
        input: SkipInput {
            segment_id: "op".into(),
            intent: Intent::Manual,
            cancelled: false,
        },
    };
    let lock = registry.state.lock().unwrap();
    assert_eq!(
        registry
            .skip_native_call(input.clone(), revision)
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    drop(lock);
    let stale = registry
        .skip_native_call(input.clone(), DecimalU64(0))
        .unwrap();
    assert_eq!(stale.result.unwrap_err().code, ErrorCode::RevisionConflict);
    assert!(stale.submission.is_none());
    assert_eq!(stale.snapshot.revision, revision);
    input.input.cancelled = true;
    let cancelled = registry.skip_native_call(input, revision).unwrap();
    assert!(matches!(
        cancelled.result,
        Ok(crate::application::skip_segments::SkipResult::Cancelled {
            status: crate::application::skip_segments::CancelStatus::BeforeAdmission,
            ..
        })
    ));
    assert!(cancelled.submission.is_none());
    assert_eq!(state.observes.load(Ordering::SeqCst), 0);
    assert_eq!(state.submits.load(Ordering::SeqCst), 0);
}

#[test]
fn native_constructor_binds_one_arc_and_keeps_production_admission_closed() {
    use crate::application::skip_segments::native_port_tests::native_sessions;
    let (sessions, state) = native_sessions();
    let registry =
        SkipHostRegistry::new_native_owner("native-host".into(), sessions.clone(), 1).unwrap();
    let port: Arc<dyn SkipPlaybackPort + Send + Sync> = sessions.clone();
    assert!(Arc::ptr_eq(&registry.port, &port));
    assert!(Arc::ptr_eq(
        &registry.native_owner.as_ref().unwrap().sessions,
        &sessions
    ));
    assert!(!registry.capability().seek_submission);
    let reference = prepared(&registry);
    let revision = registry.inspect(reference.clone()).unwrap().revision;
    let input = SessionRequest {
        session: reference,
        input: SkipInput {
            segment_id: "op".into(),
            intent: Intent::Manual,
            cancelled: false,
        },
    };
    assert_eq!(
        registry
            .skip_native_call(input.clone(), revision)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    assert_eq!(
        registry.skip(input, revision).unwrap_err().code,
        ErrorCode::Unsupported
    );
    assert_eq!(state.observes.load(Ordering::SeqCst), 0);
    assert_eq!(state.submits.load(Ordering::SeqCst), 0);
}

#[test]
fn native_same_call_has_no_diagnostic_preobserve_and_reopen_does_not_reset_attempt() {
    use crate::application::skip_segments::native_port_tests::native_sessions;
    let (sessions, state) = native_sessions();
    let mut registry =
        SkipHostRegistry::new_native_owner("native-once".into(), sessions, 1).unwrap();
    let reference = prepared(&registry);
    // Existing registry field is changed only inside this cfg(test) fixture.
    registry.seek_submission = true;
    state.legacy_reads.store(0, Ordering::SeqCst);
    let before = registry.inspect(reference.clone()).unwrap();
    let input = SessionRequest {
        session: reference.clone(),
        input: SkipInput {
            segment_id: "op".into(),
            intent: Intent::Automatic,
            cancelled: false,
        },
    };
    let result = registry
        .skip_native_call(input.clone(), before.revision)
        .unwrap();
    assert!(matches!(
        result.result,
        Ok(crate::application::skip_segments::SkipResult::Admitted { .. })
    ));
    assert!(result.submission.is_some());
    assert_eq!(result.snapshot.revision.0, before.revision.0 + 1);
    assert_eq!(state.observes.load(Ordering::SeqCst), 1);
    assert_eq!(state.submits.load(Ordering::SeqCst), 1);
    assert_eq!(state.legacy_reads.load(Ordering::SeqCst), 0);
    let reopened = open(&registry);
    assert!(reopened.reused);
    assert_eq!(reopened.snapshot.pass_id, before.pass_id);
    let duplicate = registry
        .skip_native_call(input, reopened.snapshot.revision)
        .unwrap();
    assert!(matches!(
        duplicate.result,
        Ok(crate::application::skip_segments::SkipResult::NotSubmitted { .. })
    ));
    assert!(duplicate.submission.is_none());
    assert_eq!(state.submits.load(Ordering::SeqCst), 1);
}

#[test]
fn native_core_error_after_ok_preserves_original_ticket_and_updated_snapshot() {
    use crate::application::skip_segments::native_port_tests::native_sessions;
    let (sessions, state) = native_sessions();
    let mut registry =
        SkipHostRegistry::new_native_owner("native-carrier".into(), sessions, 1).unwrap();
    let reference = prepared(&registry);
    registry.seek_submission = true;
    state.mismatch.store(true, Ordering::SeqCst);
    let before = registry.inspect(reference.clone()).unwrap().revision;
    let result = registry
        .skip_native_call(
            SessionRequest {
                session: reference,
                input: SkipInput {
                    segment_id: "op".into(),
                    intent: Intent::Manual,
                    cancelled: false,
                },
            },
            before,
        )
        .unwrap();
    assert_eq!(result.result.unwrap_err().code, ErrorCode::EngineFailed);
    assert_eq!(result.snapshot.revision.0, before.0 + 1);
    let original = state.ticket.lock().unwrap();
    let submission = result.submission.unwrap();
    assert!(submission.ticket.same_request(original.as_ref().unwrap()));
    assert_eq!(submission.receipt().engine_instance_id.0, 8);
}

#[test]
fn native_closing_after_observation_blocks_submit_and_retains_reservation() {
    use crate::application::skip_segments::native_port_tests::native_sessions;
    let (sessions, state) = native_sessions();
    let mut registry =
        SkipHostRegistry::new_native_owner("native-closing".into(), sessions, 1).unwrap();
    let reference = prepared(&registry);
    registry.seek_submission = true;
    let before = registry.inspect(reference.clone()).unwrap().revision;
    *state.close_after_observe.lock().unwrap() = Some(registry.closing.clone());
    let result = registry
        .skip_native_call(
            SessionRequest {
                session: reference,
                input: SkipInput {
                    segment_id: "op".into(),
                    intent: Intent::Manual,
                    cancelled: false,
                },
            },
            before,
        )
        .unwrap();
    assert_eq!(result.result.unwrap_err().code, ErrorCode::Unsupported);
    assert_eq!(result.snapshot.revision.0, before.0 + 1);
    assert!(result.submission.is_none());
    assert_eq!(state.observes.load(Ordering::SeqCst), 1);
    assert_eq!(state.submits.load(Ordering::SeqCst), 0);
}

struct Port {
    engine: AtomicUsize,
    calls: AtomicUsize,
}
impl SkipPlaybackPort for Port {
    fn observe(&self) -> Result<SkipObservation, ApiError> {
        Ok(SkipObservation {
            binding: SkipBinding {
                engine_instance_id: DecimalU64(self.engine.load(Ordering::SeqCst) as u64),
                resolution_artifact_id: "episode".into(),
            },
            seekability: Seekability::Yes,
            observation_authorized: true,
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
        })
    }
    fn submit(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(ControlReceipt::enqueued(input.engine_instance_id.0))
    }
}
fn setup(generation: &str, budget: usize, seek: bool) -> (SkipHostRegistry, Arc<Port>) {
    let port = Arc::new(Port {
        engine: AtomicUsize::new(7),
        calls: AtomicUsize::new(0),
    });
    (
        SkipHostRegistry::new(generation.into(), port.clone(), budget, seek).unwrap(),
        port,
    )
}
fn open(registry: &SkipHostRegistry) -> SessionView {
    registry
        .open(OpenSkipSession {
            binding: SkipBinding {
                engine_instance_id: DecimalU64(7),
                resolution_artifact_id: "episode".into(),
            },
        })
        .unwrap()
}
fn prepared(registry: &SkipHostRegistry) -> SessionRef {
    let view = open(registry);
    let snapshot = registry
        .edit(
            SessionRequest {
                session: view.session.clone(),
                input: EditInput {
                    segment_id: "op".into(),
                    kind: SegmentKind::Opening,
                    in_us: DecimalU64(10_000_000),
                    out_us: DecimalU64(20_000_000),
                },
            },
            view.snapshot.revision,
        )
        .unwrap();
    let snapshot = registry
        .verify(
            SessionRequest {
                session: view.session.clone(),
                input: VerifyInput {
                    segment_id: "op".into(),
                    annotation_revision: DecimalU64(1),
                },
            },
            snapshot.revision,
        )
        .unwrap();
    registry
        .set_policy(
            SessionRequest {
                session: view.session.clone(),
                input: PolicyInput {
                    policy: SkipPolicy {
                        opening: SkipMode::Auto,
                        ending: SkipMode::Manual,
                    },
                },
            },
            snapshot.revision,
        )
        .unwrap();
    view.session
}
#[test]
fn repeated_open_reuses_same_owner_and_does_not_reset_one_pass_guard() {
    let (registry, port) = setup("host-one", 1, true);
    let reference = prepared(&registry);
    let revision = registry.inspect(reference.clone()).unwrap().revision;
    let action = registry
        .skip(
            SessionRequest {
                session: reference.clone(),
                input: SkipInput {
                    segment_id: "op".into(),
                    intent: Intent::Automatic,
                    cancelled: false,
                },
            },
            revision,
        )
        .unwrap();
    assert!(matches!(
        action.result,
        crate::application::skip_segments::SkipResult::Admitted { .. }
    ));
    let again = open(&registry);
    assert!(again.reused);
    assert_eq!(again.session.skip_session_id, reference.skip_session_id);
    registry
        .skip(
            SessionRequest {
                session: reference,
                input: SkipInput {
                    segment_id: "op".into(),
                    intent: Intent::Automatic,
                    cancelled: false,
                },
            },
            again.snapshot.revision,
        )
        .unwrap();
    assert_eq!(port.calls.load(Ordering::SeqCst), 1);
}
#[test]
fn closed_and_foreign_generation_ids_are_not_restored() {
    let (first, _) = setup("host-one", 1, true);
    let old = open(&first);
    first
        .close(old.session.clone(), old.snapshot.revision)
        .unwrap();
    assert_eq!(
        first.inspect(old.session.clone()).unwrap_err().code,
        ErrorCode::NotFound
    );
    let next = open(&first);
    assert_ne!(next.session.skip_session_id, old.session.skip_session_id);
    let (second, _) = setup("host-two", 1, true);
    let new = open(&second);
    assert_ne!(new.session.skip_session_id, old.session.skip_session_id);
    assert_eq!(
        second.inspect(old.session).unwrap_err().code,
        ErrorCode::NotFound
    );
    assert!(!new.snapshot.durable && !new.snapshot.detection_supported);
}
#[test]
fn busy_budget_identity_exhaustion_and_stale_binding_fail_without_seek() {
    let (registry, port) = setup("host-budget", 1, true);
    let old = open(&registry);
    let guard = registry.state.lock().unwrap();
    assert_eq!(
        registry.inspect(old.session.clone()).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    drop(guard);
    port.engine.store(8, Ordering::SeqCst);
    let stored = registry.inspect(old.session.clone()).unwrap();
    assert_eq!(stored.binding.engine_instance_id, DecimalU64(7));
    assert_eq!(
        registry
            .edit(
                SessionRequest {
                    session: old.session.clone(),
                    input: EditInput {
                        segment_id: "op".into(),
                        kind: SegmentKind::Opening,
                        in_us: DecimalU64(10_000_000),
                        out_us: DecimalU64(20_000_000)
                    }
                },
                stored.revision
            )
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        registry
            .open(OpenSkipSession {
                binding: SkipBinding {
                    engine_instance_id: DecimalU64(8),
                    resolution_artifact_id: "episode".into()
                }
            })
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    registry.close(old.session, old.snapshot.revision).unwrap();
    registry.state.lock().unwrap().next_id = u64::MAX;
    assert_eq!(
        registry
            .open(OpenSkipSession {
                binding: SkipBinding {
                    engine_instance_id: DecimalU64(8),
                    resolution_artifact_id: "episode".into()
                }
            })
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    assert!(registry.state.lock().unwrap().entries.is_empty());
    assert_eq!(port.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn native_unknown_capability_default_sdk_and_close_owner_are_truthful() {
    let (registry, port) = setup("native-like", 1, false);
    let reference = prepared(&registry);
    assert!(!registry.capability().seek_submission);
    let revision = registry.inspect(reference.clone()).unwrap().revision;
    assert_eq!(
        registry
            .skip(
                SessionRequest {
                    session: reference.clone(),
                    input: SkipInput {
                        segment_id: "op".into(),
                        intent: Intent::Manual,
                        cancelled: false
                    }
                },
                revision
            )
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    assert_eq!(port.calls.load(Ordering::SeqCst), 0);
    let registry = Arc::new(registry);
    let owner = registry.owner();
    drop(owner);
    assert_eq!(
        registry.inspect(reference).unwrap_err().code,
        ErrorCode::Unsupported
    );
    assert!(!registry.capability().sessions);
    let response = PlayerSdk::default().execute(Request::new("segments.open"));
    assert_eq!(response.error.unwrap().code, ErrorCode::Unsupported);
}
#[test]
fn context_is_single_revision_authority_and_unknown_wire_fields_are_rejected() {
    let (registry, port) = setup("wire-host", 1, true);
    let reference = prepared(&registry);
    let mut request = Request::new("segments.edit");
    request.payload = serde_json::json!({"session":reference,"input":{"segment_id":"op","kind":"opening","in_us":"11000000","out_us":"21000000"}});
    assert_eq!(
        registry.execute(&request).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    request.context.expected_revision = Some("4".into());
    request.payload["input"]["expected_revision"] = serde_json::json!("4");
    assert_eq!(
        registry.execute(&request).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(port.calls.load(Ordering::SeqCst), 0);
}
