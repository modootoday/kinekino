use crate::domain::{JobId, JobRecord};

/// Immutable provenance captured by the original worker after its start CAS.
/// Not live status, publication permission, a cleanup proof, or an API request ID.
/// External input/worker JSON cannot construct this host snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobAttemptIdentityV1 {
    job_id: JobId,
    attempt: u64,
    input_sha256: String,
    source_blob_sha256: String,
    submission_request_sha256: Option<String>,
}
impl JobAttemptIdentityV1 {
    /// Only the coordinator's already-validated running record supplies identity.
    pub(super) fn from_running_record(record: &JobRecord) -> Self {
        Self {
            job_id: record.job_id.clone(),
            attempt: record.attempt.0,
            input_sha256: record.input_sha256.clone(),
            source_blob_sha256: record.input.source.sha256.clone(),
            submission_request_sha256: record
                .submission
                .as_ref()
                .map(|submission| submission.request_sha256.clone()),
        }
    }
    pub fn job_id(&self) -> &JobId {
        &self.job_id
    }
    pub fn attempt(&self) -> u64 {
        self.attempt
    }
    /// Fingerprint of the complete prepared input, distinct from source blob bytes.
    pub fn input_sha256(&self) -> &str {
        &self.input_sha256
    }
    pub fn source_blob_sha256(&self) -> &str {
        &self.source_blob_sha256
    }
    /// None preserves a historical admission without a submission fingerprint.
    pub fn submission_request_sha256(&self) -> Option<&str> {
        self.submission_request_sha256.as_deref()
    }
}
