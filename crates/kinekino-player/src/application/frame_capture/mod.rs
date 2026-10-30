#[path = "ports.rs"]
mod ports;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{
        api_values::DecimalU64,
        frame_capture::{CaptureFrame, MAX_CAPTURE_TIMEOUT_MS},
    },
};
pub use ports::FrameCapturePort;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FrameCaptureCapability {
    pub available: bool,
    pub adapter: String,
    pub ffmpeg_path: Option<String>,
    pub ffprobe_path: Option<String>,
    pub ffmpeg_version: Option<String>,
    pub ffprobe_version: Option<String>,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CapturedFrame {
    pub output_path: String,
    pub requested_source_pts_us: DecimalU64,
    pub actual_source_pts_us: DecimalU64,
    pub source_pts_ticks: String,
    pub source_time_base: String,
    pub width: u32,
    pub height: u32,
    pub output_bytes: u64,
    /// Digest of the published PNG bytes. Legacy receipts do not provide this identity.
    #[serde(default)]
    pub output_sha256: Option<String>,
    pub source_color_space: Option<String>,
    pub source_color_transfer: Option<String>,
    pub rotation_applied: bool,
    pub source_rotation_degrees: Option<i32>,
    pub subtitle_mode: String,
    pub limitations: Vec<String>,
    pub dependency: FrameCaptureCapability,
}

pub struct UnavailableFrameCapture;
impl FrameCapturePort for UnavailableFrameCapture {
    fn capability(&self) -> FrameCaptureCapability {
        FrameCaptureCapability {
            adapter: "unavailable".into(),
            unavailable_reason: Some("No frame decoder adapter configured".into()),
            ..Default::default()
        }
    }
    fn capture(&self, _: &CaptureFrame, _: u64) -> Result<CapturedFrame, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Frame capture adapter unavailable",
        ))
    }
}

pub fn execute(
    port: &impl FrameCapturePort,
    payload: &Value,
    timeout: Option<u64>,
) -> Result<Value, ApiError> {
    execute_controlled(port, payload, timeout, None)
}

pub fn execute_controlled(
    port: &impl FrameCapturePort,
    payload: &Value,
    timeout: Option<u64>,
    control: Option<&dyn crate::application::review_source::MutationControl>,
) -> Result<Value, ApiError> {
    execute_recoverable(
        port,
        payload,
        timeout,
        FrameExecutionContext {
            control,
            recovery: None,
        },
    )
}

#[derive(Clone, Copy, Default)]
pub struct FrameExecutionContext<'a> {
    pub control: Option<&'a dyn crate::application::review_source::MutationControl>,
    pub recovery: Option<&'a dyn crate::application::sdk_execution::ReceiptRecoveryControl>,
}
pub fn execute_recoverable(
    port: &impl FrameCapturePort,
    payload: &Value,
    timeout: Option<u64>,
    context: FrameExecutionContext<'_>,
) -> Result<Value, ApiError> {
    let request: CaptureFrame = serde_json::from_value(payload.clone()).map_err(|_| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Expected source_path, source_pts_us decimal string and output_path",
        )
    })?;
    request
        .validate()
        .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
    let timeout = timeout.unwrap_or(30_000);
    if timeout == 0 || timeout > MAX_CAPTURE_TIMEOUT_MS {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Capture timeout must be between 1 and 120000 ms",
        ));
    }
    Ok(json!(port.capture_recoverable(&request, timeout, context)?))
}

pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["source_path","source_pts_us","output_path"],"properties":{"source_path":{"type":"string","description":"Absolute local MP4/MKV/WebM path, maximum 16 GiB; no URLs, UNC or playlists"},"source_pts_us":{"type":"string","pattern":"^[0-9]+$","description":"Nonnegative original source PTS in integer microseconds; selects first decoded frame at or after request"},"output_path":{"type":"string","description":"New absolute local .png path; never overwritten"}}})
}
