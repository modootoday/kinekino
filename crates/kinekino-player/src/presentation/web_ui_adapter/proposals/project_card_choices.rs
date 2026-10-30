//! Captured project rows supply explicit choices, never proof of a current live proposal.
use kinekino_player::{
    application::launch::native_proposal::{NativeProposalSelection, NativeProposalTarget},
    domain::{
        projects::{ProjectProposalSnapshot, ProjectRevision},
        proposals::{Card, ProposalPayload},
    },
};
use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardChoiceError {
    InvalidProject(&'static str),
    ItemIndexOverflow,
    UnknownChoice,
}

pub struct ProjectCardChoice<'a> {
    key: String,
    capture: &'a ProjectProposalSnapshot,
    index: u32,
    card: &'a Card,
}
impl ProjectCardChoice<'_> {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn capture(&self) -> &ProjectProposalSnapshot {
        self.capture
    }
    /// This is captured content; any saved working draft remains separately available.
    pub fn card(&self) -> &Card {
        self.card
    }
    pub fn item_index(&self) -> u32 {
        self.index
    }
    pub fn live_inspection_selection(&self) -> NativeProposalSelection {
        let document = &self.capture.captured.document;
        NativeProposalSelection {
            proposal_id: document.proposal_id.clone(),
            document_revision: document.draft_revision,
            source_id: document.source.source_id().into(),
            source_revision: document.source.source_revision(),
            source_sha256: document.source.source_sha256().into(),
            target: NativeProposalTarget::Card { index: self.index },
        }
    }
}

pub struct ProjectCardChoices<'a> {
    project: &'a ProjectRevision,
    rows: Vec<ProjectCardChoice<'a>>,
}
impl<'a> ProjectCardChoices<'a> {
    /// Generation comes from the original frame owner. A payload can only name a displayed row.
    pub fn new(
        project: &'a ProjectRevision,
        generation: NonZeroU64,
    ) -> Result<Self, CardChoiceError> {
        project
            .validate()
            .map_err(CardChoiceError::InvalidProject)?;
        let mut rows = Vec::new();
        for (proposal_index, capture) in project.proposals.iter().enumerate() {
            let Some(ProposalPayload::Cards { cards }) = &capture.captured.document.draft else {
                continue;
            };
            for (index, card) in cards.iter().enumerate() {
                let index = u32::try_from(index).map_err(|_| CardChoiceError::ItemIndexOverflow)?;
                rows.push(ProjectCardChoice {
                    key: format!("project-card-{}-{proposal_index}-{index}", generation.get()),
                    capture,
                    index,
                    card,
                });
            }
        }
        Ok(Self { project, rows })
    }
    pub fn project(&self) -> &ProjectRevision {
        self.project
    }
    pub fn rows(&self) -> &[ProjectCardChoice<'a>] {
        &self.rows
    }
    /// No fallback to the first row, no deserialization of callback-supplied authority.
    pub fn resolve(&self, key: &str) -> Result<&ProjectCardChoice<'a>, CardChoiceError> {
        self.rows
            .iter()
            .find(|row| row.key == key)
            .ok_or(CardChoiceError::UnknownChoice)
    }
}

#[cfg(test)]
#[path = "project_card_choices/tests.rs"]
mod tests;
