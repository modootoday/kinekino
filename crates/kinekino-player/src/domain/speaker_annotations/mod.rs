//! Imported speaker annotations and explicit edits. No inference or media admission.
mod edits;
use crate::domain::{
    api_values::DecimalU64,
    identifiers::{valid_id, valid_sha256},
    model_catalog::{ModelTask, fixed_revision},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_DOCUMENT_BYTES: usize = 512 * 1024;
pub const MAX_HISTORY: usize = 128;
pub const MAX_SPEAKERS: usize = 128;
pub const MAX_SPANS: usize = 1024;
pub const MAX_CUES: usize = 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Speaker {
    pub id: String,
    pub label: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeakerReviewState {
    Unreviewed,
    NeedsReview,
    Reviewed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpeakerSpan {
    pub id: String,
    pub in_us: DecimalU64,
    pub out_us: DecimalU64,
    pub speaker_ids: Vec<String>,
    pub origin: String,
    pub evidence: Vec<String>,
    /// Imported annotation state, not AI quality approval. Missing stays undeclared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_state: Option<SpeakerReviewState>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CueSpeakerLink {
    pub cue_id: String,
    pub span_ids: Vec<String>,
    pub speaker_ids: Vec<String>,
    pub assignment_origin: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpeakerState {
    pub speakers: Vec<Speaker>,
    pub spans: Vec<SpeakerSpan>,
    pub links: Vec<CueSpeakerLink>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredModel {
    pub task: ModelTask,
    pub repo_id: String,
    pub revision: String,
    pub sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredSource {
    pub media_id: String,
    pub media_sha256: String,
    pub audio_track_id: String,
    pub source_revision: DecimalU64,
    pub in_us: DecimalU64,
    pub out_us: DecimalU64,
    pub cue_source_sha256: Option<String>,
    pub model_refs: Vec<DeclaredModel>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum SpeakerEdit {
    Add {
        speaker: Speaker,
    },
    Rename {
        speaker_id: String,
        label: String,
    },
    Merge {
        speaker_ids: Vec<String>,
        target: String,
    },
    Split {
        speaker_id: String,
        new_speaker: Speaker,
        span_ids: Vec<String>,
        cue_ids: Vec<String>,
    },
    AssignCues {
        cue_ids: Vec<String>,
        speaker_ids: Vec<String>,
    },
    Delete {
        speaker_id: String,
        reassign: Option<Vec<String>>,
    },
    Undo,
    Redo,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpeakerDocument {
    pub schema_version: String,
    pub document_id: String,
    pub revision: DecimalU64,
    pub parent_sha256: Option<String>,
    pub source: DeclaredSource,
    pub original: SpeakerState,
    pub edits: Vec<SpeakerEdit>,
}
pub(super) fn text(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}
pub(super) fn unique_ids(ids: &[String], allow_empty: bool) -> bool {
    (allow_empty || !ids.is_empty())
        && ids.len() <= MAX_SPANS
        && ids.iter().all(|id| valid_id(id))
        && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()
}
impl SpeakerState {
    pub fn validate(&self, source: &DeclaredSource) -> Result<(), &'static str> {
        if self.speakers.len() > MAX_SPEAKERS
            || self.spans.len() > MAX_SPANS
            || self.links.len() > MAX_CUES
        {
            return Err("Speaker editing capacity exceeded");
        }
        let ids = self
            .speakers
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>();
        let spans = self.spans.iter().map(|s| s.id.clone()).collect::<Vec<_>>();
        let cues = self
            .links
            .iter()
            .map(|s| s.cue_id.clone())
            .collect::<Vec<_>>();
        if !unique_ids(&ids, true)
            || !unique_ids(&spans, true)
            || !unique_ids(&cues, true)
            || self.speakers.iter().any(|s| !text(&s.label, 1024))
        {
            return Err("Invalid or duplicate speaker/span/cue identity");
        }
        for span in &self.spans {
            if span.in_us.0 >= span.out_us.0
                || span.in_us.0 < source.in_us.0
                || span.out_us.0 > source.out_us.0
                || !unique_ids(&span.speaker_ids, true)
                || span.speaker_ids.iter().any(|id| !ids.contains(id))
                || !text(&span.origin, 80)
                || span.evidence.len() > 16
                || span.evidence.iter().any(|e| !text(e, 128))
            {
                return Err("Invalid speaker span or declared range");
            }
        }
        for cue in &self.links {
            if !unique_ids(&cue.span_ids, true)
                || cue.span_ids.iter().any(|id| !spans.contains(id))
                || !unique_ids(&cue.speaker_ids, true)
                || cue.speaker_ids.iter().any(|id| !ids.contains(id))
                || !text(&cue.assignment_origin, 80)
            {
                return Err("Invalid cue speaker references");
            }
        }
        Ok(())
    }
}
impl SpeakerDocument {
    pub fn current(&self) -> Result<SpeakerState, &'static str> {
        if self.schema_version != "speaker-annotations/1"
            || !valid_id(&self.document_id)
            || self.revision.0 == 0
            || self.edits.len() > MAX_HISTORY
            || self
                .parent_sha256
                .as_ref()
                .is_some_and(|s| !valid_sha256(s))
            || (self.revision.0 == 1 && (self.parent_sha256.is_some() || !self.edits.is_empty()))
            || (self.revision.0 > 1 && self.parent_sha256.is_none())
        {
            return Err("Invalid speaker document revision/history");
        }
        let s = &self.source;
        if !valid_id(&s.media_id)
            || !valid_id(&s.audio_track_id)
            || !valid_sha256(&s.media_sha256)
            || s.source_revision.0 == 0
            || s.in_us.0 >= s.out_us.0
            || s.cue_source_sha256
                .as_ref()
                .is_some_and(|v| !valid_sha256(v))
            || (!self.original.links.is_empty() && s.cue_source_sha256.is_none())
            || s.model_refs.len() > 16
        {
            return Err("Invalid declared media/track/cue source");
        }
        for m in &s.model_refs {
            if !matches!(
                m.task,
                ModelTask::SpeakerSegmentation | ModelTask::SpeakerEmbedding
            ) || !fixed_revision(&m.revision)
                || !valid_sha256(&m.sha256)
                || m.repo_id.split('/').count() != 2
                || m.repo_id.split('/').any(|p| {
                    p.is_empty()
                        || p.len() > 96
                        || p == "."
                        || p == ".."
                        || !p
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
                })
            {
                return Err("Invalid declared speaker model pin");
            }
        }
        self.original.validate(s)?;
        edits::replay(self)
    }
    pub fn edit(
        &self,
        parent_sha256: String,
        edits: Vec<SpeakerEdit>,
    ) -> Result<Self, &'static str> {
        self.current()?;
        if !valid_sha256(&parent_sha256) || edits.is_empty() || edits.len() > 32 {
            return Err("Explicit bounded speaker edits and parent hash required");
        }
        let mut next = self.clone();
        next.revision = DecimalU64(
            self.revision
                .0
                .checked_add(1)
                .ok_or("Speaker revision exhausted")?,
        );
        next.parent_sha256 = Some(parent_sha256);
        next.edits.extend(edits);
        next.current()?;
        Ok(next)
    }
}
