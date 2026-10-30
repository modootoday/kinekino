//! Bounded live-host correlation records. Never evict an ID into new execution.
use super::*;
use sha2::{Digest, Sha256};
pub const MAX_RECORDS: usize = 128;
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmissionIdentity {
    pub host_scope: String,
    pub submission_id: String,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct SubmissionReceipt {
    pub submission: SubmissionIdentity,
    pub operation: HostOperationReference,
    pub request_sha256: String,
    pub acknowledged: bool,
    pub replayed: bool,
}
pub(super) struct Record {
    ordinal: u64,
    request_sha256: String,
}
impl HostExecution {
    fn validate_submission(&self, identity: &SubmissionIdentity) -> Result<(), ApiError> {
        if identity.host_scope != self.scope {
            return Err(stale());
        }
        if !crate::application::api::commands::valid_request_id(&identity.submission_id) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Bounded submission ID required",
            ));
        }
        Ok(())
    }
    fn submission_receipt(
        &self,
        identity: &SubmissionIdentity,
        record: &Record,
        state: &Retained,
        replayed: bool,
    ) -> SubmissionReceipt {
        SubmissionReceipt {
            submission: identity.clone(),
            operation: HostOperationReference {
                host_scope: self.scope.clone(),
                ordinal: record.ordinal.to_string(),
            },
            request_sha256: record.request_sha256.clone(),
            acknowledged: !state.operations.contains_key(&record.ordinal),
            replayed,
        }
    }
    pub fn inspect_submission(
        &self,
        identity: &SubmissionIdentity,
    ) -> Result<SubmissionReceipt, ApiError> {
        self.validate_submission(identity)?;
        let state = self
            .retained
            .lock()
            .map_err(|_| internal("Host submission ownership unavailable"))?;
        let record = state
            .submissions
            .get(&identity.submission_id)
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::NotFound,
                    "Submission ID was not admitted in this host",
                )
            })?;
        Ok(self.submission_receipt(identity, record, &state, false))
    }
    pub fn submit_correlated(
        &self,
        identity: &SubmissionIdentity,
        request: Request,
    ) -> Result<SubmissionReceipt, ApiError> {
        self.validate_submission(identity)?;
        request.validate()?;
        if !matches!(
            encoded_size(&request, self.max_request_bytes),
            EncodingMeasurement::Exact(_)
        ) {
            return Err(capacity());
        }
        let bytes = serde_json::to_vec(&request)
            .map_err(|_| internal("Submission request serialization unavailable"))?;
        let digest = format!("{:x}", Sha256::digest(&bytes));
        let mut state = self
            .retained
            .lock()
            .map_err(|_| internal("Host submission ownership unavailable"))?;
        if let Some(record) = state.submissions.get(&identity.submission_id) {
            if record.request_sha256 != digest {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Submission ID already belongs to a different original request",
                ));
            }
            return Ok(self.submission_receipt(identity, record, &state, true));
        }
        if state.submissions.len() >= MAX_RECORDS || state.operations.len() >= self.capacity {
            return Err(capacity());
        }
        let ordinal = state.next;
        let next = ordinal.checked_add(1).ok_or_else(capacity)?;
        let id = self.attachment.submit(request)?;
        state.operations.insert(ordinal, id);
        state.next = next;
        state.submissions.insert(
            identity.submission_id.clone(),
            Record {
                ordinal,
                request_sha256: digest,
            },
        );
        Ok(self.submission_receipt(
            identity,
            &state.submissions[&identity.submission_id],
            &state,
            false,
        ))
    }
}
