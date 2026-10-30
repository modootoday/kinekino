use super::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "edit", rename_all = "snake_case", deny_unknown_fields)]
pub enum CardEdit {
    Add {
        position: usize,
        card: CardEntry,
    },
    Duplicate {
        card_id: String,
        new_card_id: String,
        position: usize,
    },
    Delete {
        card_id: String,
    },
    Replace {
        card_id: String,
        card: CardEntry,
    },
    Reorder {
        card_ids: Vec<String>,
    },
    SetCanvas {
        canvas: CardCanvas,
    },
    SetCaption {
        caption_draft: String,
    },
}

/// Owns the working document. Callers cannot mutate its source or bypass CAS.
/// Undo is bounded in-memory editing history, not durable publication history.
pub struct CardEditor {
    document: CardDocument,
    undo: VecDeque<CardLayout>,
}

impl CardEditor {
    pub fn open(document: CardDocument) -> Result<Self, CardDocumentError> {
        document.validate()?;
        Ok(Self {
            document,
            undo: VecDeque::new(),
        })
    }
    pub fn document(&self) -> &CardDocument {
        &self.document
    }
    fn next_revision(&self, expected: DecimalU64) -> Result<DecimalU64, CardDocumentError> {
        if expected != self.document.revision {
            return Err(CardDocumentError::RevisionConflict);
        }
        self.document
            .revision
            .0
            .checked_add(1)
            .map(DecimalU64)
            .ok_or(CardDocumentError::RevisionExhausted)
    }
    pub fn apply(&mut self, expected: DecimalU64, edit: CardEdit) -> Result<(), CardDocumentError> {
        use CardDocumentError::*;
        let revision = self.next_revision(expected)?;
        let mut candidate = self.document.clone();
        let layout = &mut candidate.layout;
        match edit {
            CardEdit::Add { position, card } => {
                if position > layout.cards.len() {
                    return Err(InvalidReference);
                }
                if layout.cards.len() >= MAX_CARDS {
                    return Err(LimitExceeded);
                }
                layout.cards.insert(position, card);
            }
            CardEdit::Duplicate {
                card_id,
                new_card_id,
                position,
            } => {
                if position > layout.cards.len() {
                    return Err(InvalidReference);
                }
                if layout.cards.len() >= MAX_CARDS {
                    return Err(LimitExceeded);
                }
                let mut card = layout
                    .cards
                    .iter()
                    .find(|c| c.card_id == card_id)
                    .ok_or(InvalidReference)?
                    .clone();
                card.card_id = new_card_id;
                layout.cards.insert(position, card);
            }
            CardEdit::Delete { card_id } => {
                let index = layout
                    .cards
                    .iter()
                    .position(|c| c.card_id == card_id)
                    .ok_or(InvalidReference)?;
                layout.cards.remove(index);
            }
            CardEdit::Replace { card_id, card } => {
                if card.card_id != card_id {
                    return Err(InvalidReference);
                }
                let item = layout
                    .cards
                    .iter_mut()
                    .find(|c| c.card_id == card_id)
                    .ok_or(InvalidReference)?;
                *item = card;
            }
            CardEdit::Reorder { card_ids } => {
                if card_ids.len() != layout.cards.len() {
                    return Err(InvalidReference);
                }
                let mut old = std::mem::take(&mut layout.cards);
                for id in card_ids {
                    let index = old
                        .iter()
                        .position(|c| c.card_id == id)
                        .ok_or(InvalidReference)?;
                    layout.cards.push(old.remove(index));
                }
            }
            CardEdit::SetCanvas { canvas } => layout.canvas = canvas,
            CardEdit::SetCaption { caption_draft } => layout.caption_draft = caption_draft,
        }
        candidate.revision = revision;
        candidate.validate()?;
        self.undo.push_back(self.document.layout.clone());
        if self.undo.len() > MAX_UNDO {
            self.undo.pop_front();
        }
        self.document = candidate;
        Ok(())
    }
    pub fn undo(&mut self, expected: DecimalU64) -> Result<(), CardDocumentError> {
        let revision = self.next_revision(expected)?;
        let layout = self.undo.back().ok_or(CardDocumentError::NoUndo)?.clone();
        let mut candidate = self.document.clone();
        candidate.layout = layout;
        candidate.revision = revision;
        candidate.validate()?;
        self.undo.pop_back();
        self.document = candidate;
        Ok(())
    }
}
