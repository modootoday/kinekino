//! Original SDK capture envelope cross-check. No path execution or inferred defaults.
use crate::{
    application::api::{ApiError, ErrorCode, Request},
    domain::{
        api_values::DecimalU64,
        frame_capture::CaptureFrame,
        media_timeline::{Rounding, TimeBase, TimelineTransform},
    },
};
use serde::Deserialize;
use serde_json::Value;
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CaptureContract {
    SdkFirstSourceFrameAtOrAfter,
}
pub(super) struct CaptureExpectation<'a> {
    pub source_sha: &'a str,
    pub engine_sha: &'a str,
    pub probe_sha: &'a str,
    pub producer_cli_sha: &'a str,
    pub requested_us: u64,
    pub actual_pts: i64,
    pub time_base: TimeBase,
    pub png_sha: &'a str,
    pub png_bytes: u64,
    pub dimensions: (u32, u32),
}
#[derive(Deserialize)]
struct Pin {
    sha256: String,
    file_identity: String,
}
#[derive(Deserialize)]
struct Dependency {
    available: bool,
    adapter: String,
    ffmpeg_path: String,
    ffprobe_path: String,
}
#[derive(Deserialize)]
struct Inputs {
    source: String,
    original: Pin,
    cli: Pin,
    ffmpeg: Pin,
    ffprobe: Pin,
    dependency: Dependency,
    model_output: bool,
    actual9_binding: bool,
    quality_approved: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Process {
    cli: Pin,
    elapsed_ms: u64,
    exit_code: i32,
    pid: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    api_version: String,
    request_id: String,
    ok: bool,
    data: Frame,
    error: Value,
}
#[derive(Deserialize)]
struct Frame {
    output_path: String,
    requested_source_pts_us: DecimalU64,
    actual_source_pts_us: DecimalU64,
    source_pts_ticks: String,
    source_time_base: String,
    width: u32,
    height: u32,
    output_bytes: u64,
    output_sha256: String,
    dependency: Dependency,
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, ApiError> {
    serde_json::from_slice(bytes).map_err(|_| invalid())
}
pub(super) fn verify(
    _contract: &CaptureContract,
    inputs: &[u8],
    request: &[u8],
    response: &[u8],
    process: &[u8],
    expected: &CaptureExpectation<'_>,
) -> Result<(), ApiError> {
    let input: Inputs = decode(inputs)?;
    let request: Request = decode(request)?;
    request.validate()?;
    let payload: CaptureFrame =
        serde_json::from_value(request.payload.clone()).map_err(|_| invalid())?;
    payload.validate().map_err(|_| invalid())?;
    let response: Response = decode(response)?;
    let process: Process = decode(process)?;
    if request.command != "production.capture_frame"
        || request.context.expected_revision.is_some()
        || request.context.idempotency_key.is_some()
        || request
            .context
            .timeout_ms
            .is_none_or(|value| value == 0 || value > 120000)
        || payload.source_path != input.source
        || payload.source_pts_us.0 != expected.requested_us
    {
        return Err(invalid());
    }
    if input.original.sha256 != expected.source_sha
        || input.ffmpeg.sha256 != expected.engine_sha
        || input.ffprobe.sha256 != expected.probe_sha
        || input.cli.sha256 != expected.producer_cli_sha
        || input.model_output
        || input.actual9_binding
        || input.quality_approved
        || [
            &input.original.file_identity,
            &input.cli.file_identity,
            &input.ffmpeg.file_identity,
            &input.ffprobe.file_identity,
        ]
        .iter()
        .any(|value| value.is_empty())
    {
        return Err(invalid());
    }
    if process.exit_code != 0
        || process.pid == 0
        || process.cli.sha256 != input.cli.sha256
        || process.cli.file_identity != input.cli.file_identity
    {
        return Err(invalid());
    }
    // Elapsed is observed metadata, never a substitute for terminal exit status.
    let _observed_elapsed = process.elapsed_ms;
    if !response.ok
        || !response.error.is_null()
        || response.api_version != "1"
        || response.request_id != request.context.request_id
    {
        return Err(invalid());
    }
    let frame = response.data;
    for dependency in [&input.dependency, &frame.dependency] {
        if !dependency.available || dependency.adapter != "external_ffmpeg_cpu" {
            return Err(invalid());
        }
    }
    if input.dependency.ffmpeg_path != frame.dependency.ffmpeg_path
        || input.dependency.ffprobe_path != frame.dependency.ffprobe_path
    {
        return Err(invalid());
    }
    let ticks = frame
        .source_pts_ticks
        .parse::<i64>()
        .map_err(|_| invalid())?;
    let base = TimeBase::parse(&frame.source_time_base).map_err(|_| invalid())?;
    let actual_us = TimelineTransform::new(base, 0)
        .pts_to_us(ticks, Rounding::Down)
        .map_err(|_| invalid())?
        .value;
    if ticks.to_string() != frame.source_pts_ticks
        || ticks != expected.actual_pts
        || base != expected.time_base
        || frame.requested_source_pts_us.0 != expected.requested_us
        || frame.actual_source_pts_us.0 != actual_us
        || actual_us < expected.requested_us
        || frame.output_path != payload.output_path
        || frame.output_sha256 != expected.png_sha
        || frame.output_bytes != expected.png_bytes
        || (frame.width, frame.height) != expected.dimensions
    {
        return Err(invalid());
    }
    Ok(())
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::RevisionConflict,
        "Original capture request, response, source or producer binding differs",
    )
}
#[cfg(test)]
mod tests;
