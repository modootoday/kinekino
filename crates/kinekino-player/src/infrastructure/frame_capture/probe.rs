use super::{process, seek::TimeBase};
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::frame_capture::MAX_FRAME_PIXELS,
};
use serde_json::Value;
use std::{path::Path, process::Command, time::Instant};

pub struct MediaInfo {
    pub rotation: Option<i32>,
    pub width: u32,
    pub height: u32,
    pub color_space: Option<String>,
    pub color_transfer: Option<String>,
    pub start: u64,
    pub time_base: TimeBase,
    end: u64,
}
impl MediaInfo {
    pub fn validate(&self, pts: u64) -> Result<(), ApiError> {
        if self.width == 0
            || self.height == 0
            || u64::from(self.width) * u64::from(self.height) > MAX_FRAME_PIXELS
        {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Frame dimensions exceed 3840x2160 pixel budget",
            ));
        }
        if pts < self.start || pts >= self.end {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Requested source PTS is outside probed source duration",
            ));
        }
        if self
            .color_transfer
            .as_deref()
            .is_some_and(|value| matches!(value, "smpte2084" | "arib-std-b67"))
        {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "HDR capture requires a verified tone-mapping adapter",
            ));
        }
        Ok(())
    }
}
pub fn inspect(
    tool: &str,
    input: &Path,
    demuxer: &str,
    deadline: Instant,
    control: Option<&dyn crate::application::review_source::MutationControl>,
    recovery: Option<&dyn crate::application::sdk_execution::ReceiptRecoveryControl>,
) -> Result<MediaInfo, ApiError> {
    let mut command = Command::new(tool);
    command.args(["-v", "error", "-protocol_whitelist", "file", "-f", demuxer]);
    if demuxer == "mov" {
        command.args(["-enable_drefs", "0"]);
    }
    command
        .args([
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,color_space,color_transfer,start_time,time_base:stream_side_data=rotation:format=duration,start_time",
            "-of",
            "json",
        ])
        .arg(input);
    let output = process::run_with_recovery(&mut command, deadline, 65536, control, recovery)?;
    let json: Value = serde_json::from_slice(&output.stdout).map_err(|_| failed())?;
    let stream = json["streams"]
        .as_array()
        .and_then(|v| v.first())
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::Unsupported,
                "Source has no decodable video stream",
            )
        })?;
    let width = stream["width"]
        .as_u64()
        .and_then(|v| v.try_into().ok())
        .ok_or_else(failed)?;
    let height = stream["height"]
        .as_u64()
        .and_then(|v| v.try_into().ok())
        .ok_or_else(failed)?;
    let start = stream["start_time"]
        .as_str()
        .or_else(|| json["format"]["start_time"].as_str())
        .map(seconds_to_us)
        .transpose()?
        .unwrap_or(0);
    let duration = seconds_to_us(json["format"]["duration"].as_str().ok_or_else(failed)?)?;
    let end = start.checked_add(duration).ok_or_else(failed)?;
    Ok(MediaInfo {
        rotation: stream["side_data_list"]
            .as_array()
            .and_then(|list| list.iter().find_map(|value| value["rotation"].as_i64()))
            .and_then(|value| value.try_into().ok()),
        width,
        height,
        start,
        end,
        time_base: TimeBase::parse(stream["time_base"].as_str().ok_or_else(failed)?)?,
        color_space: stream["color_space"].as_str().map(str::to_owned),
        color_transfer: stream["color_transfer"].as_str().map(str::to_owned),
    })
}

pub fn selected_pts(log: &[u8]) -> Result<(String, String, u64), ApiError> {
    let log = std::str::from_utf8(log).map_err(|_| failed())?;
    let time_base = log
        .lines()
        .find_map(|line| {
            line.split_once("config in time_base: ")
                .map(|(_, tail)| tail.split(',').next().unwrap_or_default().trim())
        })
        .ok_or_else(failed)?;
    let (numerator, denominator) = time_base.split_once('/').ok_or_else(failed)?;
    let numerator: u128 = numerator.parse().map_err(|_| failed())?;
    let denominator: u128 = denominator.parse().map_err(|_| failed())?;
    if denominator == 0 {
        return Err(failed());
    }
    let ticks = log
        .lines()
        .filter(|line| line.contains("Parsed_showinfo") && line.contains("pts_time:"))
        .find_map(|line| {
            line.split_once(" pts:")
                .and_then(|(_, tail)| tail.split_whitespace().next())
        })
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::NotFound,
                "No frame exists at or after requested source PTS",
            )
        })?;
    let value: u128 = ticks.parse().map_err(|_| failed())?;
    let micros = value
        .checked_mul(numerator)
        .and_then(|v| v.checked_mul(1_000_000))
        .and_then(|v| v.checked_div(denominator))
        .and_then(|v| u64::try_from(v).ok())
        .ok_or_else(failed)?;
    Ok((ticks.into(), time_base.into(), micros))
}

fn seconds_to_us(value: &str) -> Result<u64, ApiError> {
    let (seconds, fraction) = value.split_once('.').unwrap_or((value, ""));
    if seconds.is_empty()
        || !seconds.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Negative or unknown source timing is unsupported",
        ));
    }
    let seconds: u64 = seconds.parse().map_err(|_| failed())?;
    let fraction = format!("{fraction:0<6}");
    let fraction: u64 = fraction[..6].parse().map_err(|_| failed())?;
    seconds
        .checked_mul(1_000_000)
        .and_then(|v| v.checked_add(fraction))
        .ok_or_else(failed)
}
fn failed() -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        "Cannot verify source timing or frame metadata",
    )
}
