//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait EpisodePlaybackEngine: Send + Sync {
    fn native_export_issuer(&self) -> Result<v1::NativeExportIssuer, ApiError> {
        Err(guarded::unsupported())
    }
    fn observe_native_guard_for_export(
        &self,
        _issuer: &v1::NativeExportIssuer,
    ) -> Result<NativeGuardObservation, ApiError> {
        Err(guarded::unsupported())
    }
    fn submit_exported_guarded_seek_v1(
        &self,
        _input: v1::ExportGuardedSeekV1,
    ) -> Result<v1::ExportedGuardedSeekV1, ApiError> {
        Err(guarded::unsupported())
    }
    fn exported_seek_references(
        &self,
        _provenance: NativeResultProvenance,
    ) -> Result<Vec<NativeResultReference>, ApiError> {
        Err(guarded::unsupported())
    }
    fn exported_seek_result(
        &self,
        _input: NativeExportResultQuery,
    ) -> Result<GuardedSeekResult, ApiError> {
        Err(guarded::unsupported())
    }
    /// Explicit opt-in only. Wrappers must not forward through legacy submission.
    fn submit_exported_guarded_seek(
        &self,
        _input: ExportGuardedSeek,
    ) -> Result<ExportedGuardedSeekSubmission, ApiError> {
        Err(guarded::unsupported())
    }
    fn observe_native_guard(&self) -> Result<NativeGuardObservation, ApiError> {
        Err(guarded::unsupported())
    }
    fn submit_guarded_seek(&self, _input: GuardedSeek) -> Result<GuardedSeekSubmission, ApiError> {
        Err(guarded::unsupported())
    }
    fn guarded_seek_result(
        &self,
        _input: GuardedSeekResultQuery,
    ) -> Result<GuardedSeekResult, ApiError> {
        Err(guarded::unsupported())
    }
    fn options_available(&self) -> bool {
        false
    }
    fn set_rate(
        &self,
        _input: SetEpisodeRate,
    ) -> Result<crate::application::playback_control::ControlReceipt, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Native playback rate unavailable",
        ))
    }
    fn select_track(&self, _input: SelectEpisodeTrack) -> Result<TrackControlReceipt, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Native playback track selection unavailable",
        ))
    }
    fn capture_frame_available(&self) -> bool {
        false
    }
    fn capture_frame(&self, _input: CaptureEpisodeFrame) -> Result<CapturedEpisodeFrame, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Native frame capture unavailable",
        ))
    }
    /// Refuse an already owned active instance. Do not silently replace or share a GUI process.
    fn open(
        &self,
        source: EpisodePlaybackSource,
        paused: bool,
        volume_percent: f64,
    ) -> Result<OpenedEpisodePlayback, ApiError>;
    /// Match the instance atomically before closing; report observed exit, not queue acceptance.
    fn close(&self, input: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError>;
    fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError>;
    fn control(
        &self,
        input: ControlEpisodePlayback,
    ) -> Result<crate::application::playback_control::ControlReceipt, ApiError>;
}
