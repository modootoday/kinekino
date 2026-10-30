use super::{native_port::NativeSkipCall, *};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        catalog::CaptureMetadata,
        episode_resolve::{EpisodeResolutionResults, StoredEpisodeResolution},
        jobs::VerifiedJobOutput,
        native_playback::NativePlayback,
        playback_control::{ControlCompletion, ControlReceipt},
        playback_session::{self, *},
        playback_status::{PlaybackStatus, project_native_status},
    },
    domain::{
        api_values::DecimalU64,
        playback_control::PlaybackControl,
        skip_segments::{
            BlockReason, Decision, Intent, Seekability, SegmentKind, SkipMode, SkipPolicy,
        },
    },
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};

#[derive(Debug)]
pub(super) struct State {
    pub(super) observes: AtomicUsize,
    pub(super) submits: AtomicUsize,
    pub(super) legacy_reads: AtomicUsize,
    pub(super) fail: AtomicBool,
    pub(super) foreign: AtomicBool,
    pub(super) missing: AtomicBool,
    pub(super) mismatch: AtomicBool,
    pub(super) position: AtomicU64,
    pub(super) close_after_observe: Mutex<Option<Arc<AtomicBool>>>,
    pub(super) ticket: Mutex<Option<NativeSeekTicket>>,
    targets: Mutex<Vec<DecimalU64>>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            observes: 0.into(),
            submits: 0.into(),
            legacy_reads: 0.into(),
            fail: false.into(),
            foreign: false.into(),
            missing: false.into(),
            mismatch: false.into(),
            position: 15.into(),
            close_after_observe: Mutex::new(None),
            ticket: Mutex::new(None),
            targets: Mutex::new(Vec::new()),
        }
    }
}
fn status(state: &State) -> PlaybackStatus {
    project_native_status(&NativePlayback {
        connected: true,
        time: Some(state.position.load(Ordering::SeqCst) as f64),
        duration: Some(100.),
        pause_observed: true,
        idle_observed: true,
        ..Default::default()
    })
}
#[derive(Debug)]
struct Authority(Arc<State>);
impl NativeGuardAuthority for Authority {
    fn matches_issuer(&self, issuer: &dyn std::any::Any) -> bool {
        issuer
            .downcast_ref::<State>()
            .is_some_and(|state| std::ptr::eq(state, self.0.as_ref()))
    }
    fn snapshot(&self) -> NativeGuardSnapshot {
        NativeGuardSnapshot {
            engine_instance_id: DecimalU64(7),
            resolution_artifact_id: if self.0.foreign.load(Ordering::SeqCst) {
                "foreign"
            } else {
                "episode"
            }
            .into(),
            playback: status(&self.0),
            seekability: Seekability::Yes,
        }
    }
    fn submit(self: Box<Self>, position_us: DecimalU64) -> Result<GuardedSeekSubmission, ApiError> {
        if self.0.fail.load(Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "original synthetic admission failure",
            ));
        }
        self.0.targets.lock().unwrap().push(position_us);
        let engine = if self.0.mismatch.load(Ordering::SeqCst) {
            8
        } else {
            7
        };
        let result =
            native_seek_submission(ControlReceipt::enqueued(engine), Arc::new(Source(engine)))?;
        *self.0.ticket.lock().unwrap() = Some(result.ticket.clone());
        Ok(result)
    }
}
#[derive(Debug)]
struct Source(u64);
impl NativeSeekResultSource for Source {
    fn identity(&self) -> (DecimalU64, String) {
        (DecimalU64(self.0), "episode".into())
    }
    fn read(&self, _: bool) -> Result<NativeSeekResultParts, ApiError> {
        Ok(NativeSeekResultParts {
            state: GuardedSeekState::Queued,
            committed: false,
            failure: None,
        })
    }
}
struct Engine(Arc<State>);
impl EpisodePlaybackEngine for Engine {
    fn observe_native_guard(&self) -> Result<NativeGuardObservation, ApiError> {
        self.0.observes.fetch_add(1, Ordering::SeqCst);
        let mut observation = playback_session::test_guarded_observation(NativePlayback::default());
        observation.guard = if self.0.missing.load(Ordering::SeqCst) {
            None
        } else {
            Some(native_guard_from_authority(Box::new(Authority(
                self.0.clone(),
            ))))
        };
        // Public diagnostics deliberately disagree with the original private view.
        observation.engine_instance_id = DecimalU64(999);
        observation.resolution_artifact_id = "diagnostic-only".into();
        if let Some(closing) = self.0.close_after_observe.lock().unwrap().as_ref() {
            closing.store(true, Ordering::Release);
        }
        Ok(observation)
    }
    fn submit_guarded_seek(&self, input: GuardedSeek) -> Result<GuardedSeekSubmission, ApiError> {
        self.0.submits.fetch_add(1, Ordering::SeqCst);
        input.guard.require_issuer(self.0.as_ref())?;
        input.guard.submit_native(input.position_us)
    }
    fn open(
        &self,
        _: EpisodePlaybackSource,
        _: bool,
        _: f64,
    ) -> Result<OpenedEpisodePlayback, ApiError> {
        panic!("no native process")
    }
    fn close(&self, _: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
        panic!("no native process")
    }
    fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
        self.0.legacy_reads.fetch_add(1, Ordering::SeqCst);
        Ok(EpisodePlaybackObservation {
            engine_instance_id: DecimalU64(7),
            resolution_artifact_id: "episode".into(),
            playback: status(&self.0),
            diagnostics: None,
            options: None,
        })
    }
    fn control(&self, _: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        panic!("no legacy control fallback")
    }
}
struct Results;
impl EpisodeResolutionResults for Results {
    fn publish(
        &self,
        _: &StoredEpisodeResolution,
        _: &CaptureMetadata,
    ) -> Result<VerifiedJobOutput, ApiError> {
        panic!("no publication")
    }
    fn read(&self, _: &str) -> Result<StoredEpisodeResolution, ApiError> {
        panic!("no resolution lookup")
    }
}
pub(super) fn native_sessions() -> (Arc<EpisodePlaybackSessions>, Arc<State>) {
    let state = Arc::new(State::default());
    (
        Arc::new(EpisodePlaybackSessions::new(
            Arc::new(Results),
            Arc::new(Engine(state.clone())),
        )),
        state,
    )
}
pub(super) fn configured_session() -> SkipSession {
    let mut session = SkipSession::new(SkipBinding {
        engine_instance_id: DecimalU64(7),
        resolution_artifact_id: "episode".into(),
    })
    .unwrap();
    session
        .edit(EditSegment {
            expected_revision: session.inspect().revision,
            segment_id: "op".into(),
            kind: SegmentKind::Opening,
            in_us: DecimalU64(10_000_000),
            out_us: DecimalU64(20_000_000),
        })
        .unwrap();
    session
        .verify(VerifySegment {
            expected_revision: session.inspect().revision,
            segment_id: "op".into(),
            annotation_revision: DecimalU64(1),
        })
        .unwrap();
    session
        .set_policy(SetSkipPolicy {
            expected_revision: session.inspect().revision,
            policy: SkipPolicy {
                opening: SkipMode::Auto,
                ending: SkipMode::Manual,
            },
        })
        .unwrap();
    session
}
fn request(session: &SkipSession, intent: Intent) -> SkipRequest {
    SkipRequest {
        expected_revision: session.inspect().revision,
        segment_id: "op".into(),
        intent,
        cancelled: false,
    }
}

#[test]
fn same_private_view_is_sampled_once_and_original_guard_and_submission_are_not_rebuilt() {
    let (sessions, state) = native_sessions();
    let closing = AtomicBool::new(false);
    let call = NativeSkipCall::new(&sessions, &closing);
    let first = call.observe().unwrap();
    let second = call.observe().unwrap();
    assert_eq!(first.binding, second.binding);
    assert_eq!(first.binding.engine_instance_id, DecimalU64(7));
    assert_eq!(first.binding.resolution_artifact_id, "episode");
    let mut session = configured_session();
    assert!(
        matches!(session.skip(request(&session, Intent::Manual), &call).unwrap(), SkipResult::Admitted { receipt, .. } if receipt.completion == ControlCompletion::Unobserved)
    );
    assert_eq!(state.observes.load(Ordering::SeqCst), 1);
    assert_eq!(state.submits.load(Ordering::SeqCst), 1);
    assert!(
        call.submit(ControlEpisodePlayback {
            engine_instance_id: DecimalU64(7),
            control: PlaybackControl::Seek {
                position_us: DecimalU64(20_000_000)
            }
        })
        .is_err()
    );
    assert_eq!(state.submits.load(Ordering::SeqCst), 1);
    let submission = call.into_submission().unwrap();
    assert!(
        submission
            .ticket
            .same_request(state.ticket.lock().unwrap().as_ref().unwrap())
    );
    assert_eq!(*state.targets.lock().unwrap(), [DecimalU64(20_000_000)]);
    assert_eq!(state.legacy_reads.load(Ordering::SeqCst), 0);
}
#[test]
fn stale_revision_and_before_cancel_never_observe_or_submit() {
    let (sessions, state) = native_sessions();
    let closing = AtomicBool::new(false);
    let mut session = configured_session();
    let mut input = request(&session, Intent::Manual);
    input.expected_revision = DecimalU64(0);
    assert_eq!(
        session
            .skip(input, &NativeSkipCall::new(&sessions, &closing))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let mut input = request(&session, Intent::Manual);
    input.cancelled = true;
    assert!(matches!(
        session
            .skip(input, &NativeSkipCall::new(&sessions, &closing))
            .unwrap(),
        SkipResult::Cancelled {
            status: CancelStatus::BeforeAdmission,
            ..
        }
    ));
    assert_eq!(state.observes.load(Ordering::SeqCst), 0);
    assert_eq!(state.submits.load(Ordering::SeqCst), 0);
}
#[test]
fn foreign_full_binding_or_missing_guard_cannot_fallback_or_submit() {
    let (sessions, state) = native_sessions();
    let closing = AtomicBool::new(false);
    let mut session = configured_session();
    state.foreign.store(true, Ordering::SeqCst);
    assert_eq!(
        session
            .skip(
                request(&session, Intent::Manual),
                &NativeSkipCall::new(&sessions, &closing)
            )
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    state.foreign.store(false, Ordering::SeqCst);
    state.missing.store(true, Ordering::SeqCst);
    assert_eq!(
        session
            .skip(
                request(&session, Intent::Manual),
                &NativeSkipCall::new(&sessions, &closing)
            )
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    assert_eq!(state.submits.load(Ordering::SeqCst), 0);
    assert_eq!(state.legacy_reads.load(Ordering::SeqCst), 0);
}
#[test]
fn admission_error_preserves_attempt_and_does_not_enable_automatic_replay() {
    let (sessions, state) = native_sessions();
    let closing = AtomicBool::new(false);
    let mut session = configured_session();
    let before = session.inspect().revision;
    state.fail.store(true, Ordering::SeqCst);
    let call = NativeSkipCall::new(&sessions, &closing);
    let error = session
        .skip(request(&session, Intent::Automatic), &call)
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::EngineFailed);
    assert_eq!(session.inspect().revision.0, before.0 + 1);
    assert!(call.into_submission().is_none());
    state.fail.store(false, Ordering::SeqCst);
    assert!(matches!(
        session
            .skip(
                request(&session, Intent::Automatic),
                &NativeSkipCall::new(&sessions, &closing)
            )
            .unwrap(),
        SkipResult::NotSubmitted {
            decision: Decision::Blocked {
                reason: BlockReason::AlreadyAttempted
            }
        }
    ));
    assert_eq!(state.submits.load(Ordering::SeqCst), 1);
    assert!(matches!(
        session
            .skip(
                request(&session, Intent::Manual),
                &NativeSkipCall::new(&sessions, &closing)
            )
            .unwrap(),
        SkipResult::Admitted { .. }
    ));
    assert_eq!(state.submits.load(Ordering::SeqCst), 2);
}
#[test]
fn backward_and_explicit_override_survive_until_explicit_new_pass() {
    let (sessions, state) = native_sessions();
    let closing = AtomicBool::new(false);
    let mut session = configured_session();
    assert!(matches!(
        session
            .skip(
                request(&session, Intent::Automatic),
                &NativeSkipCall::new(&sessions, &closing)
            )
            .unwrap(),
        SkipResult::Admitted { .. }
    ));
    state.position.store(5, Ordering::SeqCst);
    assert!(matches!(
        session
            .skip(
                request(&session, Intent::Automatic),
                &NativeSkipCall::new(&sessions, &closing)
            )
            .unwrap(),
        SkipResult::NotSubmitted {
            decision: Decision::Blocked {
                reason: BlockReason::Overridden
            }
        }
    ));
    session
        .manual_seek_override(session.inspect().revision)
        .unwrap();
    state.position.store(15, Ordering::SeqCst);
    assert!(matches!(
        session
            .skip(
                request(&session, Intent::Automatic),
                &NativeSkipCall::new(&sessions, &closing)
            )
            .unwrap(),
        SkipResult::NotSubmitted { .. }
    ));
    let pass = session.inspect().pass_id;
    session.begin_new_pass(session.inspect().revision).unwrap();
    assert_eq!(session.inspect().pass_id.0, pass.0 + 1);
    assert!(matches!(
        session
            .skip(
                request(&session, Intent::Automatic),
                &NativeSkipCall::new(&sessions, &closing)
            )
            .unwrap(),
        SkipResult::Admitted { .. }
    ));
    assert_eq!(state.submits.load(Ordering::SeqCst), 2);
}
