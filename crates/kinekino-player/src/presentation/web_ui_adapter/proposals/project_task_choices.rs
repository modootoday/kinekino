//! Captured rows name inspection targets; they grant no live mutation authority.
use kinekino_player::{
    application::launch::native_proposal::{NativeProposalSelection, NativeProposalTarget},
    domain::{
        projects::{ProjectProposalSnapshot, ProjectRevision},
        proposals::{Card, Correction, DocumentCorrection, Highlight, ProposalPayload},
    },
};
use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskChoiceError {
    InvalidProject(&'static str),
    ItemIndexOverflow,
    UnknownChoice,
}

pub enum ProjectTaskItem<'a> {
    Card(&'a Card),
    Highlight(&'a Highlight),
    CueCorrection(&'a Correction),
    DocumentCorrection(&'a DocumentCorrection),
}

pub struct ProjectTaskChoice<'a> {
    key: String,
    capture: &'a ProjectProposalSnapshot,
    index: u32,
    item: ProjectTaskItem<'a>,
}

impl ProjectTaskChoice<'_> {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn capture(&self) -> &ProjectProposalSnapshot {
        self.capture
    }
    pub fn item_index(&self) -> u32 {
        self.index
    }
    pub fn item(&self) -> &ProjectTaskItem<'_> {
        &self.item
    }
    /// Fresh SDK inspection must still check identity, support and mutation authority.
    pub fn live_inspection_selection(&self) -> NativeProposalSelection {
        let document = &self.capture.captured.document;
        let target = match &self.item {
            ProjectTaskItem::Card(_) => NativeProposalTarget::Card { index: self.index },
            ProjectTaskItem::Highlight(_) => NativeProposalTarget::Highlight { index: self.index },
            ProjectTaskItem::CueCorrection(edit) => NativeProposalTarget::CueCorrection {
                index: self.index,
                cue_id: edit.cue_id.clone(),
            },
            ProjectTaskItem::DocumentCorrection(edit) => NativeProposalTarget::DocumentCorrection {
                index: self.index,
                source_ref: edit.source_ref.clone(),
            },
        };
        NativeProposalSelection {
            proposal_id: document.proposal_id.clone(),
            document_revision: document.draft_revision,
            source_id: document.source.source_id().into(),
            source_revision: document.source.source_revision(),
            source_sha256: document.source.source_sha256().into(),
            target,
        }
    }
}

pub struct ProjectTaskChoices<'a> {
    project: &'a ProjectRevision,
    rows: Vec<ProjectTaskChoice<'a>>,
}

fn item_index(index: usize) -> Result<u32, TaskChoiceError> {
    u32::try_from(index).map_err(|_| TaskChoiceError::ItemIndexOverflow)
}

impl<'a> ProjectTaskChoices<'a> {
    pub fn new(
        project: &'a ProjectRevision,
        generation: NonZeroU64,
    ) -> Result<Self, TaskChoiceError> {
        project
            .validate()
            .map_err(TaskChoiceError::InvalidProject)?;
        let mut rows = Vec::new();
        for (proposal_index, capture) in project.proposals.iter().enumerate() {
            let mut push = |index: usize, item| {
                let index = item_index(index)?;
                rows.push(ProjectTaskChoice {
                    key: format!("project-task-{}-{proposal_index}-{index}", generation.get()),
                    capture,
                    index,
                    item,
                });
                Ok::<(), TaskChoiceError>(())
            };
            match &capture.captured.document.draft {
                Some(ProposalPayload::Cards { cards }) => {
                    for (index, card) in cards.iter().enumerate() {
                        push(index, ProjectTaskItem::Card(card))?;
                    }
                }
                Some(ProposalPayload::Highlights { highlights }) => {
                    for (index, highlight) in highlights.iter().enumerate() {
                        push(index, ProjectTaskItem::Highlight(highlight))?;
                    }
                }
                Some(ProposalPayload::Correction { edits }) => {
                    for (index, edit) in edits.iter().enumerate() {
                        push(index, ProjectTaskItem::CueCorrection(edit))?;
                    }
                }
                Some(ProposalPayload::DocumentCorrection { edits }) => {
                    for (index, edit) in edits.iter().enumerate() {
                        push(index, ProjectTaskItem::DocumentCorrection(edit))?;
                    }
                }
                // Repair requires an explicit source reference in a separate input flow.
                None => {}
            }
        }
        Ok(Self { project, rows })
    }
    pub fn project(&self) -> &ProjectRevision {
        self.project
    }
    pub fn rows(&self) -> &[ProjectTaskChoice<'a>] {
        &self.rows
    }
    /// Only keys displayed by this generation resolve; there is no first-row fallback.
    pub fn resolve(&self, key: &str) -> Result<&ProjectTaskChoice<'a>, TaskChoiceError> {
        self.rows
            .iter()
            .find(|row| row.key == key)
            .ok_or(TaskChoiceError::UnknownChoice)
    }
}

#[cfg(test)]
#[path = "project_task_choices/tests.rs"]
mod tests;
