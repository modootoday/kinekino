//! Exact live execution contracts, shared by producer and receipt reader. No defaults.
use crate::application::review_playback::ReviewContentClass;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::media_timeline::TimeBase,
    infrastructure::frame_capture::plan,
};
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, path::Path};
pub(crate) fn demuxer(class: &ReviewContentClass) -> &'static str {
    match class {
        ReviewContentClass::IsoBmffHeader => "mov",
        ReviewContentClass::MatroskaHeader => "matroska",
    }
}
fn input_options(class: &ReviewContentClass) -> Vec<&'static str> {
    match class {
        ReviewContentClass::IsoBmffHeader => vec!["-f", "mov", "-enable_drefs", "0"],
        ReviewContentClass::MatroskaHeader => vec!["-f", "matroska"],
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum LiveExecutionContract {
    VideoFirstStreamWindow { interval: String },
    AudioSourceCopyTsMono16kS16,
    RawPcmReplayMono16kS16,
    FirstDecodedOrigin,
    CaptureAtOrAfter { source_us: u64 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoleContract {
    pub role: String,
    pub root: String,
    pub contract: LiveExecutionContract,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ToolPin {
    pub role: String,
    pub path: String,
    pub sha256: String,
    pub file_identity: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProducerPin {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub file_identity: String,
    pub pid: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResourceBudget {
    pub memory_committed_bytes: u64,
    pub affinity_mask: usize,
    pub max_output_bytes: u64,
    pub cpu_threads: u8,
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid explicit live execution contract",
    )
}
fn micros(value: &str) -> Result<u64, ApiError> {
    let (seconds, fraction) = value.split_once('.').ok_or_else(invalid)?;
    if seconds.is_empty()
        || !seconds.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() != 6
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(invalid());
    }
    seconds
        .parse::<u64>()
        .ok()
        .and_then(|s| s.checked_mul(1_000_000))
        .and_then(|s| s.checked_add(fraction.parse::<u64>().ok()?))
        .ok_or_else(invalid)
}
pub(crate) fn video_arguments(
    source: &Path,
    interval: &str,
    class: &ReviewContentClass,
) -> Result<Vec<OsString>, ApiError> {
    let entries = match class {
        ReviewContentClass::IsoBmffHeader => {
            "frame=pts,best_effort_timestamp,duration,key_frame:stream=index,time_base"
        }
        ReviewContentClass::MatroskaHeader => {
            "frame=pts,best_effort_timestamp,duration,key_frame:stream=index,time_base,avg_frame_rate,r_frame_rate"
        }
    };
    let (start, end) = interval.split_once('%').ok_or_else(invalid)?;
    let start = micros(start)?;
    let end = micros(end)?;
    if end > 86_400_000_000
        || end
            .checked_sub(start)
            .is_none_or(|span| span == 0 || span > 60_000_000)
    {
        return Err(invalid());
    }
    Ok(["-v", "error", "-protocol_whitelist", "file"]
        .into_iter()
        .chain(input_options(class))
        .chain([
            "-probesize",
            "1048576",
            "-analyzeduration",
            "1000000",
            "-threads",
            "1",
            "-select_streams",
            "v:0",
            "-read_intervals",
            interval,
            "-show_frames",
            "-show_streams",
            "-show_entries",
            entries,
            "-of",
            "json",
        ])
        .map(OsString::from)
        .chain([source.as_os_str().to_owned()])
        .collect())
}
pub(crate) fn audio_arguments(
    source: &Path,
    pcm: bool,
    class: &ReviewContentClass,
) -> Vec<OsString> {
    let mut args: Vec<OsString> = [
        "-hide_banner",
        "-nostdin",
        "-nostats",
        "-loglevel",
        "info",
        "-protocol_whitelist",
        "file",
    ]
    .into_iter()
    .chain(input_options(class))
    .chain([
        "-probesize",
        "1048576",
        "-analyzeduration",
        "1000000",
        "-threads",
        "1",
        "-filter_threads",
        "1",
        "-copyts",
        "-i",
    ])
    .map(OsString::from)
    .collect();
    args.push(source.as_os_str().to_owned());
    args.extend(
        [
            "-map",
            "0:a:0",
            "-vn",
            "-sn",
            "-dn",
            "-threads",
            "1",
            "-af",
            "aformat=sample_fmts=s16:sample_rates=16000:channel_layouts=mono,ashowinfo",
            "-c:a",
            "pcm_s16le",
            "-f",
            if pcm { "s16le" } else { "null" },
            "-",
        ]
        .into_iter()
        .map(OsString::from),
    );
    args
}
pub(crate) fn replay_arguments(source: &Path) -> Vec<OsString> {
    let mut args: Vec<OsString> = [
        "-hide_banner",
        "-nostdin",
        "-nostats",
        "-loglevel",
        "info",
        "-protocol_whitelist",
        "file",
        "-f",
        "s16le",
        "-ar",
        "16000",
        "-ac",
        "1",
        "-threads",
        "1",
        "-filter_threads",
        "1",
        "-i",
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    args.push(source.as_os_str().to_owned());
    args.extend(
        [
            "-map",
            "0:a:0",
            "-vn",
            "-sn",
            "-dn",
            "-threads",
            "1",
            "-af",
            "ashowinfo",
            "-c:a",
            "pcm_s16le",
            "-f",
            "s16le",
            "-",
        ]
        .into_iter()
        .map(OsString::from),
    );
    args
}
impl LiveExecutionContract {
    pub fn arguments(
        &self,
        source: &Path,
        derived: Option<&Path>,
        base: TimeBase,
        class: &ReviewContentClass,
    ) -> Result<Vec<OsString>, ApiError> {
        match self {
            Self::VideoFirstStreamWindow { interval } => video_arguments(source, interval, class),
            Self::AudioSourceCopyTsMono16kS16 => Ok(audio_arguments(source, true, class)),
            Self::RawPcmReplayMono16kS16 => Ok(replay_arguments(derived.ok_or_else(invalid)?)),
            Self::FirstDecodedOrigin => plan::arguments(
                source,
                demuxer(class),
                plan::Selection::FirstDecoded,
                None,
                1,
            ),
            Self::CaptureAtOrAfter { source_us } => plan::arguments(
                source,
                demuxer(class),
                plan::Selection::AtOrAfter(
                    base.ticks_at_or_after(*source_us).map_err(|_| invalid())?,
                ),
                None,
                1,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_source_class_controls_every_demuxer_and_receipt_reconstruction() {
        let source = Path::new("C:/fixed/content-addressed-source");
        let base = TimeBase::new(1, 1000).unwrap();
        for (class, format) in [
            (ReviewContentClass::IsoBmffHeader, "mov"),
            (ReviewContentClass::MatroskaHeader, "matroska"),
        ] {
            for contract in [
                LiveExecutionContract::VideoFirstStreamWindow {
                    interval: "107.000000%134.000000".into(),
                },
                LiveExecutionContract::AudioSourceCopyTsMono16kS16,
                LiveExecutionContract::FirstDecodedOrigin,
                LiveExecutionContract::CaptureAtOrAfter {
                    source_us: 107_250_000,
                },
            ] {
                let args = contract.arguments(source, None, base, &class).unwrap();
                let input_format = args.windows(2).find(|pair| pair[0] == "-f").unwrap();
                assert_eq!(input_format[1], format);
                assert_eq!(
                    args.iter().any(|arg| arg == "-enable_drefs"),
                    format == "mov"
                );
                assert!(
                    args.windows(2)
                        .any(|pair| pair[0] == "-protocol_whitelist" && pair[1] == "file")
                );
                assert!(args.iter().any(|arg| arg == source.as_os_str()));
            }
        }
    }
}
