use super::{SkipHostApi, contracts::*};
use crate::{
    application::{
        api::{ApiError, ErrorCode, Request},
        playback_control::ControlReceipt,
        playback_session::{ControlEpisodePlayback, EpisodePlaybackSessions},
        skip_segments::{
            EditSegment, SetSkipPolicy, SkipObservation, SkipPlaybackPort, SkipRequest,
            SkipSession, VerifySegment,
        },
    },
    domain::{api_values::DecimalU64, identifiers::valid_id},
};
use serde_json::Value;
use std::sync::{
    Arc, Mutex, TryLockError,
    atomic::{AtomicBool, Ordering},
};

struct Entry {
    id: String,
    session: SkipSession,
}
struct State {
    next_id: u64,
    entries: Vec<Entry>,
}
pub struct SkipHostRegistry {
    generation: String,
    state: Mutex<State>,
    port: Arc<dyn SkipPlaybackPort + Send + Sync>,
    closing: Arc<AtomicBool>,
    maximum_sessions: usize,
    seek_submission: bool,
    native_owner: Option<NativeOwner>,
    export_owner: Option<ExportOwner>,
    export_enabled: bool,
}
struct NativeOwner {
    sessions: Arc<EpisodePlaybackSessions>,
}
struct ExportOwner {
    sessions: Arc<EpisodePlaybackSessions>,
    issuer: crate::application::playback_session::v1::NativeExportIssuer,
}
pub struct SkipHostOwner(Arc<SkipHostRegistry>);
impl Drop for SkipHostOwner {
    fn drop(&mut self) {
        self.0.begin_close();
    }
}
struct BoundPort<'a> {
    inner: &'a (dyn SkipPlaybackPort + Send + Sync),
    closing: &'a AtomicBool,
}
impl SkipPlaybackPort for BoundPort<'_> {
    fn observe(&self) -> Result<SkipObservation, ApiError> {
        self.inner.observe()
    }
    fn submit(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        if self.closing.load(Ordering::Acquire) {
            return Err(unavailable());
        }
        self.inner.submit(input)
    }
}
impl SkipHostRegistry {
    #[cfg(all(windows, feature = "native-proposal-window"))]
    pub(crate) fn new_routed_export_host(
        generation: String,
        prepared: super::PreparedRoutedSkipOwner,
        maximum_sessions: usize,
    ) -> Result<Self, Box<super::RoutedSkipSetupFailure>> {
        let mut registry =
            match Self::new_native_owner(generation, prepared.sessions(), maximum_sessions) {
                Ok(registry) => registry,
                Err(error) => {
                    return Err(Box::new(super::RoutedSkipSetupFailure { error, prepared }));
                }
            };
        let (sessions, issuer) = prepared.into_parts();
        registry.export_owner = Some(ExportOwner { sessions, issuer });
        registry.export_enabled = true;
        Ok(registry)
    }

    pub fn owner(self: &Arc<Self>) -> SkipHostOwner {
        SkipHostOwner(self.clone())
    }
    /// Generation is supplied by a trusted host identity adapter, never request JSON.
    pub fn new(
        generation: String,
        port: Arc<dyn SkipPlaybackPort + Send + Sync>,
        maximum_sessions: usize,
        seek_submission: bool,
    ) -> Result<Self, ApiError> {
        if !valid_id(&generation) || generation.len() > 40 || maximum_sessions == 0 {
            return Err(invalid(
                "Invalid skip host identity or explicit session budget",
            ));
        }
        Ok(Self {
            generation,
            state: Mutex::new(State {
                next_id: 1,
                entries: Vec::new(),
            }),
            port,
            closing: Arc::new(AtomicBool::new(false)),
            maximum_sessions,
            seek_submission,
            native_owner: None,
            export_owner: None,
            export_enabled: false,
        })
    }
    /// Binds one original Rust owner; does not enable advertised seek support.
    pub fn new_native_owner(
        generation: String,
        sessions: Arc<EpisodePlaybackSessions>,
        maximum_sessions: usize,
    ) -> Result<Self, ApiError> {
        let mut registry = Self::new(generation, sessions.clone(), maximum_sessions, false)?;
        registry.native_owner = Some(NativeOwner { sessions });
        Ok(registry)
    }
    /// Captures a trusted original issuer. Public wire activation remains closed until integration.
    pub fn new_native_export_owner(
        generation: String,
        sessions: Arc<EpisodePlaybackSessions>,
        issuer: crate::application::playback_session::v1::NativeExportIssuer,
        maximum_sessions: usize,
    ) -> Result<Self, ApiError> {
        let mut registry = Self::new_native_owner(generation, sessions.clone(), maximum_sessions)?;
        registry.export_owner = Some(ExportOwner { sessions, issuer });
        Ok(registry)
    }
    /// Explicit standalone wire assembly. The issuer is captured from this same port owner.
    pub fn new_standalone_export_host(
        generation: String,
        sessions: Arc<EpisodePlaybackSessions>,
        maximum_sessions: usize,
    ) -> Result<Self, ApiError> {
        let mut registry = Self::new_native_owner(generation, sessions.clone(), maximum_sessions)?;
        let issuer = sessions.native_export_issuer()?;
        registry.export_owner = Some(ExportOwner { sessions, issuer });
        registry.export_enabled = true;
        Ok(registry)
    }
    pub fn export_capability(&self) -> SkipExportCapability {
        SkipExportCapability {
            available: self.export_enabled
                && self.export_owner.is_some()
                && !self.closing.load(Ordering::Acquire),
            issuer_scope: self.export_owner.as_ref().map(|owner| owner.issuer.scope()),
        }
    }
    fn export_owner(
        &self,
        scope: crate::application::playback_session::v1::NativeIssuerScopeV1,
    ) -> Result<&ExportOwner, ApiError> {
        if self.closing.load(Ordering::Acquire) {
            return Err(unavailable());
        }
        let owner = self.export_owner.as_ref().ok_or_else(unavailable)?;
        if owner.issuer.scope() != scope {
            return Err(conflict());
        }
        Ok(owner)
    }
    fn export_origin(
        &self,
        origin: &SkipExportOrigin,
    ) -> Result<crate::application::playback_session::v1::NativeExportOriginV1, ApiError> {
        use crate::application::playback_session::v1;
        v1::NativeExportOriginV1::try_from(v1::NativeExportOriginInputV1 {
            registry_generation: self.generation.clone(),
            session: origin.session.skip_session_id.clone(),
            pass: origin.pass_id,
            segment: origin.segment_id.clone(),
            request_revision: origin.request_revision,
            engine_instance_id: origin.session.binding.engine_instance_id,
            resolution_artifact_id: origin.session.binding.resolution_artifact_id.clone(),
        })
        .map_err(ApiError::from)
    }
    /// Reads original exports without consulting the current skip entry or route.
    pub fn list_native_exports(
        &self,
        input: ListSkipExports,
    ) -> Result<crate::application::playback_session::v1::NativeExportListV1, ApiError> {
        let owner = self.export_owner(input.issuer_scope)?;
        owner.issuer.list(&self.export_origin(&input.origin)?)
    }
    pub fn peek_native_export(
        &self,
        input: ReadSkipExport,
    ) -> Result<crate::application::playback_session::v1::NativeExportResultV1, ApiError> {
        let owner = self.export_owner(input.issuer_scope)?;
        owner
            .issuer
            .peek(input.reference, &self.export_origin(&input.origin)?)
    }
    pub fn consume_native_export(
        &self,
        input: ReadSkipExport,
    ) -> Result<crate::application::playback_session::v1::NativeExportResultV1, ApiError> {
        let owner = self.export_owner(input.issuer_scope)?;
        owner
            .issuer
            .consume(input.reference, &self.export_origin(&input.origin)?)
    }
    /// Additive Rust-only path. Existing policy owns revision/reservation/cancel ordering.
    pub fn skip_native_export_call(
        &self,
        input: SkipExportRequest,
        revision: DecimalU64,
    ) -> Result<SkipExportResult, ApiError> {
        use crate::application::{
            playback_session::v1, skip_segments::native_port::NativeSkipCall,
        };
        let owner = self.export_owner(input.issuer_scope)?;
        let reference = input.session.clone();
        self.with_session(input.session, false, |session, _| {
            let (_, pass_id, annotation) = session.export_selection(&input.input.segment_id);
            let origin = SkipExportOrigin {
                session: reference.clone(),
                pass_id,
                request_revision: revision,
                segment_id: input.input.segment_id.clone(),
            };
            // Raw preparation cannot introduce a new validation error before core skip.
            let preparation = v1::NativeExportOriginInputV1 {
                registry_generation: self.generation.clone(),
                session: reference.skip_session_id.clone(),
                pass: pass_id,
                segment: input.input.segment_id.clone(),
                request_revision: revision,
                engine_instance_id: reference.binding.engine_instance_id,
                resolution_artifact_id: reference.binding.resolution_artifact_id.clone(),
            };
            let call = NativeSkipCall::new_export(
                &owner.sessions,
                &self.closing,
                &owner.issuer,
                preparation,
                annotation,
            );
            let result = session.skip(
                SkipRequest {
                    expected_revision: revision,
                    segment_id: input.input.segment_id.clone(),
                    intent: input.input.intent,
                    cancelled: input.input.cancelled,
                },
                &call,
            );
            // Take the original Ok before inspecting the core's post-admission result.
            let exported = call.into_exported();
            let (current, pass_id) = session.export_state();
            let session_state = SkipExportSessionState {
                session: reference.clone(),
                revision: current,
                pass_id,
            };
            let action = project_export(result, exported, origin);
            Ok(SkipExportResult {
                session_state,
                result: action,
            })
        })
    }
    /// Rust-only typed boundary; existing CLI dispatch and production support stay closed.
    pub fn skip_native_call(
        &self,
        input: SessionRequest<SkipInput>,
        revision: DecimalU64,
    ) -> Result<crate::application::skip_segments::NativeSkipOutcome, ApiError> {
        use crate::application::skip_segments::{NativeSkipOutcome, native_port::NativeSkipCall};
        if !self.seek_submission {
            return Err(unavailable());
        }
        let owner = self.native_owner.as_ref().ok_or_else(unavailable)?;
        self.with_session(input.session, false, |session, _| {
            let call = NativeSkipCall::new(&owner.sessions, &self.closing);
            let result = session.skip(
                SkipRequest {
                    expected_revision: revision,
                    segment_id: input.input.segment_id,
                    intent: input.input.intent,
                    cancelled: input.input.cancelled,
                },
                &call,
            );
            let submission = call.into_submission();
            Ok(NativeSkipOutcome {
                result,
                submission,
                snapshot: session.inspect(),
            })
        })
    }
    /// Blocks future admission; does not revoke a synchronous call already dispatched.
    pub fn begin_close(&self) {
        self.closing.store(true, Ordering::Release);
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, State>, ApiError> {
        if self.closing.load(Ordering::Acquire) {
            return Err(unavailable());
        }
        self.state.try_lock().map_err(|error| match error {
            TryLockError::WouldBlock => ApiError::new(
                ErrorCode::ResourceExhausted,
                "Skip host busy; request not dispatched",
            ),
            TryLockError::Poisoned(_) => {
                ApiError::new(ErrorCode::Internal, "Skip host state unavailable")
            }
        })
    }
    fn current_binding(
        &self,
        binding: &crate::application::skip_segments::SkipBinding,
    ) -> Result<(), ApiError> {
        if self.port.observe()?.binding != *binding {
            return Err(conflict());
        }
        Ok(())
    }
    pub fn open(&self, input: OpenSkipSession) -> Result<SessionView, ApiError> {
        let session = SkipSession::new(input.binding.clone())?;
        let mut state = self.lock()?;
        self.current_binding(&input.binding)?;
        if let Some(entry) = state
            .entries
            .iter()
            .find(|entry| entry.session.binding_ref() == &input.binding)
        {
            return Ok(view(entry, true));
        }
        if state.entries.len() >= self.maximum_sessions {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Skip host session budget exhausted; explicitly close an old session",
            ));
        }
        let next = state.next_id.checked_add(1).ok_or_else(|| {
            ApiError::new(
                ErrorCode::ResourceExhausted,
                "Skip session identity exhausted",
            )
        })?;
        let id = format!("skip-{}-{}", self.generation, state.next_id);
        state.entries.push(Entry { id, session });
        state.next_id = next;
        Ok(view(state.entries.last().expect("inserted session"), false))
    }
    fn with_session<T>(
        &self,
        reference: SessionRef,
        current_required: bool,
        call: impl FnOnce(&mut SkipSession, &BoundPort<'_>) -> Result<T, ApiError>,
    ) -> Result<T, ApiError> {
        if !valid_id(&reference.skip_session_id) {
            return Err(invalid("Invalid skip session identity"));
        }
        let mut state = self.lock()?;
        let entry = state
            .entries
            .iter_mut()
            .find(|entry| entry.id == reference.skip_session_id)
            .ok_or_else(not_found)?;
        if entry.session.binding_ref() != &reference.binding {
            return Err(conflict());
        }
        if current_required {
            self.current_binding(&reference.binding)?;
        }
        let port = BoundPort {
            inner: self.port.as_ref(),
            closing: self.closing.as_ref(),
        };
        call(&mut entry.session, &port)
    }
    pub fn inspect(
        &self,
        reference: SessionRef,
    ) -> Result<crate::application::skip_segments::SkipSnapshot, ApiError> {
        // Stored ephemeral state, not proof the old engine is still current.
        // This permits revision recovery/explicit close after media replacement.
        self.with_session(reference, false, |session, _| Ok(session.inspect()))
    }
    pub fn edit(
        &self,
        input: SessionRequest<EditInput>,
        revision: DecimalU64,
    ) -> Result<crate::application::skip_segments::SkipSnapshot, ApiError> {
        self.with_session(input.session, true, |session, _| {
            session.edit(EditSegment {
                expected_revision: revision,
                segment_id: input.input.segment_id,
                kind: input.input.kind,
                in_us: input.input.in_us,
                out_us: input.input.out_us,
            })
        })
    }
    pub fn verify(
        &self,
        input: SessionRequest<VerifyInput>,
        revision: DecimalU64,
    ) -> Result<crate::application::skip_segments::SkipSnapshot, ApiError> {
        self.with_session(input.session, true, |session, _| {
            session.verify(VerifySegment {
                expected_revision: revision,
                segment_id: input.input.segment_id,
                annotation_revision: input.input.annotation_revision,
            })
        })
    }
    pub fn set_policy(
        &self,
        input: SessionRequest<PolicyInput>,
        revision: DecimalU64,
    ) -> Result<crate::application::skip_segments::SkipSnapshot, ApiError> {
        self.with_session(input.session, true, |session, _| {
            session.set_policy(SetSkipPolicy {
                expected_revision: revision,
                policy: input.input.policy,
            })
        })
    }
    pub fn evaluate(
        &self,
        input: SessionRequest<EvaluateInput>,
    ) -> Result<EvaluationResult, ApiError> {
        self.with_session(input.session, true, |session, port| {
            Ok(EvaluationResult {
                decision: session.evaluate(&input.input.segment_id, input.input.intent, port)?,
                snapshot: session.inspect(),
            })
        })
    }
    pub fn skip(
        &self,
        input: SessionRequest<SkipInput>,
        revision: DecimalU64,
    ) -> Result<ActionResult, ApiError> {
        if !self.seek_submission {
            return Err(unavailable());
        }
        self.with_session(input.session, true, |session, port| {
            Ok(ActionResult {
                result: session.skip(
                    SkipRequest {
                        expected_revision: revision,
                        segment_id: input.input.segment_id,
                        intent: input.input.intent,
                        cancelled: input.input.cancelled,
                    },
                    port,
                )?,
                snapshot: session.inspect(),
            })
        })
    }
    pub fn override_session(
        &self,
        input: SessionRequest<OverrideInput>,
        revision: DecimalU64,
    ) -> Result<OverrideResult, ApiError> {
        self.with_session(input.session, true, |session, _| {
            let cancel_status = match input.input {
                OverrideInput::Cancel { segment_id } => {
                    Some(session.cancel(revision, &segment_id)?)
                }
                OverrideInput::ManualSeek => {
                    session.manual_seek_override(revision)?;
                    None
                }
                OverrideInput::BeginNewPass => {
                    session.begin_new_pass(revision)?;
                    None
                }
            };
            Ok(OverrideResult {
                snapshot: session.inspect(),
                cancel_status,
            })
        })
    }
    pub fn close(
        &self,
        reference: SessionRef,
        revision: DecimalU64,
    ) -> Result<ClosedSkipSession, ApiError> {
        if !valid_id(&reference.skip_session_id) {
            return Err(invalid("Invalid skip session identity"));
        }
        let mut state = self.lock()?;
        let index = state
            .entries
            .iter()
            .position(|entry| entry.id == reference.skip_session_id)
            .ok_or_else(not_found)?;
        let snapshot = state.entries[index].session.inspect();
        if snapshot.binding != reference.binding || snapshot.revision != revision {
            return Err(conflict());
        }
        state.entries.remove(index);
        Ok(ClosedSkipSession {
            skip_session_id: reference.skip_session_id,
            closed: true,
            playback_closed: false,
        })
    }
}
impl SkipHostApi for SkipHostRegistry {
    fn export_capability(&self) -> SkipExportCapability {
        SkipHostRegistry::export_capability(self)
    }
    fn capability(&self) -> SkipHostCapability {
        let sessions = !self.closing.load(Ordering::Acquire);
        SkipHostCapability {
            sessions,
            seek_submission: sessions && self.seek_submission,
        }
    }
    fn execute(&self, request: &Request) -> Result<Value, ApiError> {
        super::dispatch::execute(self, request)
    }
}
fn view(entry: &Entry, reused: bool) -> SessionView {
    let snapshot = entry.session.inspect();
    SessionView {
        session: SessionRef {
            skip_session_id: entry.id.clone(),
            binding: snapshot.binding.clone(),
        },
        snapshot,
        reused,
    }
}
fn project_export(
    result: Result<crate::application::skip_segments::SkipResult, ApiError>,
    exported: Option<crate::application::playback_session::v1::ExportedGuardedSeekV1>,
    origin: SkipExportOrigin,
) -> SkipExportAction {
    use crate::application::{playback_session::v1, skip_segments::SkipResult};
    if let Some(exported) = exported {
        let (submission, reference, metadata) = exported.into_parts();
        let receipt = submission.receipt().clone();
        return match result {
            Err(error) => SkipExportAction::AdmittedWithError {
                receipt,
                reference,
                metadata,
                error: v1::NativeExportCoreErrorV1::from_api(&error),
            },
            Ok(_) => SkipExportAction::Admitted {
                receipt,
                reference,
                metadata,
            },
        };
    }
    match result {
        Ok(SkipResult::NotSubmitted { decision }) => SkipExportAction::NotSubmitted { decision },
        Ok(SkipResult::Cancelled { status, .. }) => SkipExportAction::Cancelled { status },
        Err(error) => SkipExportAction::NoSubmissionReturned {
            error: v1::NativeExportCoreErrorV1::from_api(&error),
            origin,
        },
        Ok(SkipResult::Admitted { .. }) => SkipExportAction::NoSubmissionReturned {
            error: v1::NativeExportCoreErrorV1::from_api(&ApiError::new(
                ErrorCode::Internal,
                "Core admitted without returning an original export submission; completion unobserved",
            )),
            origin,
        },
    }
}
fn invalid(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn conflict() -> ApiError {
    ApiError::new(
        ErrorCode::RevisionConflict,
        "Skip session revision or playback binding changed",
    )
}
fn not_found() -> ApiError {
    ApiError::new(
        ErrorCode::NotFound,
        "Skip session is unknown, closed or from another host",
    )
}
fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Explicit live skip host or observed seek submission unavailable",
    )
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
mod export_tests {
    use super::*;
    use crate::application::{
        playback_session::{
            self, NativeExportIssuerSourceV1, NativeSeekResultParts, NativeSeekResultSource, v1,
        },
        skip_segments::{CancelStatus, SkipResult, native_port_tests::native_sessions},
    };
    use std::sync::atomic::AtomicUsize;
    const SCOPE: &str = "nsi1.abcdefabcdefabcdefabcdefabcdefab";
    const REFERENCE: &str = "nsr1.abcdefabcdefabcdefabcdefabcdefab.0000000000000001";
    struct CaptureEngine {
        captures: Arc<AtomicUsize>,
        failure: Option<ErrorCode>,
    }
    impl playback_session::EpisodePlaybackEngine for CaptureEngine {
        fn native_export_issuer(&self) -> Result<v1::NativeExportIssuer, ApiError> {
            self.captures.fetch_add(1, Ordering::SeqCst);
            if let Some(code) = self.failure {
                return Err(ApiError::new(code, "original capture failure"));
            }
            Ok(issuer(Arc::new(AtomicUsize::new(0))))
        }
        fn open(
            &self,
            _: playback_session::EpisodePlaybackSource,
            _: bool,
            _: f64,
        ) -> Result<playback_session::OpenedEpisodePlayback, ApiError> {
            panic!("capture must not open media")
        }
        fn close(
            &self,
            _: playback_session::CloseEpisodePlayback,
        ) -> Result<playback_session::ClosedEpisodePlayback, ApiError> {
            panic!("capture must not close media")
        }
        fn observe(&self) -> Result<playback_session::EpisodePlaybackObservation, ApiError> {
            panic!("capture must not observe")
        }
        fn control(&self, _: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
            panic!("capture must not submit")
        }
    }
    struct CaptureResults;
    impl crate::application::episode_resolve::EpisodeResolutionResults for CaptureResults {
        fn publish(
            &self,
            _: &crate::application::episode_resolve::StoredEpisodeResolution,
            _: &crate::application::catalog::CaptureMetadata,
        ) -> Result<crate::application::jobs::VerifiedJobOutput, ApiError> {
            panic!("no publication")
        }
        fn read(
            &self,
            _: &str,
        ) -> Result<crate::application::episode_resolve::StoredEpisodeResolution, ApiError>
        {
            panic!("capture must not resolve media")
        }
    }
    fn capture_sessions(
        failure: Option<ErrorCode>,
    ) -> (Arc<EpisodePlaybackSessions>, Arc<AtomicUsize>) {
        let captures = Arc::new(AtomicUsize::new(0));
        (
            Arc::new(EpisodePlaybackSessions::new(
                Arc::new(CaptureResults),
                Arc::new(CaptureEngine {
                    captures: captures.clone(),
                    failure,
                }),
            )),
            captures,
        )
    }
    #[test]
    fn standalone_factory_captures_once_from_same_sessions_and_only_enables_export_wire() {
        let (sessions, captures) = capture_sessions(None);
        let registry =
            SkipHostRegistry::new_standalone_export_host("original".into(), sessions.clone(), 1)
                .unwrap();
        assert_eq!(captures.load(Ordering::SeqCst), 1);
        assert!(Arc::ptr_eq(
            &sessions,
            &registry.native_owner.as_ref().unwrap().sessions
        ));
        assert!(Arc::ptr_eq(
            &sessions,
            &registry.export_owner.as_ref().unwrap().sessions
        ));
        let port: Arc<dyn SkipPlaybackPort + Send + Sync> = sessions;
        assert!(Arc::ptr_eq(&port, &registry.port));
        assert!(registry.export_capability().available);
        assert!(!registry.capability().seek_submission);
        let mut request = Request::new(super::super::EXPORT_LIST);
        request.payload = serde_json::json!({"issuer_scope":SCOPE,"origin":origin()});
        let result = registry.execute(&request).unwrap();
        assert!(
            v1::encode_native_export_v1(&result).unwrap().len() <= v1::NATIVE_EXPORT_JSON_BYTES
        );
        registry.begin_close();
        assert!(!registry.export_capability().available);
        assert_eq!(
            registry.execute(&request).unwrap_err().code,
            ErrorCode::Unsupported
        );
    }
    #[test]
    fn standalone_factory_preserves_unsupported_and_original_capture_failure_without_fallback() {
        let (sessions, state) = native_sessions();
        assert_eq!(
            SkipHostRegistry::new_standalone_export_host("original".into(), sessions, 1)
                .err()
                .unwrap()
                .code,
            ErrorCode::Unsupported
        );
        assert_eq!(state.observes.load(Ordering::SeqCst), 0);
        assert_eq!(state.submits.load(Ordering::SeqCst), 0);
        for code in [ErrorCode::Unsupported, ErrorCode::EngineFailed] {
            let (sessions, captures) = capture_sessions(Some(code));
            let error =
                SkipHostRegistry::new_standalone_export_host("original".into(), sessions, 1)
                    .err()
                    .unwrap();
            assert_eq!(error.code, code);
            assert_eq!(error.message, "original capture failure");
            assert_eq!(captures.load(Ordering::SeqCst), 1);
        }
    }
    #[test]
    fn invalid_factory_budget_and_identity_are_rejected_before_capture() {
        let (sessions, captures) = capture_sessions(None);
        assert!(
            SkipHostRegistry::new_standalone_export_host("!".into(), sessions.clone(), 1).is_err()
        );
        assert!(
            SkipHostRegistry::new_standalone_export_host("original".into(), sessions, 0).is_err()
        );
        assert_eq!(captures.load(Ordering::SeqCst), 0);
    }
    #[test]
    fn default_native_and_external_issuer_factories_keep_wire_closed() {
        let (sessions, captures) = capture_sessions(None);
        let normal = SkipHostRegistry::new("original".into(), sessions.clone(), 1, false).unwrap();
        let native =
            SkipHostRegistry::new_native_owner("original".into(), sessions.clone(), 1).unwrap();
        let external = SkipHostRegistry::new_native_export_owner(
            "original".into(),
            sessions,
            issuer(Arc::new(AtomicUsize::new(0))),
            1,
        )
        .unwrap();
        for registry in [normal, native, external] {
            assert!(!registry.export_capability().available);
            assert!(!registry.capability().seek_submission);
        }
        assert_eq!(captures.load(Ordering::SeqCst), 0);
    }
    #[test]
    fn explicit_host_wire_schema_and_capabilities_agree_and_close_together() {
        let (sessions, _) = capture_sessions(None);
        let registry = Arc::new(
            SkipHostRegistry::new_standalone_export_host("original".into(), sessions.clone(), 1)
                .unwrap(),
        );
        let sdk = crate::application::api::PlayerSdk::new(
            crate::infrastructure::subtitle_files::SubtitleFiles,
        )
        .with_shared_episode_playback(sessions)
        .with_skip_host(registry.clone());
        for available in [true, false] {
            if !available {
                registry.begin_close();
            }
            let capabilities = sdk.execute(Request::new("capabilities")).data.unwrap();
            assert_eq!(capabilities["skip_exports"]["available"], available);
            for command in super::super::EXPORT_COMMANDS {
                assert!(capabilities["commands"].as_array().unwrap().iter().any(
                    |entry| entry["command"] == *command
                        && entry["status"]
                            == if available {
                                "supported"
                            } else {
                                "unsupported"
                            }
                ));
                let mut request = Request::new("schema");
                request.payload = serde_json::json!({"command":command});
                assert_eq!(
                    sdk.execute(request).data.unwrap()["status"],
                    if available {
                        "supported"
                    } else {
                        "unsupported"
                    }
                );
                assert_eq!(
                    super::super::schema::describe(
                        &serde_json::json!({"command":command}),
                        registry.capability()
                    )
                    .unwrap()["status"],
                    "unsupported"
                );
            }
            assert!(!registry.capability().seek_submission);
        }
    }
    #[derive(Debug)]
    struct IssuerSource {
        calls: Arc<AtomicUsize>,
    }
    impl NativeExportIssuerSourceV1 for IssuerSource {
        fn scope(&self) -> v1::NativeIssuerScopeV1 {
            SCOPE.parse().unwrap()
        }
        fn list(&self, _: &v1::NativeExportOriginV1) -> Result<v1::NativeExportListV1, ApiError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            v1::NativeExportListV1::new(self.scope(), Vec::new())
        }
        fn read(
            &self,
            _: v1::NativeResultRefV1,
            _: &v1::NativeExportOriginV1,
            _: bool,
        ) -> Result<v1::NativeExportResultV1, ApiError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Err(ApiError::new(
                ErrorCode::SourceExpired,
                "original synthetic owner expiry",
            ))
        }
    }
    fn issuer(calls: Arc<AtomicUsize>) -> v1::NativeExportIssuer {
        v1::NativeExportIssuer::from_original_source(Arc::new(IssuerSource { calls }))
    }
    fn origin() -> SkipExportOrigin {
        SkipExportOrigin {
            session: SessionRef {
                skip_session_id: "skip-original-1".into(),
                binding: crate::application::skip_segments::SkipBinding {
                    engine_instance_id: DecimalU64(7),
                    resolution_artifact_id: "a".repeat(64),
                },
            },
            pass_id: DecimalU64(1),
            request_revision: DecimalU64(2),
            segment_id: "op".into(),
        }
    }
    #[derive(Debug)]
    struct ResultSource;
    impl NativeSeekResultSource for ResultSource {
        fn identity(&self) -> (DecimalU64, String) {
            (DecimalU64(7), "a".repeat(64))
        }
        fn read(&self, _: bool) -> Result<NativeSeekResultParts, ApiError> {
            panic!("Projection must not issue a new read")
        }
    }
    fn original_export() -> v1::ExportedGuardedSeekV1 {
        let origin = v1::NativeExportOriginV1::try_from(v1::NativeExportOriginInputV1 {
            registry_generation: "original".into(),
            session: "skip-original-1".into(),
            pass: DecimalU64(1),
            segment: "op".into(),
            request_revision: DecimalU64(2),
            engine_instance_id: DecimalU64(7),
            resolution_artifact_id: "a".repeat(64),
        })
        .unwrap();
        let annotation =
            v1::NativeExportAnnotationV1::try_from(v1::NativeExportAnnotationInputV1 {
                annotation_revision: DecimalU64(11),
                verified_revision: DecimalU64(11),
                timebase: crate::domain::skip_segments::SegmentTimebase::EnginePlayback,
            })
            .unwrap();
        let metadata = v1::NativeExportMetadataV1::try_from(v1::NativeExportMetadataInputV1 {
            origin,
            annotation,
            actual_target_us: DecimalU64(20_000_000),
        })
        .unwrap();
        let submission = playback_session::native_seek_submission(
            ControlReceipt::enqueued(7),
            Arc::new(ResultSource),
        )
        .unwrap();
        v1::ExportedGuardedSeekV1::from_original(
            submission,
            REFERENCE.parse().unwrap(),
            metadata,
            SCOPE.parse().unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn post_admission_error_preserves_original_reference_and_independent_annotation_target() {
        let error = ApiError::new(ErrorCode::RevisionConflict, "original post-admission error");
        let result = project_export(Err(error), Some(original_export()), origin());
        let SkipExportAction::AdmittedWithError {
            reference,
            metadata,
            error,
            ..
        } = result
        else {
            panic!("Original Ok export lost")
        };
        assert_eq!(reference.to_string(), REFERENCE);
        assert_eq!(metadata.origin().request_revision(), DecimalU64(2));
        assert_eq!(metadata.annotation().annotation_revision(), DecimalU64(11));
        assert_eq!(metadata.actual_target_us(), DecimalU64(20_000_000));
        let value = serde_json::to_value(error).unwrap();
        assert_eq!(value["code"], "revision_conflict");
        assert_eq!(value["message_prefix"], "original post-admission error");
    }
    #[test]
    fn missing_submission_is_explicit_and_cancel_does_not_invent_reference_or_target() {
        let result = project_export(
            Err(ApiError::new(ErrorCode::EngineFailed, "ambiguous original")),
            None,
            origin(),
        );
        assert!(matches!(
            result,
            SkipExportAction::NoSubmissionReturned { .. }
        ));
        let result = project_export(
            Ok(SkipResult::Cancelled {
                session_revision: DecimalU64(3),
                status: CancelStatus::BeforeAdmission,
            }),
            None,
            origin(),
        );
        let value = serde_json::to_value(result).unwrap();
        assert_eq!(value["state"], "cancelled");
        assert!(value.get("reference").is_none());
        assert!(value.get("actual_target_us").is_none());
    }
    #[test]
    fn original_reads_do_not_lookup_current_session_or_observe_current_route() {
        let (sessions, state) = native_sessions();
        let calls = Arc::new(AtomicUsize::new(0));
        let registry = SkipHostRegistry::new_native_export_owner(
            "original".into(),
            sessions,
            issuer(calls.clone()),
            1,
        )
        .unwrap();
        assert!(!registry.export_capability().available);
        let list = registry
            .list_native_exports(ListSkipExports {
                issuer_scope: SCOPE.parse().unwrap(),
                origin: origin(),
            })
            .unwrap();
        assert!(list.items().is_empty()); // There is no current skip entry to reconstruct.
        let error = registry
            .consume_native_export(ReadSkipExport {
                issuer_scope: SCOPE.parse().unwrap(),
                reference: REFERENCE.parse().unwrap(),
                origin: origin(),
            })
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::SourceExpired);
        assert_eq!(error.message, "original synthetic owner expiry");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(state.legacy_reads.load(Ordering::SeqCst), 0);
        assert_eq!(state.observes.load(Ordering::SeqCst), 0);
        assert_eq!(state.submits.load(Ordering::SeqCst), 0);
        let foreign = "nsr1.12345678901234567890123456789012.0000000000000001"
            .parse()
            .unwrap();
        assert_eq!(
            registry
                .peek_native_export(ReadSkipExport {
                    issuer_scope: SCOPE.parse().unwrap(),
                    reference: foreign,
                    origin: origin()
                })
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
    #[test]
    fn stale_then_unverified_cancel_keep_core_order_and_perform_no_export_observe() {
        use crate::domain::skip_segments::{Intent, SegmentKind};
        let (sessions, state) = native_sessions();
        let registry = SkipHostRegistry::new_native_export_owner(
            "cancel-order".into(),
            sessions,
            issuer(Arc::new(AtomicUsize::new(0))),
            1,
        )
        .unwrap();
        let reference = registry
            .open(OpenSkipSession {
                binding: crate::application::skip_segments::SkipBinding {
                    engine_instance_id: DecimalU64(7),
                    resolution_artifact_id: "episode".into(),
                },
            })
            .unwrap()
            .session;
        registry
            .edit(
                SessionRequest {
                    session: reference.clone(),
                    input: EditInput {
                        segment_id: "op".into(),
                        kind: SegmentKind::Opening,
                        in_us: DecimalU64(10_000_000),
                        out_us: DecimalU64(20_000_000),
                    },
                },
                DecimalU64(1),
            )
            .unwrap();
        let calls_before = state.legacy_reads.load(Ordering::SeqCst);
        let input = SkipExportRequest {
            issuer_scope: SCOPE.parse().unwrap(),
            session: reference,
            input: SkipInput {
                segment_id: "op".into(),
                intent: Intent::Manual,
                cancelled: true,
            },
        };
        let stale = registry
            .skip_native_export_call(input.clone(), DecimalU64(1))
            .unwrap();
        assert!(matches!(
            stale.result,
            SkipExportAction::NoSubmissionReturned { .. }
        ));
        assert_eq!(stale.session_state.revision, DecimalU64(2));
        let mut observe = input.clone();
        observe.input.cancelled = false;
        let unsupported = registry
            .skip_native_export_call(observe, DecimalU64(2))
            .unwrap();
        let SkipExportAction::NoSubmissionReturned { error, .. } = unsupported.result else {
            panic!("P0 default Unsupported must not fall back to legacy observation")
        };
        assert_eq!(serde_json::to_value(error).unwrap()["code"], "unsupported");
        assert_eq!(unsupported.session_state.revision, DecimalU64(2));
        let cancelled = registry
            .skip_native_export_call(input, DecimalU64(2))
            .unwrap();
        assert!(matches!(
            cancelled.result,
            SkipExportAction::Cancelled {
                status: CancelStatus::BeforeAdmission
            }
        ));
        assert_eq!(cancelled.session_state.revision, DecimalU64(3));
        assert_eq!(state.observes.load(Ordering::SeqCst), 0);
        assert_eq!(state.submits.load(Ordering::SeqCst), 0);
        assert_eq!(state.legacy_reads.load(Ordering::SeqCst), calls_before);
    }
}
