//! Prepare a complete immutable card project and explicit assets for one bundle attempt.
mod catalog;
mod execution;
mod schema;
pub use catalog::*;
pub use execution::*;
pub const EXPORT_OPERATION: &str = "production.export_cards";
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        card_projects::{CardProjectStore, CardPublication},
    },
    domain::{
        api_values::DecimalU64,
        card_document::{CardCanvas, CardRole},
        identifiers::{valid_id, valid_sha256},
    },
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedCardProjectPin {
    pub document_id: String,
    pub publication_revision: DecimalU64,
    pub publication_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedCardFrame {
    pub artifact_id: String,
    pub path: String,
    pub png_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardBundleRequest {
    pub project: SavedCardProjectPin,
    pub frames: Vec<SelectedCardFrame>,
    pub output_directory: String,
    pub review_artifact_only: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleTemplatePin {
    #[serde(default, skip_serializing_if = "is_false")]
    pub role_layouts_and_pages: bool,
    /// Absent in historical descriptors means the original frame-required contract.
    #[serde(default = "required_frame", skip_serializing_if = "is_required_frame")]
    pub frame_required: bool,
    pub canvas: CardCanvas,
    pub common_font_sha256: Vec<String>,
    pub license_refs: Vec<String>,
    pub limitations: Vec<String>,
}
fn required_frame() -> bool {
    true
}
fn is_false(value: &bool) -> bool {
    !*value
}
fn is_required_frame(value: &bool) -> bool {
    *value
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleFrameObservation {
    pub sha256: String,
    pub bytes: DecimalU64,
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedBundleCard {
    pub position: DecimalU64,
    pub card_id: String,
    pub output_name: String,
    pub frame: Option<SelectedCardFrame>,
    pub observation: Option<BundleFrameObservation>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedCardBundle {
    pub schema_version: String,
    pub project: SavedCardProjectPin,
    pub publication: CardPublication,
    pub template: BundleTemplatePin,
    pub cards: Vec<PreparedBundleCard>,
    pub output_directory: String,
    pub review_artifact_only: bool,
}
/// Static product adapters verify real PNG bytes and common font/template assets.
/// A JSON payload never supplies this adapter or asserts its own asset admission.
pub trait CardBundleAssets: Send + Sync {
    fn verify_template(&self, canvas: &CardCanvas) -> Result<BundleTemplatePin, ApiError>;
    fn inspect_frame(
        &self,
        selection: &SelectedCardFrame,
    ) -> Result<BundleFrameObservation, ApiError>;
    fn verify_new_output_directory(&self, path: &str) -> Result<(), ApiError>;
}

pub struct RenderedBundleCard {
    pub png: Vec<u8>,
    pub provenance: serde_json::Value,
}
pub trait CardBundleRasterizer: Send + Sync {
    fn read_frame_png(&self, selection: &SelectedCardFrame) -> Result<Vec<u8>, ApiError>;
    fn render_card(
        &self,
        bundle: &PreparedCardBundle,
        position: DecimalU64,
    ) -> Result<RenderedBundleCard, ApiError>;
}

pub(crate) fn card_output_name(position: usize, role: CardRole) -> Result<String, ApiError> {
    if position >= crate::domain::card_document::MAX_CARDS {
        return Err(error(
            ErrorCode::InvalidArgument,
            "Card position exceeds bundle limit",
        ));
    }
    let role = match role {
        CardRole::Cover => "cover",
        CardRole::Body => "body",
        CardRole::Closing => "closing",
    };
    Ok(format!("{:03}-{role}.png", position + 1))
}
fn error(code: ErrorCode, message: &str) -> ApiError {
    ApiError::new(code, message)
}

pub fn prepare(
    store: &dyn CardProjectStore,
    assets: &dyn CardBundleAssets,
    request: CardBundleRequest,
) -> Result<PreparedCardBundle, ApiError> {
    if !request.review_artifact_only {
        return Err(error(
            ErrorCode::NeedsInput,
            "Current card renderer produces explicitly unapproved review artifacts",
        ));
    }
    if !valid_id(&request.project.document_id)
        || request.project.publication_revision.0 == 0
        || !valid_sha256(&request.project.publication_sha256)
    {
        return Err(error(
            ErrorCode::InvalidArgument,
            "Exact saved card publication identity is required",
        ));
    }
    let saved = store.load(
        &request.project.document_id,
        Some(request.project.publication_revision),
    )?;
    if saved.publication_sha256 != request.project.publication_sha256
        || saved.publication.document.document_id != request.project.document_id
        || saved.publication.publication_revision != request.project.publication_revision
        || crate::application::card_projects::publication_sha256(&saved.publication)?
            != request.project.publication_sha256
    {
        return Err(error(
            ErrorCode::RevisionConflict,
            "Saved card publication differs from the selected SHA256",
        ));
    }
    let document = &saved.publication.document;
    document
        .validate()
        .map_err(|_| error(ErrorCode::NeedsInput, "Invalid saved card document"))?;
    if document.layout.cards.is_empty() {
        return Err(error(ErrorCode::NeedsInput, "Card bundle has no cards"));
    }
    let mut frames = BTreeMap::new();
    for frame in &request.frames {
        if !valid_id(&frame.artifact_id)
            || !valid_sha256(&frame.png_sha256)
            || frames.insert(frame.artifact_id.as_str(), frame).is_some()
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Frame selections must have unique valid identities and SHA256",
            ));
        }
    }
    if frames.len() > crate::domain::card_document::MAX_CARDS {
        return Err(error(
            ErrorCode::ResourceExhausted,
            "Too many card frame selections",
        ));
    }
    let template = assets.verify_template(&document.layout.canvas)?;
    if serde_json::to_vec(&template.canvas)
        .map_err(|_| error(ErrorCode::Internal, "Invalid template admission"))?
        != serde_json::to_vec(&document.layout.canvas)
            .map_err(|_| error(ErrorCode::Internal, "Invalid canvas"))?
        || template.common_font_sha256.is_empty()
        || template.common_font_sha256.len() != template.license_refs.len()
        || template.common_font_sha256.iter().any(|s| !valid_sha256(s))
        || template.license_refs.iter().any(|s| s.trim().is_empty())
    {
        return Err(error(
            ErrorCode::EngineFailed,
            "Template adapter returned mismatched canvas or unlicensed font pins",
        ));
    }
    let mut used = BTreeSet::new();
    let mut cards = Vec::new();
    for (position, card) in document.layout.cards.iter().enumerate() {
        if card.content.front.trim().is_empty()
            || card.content.back.trim().is_empty()
            || card.alt_text_draft.trim().is_empty()
        {
            return Err(error(
                ErrorCode::NeedsInput,
                "Current two-slot card template requires front, back and alt text drafts",
            ));
        }
        if !template.frame_required {
            if card.frame.is_some() {
                return Err(error(
                    ErrorCode::InvalidArgument,
                    "Text-only template forbids a frame reference",
                ));
            }
            cards.push(PreparedBundleCard {
                position: DecimalU64(position as u64),
                card_id: card.card_id.clone(),
                output_name: card_output_name(position, card.role)?,
                frame: None,
                observation: None,
            });
            continue;
        }
        let pin = card.frame.as_ref().ok_or_else(|| {
            error(
                ErrorCode::NeedsInput,
                "Selected card requires an explicit frame reference",
            )
        })?;
        let selected = frames
            .get(pin.artifact_id.as_str())
            .ok_or_else(|| error(ErrorCode::NeedsInput, "No PNG selection for a card frame"))?;
        if selected.png_sha256 != pin.png_sha256 {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Selected PNG differs from the saved frame pin",
            ));
        }
        used.insert(selected.artifact_id.as_str());
        let observation = assets.inspect_frame(selected)?;
        if observation.sha256 != pin.png_sha256
            || observation.bytes.0 == 0
            || observation.width == 0
            || observation.height == 0
        {
            return Err(error(
                ErrorCode::SourceExpired,
                "Observed PNG differs from the saved frame pin",
            ));
        }
        cards.push(PreparedBundleCard {
            position: DecimalU64(position as u64),
            card_id: card.card_id.clone(),
            output_name: card_output_name(position, card.role)?,
            frame: Some((*selected).clone()),
            observation: Some(observation),
        });
    }
    if used.len() != frames.len() {
        return Err(error(
            ErrorCode::InvalidArgument,
            "Unused PNG selections are not admitted",
        ));
    }
    assets.verify_new_output_directory(&request.output_directory)?;
    Ok(PreparedCardBundle {
        schema_version: "1".into(),
        project: request.project,
        publication: saved.publication,
        template,
        cards,
        output_directory: request.output_directory,
        review_artifact_only: true,
    })
}
