use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{
        api_values::DecimalU64,
        clip_document::{ClipSourcePin, SourceClock, SourceTrack, SourceTrackKind},
        identifiers::valid_id,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub use crate::application::clip_render::{ClipSourceObservation as ProbeFacts, DurationBasis};
#[derive(Deserialize)]
struct Probe {
    streams: Vec<Stream>,
    format: Option<Format>,
}
#[derive(Deserialize)]
struct Format {
    duration: Option<String>,
}
#[derive(Deserialize)]
struct Stream {
    index: u32,
    codec_type: String,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    time_base: Option<String>,
    start_pts: Option<i64>,
    duration_ts: Option<i64>,
    color_transfer: Option<String>,
    disposition: Option<Disposition>,
    side_data_list: Option<Vec<SideData>>,
}
#[derive(Deserialize)]
struct Disposition {
    attached_pic: Option<u8>,
}
#[derive(Deserialize)]
struct SideData {
    rotation: Option<i32>,
}
fn unavailable(message: &str) -> ApiError {
    ApiError::new(ErrorCode::NeedsInput, message)
}
fn rational(value: &str) -> Result<(u32, u32), ApiError> {
    let (a, b) = value
        .split_once('/')
        .ok_or_else(|| unavailable("Missing rational source time base"))?;
    let a = a
        .parse::<u32>()
        .map_err(|_| unavailable("Invalid source time base"))?;
    let b = b
        .parse::<u32>()
        .map_err(|_| unavailable("Invalid source time base"))?;
    if a == 0 || b == 0 {
        return Err(unavailable("Invalid source time base"));
    }
    Ok((a, b))
}
fn seconds(value: &str) -> Result<u64, ApiError> {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.len() > 6
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(unavailable(
            "Container duration must be an exact positive microsecond decimal",
        ));
    }
    let whole = whole
        .parse::<u64>()
        .map_err(|_| unavailable("Container duration overflow"))?;
    let digits = fraction.len() as u32;
    let fraction = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse::<u64>()
            .map_err(|_| unavailable("Invalid duration fraction"))?
    };
    whole
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(fraction * 10u64.pow(6 - digits)))
        .ok_or_else(|| unavailable("Container duration overflow"))
}
pub(super) fn parse(
    raw: &[u8],
    media_id: &str,
    source_sha256: &str,
    probe_sha256: &str,
) -> Result<ProbeFacts, ApiError> {
    let probe: Probe =
        serde_json::from_slice(raw).map_err(|_| unavailable("Invalid bounded clip probe JSON"))?;
    if probe.streams.is_empty() || probe.streams.len() > 64 {
        return Err(unavailable("Clip probe stream count is outside the bound"));
    }
    let reference = probe
        .streams
        .iter()
        .filter(|stream| {
            stream.codec_type == "video"
                && stream
                    .disposition
                    .as_ref()
                    .and_then(|disposition| disposition.attached_pic)
                    .unwrap_or(0)
                    == 0
        })
        .min_by_key(|stream| stream.index)
        .ok_or_else(|| unavailable("No ordinary video clock reference stream"))?;
    let width = reference
        .width
        .filter(|value| *value > 0)
        .ok_or_else(|| unavailable("Missing video width"))?;
    let height = reference
        .height
        .filter(|value| *value > 0)
        .ok_or_else(|| unavailable("Missing video height"))?;
    if u64::from(width) * u64::from(height) > crate::domain::frame_capture::MAX_FRAME_PIXELS {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Clip source dimensions exceed the pixel bound",
        ));
    }
    let (numerator, denominator) = rational(
        reference
            .time_base
            .as_deref()
            .ok_or_else(|| unavailable("Missing source time base"))?,
    )?;
    let origin = reference
        .start_pts
        .ok_or_else(|| unavailable("Missing integer reference stream origin; no inferred zero"))?;
    let (duration_us, duration_basis) =
        if let Some(ticks) = reference.duration_ts.filter(|value| *value > 0) {
            let micros = (ticks as u128)
                .checked_mul(u128::from(numerator))
                .and_then(|value| value.checked_mul(1_000_000))
                .and_then(|value| value.checked_div(u128::from(denominator)))
                .and_then(|value| u64::try_from(value).ok())
                .ok_or_else(|| unavailable("Reference stream duration overflow"))?;
            (micros, DurationBasis::ReferenceStreamTicks)
        } else {
            (
                seconds(
                    probe
                        .format
                        .as_ref()
                        .and_then(|format| format.duration.as_deref())
                        .ok_or_else(|| unavailable("No bounded reported duration"))?,
                )?,
                DurationBasis::ContainerReportedSpan,
            )
        };
    if duration_us == 0 {
        return Err(unavailable("Clip duration is empty"));
    }
    let mut indexes = BTreeSet::new();
    let mut tracks = Vec::new();
    for stream in &probe.streams {
        if !indexes.insert(stream.index) {
            return Err(unavailable("Duplicate container stream index"));
        }
        let kind = match stream.codec_type.as_str() {
            "video" => SourceTrackKind::Video,
            "audio" => SourceTrackKind::Audio,
            "subtitle" => SourceTrackKind::Subtitle,
            _ => continue,
        };
        let codec = stream
            .codec_name
            .as_deref()
            .filter(|codec| valid_id(codec))
            .ok_or_else(|| unavailable("Unknown clip stream codec"))?;
        tracks.push(SourceTrack {
            stream_index: stream.index,
            kind,
            codec_id: codec.into(),
        });
    }
    tracks.sort_by_key(|track| track.stream_index);
    let raw_probe_sha256 = format!("{:x}", Sha256::digest(raw));
    #[derive(Serialize)]
    struct Identity<'a> {
        contract: &'static str,
        source_sha256: &'a str,
        probe_sha256: &'a str,
        raw_probe_sha256: &'a str,
        reference_stream_index: u32,
        duration_basis: DurationBasis,
    }
    let identity = Identity {
        contract: "clip-source-probe/1",
        source_sha256,
        probe_sha256,
        raw_probe_sha256: &raw_probe_sha256,
        reference_stream_index: reference.index,
        duration_basis,
    };
    let observation_sha256 = format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(&identity)
                .map_err(|_| unavailable("Invalid clip observation identity"))?
        )
    );
    Ok(ProbeFacts {
        source: ClipSourcePin {
            media_id: media_id.into(),
            sha256: source_sha256.into(),
            duration_us: DecimalU64(duration_us),
            width,
            height,
            clock: SourceClock {
                origin_pts: origin.to_string(),
                time_base_numerator: numerator,
                time_base_denominator: denominator,
                observation_sha256,
            },
            tracks,
        },
        reference_stream_index: reference.index,
        duration_basis,
        raw_probe_sha256,
        probe_sha256: probe_sha256.into(),
        color_transfer: reference.color_transfer.clone(),
        rotation_degrees: reference
            .side_data_list
            .as_ref()
            .and_then(|list| list.iter().find_map(|side| side.rotation)),
    })
}
#[cfg(test)]
mod tests;
