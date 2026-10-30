use super::{InspectedMedia, ProbeContainer, ProbePlan, TrackKind};
use crate::application::api::{ApiError, ErrorCode};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) const MAX_PROBE_OUTPUT: usize = 65536;
pub(super) fn inspect(raw: &[u8], plan: &ProbePlan) -> Result<InspectedMedia, ApiError> {
    plan.validate()?;
    if raw.is_empty() || raw.len() > MAX_PROBE_OUTPUT {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Probe response exceeds bounded metadata contract",
        ));
    }
    let value: Value = serde_json::from_slice(raw).map_err(|_| invalid())?;
    let format = value["format"]["format_name"]
        .as_str()
        .ok_or_else(invalid)?;
    let format_matches = format.split(',').any(|name| match plan.container {
        ProbeContainer::Mp4 => matches!(name, "mov" | "mp4"),
        ProbeContainer::Matroska => matches!(name, "matroska" | "webm"),
        ProbeContainer::MpegTs => name == "mpegts",
    });
    if !format_matches {
        return Err(mismatch());
    }
    let duration_us = seconds_to_us(value["format"]["duration"].as_str().ok_or_else(invalid)?)?;
    if duration_us < plan.minimum_duration_us || duration_us > plan.maximum_duration_us {
        return Err(mismatch());
    }
    let streams = value["streams"].as_array().ok_or_else(invalid)?;
    if streams.len() != plan.tracks.len() {
        return Err(mismatch());
    }
    let mut indices = BTreeSet::new();
    for stream in streams {
        let index =
            u32::try_from(stream["index"].as_u64().ok_or_else(invalid)?).map_err(|_| invalid())?;
        if !indices.insert(index) {
            return Err(invalid());
        }
        let expected = plan
            .tracks
            .iter()
            .find(|track| track.index == index)
            .ok_or_else(mismatch)?;
        let kind = match stream["codec_type"].as_str() {
            Some("video") => TrackKind::Video,
            Some("audio") => TrackKind::Audio,
            Some("subtitle") => TrackKind::Subtitle,
            _ => return Err(mismatch()),
        };
        let codec = stream["codec_name"]
            .as_str()
            .filter(|codec| !codec.is_empty() && *codec != "unknown")
            .ok_or_else(invalid)?;
        if kind != expected.kind || expected.codec.as_deref().is_some_and(|name| name != codec) {
            return Err(mismatch());
        }
        let packets = stream["nb_read_packets"]
            .as_u64()
            .or_else(|| {
                stream["nb_read_packets"]
                    .as_str()
                    .and_then(|value| value.parse::<u64>().ok())
            })
            .ok_or_else(invalid)?;
        if packets == 0 && kind != TrackKind::Subtitle {
            return Err(invalid());
        }
        if kind == TrackKind::Video
            && (stream["width"].as_u64().unwrap_or(0) == 0
                || stream["height"].as_u64().unwrap_or(0) == 0)
        {
            return Err(invalid());
        }
        if kind == TrackKind::Audio
            && (stream["channels"].as_u64().unwrap_or(0) == 0
                || stream["sample_rate"]
                    .as_str()
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or(0)
                    == 0)
        {
            return Err(invalid());
        }
    }
    Ok(InspectedMedia {
        duration_us,
        streams: streams.len(),
    })
}
fn seconds_to_us(value: &str) -> Result<u64, ApiError> {
    let (seconds, fraction) = value.split_once('.').unwrap_or((value, ""));
    if seconds.is_empty()
        || !seconds.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
        || fraction.as_bytes().iter().skip(6).any(|b| *b != b'0')
    {
        return Err(invalid());
    }
    let seconds = seconds.parse::<u64>().map_err(|_| invalid())?;
    let mut micros = 0_u64;
    for position in 0..6 {
        micros = micros * 10
            + u64::from(fraction.as_bytes().get(position).copied().unwrap_or(b'0') - b'0');
    }
    seconds
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(micros))
        .ok_or_else(invalid)
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        "Probe did not report valid readable media metadata",
    )
}
fn mismatch() -> ApiError {
    ApiError::new(
        ErrorCode::SourceExpired,
        "Downloaded container, duration or tracks differ from selected media",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::download::MediaIdentity, infrastructure::download::media_probe::ExpectedTrack,
    };
    use serde_json::json;
    fn plan() -> ProbePlan {
        ProbePlan {
            identity: MediaIdentity {
                media: "fixture".into(),
                tracks: "video-audio".into(),
            },
            container: ProbeContainer::Mp4,
            tracks: vec![
                ExpectedTrack {
                    index: 0,
                    kind: TrackKind::Video,
                    codec: Some("h264".into()),
                },
                ExpectedTrack {
                    index: 1,
                    kind: TrackKind::Audio,
                    codec: Some("aac".into()),
                },
            ],
            minimum_duration_us: 900_000,
            maximum_duration_us: 1_100_000,
        }
    }
    fn raw() -> Value {
        json!({"format":{"format_name":"mov,mp4,m4a,3gp,3g2,mj2","duration":"1.000000"},"streams":[{"index":0,"codec_type":"video","codec_name":"h264","width":640,"height":360,"nb_read_packets":"25"},{"index":1,"codec_type":"audio","codec_name":"aac","channels":2,"sample_rate":"48000","nb_read_packets":"47"}]})
    }
    #[test]
    fn selected_container_tracks_and_read_packets_are_required() {
        let result = inspect(&serde_json::to_vec(&raw()).unwrap(), &plan()).unwrap();
        assert_eq!(result.duration_us, 1_000_000);
        assert_eq!(result.streams, 2);
        let mut missing = raw();
        missing["streams"][1]["nb_read_packets"] = Value::Null;
        assert!(inspect(&serde_json::to_vec(&missing).unwrap(), &plan()).is_err());
    }
    #[test]
    fn track_loss_extra_track_and_changed_duration_are_rejected() {
        let mut extra = raw();
        extra["streams"].as_array_mut().unwrap().push(
            json!({"index":2,"codec_type":"subtitle","codec_name":"webvtt","nb_read_packets":"1"}),
        );
        assert_eq!(
            inspect(&serde_json::to_vec(&extra).unwrap(), &plan())
                .err()
                .unwrap()
                .code,
            ErrorCode::SourceExpired
        );
        let mut missing = raw();
        missing["streams"].as_array_mut().unwrap().pop();
        assert_eq!(
            inspect(&serde_json::to_vec(&missing).unwrap(), &plan())
                .err()
                .unwrap()
                .code,
            ErrorCode::SourceExpired
        );
        let mut duration = raw();
        duration["format"]["duration"] = json!("2.0");
        assert_eq!(
            inspect(&serde_json::to_vec(&duration).unwrap(), &plan())
                .err()
                .unwrap()
                .code,
            ErrorCode::SourceExpired
        );
        let mut duplicate = raw();
        duplicate["streams"][1]["index"] = json!(0);
        assert!(inspect(&serde_json::to_vec(&duplicate).unwrap(), &plan()).is_err());
    }
    #[test]
    fn alternate_containers_require_matching_probe_format() {
        for (container, format) in [
            (ProbeContainer::Matroska, "matroska,webm"),
            (ProbeContainer::MpegTs, "mpegts"),
        ] {
            let mut selected = plan();
            selected.container = container;
            assert!(inspect(&serde_json::to_vec(&raw()).unwrap(), &selected).is_err());
            let mut matching = raw();
            matching["format"]["format_name"] = json!(format);
            assert!(inspect(&serde_json::to_vec(&matching).unwrap(), &selected).is_ok());
        }
    }
    #[test]
    fn timing_is_exact_and_never_accepts_unknown_negative_or_overflow() {
        assert_eq!(seconds_to_us("1.0000000").unwrap(), 1_000_000);
        for value in ["N/A", "-1.0", "1e3", "0.0000001", "18446744073709551615.0"] {
            assert!(seconds_to_us(value).is_err(), "{value}");
        }
    }
}
