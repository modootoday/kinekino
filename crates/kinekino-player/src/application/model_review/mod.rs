#[path = "ports.rs"]
mod ports;
pub use ports::ReviewStore;
pub mod schema;
use crate::{
    application::api::{ApiError, ErrorCode, Request},
    domain::model_review::{
        RecordReviewDecision, ReviewDecision, ReviewPackage, valid_id, valid_sha256,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub trait ReviewSession {
    fn package(&self) -> &ReviewPackage;
    fn read_text_asset(&self, asset_sha256: &str) -> Result<ReviewTextAsset, ApiError>;
    /// Internal evidence adapters only. Implementations retain package/file custody;
    /// reads must bind both declared asset ID and SHA, never interpret an embedded path.
    /// Existing providers stay explicitly unsupported rather than fabricating bytes.
    fn read_evidence_asset(
        &self,
        _asset_id: &str,
        _sha256: &str,
        _max_bytes: u64,
    ) -> Result<Vec<u8>, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Binary review evidence reader unavailable",
        ))
    }
    fn append(&self, decision: &ReviewDecision) -> Result<StoredDecision, ApiError>;
    /// Prefix classification only, backed by the session's already verified full asset.
    fn read_evidence_prefix(
        &self,
        _asset_id: &str,
        _sha256: &str,
        _max_prefix: u64,
    ) -> Result<Vec<u8>, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Guarded review evidence prefix reader unavailable",
        ))
    }
    fn decisions(&self, item_id: &str) -> Result<Vec<StoredDecision>, ApiError>;
}
pub const MAX_BINARY_EVIDENCE_BYTES: u64 = 64 * 1024 * 1024;
pub struct UnavailableReviewStore;
impl ReviewStore for UnavailableReviewStore {
    fn available(&self) -> bool {
        false
    }
    fn open(&self, _: &str) -> Result<Box<dyn ReviewSession>, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Review storage adapter unavailable",
        ))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredDecision {
    pub decision_sha256: String,
    pub decision: ReviewDecision,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectReviewPackage {
    pub package_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadReviewTextAsset {
    pub package_sha256: String,
    pub asset_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewTextAsset {
    pub package_sha256: String,
    pub asset_sha256: String,
    pub bytes: crate::domain::api_values::DecimalU64,
    pub media_type: String,
    pub utf8: String,
}
pub const MAX_TEXT_ASSET_BYTES: u64 = 65_536;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListReviewDecisions {
    pub package_sha256: String,
    pub item_id: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewPackageView {
    pub package_sha256: String,
    pub package: ReviewPackage,
    pub evidence_verified: bool,
    pub review_ready: bool,
    pub automatic_application_enabled: bool,
}
pub fn is_command(command: &str) -> bool {
    matches!(
        command,
        "reviews.inspect"
            | "reviews.record_decision"
            | "reviews.decisions"
            | "reviews.read_text_asset"
    )
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
pub fn read_text_asset(
    store: &impl ReviewStore,
    input: ReadReviewTextAsset,
) -> Result<ReviewTextAsset, ApiError> {
    if !valid_sha256(&input.package_sha256) || !valid_sha256(&input.asset_sha256) {
        return Err(invalid("Review package and asset SHA256 required"));
    }
    let session = store.open(&input.package_sha256)?;
    text_asset(session.package(), &input.asset_sha256)?;
    session.read_text_asset(&input.asset_sha256)
}
pub(crate) fn text_asset(package: &ReviewPackage, sha: &str) -> Result<(), ApiError> {
    let assets = package.assets();
    let asset = assets
        .iter()
        .find(|asset| asset.sha256 == sha)
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::NotFound,
                "Asset is not referenced by this review package",
            )
        })?;
    let mime = asset.media_type.split(';').next().unwrap_or("").trim();
    if asset.bytes.0 > MAX_TEXT_ASSET_BYTES
        || !(mime.starts_with("text/") || mime == "application/json")
        || assets
            .iter()
            .any(|other| other.sha256 == asset.sha256 && other.media_type != asset.media_type)
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Review text requires unambiguous text or JSON evidence at most 64 KiB",
        ));
    }
    Ok(())
}
pub fn inspect(
    store: &impl ReviewStore,
    input: InspectReviewPackage,
) -> Result<ReviewPackageView, ApiError> {
    if !valid_sha256(&input.package_sha256) {
        return Err(invalid("Review package SHA256 required"));
    }
    let session = store.open(&input.package_sha256)?;
    let package = session.package().clone();
    package.validate().map_err(invalid)?;
    let ready = package.items.iter().all(|i| i.baseline.is_some());
    Ok(ReviewPackageView {
        package_sha256: input.package_sha256,
        package,
        evidence_verified: true,
        review_ready: ready,
        automatic_application_enabled: false,
    })
}
pub fn record(
    store: &impl ReviewStore,
    input: RecordReviewDecision,
) -> Result<StoredDecision, ApiError> {
    input.validate().map_err(invalid)?;
    let session = store.open(&input.package_sha256)?;
    if !session
        .package()
        .items
        .iter()
        .any(|i| i.item_id == input.item_id)
    {
        return Err(ApiError::new(ErrorCode::NotFound, "Review item not found"));
    }
    let decision = input.bind(session.package()).map_err(invalid)?;
    session.append(&decision)
}
pub fn decisions(
    store: &impl ReviewStore,
    input: ListReviewDecisions,
) -> Result<Vec<StoredDecision>, ApiError> {
    if !valid_sha256(&input.package_sha256) || !valid_id(&input.item_id) {
        return Err(invalid("Invalid review package or item identity"));
    }
    let session = store.open(&input.package_sha256)?;
    if !session
        .package()
        .items
        .iter()
        .any(|i| i.item_id == input.item_id)
    {
        return Err(ApiError::new(ErrorCode::NotFound, "Review item not found"));
    }
    session.decisions(&input.item_id)
}
pub fn execute(request: &Request, store: &impl ReviewStore) -> Result<Value, ApiError> {
    if request.context.expected_revision.is_some()
        || request.context.idempotency_key.is_some()
        || request.context.timeout_ms.is_some()
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Review revision, idempotency and operation timeout are unavailable",
        ));
    }
    let parse = || invalid("Invalid review payload or unknown field");
    let encode = || ApiError::new(ErrorCode::Internal, "Invalid review response");
    match request.command.as_str() {
        "reviews.read_text_asset" => serde_json::to_value(read_text_asset(
            store,
            serde_json::from_value(request.payload.clone()).map_err(|_| parse())?,
        )?)
        .map_err(|_| encode()),
        "reviews.inspect" => serde_json::to_value(inspect(
            store,
            serde_json::from_value(request.payload.clone()).map_err(|_| parse())?,
        )?)
        .map_err(|_| encode()),
        "reviews.record_decision" => serde_json::to_value(record(
            store,
            serde_json::from_value(request.payload.clone()).map_err(|_| parse())?,
        )?)
        .map_err(|_| encode()),
        "reviews.decisions" => serde_json::to_value(decisions(
            store,
            serde_json::from_value(request.payload.clone()).map_err(|_| parse())?,
        )?)
        .map_err(|_| encode()),
        _ => Err(parse()),
    }
}
