use crate::application::OutputPublication;
use kinekino_jobs::domain::VerifiedJobOutput;
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::protocol::domain::{WorkerOutcomeCandidate, WorkerResult};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Take},
    sync::Arc,
    time::Instant,
};

/// Product immutable_storage_paths adapter holds its original directory guards.
/// It must reopen the same ordinary output inside that root, rejecting reparse.
/// No caller JSON path is accepted by this storage connector.
pub trait GuardedOutputFile: Send + Sync {
    fn path(&self) -> &str;
    fn open_actual(&self) -> Result<File, ApiError>;
}
pub struct JsonOutputPublication {
    file: Arc<dyn GuardedOutputFile>,
}
impl JsonOutputPublication {
    pub fn new(file: Arc<dyn GuardedOutputFile>) -> Self {
        Self { file }
    }
}
const LIMIT: u64 = 128 * 1024;
pub(super) fn encode(output: &WorkerOutcomeCandidate) -> Result<Vec<u8>, ApiError> {
    let WorkerResult::Complete { scores } = output.result() else {
        return Err(ApiError::new(
            ErrorCode::EngineFailed,
            "Only complete wire data can be published",
        ));
    };
    let actual_runtime=output.actual_runtime().map(|r|json!({"provider":r.provider(),"precision":r.precision(),"intra_threads":r.intra_threads(),"inter_threads":r.inter_threads(),"modules":r.modules().iter().map(|m|json!({"id":m.pin().id(),"sha256":m.pin().sha256(),"bytes":m.pin().bytes().to_string(),"canonical_path":m.canonical_path()})).collect::<Vec<_>>()}));
    let bytes=serde_json::to_vec(&json!({"schema_version":"1","kind":"vad_wire_observation","request_sha256":output.context().request().sha256(),"descriptor_sha256":output.context().descriptor().raw().sha256(),"worker_outcome_sha256":output.raw().sha256(),"worker_outcome_bytes":output.raw().bytes().to_string(),"start_sample":output.context().input().start().to_string(),"end_sample":output.context().input().end().to_string(),"scores":scores,"actual_runtime":actual_runtime,"runtime_admission_verified":false})).map_err(|_|ApiError::new(ErrorCode::Internal,"VAD output serialization failed"))?;
    if bytes.len() as u64 > LIMIT {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "VAD output exceeds bound",
        ));
    };
    Ok(bytes)
}
impl OutputPublication for JsonOutputPublication {
    fn publish(&self, output: &WorkerOutcomeCandidate, deadline: Instant) -> Result<(), ApiError> {
        kinekino_core::file_publication::publish_before(
            self.file.path(),
            &encode(output)?,
            Some(deadline),
        )
    }
    fn verify(
        &self,
        output: &WorkerOutcomeCandidate,
        deadline: Instant,
    ) -> Result<VerifiedJobOutput, ApiError> {
        if Instant::now() >= deadline {
            return Err(ApiError::new(ErrorCode::Timeout, "VAD verification cutoff"));
        };
        let expected = encode(output)?;
        let mut bytes = Vec::new();
        let mut reader: Take<File> = self.file.open_actual()?.take(LIMIT + 1);
        reader
            .read_to_end(&mut bytes)
            .map_err(kinekino_core::file_publication::io_error)?;
        if bytes != expected {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Published VAD bytes changed or exceeded bound",
            ));
        };
        if Instant::now() >= deadline {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "VAD verification completed after cutoff",
            ));
        };
        let verified = VerifiedJobOutput {
            path: self.file.path().to_owned(),
            sha256: format!("{:x}", Sha256::digest(&bytes)),
            bytes: DecimalU64(bytes.len() as u64),
            receipt: json!({"schema_version":"1","worker_outcome_sha256":output.raw().sha256(),"request_sha256":output.context().request().sha256(),"runtime_admission_verified":false}),
        };
        verified
            .validate()
            .map_err(|m| ApiError::new(ErrorCode::Internal, m))?;
        Ok(verified)
    }
}
