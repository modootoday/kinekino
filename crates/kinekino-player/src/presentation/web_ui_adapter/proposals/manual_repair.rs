//! Explicit presentation input only. SDK validation, publication and CAS remain unchanged.
use super::*;

pub struct ManualRepairSource {
    pub choice: String,
    pub text: String,
}

pub struct ManualRepairEntry<'a> {
    pub task: ProposalTask,
    pub sources: Vec<ManualRepairSource>,
    pub selected_source: Option<&'a str>,
    pub generated_raw: &'a str,
    pub import_error: Option<&'a str>,
}

impl ReviewSession {
    /// A missing typed draft is displayed without creating an item or choosing evidence.
    pub fn manual_repair_entry(&self) -> Result<ManualRepairEntry<'_>, ViewError> {
        if self.draft.is_some() {
            return Err(ViewError::ExistingDraft);
        }
        if self.stored.document.state != DraftState::NeedsRepair {
            return Err(ViewError::InvalidSnapshot);
        }
        let source = &self.stored.document.source;
        let texts: Vec<&str> = match source {
            SourceSnapshot::Document(source) => {
                source.texts.iter().map(|text| text.text.as_str()).collect()
            }
            SourceSnapshot::Cue(source) => {
                source.cues.iter().map(|cue| cue.text.as_str()).collect()
            }
        };
        let position = self.manual_source_position().ok();
        Ok(ManualRepairEntry {
            task: self.stored.document.generated.task,
            selected_source: position.and_then(|index| texts.get(index).copied()),
            sources: texts
                .iter()
                .enumerate()
                .map(|(index, text)| ManualRepairSource {
                    choice: (index + 1).to_string(),
                    text: (*text).to_owned(),
                })
                .collect(),
            generated_raw: &self.stored.document.generated.raw_text,
            import_error: self.stored.document.import_error.as_deref(),
        })
    }

    /// One explicit source choice; no cue is selected from the model output or media time.
    pub fn choose_manual_source(
        &mut self,
        scope: &ReviewScope,
        choice: String,
    ) -> Result<(), ViewError> {
        self.ready(scope)?;
        if self.draft.is_some() {
            return Err(ViewError::ExistingDraft);
        }
        if self.stored.document.source.as_document().is_some() {
            return self.choose_document_text(scope, choice);
        }
        self.document_choice = choice;
        self.document_selection = None;
        let position = self.manual_source_position()?;
        self.stored
            .document
            .source
            .cues()
            .get(position)
            .ok_or(ViewError::InvalidSelection)?;
        Ok(())
    }

    /// Start an empty human input buffer. This neither saves nor accepts a proposal.
    pub fn begin_manual_repair(&mut self, scope: &ReviewScope) -> Result<(), ViewError> {
        self.ready(scope)?;
        if self.draft.is_some() {
            return Err(ViewError::ExistingDraft);
        }
        if self.stored.document.state != DraftState::NeedsRepair {
            return Err(ViewError::InvalidSnapshot);
        }
        if self.stored.document.source.as_document().is_some() {
            return self.begin_document_repair(scope);
        }
        let position = self.manual_source_position()?;
        let cue = self
            .stored
            .document
            .source
            .cues()
            .get(position)
            .ok_or(ViewError::InvalidSelection)?;
        let reference = cue.cue_id.clone();
        let draft = match self.stored.document.generated.task {
            ProposalTask::Correction => ProposalPayload::Correction {
                edits: vec![Correction {
                    cue_id: reference,
                    text: String::new(),
                }],
            },
            ProposalTask::Cards => ProposalPayload::Cards {
                cards: vec![Card {
                    front: String::new(),
                    back: String::new(),
                    source_refs: vec![reference],
                }],
            },
            ProposalTask::Highlights => ProposalPayload::Highlights {
                highlights: vec![Highlight {
                    title: String::new(),
                    source_refs: vec![reference],
                }],
            },
        };
        self.set_draft(scope, draft)
    }

    fn manual_source_position(&self) -> Result<usize, ViewError> {
        self.document_choice
            .trim()
            .parse::<usize>()
            .ok()
            .and_then(|position| position.checked_sub(1))
            .ok_or(ViewError::InvalidSelection)
    }
}

#[cfg(test)]
#[path = "manual_repair/tests.rs"]
mod tests;
