use crate::{
    application::{
        api::{ApiError, ErrorCode},
        episode_resolve::StoredEpisodeResolution,
        playback_control::ControlCompletion,
        playback_status::PlaybackStatus,
    },
    domain::{api_values::DecimalU64, request_referer::RequestReferer},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpenEpisodePlayback {
    pub resolution_artifact_id: String,
    pub paused: bool,
    pub volume_percent: f64,
}

impl OpenEpisodePlayback {
    pub(super) fn validate(&self) -> Result<(), ApiError> {
        if !crate::domain::identifiers::valid_sha256(&self.resolution_artifact_id)
            || !self.volume_percent.is_finite()
            || !(0.0..=100.0).contains(&self.volume_percent)
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid episode playback request",
            ));
        }
        Ok(())
    }
}

/// Constructed only after the application result reader has reverified stored evidence.
/// Historical bytes do not prove current URL validity or transferable browser credentials.
pub struct EpisodePlaybackSource {
    artifact_id: String,
    url: String,
    referer: RequestReferer,
}

impl EpisodePlaybackSource {
    #[cfg(test)]
    pub(crate) fn synthetic_replacement_source(referer: RequestReferer) -> Self {
        Self {
            artifact_id: "b".repeat(64),
            url: "https://media.example.invalid/fixture.m3u8".into(),
            referer,
        }
    }
    pub(super) fn from_result(
        artifact_id: String,
        result: StoredEpisodeResolution,
    ) -> Result<Self, ApiError> {
        result.validate()?;
        Ok(Self {
            artifact_id,
            url: result.manifest.url,
            referer: result.manifest.referer,
        })
    }
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn referer(&self) -> &RequestReferer {
        &self.referer
    }
}

/// Process creation only; loading, decoding and command completion remain unobserved.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpenedEpisodePlayback {
    pub engine_instance_id: DecimalU64,
    pub resolution_artifact_id: String,
    pub completion: ControlCompletion,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CloseEpisodePlayback {
    pub engine_instance_id: DecimalU64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ControlEpisodePlayback {
    pub engine_instance_id: DecimalU64,
    pub control: crate::domain::playback_control::PlaybackControl,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClosedEpisodePlayback {
    pub engine_instance_id: DecimalU64,
    pub engine_exited: bool,
    pub ipc_worker_exited: bool,
    pub stderr_worker_exited: bool,
    pub stdout_worker_exited: bool,
    pub diagnostics: NativeEngineDiagnostics,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodePlaybackObservation {
    pub engine_instance_id: DecimalU64,
    pub resolution_artifact_id: String,
    pub playback: PlaybackStatus,
    pub diagnostics: Option<NativeEngineDiagnostics>,
    #[serde(default)]
    pub options: Option<super::PlaybackOptionsObservation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeEngineDiagnostics {
    pub artifact: crate::application::catalog::RawArtifact,
    pub stderr_sha256: String,
    pub stderr_bytes: DecimalU64,
    pub stdout_sha256: String,
    pub stdout_bytes: DecimalU64,
    pub complete: bool,
    pub truncated: bool,
    pub read_failed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureEpisodeFrame {
    pub engine_instance_id: DecimalU64,
    pub output_path: String,
}
impl CaptureEpisodeFrame {
    pub(super) fn validate(&self) -> Result<(), ApiError> {
        if self.engine_instance_id.0 == 0
            || !crate::domain::frame_capture::is_absolute_local_path(&self.output_path)
            || self.output_path.len() > 32767
            || self.output_path.chars().any(char::is_control)
            || !self.output_path.to_ascii_lowercase().ends_with(".png")
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Native capture requires an instance and a new absolute local PNG path",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapturedEpisodeFrame {
    pub engine_instance_id: DecimalU64,
    pub resolution_artifact_id: String,
    pub output_path: String,
    pub output_sha256: String,
    pub output_bytes: DecimalU64,
    pub width: u32,
    pub height: u32,
    pub position_before_us: Option<DecimalU64>,
    pub position_after_us: Option<DecimalU64>,
    pub frame_time_exact: bool,
    pub subtitles_included: bool,
    pub window_capture: bool,
    pub color_parity_verified: bool,
}
