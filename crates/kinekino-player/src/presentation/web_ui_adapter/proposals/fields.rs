use kinekino_player::domain::proposals::ProposalPayload;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextFieldKind {
    CueCorrection,
    DocumentCorrection,
    CardFront,
    CardBack,
    HighlightTitle,
}

/// Opaque input identity, valid only for the session and working draft that issued it.
#[derive(Clone, Debug)]
pub struct TextInput {
    pub(super) owner: Arc<()>,
    pub(super) generation: u64,
    kind: TextFieldKind,
    index: usize,
}
impl PartialEq for TextInput {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner, &other.owner)
            && self.generation == other.generation
            && self.kind == other.kind
            && self.index == other.index
    }
}
impl Eq for TextInput {}

#[derive(Clone, Debug)]
pub struct TextField {
    pub kind: TextFieldKind,
    pub text: String,
    pub input: TextInput,
}

#[derive(Clone, Debug)]
pub enum ReviewItem {
    Card {
        position: usize,
        source_refs: Vec<String>,
        front: TextField,
        back: TextField,
    },
    Highlight {
        position: usize,
        source_refs: Vec<String>,
        title: TextField,
    },
}

/// Positions belong to this draft generation, not persistent CardDocument or clip identities.
pub(super) fn items(owner: &Arc<()>, generation: u64, draft: &ProposalPayload) -> Vec<ReviewItem> {
    let fields = project(owner, generation, draft);
    match draft {
        ProposalPayload::Cards { cards } => cards
            .iter()
            .zip(fields.as_chunks::<2>().0)
            .enumerate()
            .map(|(position, (card, fields))| ReviewItem::Card {
                position,
                source_refs: card.source_refs.clone(),
                front: fields[0].clone(),
                back: fields[1].clone(),
            })
            .collect(),
        ProposalPayload::Highlights { highlights } => highlights
            .iter()
            .zip(fields)
            .enumerate()
            .map(|(position, (highlight, title))| ReviewItem::Highlight {
                position,
                source_refs: highlight.source_refs.clone(),
                title,
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub(super) fn project(owner: &Arc<()>, generation: u64, draft: &ProposalPayload) -> Vec<TextField> {
    let mut fields = Vec::new();
    let mut add = |kind, index, text: &str| {
        fields.push(TextField {
            kind,
            text: text.into(),
            input: TextInput {
                owner: owner.clone(),
                generation,
                kind,
                index,
            },
        });
    };
    match draft {
        ProposalPayload::Correction { edits } => {
            for (index, edit) in edits.iter().enumerate() {
                add(TextFieldKind::CueCorrection, index, &edit.text);
            }
        }
        ProposalPayload::DocumentCorrection { edits } => {
            for (index, edit) in edits.iter().enumerate() {
                add(TextFieldKind::DocumentCorrection, index, &edit.text);
            }
        }
        ProposalPayload::Cards { cards } => {
            for (index, card) in cards.iter().enumerate() {
                add(TextFieldKind::CardFront, index, &card.front);
                add(TextFieldKind::CardBack, index, &card.back);
            }
        }
        ProposalPayload::Highlights { highlights } => {
            for (index, highlight) in highlights.iter().enumerate() {
                add(TextFieldKind::HighlightTitle, index, &highlight.title);
            }
        }
    }
    fields
}

pub(super) fn replace(draft: &mut ProposalPayload, input: &TextInput, text: String) -> bool {
    let target = match (draft, input.kind) {
        (ProposalPayload::Correction { edits }, TextFieldKind::CueCorrection) => {
            edits.get_mut(input.index).map(|edit| &mut edit.text)
        }
        (ProposalPayload::DocumentCorrection { edits }, TextFieldKind::DocumentCorrection) => {
            edits.get_mut(input.index).map(|edit| &mut edit.text)
        }
        (ProposalPayload::Cards { cards }, TextFieldKind::CardFront) => {
            cards.get_mut(input.index).map(|card| &mut card.front)
        }
        (ProposalPayload::Cards { cards }, TextFieldKind::CardBack) => {
            cards.get_mut(input.index).map(|card| &mut card.back)
        }
        (ProposalPayload::Highlights { highlights }, TextFieldKind::HighlightTitle) => highlights
            .get_mut(input.index)
            .map(|highlight| &mut highlight.title),
        _ => None,
    };
    if let Some(target) = target {
        *target = text;
        true
    } else {
        false
    }
}
