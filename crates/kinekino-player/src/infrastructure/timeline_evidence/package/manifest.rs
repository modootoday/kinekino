use super::super::capture_execution::CaptureContract;
use super::super::original_execution::{ExecutionContract, ObservedCompletion};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CaptureTrace {
    pub inputs: EvidenceReference,
    pub request: EvidenceReference,
    pub response: EvidenceReference,
    pub process: EvidenceReference,
    pub contract: CaptureContract,
}
use crate::domain::{
    api_values::DecimalU64,
    review_source::{EvidenceReference, MediaReference},
};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OriginalTrace {
    pub command: EvidenceReference,
    pub inputs: EvidenceReference,
    pub process: EvidenceReference,
    pub contract: ExecutionContract,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    pub schema_version: String,
    pub source_request_sha256: String,
    pub media: MediaReference,
    pub mapping_revision: DecimalU64,
    pub audio: Audio,
    pub video: Video,
    pub origin_capture: EvidenceReference,
    pub start_capture: EvidenceReference,
    pub end_capture: EvidenceReference,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Audio {
    pub log: EvidenceReference,
    pub full_wav: EvidenceReference,
    pub clip_wav: EvidenceReference,
    pub replay_pcm: EvidenceReference,
    pub clip_start_sample: DecimalU64,
    pub engine: EvidenceReference,
    pub execution: EvidenceReference,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Video {
    pub frames: EvidenceReference,
    pub engine: EvidenceReference,
    pub execution: EvidenceReference,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AudioExecution {
    pub trace: OriginalTrace,
    pub schema_version: String,
    pub source_sha256: String,
    pub engine_sha256: String,
    pub log_sha256: String,
    pub replay_sha256: String,
    pub completion: ObservedCompletion,
    pub copy_timestamps: bool,
    pub sample_rate: u32,
    pub channels: u32,
    pub sample_format: String,
    pub time_base: String,
    pub async_correction: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VideoExecution {
    pub trace: OriginalTrace,
    pub schema_version: String,
    pub source_sha256: String,
    pub engine_sha256: String,
    pub frames_sha256: String,
    pub stream_index: u32,
    pub completion: ObservedCompletion,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Capture {
    pub trace: CaptureTrace,
    pub schema_version: String,
    pub source_sha256: String,
    pub engine: EvidenceReference,
    pub png: EvidenceReference,
    pub requested_source_us: DecimalU64,
    pub actual_pts: i64,
    pub time_base: String,
    pub exit_code: i32,
    pub complete: bool,
}
