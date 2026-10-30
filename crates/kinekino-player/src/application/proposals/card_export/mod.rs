//! Immutable accepted Card to an explicitly unapproved review artifact.
#[path = "ports.rs"]
mod ports;
pub use ports::CardImagePort;
pub(crate) mod schema;
use super::{InspectProposal, ProposalStore};
use crate::{
    application::api::{ApiError, ErrorCode, Request},
    domain::{
        api_values::DecimalU64,
        proposals::{ProposalPayload, output_review::OutputReviewTarget},
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const COMMAND: &str = "production.export_card_review_png";
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateSelection {
    pub id: String,
    pub version: String,
    pub declaration_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportCardReviewPng {
    pub target: OutputReviewTarget,
    pub position: DecimalU64,
    pub frame_path: String,
    pub frame_sha256: String,
    pub template: TemplateSelection,
    pub output_path: String,
    pub review_artifact_only: bool,
}
pub struct SelectedAcceptedCard<'a> {
    pub target: &'a OutputReviewTarget,
    pub position: u64,
    pub front: &'a str,
    pub back: &'a str,
    pub source_refs: &'a [String],
}
pub struct PreparedReviewPng {
    pub bytes: Vec<u8>,
    pub provenance: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardReviewArtifact {
    pub target: OutputReviewTarget,
    pub position: DecimalU64,
    pub source_refs: Vec<String>,
    pub output_path: String,
    pub output_sha256: String,
    pub output_bytes: DecimalU64,
    pub state: String,
    pub quality_approved: bool,
    pub template_approved: bool,
    pub overwrite: bool,
    pub limitations: Vec<String>,
    pub cancellation_supported: bool,
    pub timeout_supported: bool,
    pub provenance: Value,
}
pub fn execute(
    store: &dyn ProposalStore,
    port: Option<&dyn CardImagePort>,
    input: ExportCardReviewPng,
) -> Result<CardReviewArtifact, ApiError> {
    if !input.review_artifact_only {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Only an explicitly acknowledged unapproved review artifact is available",
        ));
    }
    input
        .target
        .validate_stored()
        .map_err(|e| ApiError::new(ErrorCode::InvalidArgument, e))?;
    for path in [&input.frame_path, &input.output_path] {
        if !crate::domain::frame_capture::is_absolute_local_path(path)
            || !path.to_ascii_lowercase().ends_with(".png")
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Frame and review output must be explicit absolute local PNG paths",
            ));
        }
    }
    if input.frame_path == input.output_path {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Review output must be distinct from the input frame",
        ));
    }
    if !crate::domain::identifiers::valid_sha256(&input.frame_sha256)
        || !crate::domain::identifiers::valid_sha256(&input.template.declaration_sha256)
        || input.output_path.trim().is_empty()
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Explicit frame/template SHA and new output path required",
        ));
    }
    if !store.available() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Proposal storage unavailable",
        ));
    }
    let port = port.ok_or_else(|| {
        ApiError::new(
            ErrorCode::Unsupported,
            "Card review renderer is not configured",
        )
    })?;
    let state = super::inspect(
        store,
        InspectProposal {
            proposal_id: input.target.proposal_id.clone(),
        },
    )?;
    input
        .target
        .validate_against(&state)
        .map_err(|e| ApiError::new(ErrorCode::RevisionConflict, e))?;
    let receipt = state.acceptance.as_ref().ok_or_else(|| {
        ApiError::new(
            ErrorCode::NeedsInput,
            "Explicit accepted Card output required",
        )
    })?;
    let ProposalPayload::Cards { cards } = &receipt.output else {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Selected accepted output is not Cards",
        ));
    };
    let index = usize::try_from(input.position.0).map_err(|_| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Card position exceeds platform range",
        )
    })?;
    let card = cards.get(index).ok_or_else(|| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Card position is outside the accepted output",
        )
    })?;
    let artifact = port.render(
        SelectedAcceptedCard {
            target: &input.target,
            position: input.position.0,
            front: &card.front,
            back: &card.back,
            source_refs: &card.source_refs,
        },
        &input,
    )?;
    if artifact.bytes.is_empty()
        || artifact.bytes.len() as u64 > crate::domain::frame_capture::MAX_FRAME_PNG_BYTES
    {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Review PNG exceeds the shared output byte limit",
        ));
    }
    let result = CardReviewArtifact {
        target: input.target,
        position: input.position,
        source_refs: card.source_refs.clone(),
        output_path: input.output_path,
        output_sha256: format!("{:x}", Sha256::digest(&artifact.bytes)),
        output_bytes: DecimalU64(artifact.bytes.len() as u64),
        state: "review_artifact".into(),
        quality_approved: false,
        template_approved: false,
        overwrite: false,
        limitations: LIMITATIONS.iter().map(|x| (*x).into()).collect(),
        cancellation_supported: false,
        timeout_supported: false,
        provenance: artifact.provenance,
    };
    port.publish_new(&result.output_path, &artifact.bytes)?;
    Ok(result)
}
pub(crate) fn execute_request(
    request: &Request,
    store: &dyn ProposalStore,
    port: Option<&dyn CardImagePort>,
) -> Result<Value, ApiError> {
    request.validate()?;
    if request.command != COMMAND
        || request.context.expected_revision.is_some()
        || request.context.idempotency_key.is_some()
        || request.context.timeout_ms.is_some()
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Synchronous review PNG has no CAS, retry, cancellation or wall deadline contract",
        ));
    }
    let input = serde_json::from_value(request.payload.clone()).map_err(|_| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Expected explicit accepted Card/frame/template/review-only request",
        )
    })?;
    serde_json::to_value(execute(store, port, input)?)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Cannot encode review PNG receipt"))
}
pub const LIMITATIONS: [&str; 7] = [
    "glyph_coverage_unmeasured",
    "ink_overflow_unmeasured",
    "host_font_fallback_uncontrolled",
    "wall_deadline_not_provided",
    "frame_placement_not_pixel_verified",
    "template_approval_not_checked",
    "cancellation_not_provided",
];
pub fn support(available: bool) -> Value {
    json!({"review_artifact_available":available,"production_quality_export":false,"template_approved":false,"quality_approved":false,"cancellation_supported":false,"timeout_supported":false,"limitations":LIMITATIONS,"overwrite":false})
}
#[cfg(test)]
mod tests;
