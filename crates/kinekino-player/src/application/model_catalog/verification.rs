//! Point-in-time file integrity, never a persisted executable model admission.
use super::*;
use crate::{application::review_source::MutationControl, domain::api_values::DecimalU64};
use std::time::{Duration, Instant};
pub const MAX_VERIFY_BYTES: u64 = 1_099_511_627_776;
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyModel {
    pub model_id: String,
    pub revision: String,
    pub catalog_sha256: String,
    pub maximum_bytes: DecimalU64,
}
impl VerifyModel {
    pub fn validate(&self) -> Result<(), ApiError> {
        if !crate::domain::identifiers::valid_id(&self.model_id)
            || !fixed_revision(&self.revision)
            || !crate::domain::identifiers::valid_sha256(&self.catalog_sha256)
            || !(1..=MAX_VERIFY_BYTES).contains(&self.maximum_bytes.0)
        {
            return Err(invalid(
                "Exact model/catalog pins and positive bounded byte budget required",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFileCheckState {
    Missing,
    SizeMismatch,
    HashMismatch,
    Matched,
}
#[derive(Debug, Serialize)]
pub struct ModelFileCheck {
    pub path: String,
    pub expected_bytes: DecimalU64,
    pub expected_sha256: String,
    pub observed_bytes: Option<DecimalU64>,
    pub observed_sha256: Option<String>,
    pub state: ModelFileCheckState,
}
#[derive(Debug, Serialize)]
pub struct ModelVerification {
    pub model_id: String,
    pub revision: String,
    pub files: Vec<ModelFileCheck>,
}
pub fn verify_model(
    port: &dyn ModelCatalogPort,
    input: &VerifyModel,
    control: &dyn MutationControl,
) -> Result<Value, ApiError> {
    input.validate()?;
    control.checkpoint()?;
    if !port.verification_available() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Model file verification unavailable",
        ));
    }
    let (catalog, digest) = port.snapshot()?;
    control.checkpoint()?;
    catalog.validate().map_err(invalid)?;
    if digest != input.catalog_sha256 {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Catalog pin differs",
        ));
    }
    let model = catalog
        .models
        .iter()
        .find(|m| m.model_id == input.model_id && m.revision == input.revision)
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Model revision not declared"))?;
    let total = model
        .files
        .iter()
        .try_fold(0u64, |total, file| total.checked_add(file.bytes.0))
        .ok_or_else(|| ApiError::new(ErrorCode::ResourceExhausted, "Model byte sum overflow"))?;
    if total > input.maximum_bytes.0 {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Declared model exceeds verification byte budget",
        ));
    }
    let result = port.verify_files(input, control)?;
    control.checkpoint()?;
    if result.model_id != input.model_id
        || result.revision != input.revision
        || result.files.len() != model.files.len()
        || result.files.iter().zip(&model.files).any(|(check, file)| {
            check.path != file.path
                || check.expected_bytes != file.bytes
                || check.expected_sha256 != file.sha256
                || check.state == ModelFileCheckState::Matched
                    && (check.observed_bytes != Some(file.bytes)
                        || check.observed_sha256.as_ref() != Some(&file.sha256))
        })
    {
        return Err(ApiError::new(
            ErrorCode::Internal,
            "Verification identity differs",
        ));
    }
    let matched = result
        .files
        .iter()
        .all(|file| file.state == ModelFileCheckState::Matched);
    let report = json!({"catalog_sha256":input.catalog_sha256,"data":{"verification":result,"all_declared_files_matched":matched,"integrity":if matched{"matched_at_verification"}else{"not_matched"},"persistent_admission":false},"runtime_admitted":false,"quality_approved":false,"license_permission":"not_verified"});
    if !matched {
        let mut error = ApiError::new(
            ErrorCode::NeedsInput,
            "Declared model files did not match; review integrity report",
        );
        error.details = report;
        return Err(error);
    }
    Ok(report)
}
struct Deadline<'a> {
    at: Instant,
    external: Option<&'a dyn MutationControl>,
}
impl MutationControl for Deadline<'_> {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if let Some(external) = self.external {
            external.checkpoint()?;
        }
        if Instant::now() >= self.at {
            Err(ApiError::new(
                ErrorCode::Timeout,
                "Model verification deadline expired",
            ))
        } else {
            Ok(())
        }
    }
}
pub(super) fn execute(
    port: &dyn ModelCatalogPort,
    request: &Request,
    external: Option<&dyn MutationControl>,
) -> Result<Value, ApiError> {
    let input: VerifyModel = serde_json::from_value(request.payload.clone())
        .map_err(|_| invalid("Expected exact model/catalog pins and maximum_bytes"))?;
    let millis = request.context.timeout_ms.unwrap_or(30000);
    if !(1..=120000).contains(&millis) {
        return Err(invalid("Verification timeout must be 1..120000ms"));
    }
    verify_model(
        port,
        &input,
        &Deadline {
            at: Instant::now() + Duration::from_millis(millis),
            external,
        },
    )
}
pub(super) fn input_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["model_id","revision","catalog_sha256","maximum_bytes"],"properties":{"model_id":{"type":"string","minLength":1,"maxLength":80},"revision":{"type":"string","pattern":"^[a-f0-9]{40}$"},"catalog_sha256":{"type":"string","pattern":"^[a-f0-9]{64}$"},"maximum_bytes":{"type":"string","pattern":"^[0-9]+$","description":"1..1099511627776 bytes; sum of declared model bytes must fit before any model file read"}}})
}
