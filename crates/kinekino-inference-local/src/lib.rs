//! Local-pack adapter: whisper.cpp CLI (`whisper-cli -oj`) behind `InferenceBackend`.
//! OS-independent std process spawn (Linux first). The pack itself (binary + ggml model) is
//! installed by the user; this crate never downloads anything.
mod measurement_prep;
mod whisper;

pub use measurement_prep::{
    FIXTURE_DURATION_SECS, FIXTURE_SAMPLE_RATE, MeasuredValue, MeasurementItem, MeasurementRecord,
    MeasurementStatus, PROCEDURE_STEPS, PrepReport, WavInfo, assess_prep, blank_record,
    format_fixture_wav, inspect_wav, required_measurement_items, start_measurement,
};
pub use whisper::{
    LOCAL_CAPS, WhisperCliBackend, WhisperConfig, parse_whisper_json, register_whisper_cli,
};
