//! One synchronous skip call; no policy, shared guard cache, or result registry.
use super::{SkipBinding, SkipObservation, SkipPlaybackPort, SkipResult, SkipSnapshot};
use crate::application::playback_session::v1;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_control::ControlReceipt,
        playback_session::{
            ControlEpisodePlayback, EpisodePlaybackSessions, GuardedSeek, GuardedSeekSubmission,
            NativeGuardView, NativeObservationGuard,
        },
    },
    domain::playback_control::PlaybackControl,
};
use std::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};

struct ExportCall<'a> {
    issuer: &'a v1::NativeExportIssuer,
    origin: v1::NativeExportOriginInputV1,
    annotation: Option<v1::NativeExportAnnotationInputV1>,
}

/// Rust-only owner-bearing result. An API failure after admission cannot discard its ticket.
#[derive(Debug)]
pub struct NativeSkipOutcome {
    pub result: Result<SkipResult, ApiError>,
    pub submission: Option<GuardedSeekSubmission>,
    pub snapshot: SkipSnapshot,
}
#[derive(Default)]
struct CallState {
    observed: bool,
    view: Option<NativeGuardView>,
    guard: Option<NativeObservationGuard>,
    submitted: bool,
    submission: Option<GuardedSeekSubmission>,
    exported: Option<v1::ExportedGuardedSeekV1>,
}
pub(super) struct NativeSkipCall<'a> {
    sessions: &'a EpisodePlaybackSessions,
    closing: &'a AtomicBool,
    state: RefCell<CallState>,
    export: Option<ExportCall<'a>>,
}
impl<'a> NativeSkipCall<'a> {
    pub(super) fn new(sessions: &'a EpisodePlaybackSessions, closing: &'a AtomicBool) -> Self {
        Self {
            sessions,
            closing,
            state: RefCell::new(CallState::default()),
            export: None,
        }
    }
    pub(super) fn into_submission(self) -> Option<GuardedSeekSubmission> {
        self.state.into_inner().submission
    }
    pub(super) fn new_export(
        sessions: &'a EpisodePlaybackSessions,
        closing: &'a AtomicBool,
        issuer: &'a v1::NativeExportIssuer,
        origin: v1::NativeExportOriginInputV1,
        annotation: Option<v1::NativeExportAnnotationInputV1>,
    ) -> Self {
        let mut call = Self::new(sessions, closing);
        call.export = Some(ExportCall {
            issuer,
            origin,
            annotation,
        });
        call
    }
    pub(super) fn into_exported(self) -> Option<v1::ExportedGuardedSeekV1> {
        self.state.into_inner().exported
    }
    fn require_open(&self) -> Result<(), ApiError> {
        if self.closing.load(Ordering::Acquire) {
            Err(ApiError::new(
                ErrorCode::Unsupported,
                "Skip host closing; native submission unavailable",
            ))
        } else {
            Ok(())
        }
    }
}
fn busy() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "Native skip call already borrowed",
    )
}
fn projection(view: &NativeGuardView) -> SkipObservation {
    SkipObservation {
        binding: SkipBinding {
            engine_instance_id: view.identity().engine_instance_id(),
            resolution_artifact_id: view.identity().resolution_artifact_id().into(),
        },
        playback: view.playback().clone(),
        seekability: view.seekability(),
        observation_authorized: view.observation_authorized(),
    }
}
impl SkipPlaybackPort for NativeSkipCall<'_> {
    fn observe(&self) -> Result<SkipObservation, ApiError> {
        self.require_open()?;
        {
            let mut state = self.state.try_borrow_mut().map_err(|_| busy())?;
            if let Some(view) = &state.view {
                return Ok(projection(view));
            }
            if state.observed {
                return Err(ApiError::new(
                    ErrorCode::NeedsInput,
                    "Original native observation failed; no implicit retry",
                ));
            }
            state.observed = true;
        }
        // No RefCell borrow or skip callback is passed into the SDK observation wait.
        let observation = match &self.export {
            Some(export) => self
                .sessions
                .observe_native_guard_for_export(export.issuer)?,
            None => self.sessions.observe_native_guard()?,
        };
        let (view, guard) = observation.into_guarded_parts()?;
        let observation = projection(&view);
        let mut state = self.state.try_borrow_mut().map_err(|_| busy())?;
        state.view = Some(view);
        state.guard = Some(guard);
        Ok(observation)
    }
    fn submit(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        self.require_open()?;
        let PlaybackControl::Seek { position_us } = input.control else {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Native skip accepts only the original seek target",
            ));
        };
        let mut state = self.state.try_borrow_mut().map_err(|_| busy())?;
        if state.submitted {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native skip guard already consumed",
            ));
        }
        let view = state.view.as_ref().ok_or_else(|| {
            ApiError::new(
                ErrorCode::NeedsInput,
                "Native skip requires the original observation",
            )
        })?;
        if view.identity().engine_instance_id() != input.engine_instance_id {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native skip command differs from observed engine",
            ));
        }
        let guard = state.guard.take().ok_or_else(|| {
            ApiError::new(
                ErrorCode::NeedsInput,
                "Native skip has no unused original guard",
            )
        })?;
        state.submitted = true;
        self.require_open()?;
        // This borrow is private to this stack call, never an Active/registry/router lock.
        // Establish it before admission so storing an Ok submission cannot fail afterwards.
        if let Some(export) = &self.export {
            let origin = v1::NativeExportOriginV1::try_from(export.origin.clone())
                .map_err(ApiError::from)?;
            let annotation = export.annotation.ok_or_else(|| {
                ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Export requires the selected original annotation",
                )
            })?;
            let annotation =
                v1::NativeExportAnnotationV1::try_from(annotation).map_err(ApiError::from)?;
            let exported =
                self.sessions
                    .submit_exported_guarded_seek_v1(v1::ExportGuardedSeekV1 {
                        seek: GuardedSeek { guard, position_us },
                        export: v1::PreparedNativeResultExportV1::new(origin, annotation),
                    })?;
            let receipt = exported.receipt().clone();
            state.exported = Some(exported);
            return Ok(receipt);
        }
        let submission = self
            .sessions
            .submit_guarded_seek(GuardedSeek { guard, position_us })?;
        let receipt = submission.receipt().clone();
        state.submission = Some(submission);
        Ok(receipt)
    }
}
