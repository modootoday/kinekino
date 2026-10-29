use crate::application::RequestEncoder;
use kinekino_jobs::application::JobAttemptIdentityV1;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::protocol::domain::DecodedDescriptor;
use serde_json::json;

/// Internal transport declaration only. Worker support is a separate gate.
pub struct PinnedRequestEncoder;
impl RequestEncoder for PinnedRequestEncoder {
    fn encode(
        &self,
        identity: &JobAttemptIdentityV1,
        descriptor: &DecodedDescriptor,
    ) -> Result<Vec<u8>, ApiError> {
        let context =
            crate::application::descriptor_snapshot(descriptor.profile(), descriptor.input());
        serde_json::to_vec(&json!({"schema_version":"1","job_id":identity.job_id().as_str(),"attempt":identity.attempt().to_string(),"input_sha256":identity.input_sha256(),"source_blob_sha256":identity.source_blob_sha256(),"submission_request_sha256":identity.submission_request_sha256(),"descriptor_sha256":descriptor.raw().sha256(),"context":context})).map_err(|_|ApiError::new(ErrorCode::Internal,"Internal VAD request serialization failed"))
    }
}
