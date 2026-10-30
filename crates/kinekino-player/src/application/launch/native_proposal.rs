//! Local launch wire contract, independent of presentation and Native platform types.
mod options;
pub use options::{NativeProposalLaunchOptions, native_proposal_options};

use crate::{
    application::api::{ApiError, ErrorCode, RequestContext},
    domain::{api_values::DecimalU64, proposals::DocumentTextRef},
};
use serde::{Deserialize, Serialize};

pub const NATIVE_PROPOSAL_SCHEMA_VERSION: &str = "1";
pub const MAX_NATIVE_PROPOSAL_REQUEST_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProposalLaunchRequest {
    pub schema_version: String,
    pub context: RequestContext,
    pub selection: NativeProposalSelection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProposalSelection {
    pub proposal_id: String,
    pub document_revision: DecimalU64,
    pub source_id: String,
    pub source_revision: DecimalU64,
    pub source_sha256: String,
    pub target: NativeProposalTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NativeProposalTarget {
    Card {
        index: u32,
    },
    Highlight {
        index: u32,
    },
    CueCorrection {
        index: u32,
        cue_id: String,
    },
    DocumentCorrection {
        index: u32,
        source_ref: DocumentTextRef,
    },
    DocumentRepair {
        selection: DocumentRepairSelection,
    },
    ManualRepair {
        task: crate::domain::proposals::ProposalTask,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DocumentRepairSelection {
    Unselected {},
    Selected { source_ref: DocumentTextRef },
}

impl NativeProposalTarget {
    /// Validate the platform conversion without selecting a default item.
    pub fn checked_item_index(&self) -> Result<Option<usize>, ApiError> {
        let index = match self {
            Self::Card { index }
            | Self::Highlight { index }
            | Self::CueCorrection { index, .. }
            | Self::DocumentCorrection { index, .. } => *index,
            Self::DocumentRepair { .. } | Self::ManualRepair { .. } => return Ok(None),
        };
        usize::try_from(index).map(Some).map_err(|_| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Proposal index exceeds platform range",
            )
        })
    }
}

pub fn decode_native_proposal_request(
    bytes: &[u8],
) -> Result<NativeProposalLaunchRequest, ApiError> {
    if bytes.len() > MAX_NATIVE_PROPOSAL_REQUEST_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Native proposal launch request exceeds 64 KiB",
        ));
    }
    let request: NativeProposalLaunchRequest = serde_json::from_slice(bytes).map_err(|error| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            format!("Invalid native proposal launch JSON: {error}"),
        )
    })?;
    request.context.validate()?;
    if request.schema_version != NATIVE_PROPOSAL_SCHEMA_VERSION {
        let mut error = ApiError::new(
            ErrorCode::Unsupported,
            "Unsupported native proposal launch schema version",
        );
        error.request_id.clone_from(&request.context.request_id);
        return Err(error);
    }
    request.selection.target.checked_item_index()?;
    // Snapshot identity, target membership and supported inspect context remain SDK/UI policy.
    Ok(request)
}

#[cfg(test)]
mod tests;
