//! Engine-free admission policy for one saved manual clip publication.
use super::{SavedClipProjectPin, encoder_requirements};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        clip_projects::{self, ClipProjectStore},
    },
    domain::{clip_document::ClipDocument, identifiers::valid_sha256},
};
use serde::{Deserialize, Serialize};
pub const RENDER_OPERATION: &str = "production.render_short";
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipRenderRequest {
    pub project: SavedClipProjectPin,
    pub review_artifact_only: bool,
}
impl ClipRenderRequest {
    pub fn schema() -> serde_json::Value {
        serde_json::json!({"type":"object","additionalProperties":false,"required":["project","review_artifact_only"],"properties":{"project":{"type":"object","additionalProperties":false,"required":["document_id","publication_revision","publication_sha256"],"properties":{"document_id":{"type":"string","pattern":"^[A-Za-z0-9_.-]{1,80}$"},"publication_revision":{"type":"string","pattern":"^[1-9][0-9]*$"},"publication_sha256":{"type":"string","pattern":"^[0-9a-f]{64}$"}}},"review_artifact_only":{"const":true}}})
    }
}
pub fn capture_render_document(
    store: &dyn ClipProjectStore,
    request: &ClipRenderRequest,
) -> Result<ClipDocument, ApiError> {
    if !request.review_artifact_only || !valid_sha256(&request.project.publication_sha256) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Pinned review clip publication required",
        ));
    }
    let saved = clip_projects::inspect(
        store,
        &request.project.document_id,
        Some(request.project.publication_revision),
    )?;
    if saved.publication_sha256 != request.project.publication_sha256 {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Saved clip publication differs from requested pin",
        ));
    }
    let document = saved.publication.document;
    encoder_requirements(&document)?;
    if document.segments.len() != 1 || document.segments.iter().any(|s| s.subject_ref.is_some()) {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Manual renderer currently requires one segment and no subject overlay",
        ));
    }
    let range = &document.segments[0].range;
    if range
        .end_us
        .0
        .checked_sub(range.start_us.0)
        .is_none_or(|duration| duration == 0 || duration > 60_000_000)
    {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Manual clip duration must be positive and at most 60 seconds",
        ));
    }
    Ok(document)
}
