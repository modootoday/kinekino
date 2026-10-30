use super::*;
use std::collections::VecDeque;
pub const MAX_UNDO: usize = 64;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "edit", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClipEdit {
    Add {
        position: usize,
        segment: ClipSegment,
    },
    Duplicate {
        segment_id: String,
        new_segment_id: String,
        position: usize,
    },
    Delete {
        segment_id: String,
    },
    Replace {
        segment_id: String,
        segment: ClipSegment,
    },
    Reorder {
        segment_ids: Vec<String>,
    },
    SetPreset {
        preset: ClipOutputPreset,
    },
    SetCutMode {
        cut_mode: CutMode,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipEditError {
    InvalidDocument(&'static str),
    InvalidReference,
    RevisionConflict,
    RevisionExhausted,
    LimitExceeded,
    NoUndo,
}
#[derive(Clone)]
struct Working {
    segments: Vec<ClipSegment>,
    preset: ClipOutputPreset,
    cut_mode: CutMode,
}
impl Working {
    fn capture(document: &ClipDocument) -> Self {
        Self {
            segments: document.segments.clone(),
            preset: document.preset.clone(),
            cut_mode: document.cut_mode,
        }
    }
    fn restore(self, document: &mut ClipDocument) {
        document.segments = self.segments;
        document.preset = self.preset;
        document.cut_mode = self.cut_mode;
    }
}
/// Owns working edits. Source declarations and project identity cannot be mutated here.
/// Undo is bounded session history, separate from durable publication history.
pub struct ClipEditor {
    document: ClipDocument,
    undo: VecDeque<Working>,
}
impl ClipEditor {
    pub fn open(document: ClipDocument) -> Result<Self, ClipEditError> {
        document
            .validate()
            .map_err(ClipEditError::InvalidDocument)?;
        Ok(Self {
            document,
            undo: VecDeque::new(),
        })
    }
    pub fn document(&self) -> &ClipDocument {
        &self.document
    }
    fn next_revision(&self, expected: DecimalU64) -> Result<DecimalU64, ClipEditError> {
        if expected != self.document.revision {
            return Err(ClipEditError::RevisionConflict);
        }
        self.document
            .revision
            .0
            .checked_add(1)
            .map(DecimalU64)
            .ok_or(ClipEditError::RevisionExhausted)
    }
    pub fn apply(&mut self, expected: DecimalU64, edit: ClipEdit) -> Result<(), ClipEditError> {
        use ClipEditError::*;
        let revision = self.next_revision(expected)?;
        let mut candidate = self.document.clone();
        match edit {
            ClipEdit::Add { position, segment } => {
                if position > candidate.segments.len() {
                    return Err(InvalidReference);
                }
                if candidate.segments.len() >= MAX_SEGMENTS {
                    return Err(LimitExceeded);
                }
                candidate.segments.insert(position, segment);
            }
            ClipEdit::Duplicate {
                segment_id,
                new_segment_id,
                position,
            } => {
                if position > candidate.segments.len() {
                    return Err(InvalidReference);
                }
                if candidate.segments.len() >= MAX_SEGMENTS {
                    return Err(LimitExceeded);
                }
                let mut segment = candidate
                    .segments
                    .iter()
                    .find(|segment| segment.segment_id == segment_id)
                    .ok_or(InvalidReference)?
                    .clone();
                segment.segment_id = new_segment_id;
                candidate.segments.insert(position, segment);
            }
            ClipEdit::Delete { segment_id } => {
                let index = candidate
                    .segments
                    .iter()
                    .position(|segment| segment.segment_id == segment_id)
                    .ok_or(InvalidReference)?;
                candidate.segments.remove(index);
            }
            ClipEdit::Replace {
                segment_id,
                segment,
            } => {
                if segment.segment_id != segment_id {
                    return Err(InvalidReference);
                }
                *candidate
                    .segments
                    .iter_mut()
                    .find(|item| item.segment_id == segment_id)
                    .ok_or(InvalidReference)? = segment;
            }
            ClipEdit::Reorder { segment_ids } => {
                if segment_ids.len() != candidate.segments.len() {
                    return Err(InvalidReference);
                }
                let mut previous = std::mem::take(&mut candidate.segments);
                for id in segment_ids {
                    let index = previous
                        .iter()
                        .position(|item| item.segment_id == id)
                        .ok_or(InvalidReference)?;
                    candidate.segments.push(previous.remove(index));
                }
            }
            ClipEdit::SetPreset { preset } => candidate.preset = preset,
            ClipEdit::SetCutMode { cut_mode } => candidate.cut_mode = cut_mode,
        }
        candidate.revision = revision;
        candidate.validate().map_err(InvalidDocument)?;
        self.undo.push_back(Working::capture(&self.document));
        if self.undo.len() > MAX_UNDO {
            self.undo.pop_front();
        }
        self.document = candidate;
        Ok(())
    }
    pub fn undo(&mut self, expected: DecimalU64) -> Result<(), ClipEditError> {
        let revision = self.next_revision(expected)?;
        let working = self.undo.back().ok_or(ClipEditError::NoUndo)?.clone();
        let mut candidate = self.document.clone();
        working.restore(&mut candidate);
        candidate.revision = revision;
        candidate
            .validate()
            .map_err(ClipEditError::InvalidDocument)?;
        self.undo.pop_back();
        self.document = candidate;
        Ok(())
    }
}
