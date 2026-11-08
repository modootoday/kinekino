//! Export one fixed stored project revision to a new local file (create-only, no overwrite).
use super::{InspectProject, ProjectExportTarget, ProjectStore, inspect_with_proposals, invalid};
use crate::{application::proposals::ProposalStore, domain::api_values::DecimalU64};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Component, Path};

pub const FORMAT: &str = "kinekino.project-export";
pub const FORMAT_VERSION: &str = "1";
pub const COLLISION_POLICY: &str = "reject";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportProject {
    pub project_id: String,
    pub revision: DecimalU64,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportedProject {
    pub project_id: String,
    pub revision: DecimalU64,
    pub revision_sha256: String,
    pub output_path: String,
    pub output_sha256: String,
    pub output_bytes: DecimalU64,
    pub format: String,
    pub format_version: String,
    pub collision_policy: String,
    pub overwrite: bool,
}

pub fn export(
    store: &dyn ProjectStore,
    proposals: &dyn ProposalStore,
    target: &dyn ProjectExportTarget,
    input: ExportProject,
) -> Result<ExportedProject, ApiError> {
    if !store.available() {
        return Err(super::unavailable());
    }
    validate_output_path(&input.output_path)?;
    if input.revision.0 == 0 {
        return Err(invalid("Invalid project identity or revision"));
    }
    let state = inspect_with_proposals(
        store,
        proposals,
        InspectProject {
            project_id: input.project_id.clone(),
            revision: Some(input.revision),
        },
    )?;
    if state.project.project_id != input.project_id || state.project.revision != input.revision {
        return Err(ApiError::new(
            ErrorCode::Internal,
            "Stored project revision does not match the requested identity",
        ));
    }
    let document = serde_json::json!({
        "format": FORMAT,
        "format_version": FORMAT_VERSION,
        "revision_sha256": state.revision_sha256,
        "project": state.project,
    });
    let bytes = serde_json::to_vec(&document)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Project export serialization failed"))?;
    target.publish_new(&input.output_path, &bytes)?;
    Ok(ExportedProject {
        project_id: input.project_id,
        revision: input.revision,
        revision_sha256: state.revision_sha256,
        output_path: input.output_path,
        output_sha256: kinekino_media_contracts::hex_lower(Sha256::digest(&bytes)),
        output_bytes: DecimalU64(bytes.len() as u64),
        format: FORMAT.into(),
        format_version: FORMAT_VERSION.into(),
        collision_policy: COLLISION_POLICY.into(),
        overwrite: false,
    })
}

fn validate_output_path(path: &str) -> Result<(), ApiError> {
    let output = Path::new(path);
    let names_file = matches!(output.components().next_back(), Some(Component::Normal(_)));
    if path.trim().is_empty()
        || !output.is_absolute()
        || !names_file
        || output
            .components()
            .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(invalid(
            "Explicit absolute output path naming a new file is required",
        ));
    }
    Ok(())
}
