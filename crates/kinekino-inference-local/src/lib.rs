//! Local-pack adapter: whisper.cpp CLI (`whisper-cli -oj`) behind `InferenceBackend`.
//! OS-independent std process spawn (Linux first). The pack itself (binary + ggml model) is
//! installed by the user; this crate never downloads anything.
mod whisper;

pub use whisper::{
    LOCAL_CAPS, WhisperCliBackend, WhisperConfig, parse_whisper_json, register_whisper_cli,
};
