//! Explicit native lifetime; no GUI events, operating-system API or engine command strings.
#[path = "ports.rs"]
mod ports;
pub use ports::EpisodePlaybackEngine;
mod guarded;
mod native_result_export;
pub(crate) use native_result_export::NativeExportIssuerSourceV1;
pub use native_result_export::v1;
pub use native_result_export::{
    ExportGuardedSeek, ExportedGuardedSeekSubmission, NativeExportResultQuery,
    NativeResultProvenance, NativeResultReference, PreparedNativeResultExport,
};
pub(crate) use native_result_export::{NativeExportReadOwner, PreparedNativeExportReadOwner};
mod options;
pub use guarded::{
    FileAttribution, GuardedSeek, GuardedSeekFailure, GuardedSeekFailureStage, GuardedSeekResult,
    GuardedSeekResultQuery, GuardedSeekState, GuardedSeekSubmission, NativeGuardIdentity,
    NativeGuardObservation, NativeGuardView, NativeObservationGuard, NativePropertySample,
    NativeReceiveObservation, NativeSeekTicket,
};
pub(crate) use guarded::{NativeGuardAuthority, NativeGuardSnapshot, native_guard_from_authority};
pub(crate) use guarded::{NativeSeekResultParts, NativeSeekResultSource, native_seek_submission};
#[cfg(test)]
mod guarded_tests;
#[cfg(test)]
pub(crate) use guarded::test_observation as test_guarded_observation;
#[cfg(test)]
mod tests;
pub mod transition;
mod types;
pub use options::{
    PlaybackOptionsObservation, SelectEpisodeTrack, SetEpisodeRate, TrackControlReceipt,
};
pub use types::{
    CaptureEpisodeFrame, CapturedEpisodeFrame, CloseEpisodePlayback, ClosedEpisodePlayback,
    ControlEpisodePlayback, EpisodePlaybackObservation, EpisodePlaybackSource,
    NativeEngineDiagnostics, OpenEpisodePlayback, OpenedEpisodePlayback,
};

use crate::application::{
    api::{ApiError, ErrorCode},
    episode_resolve::EpisodeResolutionResults,
};
use std::sync::Arc;

pub struct EpisodePlaybackSessions {
    results: Arc<dyn EpisodeResolutionResults>,
    engine: Arc<dyn EpisodePlaybackEngine>,
}

impl EpisodePlaybackSessions {
    pub fn native_export_issuer(&self) -> Result<v1::NativeExportIssuer, ApiError> {
        self.engine.native_export_issuer()
    }
    pub fn observe_native_guard_for_export(
        &self,
        issuer: &v1::NativeExportIssuer,
    ) -> Result<NativeGuardObservation, ApiError> {
        self.engine.observe_native_guard_for_export(issuer)
    }
    pub fn submit_exported_guarded_seek_v1(
        &self,
        input: v1::ExportGuardedSeekV1,
    ) -> Result<v1::ExportedGuardedSeekV1, ApiError> {
        self.engine.submit_exported_guarded_seek_v1(input)
    }
    pub fn submit_exported_guarded_seek(
        &self,
        input: ExportGuardedSeek,
    ) -> Result<ExportedGuardedSeekSubmission, ApiError> {
        self.engine.submit_exported_guarded_seek(input)
    }
    pub fn exported_seek_references(
        &self,
        provenance: NativeResultProvenance,
    ) -> Result<Vec<NativeResultReference>, ApiError> {
        self.engine.exported_seek_references(provenance)
    }
    pub fn exported_seek_result(
        &self,
        input: NativeExportResultQuery,
    ) -> Result<GuardedSeekResult, ApiError> {
        self.engine.exported_seek_result(input)
    }
    pub fn observe_native_guard(&self) -> Result<NativeGuardObservation, ApiError> {
        self.engine.observe_native_guard()
    }
    pub fn submit_guarded_seek(
        &self,
        input: GuardedSeek,
    ) -> Result<GuardedSeekSubmission, ApiError> {
        self.engine.submit_guarded_seek(input)
    }
    pub fn guarded_seek_result(
        &self,
        input: GuardedSeekResultQuery,
    ) -> Result<GuardedSeekResult, ApiError> {
        self.engine.guarded_seek_result(input)
    }
    pub fn options_available(&self) -> bool {
        self.engine.options_available()
    }
    pub fn set_rate(
        &self,
        input: SetEpisodeRate,
    ) -> Result<crate::application::playback_control::ControlReceipt, ApiError> {
        input.validate()?;
        self.engine.set_rate(input)
    }
    pub fn select_track(&self, input: SelectEpisodeTrack) -> Result<TrackControlReceipt, ApiError> {
        input.validate()?;
        self.engine.select_track(input)
    }
    pub fn capture_frame_available(&self) -> bool {
        self.engine.capture_frame_available()
    }
    pub fn capture_frame(
        &self,
        input: CaptureEpisodeFrame,
    ) -> Result<CapturedEpisodeFrame, ApiError> {
        input.validate()?;
        self.engine.capture_frame(input)
    }
    pub fn new(
        results: Arc<dyn EpisodeResolutionResults>,
        engine: Arc<dyn EpisodePlaybackEngine>,
    ) -> Self {
        Self { results, engine }
    }
    pub fn open(&self, input: OpenEpisodePlayback) -> Result<OpenedEpisodePlayback, ApiError> {
        input.validate()?;
        let result = self.results.read(&input.resolution_artifact_id)?;
        let source = EpisodePlaybackSource::from_result(input.resolution_artifact_id, result)?;
        self.engine.open(source, input.paused, input.volume_percent)
    }
    pub fn close(&self, input: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
        if input.engine_instance_id.0 == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Native engine instance ID must be nonzero",
            ));
        }
        self.engine.close(input)
    }
    pub fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
        self.engine.observe()
    }
    pub fn control(
        &self,
        input: ControlEpisodePlayback,
    ) -> Result<crate::application::playback_control::ControlReceipt, ApiError> {
        if input.engine_instance_id.0 == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Native engine instance ID must be nonzero",
            ));
        }
        input
            .control
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        self.engine.control(input)
    }
}
