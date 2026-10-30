//! Source-bound selection over existing timed cue snapshots; no AI acceptance or rendering.
pub mod frames;
pub mod layout;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_source::MutationControl,
    },
    domain::{
        api_values::DecimalU64,
        clip_document::{
            ArtifactPin, ClipSegment, ClipSourcePin, NormalizedCrop, SourceBoundArtifact,
        },
        identifiers::{valid_id, valid_sha256},
        proposals::CueSourceSnapshot,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipSubtitleOrigin {
    ExplicitManualSnapshot,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipSubtitleTimeBasis {
    MediaRelativeUs,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipSubtitleDocument {
    pub schema_version: String,
    pub media: ClipSourcePin,
    pub time_basis: ClipSubtitleTimeBasis,
    pub origin: ClipSubtitleOrigin,
    /// Existing cue snapshot identity, declared original text SHA and stable IDs are preserved.
    pub cues: CueSourceSnapshot,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipSubtitleFontRef {
    pub asset_path: String,
    pub sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipSubtitleAlignment {
    Left,
    Center,
    Right,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipSubtitleStyle {
    pub schema_version: String,
    pub style_id: String,
    pub revision: DecimalU64,
    pub fonts: Vec<ClipSubtitleFontRef>,
    pub font_px: u32,
    pub line_height_px: u32,
    pub box_normalized: NormalizedCrop,
    pub foreground_rgba: [u8; 4],
    pub background_rgba: [u8; 4],
    pub alignment: ClipSubtitleAlignment,
    pub max_lines: u32,
}
#[derive(Debug, Clone, Serialize)]
pub struct ClipSubtitleFontPin {
    pub asset_path: String,
    pub sha256: String,
    pub bytes: DecimalU64,
    pub license: String,
    pub release_manifest_sha256: String,
}
pub trait ClipSubtitleDocumentLease {
    fn document(&self) -> &ClipSubtitleDocument;
}
pub trait ClipSubtitleStyleLease {
    fn style(&self) -> &ClipSubtitleStyle;
    fn fonts(&self) -> &[ClipSubtitleFontPin];
}
pub trait ClipSubtitleAdmission {
    type Document: ClipSubtitleDocumentLease;
    type Style: ClipSubtitleStyleLease;
    fn open_document(
        &self,
        expected: &SourceBoundArtifact,
        control: &dyn MutationControl,
    ) -> Result<Self::Document, ApiError>;
    fn open_style(
        &self,
        expected: &ArtifactPin,
        control: &dyn MutationControl,
    ) -> Result<Self::Style, ApiError>;
}
#[derive(Debug, Serialize)]
pub struct SelectedClipSubtitleCue {
    pub cue_id: String,
    pub text: String,
    pub source_start_us: DecimalU64,
    pub source_end_us: DecimalU64,
    pub output_start_us: DecimalU64,
    pub output_end_us: DecimalU64,
}
/// Original document/style/font owners survive until the renderer and publication finish.
/// Not serializable: caller JSON cannot reconstruct admitted authority.
pub struct PreparedClipSubtitles<D: ClipSubtitleDocumentLease, S: ClipSubtitleStyleLease> {
    segment: ClipSegment,
    selection: crate::domain::clip_document::SubtitleSelection,
    document: D,
    style: S,
    cues: Vec<SelectedClipSubtitleCue>,
    omitted_outside_clip: Vec<String>,
}
pub type AdmittedClipSubtitles<P> = PreparedClipSubtitles<
    <P as ClipSubtitleAdmission>::Document,
    <P as ClipSubtitleAdmission>::Style,
>;
impl<D: ClipSubtitleDocumentLease, S: ClipSubtitleStyleLease> PreparedClipSubtitles<D, S> {
    pub fn require_binding(
        &self,
        source: &ClipSourcePin,
        segment: &ClipSegment,
    ) -> Result<(), ApiError> {
        if serde_json::to_value(source).map_err(|_| invalid("Invalid source pin"))?
            != serde_json::to_value(&self.document().document().media)
                .map_err(|_| invalid("Invalid subtitle source pin"))?
            || serde_json::to_value(segment).map_err(|_| invalid("Invalid clip segment"))?
                != serde_json::to_value(&self.segment)
                    .map_err(|_| invalid("Invalid admitted subtitle segment"))?
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Subtitles require the original admitted media and segment",
            ));
        }
        Ok(())
    }
    pub fn selection(&self) -> &crate::domain::clip_document::SubtitleSelection {
        &self.selection
    }
    pub fn document(&self) -> &D {
        &self.document
    }
    pub fn style(&self) -> &S {
        &self.style
    }
    pub fn cues(&self) -> &[SelectedClipSubtitleCue] {
        &self.cues
    }
    pub fn omitted_outside_clip(&self) -> &[String] {
        &self.omitted_outside_clip
    }
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
pub fn validate_subtitle_style(style: &ClipSubtitleStyle) -> Result<(), ApiError> {
    style.box_normalized.validate().map_err(invalid)?;
    if style.schema_version != "clip-subtitle-style/1"
        || !valid_id(&style.style_id)
        || style.revision.0 == 0
        || style.fonts.is_empty()
        || style.fonts.len() > 8
        || !(1..=512).contains(&style.font_px)
        || style.line_height_px < style.font_px
        || style.line_height_px > 1024
        || !(1..=16).contains(&style.max_lines)
        || style.foreground_rgba[3] == 0
    {
        return Err(invalid("Invalid explicit subtitle style or text bounds"));
    }
    let mut paths = BTreeSet::new();
    if style.fonts.iter().any(|font| {
        !paths.insert(&font.asset_path)
            || font.asset_path.is_empty()
            || font.asset_path.len() > 256
            || font.asset_path.chars().any(char::is_control)
            || !valid_sha256(&font.sha256)
    }) {
        return Err(invalid("Distinct pinned common font references required"));
    }
    Ok(())
}
pub fn prepare_subtitles<P: ClipSubtitleAdmission>(
    source: &ClipSourcePin,
    segment: &ClipSegment,
    admission: &P,
    control: &dyn MutationControl,
) -> Result<Option<AdmittedClipSubtitles<P>>, ApiError> {
    control.checkpoint()?;
    let Some(selection) = &segment.subtitles else {
        return Ok(None);
    };
    if segment.media_id != source.media_id
        || segment.range.start_us.0 >= segment.range.end_us.0
        || segment.range.end_us.0 > source.duration_us.0
        || selection.document.source_sha256 != source.sha256
        || !valid_id(&selection.document.artifact.artifact_id)
        || selection.document.artifact.revision.0 == 0
        || !valid_sha256(&selection.document.artifact.sha256)
        || !valid_id(&selection.style.artifact_id)
        || selection.style.revision.0 == 0
        || !valid_sha256(&selection.style.sha256)
    {
        return Err(invalid(
            "Source-bound subtitle document/style selection required",
        ));
    }
    let mut selected = BTreeSet::new();
    if selection.cue_ids.is_empty()
        || selection.cue_ids.len() > crate::domain::clip_document::MAX_SELECTED_CUES
        || selection
            .cue_ids
            .iter()
            .any(|id| !valid_id(id) || !selected.insert(id.as_str()))
    {
        return Err(invalid("Distinct selected cue IDs required"));
    }
    let document = admission.open_document(&selection.document, control)?;
    control.checkpoint()?;
    let record = document.document();
    record.cues.validate().map_err(invalid)?;
    if record.schema_version != "clip-subtitle-document/1"
        || record.media != *source
        || record.cues.source_id != selection.document.artifact.artifact_id
        || record.cues.source_revision != selection.document.artifact.revision
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Subtitle document media, clock or publication differs from selection",
        ));
    }
    let style = admission.open_style(&selection.style, control)?;
    control.checkpoint()?;
    validate_subtitle_style(style.style())?;
    if style.style().style_id != selection.style.artifact_id
        || style.style().revision != selection.style.revision
        || style.fonts().len() != style.style().fonts.len()
        || style
            .fonts()
            .iter()
            .zip(&style.style().fonts)
            .any(|(actual, expected)| {
                actual.asset_path != expected.asset_path
                    || actual.sha256 != expected.sha256
                    || actual.bytes.0 == 0
                    || actual.license.trim().is_empty()
                    || !valid_sha256(&actual.release_manifest_sha256)
            })
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Subtitle style or licensed common font pins differ",
        ));
    }
    let mut cues = Vec::new();
    let mut omitted_outside_clip = Vec::new();
    for cue in &record.cues.cues {
        if !selected.remove(cue.cue_id.as_str()) {
            continue;
        }
        control.checkpoint()?;
        let (Some(start), Some(end)) = (cue.start_us, cue.end_us) else {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Selected subtitle cue has no explicit media-relative timing",
            ));
        };
        if end.0 > source.duration_us.0 {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Subtitle cue exceeds the pinned media duration",
            ));
        }
        let a = start.0.max(segment.range.start_us.0);
        let b = end.0.min(segment.range.end_us.0);
        if a >= b {
            omitted_outside_clip.push(cue.cue_id.clone());
            continue;
        }
        cues.push(SelectedClipSubtitleCue {
            cue_id: cue.cue_id.clone(),
            text: cue.text.clone(),
            source_start_us: start,
            source_end_us: end,
            output_start_us: DecimalU64(a - segment.range.start_us.0),
            output_end_us: DecimalU64(b - segment.range.start_us.0),
        });
    }
    if !selected.is_empty() {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Selected subtitle cue ID is absent from the immutable document",
        ));
    }
    control.checkpoint()?;
    Ok(Some(PreparedClipSubtitles {
        segment: segment.clone(),
        selection: selection.clone(),
        document,
        style,
        cues,
        omitted_outside_clip,
    }))
}
