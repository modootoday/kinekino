//! Wire correlation derived from original coordinator provenance, not permission.
use crate::application::{
    api::{ApiError, ErrorCode},
    download::{DOWNLOAD_OPERATION, DownloadRequest},
    jobs::JobAttemptIdentityV1,
};
use kinekino_jobs::domain::PreparedJobInput;
use sha2::{Digest, Sha256};

pub(super) struct BoundRequest {
    pub digest: [u8; 32],
    pub maximum_bytes: u64,
}
impl BoundRequest {
    pub fn new(
        identity: &JobAttemptIdentityV1,
        input: &PreparedJobInput,
    ) -> Result<Self, ApiError> {
        if input.operation != DOWNLOAD_OPERATION
            || input.sha256().ok().as_deref() != Some(identity.input_sha256())
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Wire request input differs from original attempt",
            ));
        }
        let request = DownloadRequest::decode(input.parameters.clone())?;
        let mut hash = Sha256::new();
        hash.update(b"source-attempt-wire/1\0");
        field(&mut hash, identity.job_id().as_str().as_bytes());
        hash.update(identity.attempt().to_be_bytes());
        field(&mut hash, identity.input_sha256().as_bytes());
        field(&mut hash, identity.source_blob_sha256().as_bytes());
        match identity.submission_request_sha256() {
            Some(value) => {
                hash.update([1]);
                field(&mut hash, value.as_bytes());
            }
            None => hash.update([0]),
        }
        hash.update(request.maximum_bytes.0.to_be_bytes());
        Ok(Self {
            digest: hash.finalize().into(),
            maximum_bytes: request.maximum_bytes.0,
        })
    }
}
fn field(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}
