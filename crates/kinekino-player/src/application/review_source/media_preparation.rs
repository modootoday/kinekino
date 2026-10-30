//! Explicit local selection. Package custody is not historical media or clock verification.
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{
        api_values::DecimalU64,
        model_review::{AutomatedCheck, CheckOutcome, EvidenceAsset, ReviewPackage},
        review_source::ReviewCase,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaSelectionRelation {
    pub schema_version: String,
    pub base_case: ReviewCase,
    pub original_candidate_sha256: String,
    pub media: EvidenceAsset,
    pub selection_authority: String,
    pub historical_media_verified: bool,
    pub timeline_verified: bool,
    pub quality_approved: bool,
}
/// Built from exact retained base bytes. No paths, storage, decoder or inference policy.
pub struct PreparedMediaPackage {
    pub body: Vec<u8>,
    pub case: ReviewCase,
    pub relation: MediaSelectionRelation,
    pub relation_asset: EvidenceAsset,
    pub relation_bytes: Vec<u8>,
    pub base_asset: EvidenceAsset,
}
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn encode(value: &impl Serialize) -> Result<Vec<u8>, ApiError> {
    serde_json::to_vec(value)
        .map_err(|_| ApiError::new(ErrorCode::EngineFailed, "Media preparation encoding failed"))
}
fn blob(id: String, bytes: &[u8]) -> EvidenceAsset {
    EvidenceAsset {
        asset_id: id,
        sha256: digest(bytes),
        bytes: DecimalU64(bytes.len() as u64),
        media_type: "application/json".into(),
    }
}
pub fn prepare(
    base_bytes: &[u8],
    base_case: &ReviewCase,
    media: EvidenceAsset,
) -> Result<PreparedMediaPackage, ApiError> {
    base_case
        .validate()
        .map_err(|_| invalid("Invalid base case"))?;
    if base_bytes.len() > 524288 || digest(base_bytes) != base_case.package_sha256 {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Base package bytes differ",
        ));
    }
    if !crate::domain::review_source::valid_review_object_id(&media.asset_id)
        || !crate::domain::model_review::valid_sha256(&media.sha256)
        || media.bytes.0 == 0
        || media.bytes.0 > 1_073_741_824
        || !matches!(
            media.media_type.as_str(),
            "video/mp4"
                | "video/webm"
                | "video/x-matroska"
                | "video/quicktime"
                | "audio/wav"
                | "audio/x-wav"
                | "audio/mpeg"
                | "audio/flac"
                | "audio/ogg"
                | "audio/mp4"
                | "audio/webm"
        )
    {
        return Err(invalid("Invalid selected media metadata"));
    }
    let mut package: ReviewPackage =
        serde_json::from_slice(base_bytes).map_err(|_| invalid("Invalid base package"))?;
    package.validate().map_err(invalid)?;
    let item = package
        .items
        .iter()
        .find(|item| item.item_id == base_case.item_id)
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Base item missing"))?;
    if item.source.sha256 != base_case.source_request_sha256 {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Base source request differs",
        ));
    }
    let relation = MediaSelectionRelation {
        schema_version: "review-media-selection/1".into(),
        base_case: base_case.clone(),
        original_candidate_sha256: digest(&encode(&item.candidate)?),
        media,
        selection_authority: "explicit_local_selection".into(),
        historical_media_verified: false,
        timeline_verified: false,
        quality_approved: false,
    };
    let relation_bytes = encode(&relation)?;
    let key = digest(&relation_bytes);
    let prefix = format!("selected-{}", &key[..24]);
    let relation_asset = blob(format!("{prefix}-relation"), &relation_bytes);
    let base_asset = blob(format!("{prefix}-base"), base_bytes);
    let added = vec![
        relation.media.clone(),
        relation_asset.clone(),
        base_asset.clone(),
    ];
    if package
        .assets()
        .iter()
        .any(|existing| added.iter().any(|new| new.asset_id == existing.asset_id))
    {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Selection asset ID already exists",
        ));
    }
    let item = package
        .items
        .iter_mut()
        .find(|item| item.item_id == base_case.item_id)
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Base item missing"))?;
    item.candidate.automated_checks.push(AutomatedCheck {
        check_id: prefix,
        outcome: CheckOutcome::NotRun,
        summary: "Explicit local media selection only; historical media, timeline and user quality remain unverified".into(),
        evidence: added,
    });
    package.validate().map_err(invalid)?;
    let body = encode(&package)?;
    if body.len() > 524288 {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Prepared package exceeds 512KiB",
        ));
    }
    Ok(PreparedMediaPackage {
        case: ReviewCase {
            package_sha256: digest(&body),
            ..base_case.clone()
        },
        body,
        relation,
        relation_asset,
        relation_bytes,
        base_asset,
    })
}
