//! Local whisper measurement preparation: a format fixture, required items, and
//! a reproduction procedure. This module never downloads a model and never
//! starts a real measurement.
use crate::WhisperConfig;
use kinekino_inference::InferenceError;
use kinekino_inference::executor::{PcmTrack, SpeechRegion, wav_segment};

pub const FIXTURE_SAMPLE_RATE: u32 = 16_000;
pub const FIXTURE_DURATION_SECS: f64 = 2.0;

/// Reproduction steps for a later, approved measurement. Installing the pack and
/// running whisper-cli are operator actions; this crate does not download them.
pub const PROCEDURE_STEPS: &[&str] = &[
    "Install whisper-cli and the ggml model as existing local files; this crate does not download them.",
    "Point WhisperConfig.program at the existing whisper-cli and WhisperConfig.model at the existing ggml file.",
    "Generate the 16 kHz mono PCM16 format fixture in-process (two seconds of silence).",
    "After explicit approval, run three transcriptions of the chosen fixture and record every required measurement item, including machine conditions.",
    "Automatic numbers are not user quality approval.",
    "A licensed speech fixture and character error rate are recorded only after approval supplies that fixture; this prep does not include one.",
];

const ITEMS: &[MeasurementItem] = &[
    MeasurementItem {
        id: "availability",
        unit: "enum",
        requires_speech_reference: false,
    },
    MeasurementItem {
        id: "process_exit",
        unit: "code",
        requires_speech_reference: false,
    },
    MeasurementItem {
        id: "wall_time_median_s",
        unit: "s",
        requires_speech_reference: false,
    },
    MeasurementItem {
        id: "peak_memory_bytes",
        unit: "bytes",
        requires_speech_reference: false,
    },
    MeasurementItem {
        id: "real_time_factor",
        unit: "ratio",
        requires_speech_reference: false,
    },
    MeasurementItem {
        id: "transcript_text",
        unit: "text",
        requires_speech_reference: false,
    },
    MeasurementItem {
        id: "character_error_rate",
        unit: "ratio",
        requires_speech_reference: true,
    },
    MeasurementItem {
        id: "chunk_bounds",
        unit: "bool",
        requires_speech_reference: false,
    },
    MeasurementItem {
        id: "provenance_backend",
        unit: "text",
        requires_speech_reference: false,
    },
    MeasurementItem {
        id: "provenance_model",
        unit: "text",
        requires_speech_reference: false,
    },
    MeasurementItem {
        id: "provenance_cost_usd",
        unit: "usd",
        requires_speech_reference: false,
    },
    MeasurementItem {
        id: "language",
        unit: "text",
        requires_speech_reference: false,
    },
    MeasurementItem {
        id: "measured",
        unit: "bool",
        requires_speech_reference: false,
    },
    MeasurementItem {
        id: "conditions",
        unit: "text",
        requires_speech_reference: false,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasurementItem {
    pub id: &'static str,
    pub unit: &'static str,
    pub requires_speech_reference: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasuredValue {
    pub id: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementStatus {
    NotRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasurementRecord {
    pub status: MeasurementStatus,
    pub items: Vec<MeasuredValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WavInfo {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepReport {
    pub fixture_ready: bool,
    pub items_ready: bool,
    pub procedure_ready: bool,
    pub artifacts_ready: bool,
    pub binary_present: bool,
    pub model_present: bool,
    pub measured: bool,
    pub inference_invocations: u64,
}

/// Builds a silent 16 kHz mono PCM16 WAV of [`FIXTURE_DURATION_SECS`].
pub fn format_fixture_wav() -> Result<Vec<u8>, InferenceError> {
    let samples = usize::try_from(FIXTURE_SAMPLE_RATE)
        .ok()
        .and_then(|rate| rate.checked_mul(2))
        .ok_or_else(|| InferenceError::InvalidInput("fixture too long".into()))?;
    let track = PcmTrack {
        sample_rate: FIXTURE_SAMPLE_RATE,
        samples: vec![0; samples],
    };
    wav_segment(
        &track,
        SpeechRegion {
            start: 0.0,
            end: FIXTURE_DURATION_SECS,
        },
    )
}

/// Reads a canonical 44-byte PCM WAV header produced by [`format_fixture_wav`].
pub fn inspect_wav(bytes: &[u8]) -> Result<WavInfo, InferenceError> {
    let bad = |m: &str| InferenceError::InvalidInput(m.into());
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(bad("expected a PCM WAV"));
    }
    if &bytes[12..16] != b"fmt " {
        return Err(bad("expected a fmt chunk"));
    }
    if read_u16(bytes, 20) != 1 {
        return Err(bad("expected PCM"));
    }
    let channels = read_u16(bytes, 22);
    let sample_rate = read_u32(bytes, 24);
    let bits_per_sample = read_u16(bytes, 34);
    if channels == 0
        || sample_rate == 0
        || bits_per_sample == 0
        || !bits_per_sample.is_multiple_of(8)
    {
        return Err(bad("invalid PCM header"));
    }
    if &bytes[36..40] != b"data" {
        return Err(bad("expected a data chunk"));
    }
    let data_bytes = u64::from(read_u32(bytes, 40));
    let frame = u64::from(channels) * u64::from(bits_per_sample / 8);
    let duration_secs = data_bytes as f64 / (f64::from(sample_rate) * frame as f64);
    Ok(WavInfo {
        sample_rate,
        channels,
        bits_per_sample,
        duration_secs,
    })
}

pub fn required_measurement_items() -> &'static [MeasurementItem] {
    ITEMS
}

pub fn blank_record() -> MeasurementRecord {
    MeasurementRecord {
        status: MeasurementStatus::NotRun,
        items: ITEMS
            .iter()
            .map(|item| MeasuredValue {
                id: item.id.to_string(),
                value: None,
            })
            .collect(),
    }
}

/// Reports whether prep artifacts are ready. Never spawns whisper-cli, never
/// downloads a model, and never marks the run as measured.
pub fn assess_prep(config: &WhisperConfig) -> PrepReport {
    let fixture_ready = format_fixture_wav()
        .ok()
        .and_then(|wav| inspect_wav(&wav).ok())
        .is_some_and(|info| {
            info.sample_rate == FIXTURE_SAMPLE_RATE
                && info.channels == 1
                && info.bits_per_sample == 16
                && (info.duration_secs - FIXTURE_DURATION_SECS).abs() < 0.001
        });
    let ids: Vec<&str> = ITEMS.iter().map(|item| item.id).collect();
    let needed = [
        "availability",
        "process_exit",
        "wall_time_median_s",
        "peak_memory_bytes",
        "real_time_factor",
        "transcript_text",
        "character_error_rate",
        "chunk_bounds",
        "provenance_backend",
        "provenance_model",
        "provenance_cost_usd",
        "language",
        "measured",
        "conditions",
    ];
    let items_ready = needed.iter().all(|id| ids.contains(id))
        && ITEMS
            .iter()
            .any(|item| item.id == "character_error_rate" && item.requires_speech_reference);
    let text = PROCEDURE_STEPS.join("\n").to_ascii_lowercase();
    let procedure_ready = [
        "does not download",
        "whisper-cli",
        "16 khz",
        "approval",
        "three",
        "quality",
        "speech fixture",
    ]
    .iter()
    .all(|needle| text.contains(needle));
    // File-presence only: the same checks WhisperCliBackend::availability uses, without spawning.
    PrepReport {
        fixture_ready,
        items_ready,
        procedure_ready,
        artifacts_ready: fixture_ready && items_ready && procedure_ready,
        binary_present: config.program.is_file(),
        model_present: config.model.is_file(),
        measured: false,
        inference_invocations: 0,
    }
}

/// Refuses to run a measurement until there is an explicit approval.
pub fn start_measurement(_config: &WhisperConfig) -> Result<MeasurementRecord, InferenceError> {
    Err(InferenceError::Unsupported(
        "local whisper measurement requires explicit approval; this slice is prep only".into(),
    ))
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}
