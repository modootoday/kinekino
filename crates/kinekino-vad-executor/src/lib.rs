//! Inert VAD executor adapter candidate; no operation registration or native launcher.
pub mod application;
pub mod domain;
pub mod infrastructure;
pub use kinekino_process::domain::deadline::Deadline;
pub use kinekino_vad::application::ProfileAdmission;
pub use kinekino_vad::domain::{ArtifactPin, MAX_AUDIO_BYTES, VadInput, VadProfile};
pub use kinekino_vad::protocol::application::{MAX_DESCRIPTOR_BYTES, decode_descriptor};
pub use kinekino_vad::protocol::{
    application::decode_worker_outcome, domain::ExpectedWorkerContext,
};
