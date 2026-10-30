use crate::{
    application::{
        api::{ApiError, ErrorCode},
        jobs::{JobRecord, JobState, SubmitJob},
    },
    domain::{
        api_values::DecimalU64,
        job_batch::{BatchItemPin, BatchPlan},
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const MAX_BATCH_INPUT_BYTES: usize = 512 * 1024;
pub const MAX_BATCH_DOCUMENT_BYTES: usize = 24 * 1024 * 1024;
pub(super) fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
pub(super) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn request_sha256(request: &SubmitJob) -> Result<String, ApiError> {
    let bytes = serde_json::to_vec(request).map_err(|_| invalid("Invalid batch request"))?;
    if bytes.len() > kinekino_jobs::domain::MAX_PARAMETERS_BYTES + 4096 {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Batch request byte limit exceeded",
        ));
    }
    Ok(hash(&bytes))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchJobInput {
    pub item_id: String,
    pub request: SubmitJob,
}
/// A wire payload cannot claim that source/parameters have passed capture.
#[derive(Debug)]
pub struct CapturedBatchInput {
    input: BatchJobInput,
}
impl CapturedBatchInput {
    pub fn new(
        item_id: String,
        captured: crate::application::jobs::CapturedJobRequest,
    ) -> Result<Self, ApiError> {
        if !crate::domain::identifiers::valid_id(&item_id) {
            return Err(invalid("Invalid captured batch item identity"));
        }
        Ok(Self {
            input: BatchJobInput {
                item_id,
                request: captured.into_request(),
            },
        })
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum BatchProgress {
    Waiting,
    SubmitPending,
    /// A submission error is not proof that admission did not occur. Replay the same key.
    SubmitError {
        error: Box<ApiError>,
    },
    Observed {
        job: Box<JobRecord>,
        last_error: Option<Box<ApiError>>,
    },
    RetryPending {
        before: Box<JobRecord>,
        last_error: Option<Box<ApiError>>,
    },
}
impl BatchProgress {
    pub fn job(&self) -> Option<&JobRecord> {
        match self {
            Self::Observed { job, .. } => Some(job),
            Self::RetryPending { before, .. } => Some(before),
            _ => None,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchEntry {
    pub input: BatchJobInput,
    pub progress: BatchProgress,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchDocument {
    pub plan: BatchPlan,
    pub revision: DecimalU64,
    pub entries: Vec<BatchEntry>,
}
impl BatchDocument {
    pub fn from_captured(
        scope_sha256: String,
        batch_id: String,
        inputs: Vec<CapturedBatchInput>,
    ) -> Result<Self, ApiError> {
        Self::new(
            scope_sha256,
            batch_id,
            inputs.into_iter().map(|captured| captured.input).collect(),
        )
    }
    pub(super) fn new(
        scope_sha256: String,
        batch_id: String,
        inputs: Vec<BatchJobInput>,
    ) -> Result<Self, ApiError> {
        if inputs.is_empty() || inputs.len() > crate::domain::job_batch::MAX_BATCH_ITEMS {
            return Err(invalid("Batch requires1..128 inputs"));
        }
        let items = inputs
            .iter()
            .map(|input| {
                Ok(BatchItemPin {
                    item_id: input.item_id.clone(),
                    request_sha256: request_sha256(&input.request)?,
                })
            })
            .collect::<Result<Vec<_>, ApiError>>()?;
        let document = Self {
            plan: BatchPlan {
                schema_version: "1".into(),
                scope_sha256,
                batch_id,
                items,
            },
            revision: DecimalU64(1),
            entries: inputs
                .into_iter()
                .map(|input| BatchEntry {
                    input,
                    progress: BatchProgress::Waiting,
                })
                .collect(),
        };
        document.validate()?;
        Ok(document)
    }
    pub fn validate(&self) -> Result<(), ApiError> {
        self.plan.validate().map_err(invalid)?;
        if self.revision.0 == 0 || self.entries.len() != self.plan.items.len() {
            return Err(invalid("Invalid batch revision or entry count"));
        }
        let inputs = self
            .entries
            .iter()
            .map(|entry| &entry.input)
            .collect::<Vec<_>>();
        if serde_json::to_vec(&inputs)
            .map_err(|_| invalid("Invalid batch input"))?
            .len()
            > MAX_BATCH_INPUT_BYTES
        {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Batch input byte limit exceeded",
            ));
        }
        for (entry, pin) in self.entries.iter().zip(&self.plan.items) {
            if entry
                .input
                .request
                .expected_source_sha256
                .as_deref()
                .is_some_and(|sha| !crate::domain::identifiers::valid_sha256(sha))
            {
                return Err(invalid("Invalid captured batch source SHA"));
            }
            if entry.input.item_id != pin.item_id
                || request_sha256(&entry.input.request)? != pin.request_sha256
            {
                return Err(invalid("Batch input differs from its immutable plan"));
            }
            if let Some(job) = entry.progress.job() {
                job.validate().map_err(invalid)?;
                if entry
                    .input
                    .request
                    .expected_source_sha256
                    .as_ref()
                    .is_some_and(|sha| job.input.source.sha256 != *sha)
                {
                    return Err(invalid(
                        "Observed job source differs from its captured batch pin",
                    ));
                }
                let key = self.plan.submission_key(&pin.item_id).map_err(invalid)?;
                if job
                    .submission
                    .as_ref()
                    .is_none_or(|submission| submission.key_sha256 != hash(key.as_bytes()))
                {
                    return Err(invalid(
                        "Observed job is not bound to this batch item submission",
                    ));
                }
                if matches!(entry.progress, BatchProgress::RetryPending { .. })
                    && (!job.state.terminal() || job.state == JobState::Completed)
                {
                    return Err(invalid(
                        "Retry intent requires a non-completed terminal observation; Jobs decides admission",
                    ));
                }
            }
        }
        if serde_json::to_vec(self)
            .map_err(|_| invalid("Invalid batch document"))?
            .len()
            > MAX_BATCH_DOCUMENT_BYTES
        {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Batch document byte limit exceeded",
            ));
        }
        Ok(())
    }
}
