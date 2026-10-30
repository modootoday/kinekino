//! Readback values validate layout, never mint a Jobs completion/quality authority.
mod contents;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        asset_bundle::{AssetBundleFile, MAX_ASSET_BUNDLE_BYTES, MAX_ASSET_RENDER_UNITS},
    },
    domain::{
        api_values::DecimalU64,
        identifiers::{valid_id, valid_sha256},
    },
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_ASSET_PUBLICATION_BYTES: u64 = 32 * 1024 * 1024;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectAssetArtifact {
    pub artifact_id: String,
    pub publication_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetArtifactInspection {
    pub publication: AssetArtifactPublication,
    pub publication_sha256: String,
    pub publication_bytes: DecimalU64,
    pub manifest: crate::application::asset_bundle::AssetBundleManifest,
}
/// A point-in-time readback, never an admission lease or completion/quality authority.
pub trait AssetArtifactInspector: Send + Sync {
    fn inspect_artifact(
        &self,
        input: &InspectAssetArtifact,
        control: &dyn crate::application::review_source::MutationControl,
    ) -> Result<AssetArtifactInspection, ApiError>;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetExecutionBinding {
    pub job_id: String,
    pub attempt: DecimalU64,
    pub input_sha256: String,
    pub source_blob_sha256: String,
    pub submission_request_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetArtifactPublication {
    pub schema_version: String,
    pub artifact_id: String,
    pub execution: AssetExecutionBinding,
    pub bundle_manifest: AssetBundleFile,
    pub files: Vec<AssetBundleFile>,
    pub review_artifact_only: bool,
    pub quality_approved: bool,
}
fn corrupt() -> ApiError {
    ApiError::new(
        ErrorCode::NeedsInput,
        "Asset artifact publication layout is invalid; preserve original files",
    )
}
impl AssetArtifactPublication {
    pub fn validate(&self, expected_id: &str, publication_bytes: u64) -> Result<(), ApiError> {
        let job: Result<kinekino_jobs::domain::JobId, _> = self.execution.job_id.clone().try_into();
        if self.schema_version != "asset-artifact/1"
            || !valid_id(expected_id)
            || self.artifact_id != expected_id
            || job.is_err()
            || self.execution.attempt.0 == 0
            || self.artifact_id
                != format!(
                    "{}-attempt-{}",
                    self.execution.job_id, self.execution.attempt.0
                )
            || !self.review_artifact_only
            || self.quality_approved
            || publication_bytes == 0
            || publication_bytes > MAX_ASSET_PUBLICATION_BYTES
            || [
                &self.execution.input_sha256,
                &self.execution.source_blob_sha256,
                &self.execution.submission_request_sha256,
            ]
            .iter()
            .any(|sha| !valid_sha256(sha))
            || self.files.len() < 5
            || self.files.len() > MAX_ASSET_RENDER_UNITS * 3 + 2
            || !(self.files.len() - 2).is_multiple_of(3)
        {
            return Err(corrupt());
        }
        let mut names = BTreeSet::new();
        let mut total = publication_bytes;
        for (index, file) in self.files.iter().enumerate() {
            if !valid_sha256(&file.sha256)
                || file.bytes.0 == 0
                || file.bytes.0 > MAX_ASSET_BUNDLE_BYTES
                || !names.insert(&file.name)
            {
                return Err(corrupt());
            }
            let valid = if index == 0 {
                file.name == "project.json" && file.media_type == "application/json"
            } else if index == self.files.len() - 1 {
                file.name == "manifest.json"
                    && file.media_type == "application/json"
                    && *file == self.bundle_manifest
                    && file.bytes.0 <= MAX_ASSET_PUBLICATION_BYTES
            } else {
                let unit = (index - 1) / 3;
                let kind = (index - 1) % 3;
                let expected_suffix = match kind {
                    0 => ".rgba8",
                    1 => ".png",
                    _ => ".mask.png",
                };
                let expected_tail = format!("-unit-{unit:06}{expected_suffix}");
                file.name
                    .strip_prefix("frame-")
                    .and_then(|name| name.strip_suffix(&expected_tail))
                    .is_some_and(|position| {
                        position.len() == 6
                            && position.bytes().all(|b| b.is_ascii_digit())
                            && position.parse::<usize>().is_ok_and(|position| {
                                position < crate::domain::asset_generation::MAX_ASSET_FRAMES
                            })
                    })
                    && file.media_type
                        == if kind == 0 {
                            "application/octet-stream"
                        } else {
                            "image/png"
                        }
            };
            if !valid {
                return Err(corrupt());
            }
            total = total
                .checked_add(file.bytes.0)
                .filter(|total| *total <= MAX_ASSET_BUNDLE_BYTES)
                .ok_or_else(corrupt)?;
        }
        Ok(())
    }
}
