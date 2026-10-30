//! Shared editable project values. Validation confers no media custody or encoder availability.
mod edit;
#[cfg(test)]
mod tests;
mod validation;
mod values;
use crate::domain::api_values::DecimalU64;
pub use edit::{ClipEdit, ClipEditError, ClipEditor, MAX_UNDO};
use serde::{Deserialize, Serialize};
pub use values::*;
pub const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;
pub const MAX_SOURCES: usize = 128;
pub const MAX_SEGMENTS: usize = 128;
pub const MAX_CROP_KEYS: usize = 128;
pub const MAX_SELECTED_CUES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactPin {
    pub artifact_id: String,
    pub revision: DecimalU64,
    pub sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceBoundArtifact {
    pub artifact: ArtifactPin,
    pub source_sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipSourcePin {
    pub media_id: String,
    pub sha256: String,
    pub duration_us: DecimalU64,
    pub width: u32,
    pub height: u32,
    pub clock: SourceClock,
    /// Container-scoped stream indexes, not engine/native playback track IDs.
    pub tracks: Vec<SourceTrack>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubtitleSelection {
    pub document: SourceBoundArtifact,
    pub cue_ids: Vec<String>,
    /// A shared style/font asset contract, never an arbitrary font file path.
    pub style: ArtifactPin,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipSegment {
    pub segment_id: String,
    pub media_id: String,
    pub range: SourceRange,
    pub video_stream_index: u32,
    pub crop_keys: Vec<CropKey>,
    pub audio: AudioSelection,
    pub subtitles: Option<SubtitleSelection>,
    pub subject_ref: Option<SourceBoundArtifact>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipOutputPreset {
    pub catalog: ArtifactPin,
    pub width: u32,
    pub height: u32,
    pub frame_rate: FrameRate,
    pub container: Container,
    pub video_codec: VideoCodec,
    pub audio_codec: Option<AudioCodec>,
    pub color: OutputColor,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipDocument {
    pub schema_version: String,
    pub document_id: String,
    pub revision: DecimalU64,
    pub sources: Vec<ClipSourcePin>,
    pub segments: Vec<ClipSegment>,
    pub preset: ClipOutputPreset,
    pub cut_mode: CutMode,
}
