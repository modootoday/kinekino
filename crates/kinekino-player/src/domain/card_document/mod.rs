//! Editable card projects. Source snapshots remain immutable; edits never accept AI output.
mod edit;
#[cfg(test)]
mod tests;
mod validation;

use crate::domain::{
    api_values::DecimalU64,
    proposals::{Card, CueSourceSnapshot},
};
pub use edit::{CardEdit, CardEditor};
use serde::{Deserialize, Serialize};

pub const MAX_CARDS: usize = 128;
pub const MAX_DOCUMENT_BYTES: usize = 512 * 1024;
pub const MAX_UNDO: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardRole {
    Cover,
    Body,
    Closing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardTextKind {
    Quotation,
    Summary,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardFrameReference {
    pub artifact_id: String,
    pub png_sha256: String,
    pub source_sha256: String,
    // A declaration, not authenticated producer timing or proof of image rights.
    pub declared_source_pts_us: Option<DecimalU64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardEntry {
    pub card_id: String,
    pub role: CardRole,
    pub text_kind: CardTextKind,
    pub content: Card,
    pub frame: Option<CardFrameReference>,
    pub alt_text_draft: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardCanvas {
    pub width: u32,
    pub height: u32,
    pub template_id: String,
    pub template_version: String,
    pub template_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardLayout {
    pub canvas: CardCanvas,
    pub cards: Vec<CardEntry>,
    pub caption_draft: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardDocument {
    pub schema_version: String,
    pub document_id: String,
    pub revision: DecimalU64,
    pub source: CueSourceSnapshot,
    pub layout: CardLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardDocumentError {
    InvalidDocument,
    InvalidReference,
    InvalidQuotation,
    RevisionConflict,
    RevisionExhausted,
    NoUndo,
    LimitExceeded,
}
