mod publication;
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        card_bundle::{self, CardBundleRequest, PreparedCardBundle},
        card_projects::CardProjectStore,
    },
    domain::api_values::DecimalU64,
    infrastructure::card_export::CardBundleFiles,
};
use kinekino_jobs::{
    application::{ExecutionControl, JobExecutor, SourceBlobStore},
    domain::{MAX_PARAMETERS_BYTES, PreparedJobInput, VerifiedJobOutput},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::Arc;
pub const OPERATION: &str = card_bundle::EXPORT_OPERATION;
pub struct CardBundleJobExecutor {
    projects: Arc<dyn CardProjectStore>,
}
impl CardBundleJobExecutor {
    pub fn new(projects: Arc<dyn CardProjectStore>) -> Self {
        Self { projects }
    }
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn parse(operation: &str, parameters: Value) -> Result<CardBundleRequest, ApiError> {
    if operation != OPERATION {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Card bundle operation unavailable",
        ));
    }
    if serde_json::to_vec(&parameters)
        .map_err(|_| invalid("Invalid bundle parameters"))?
        .len()
        > MAX_PARAMETERS_BYTES
    {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Bundle parameter byte limit exceeded",
        ));
    }
    serde_json::from_value(parameters).map_err(|_| invalid("Expected strict card bundle request"))
}
impl JobExecutor for CardBundleJobExecutor {
    fn supports_batch_capture(&self, operation: &str) -> bool {
        self.supports(operation)
    }
    fn operations(&self) -> Vec<String> {
        vec![OPERATION.into()]
    }
    fn supports(&self, operation: &str) -> bool {
        operation == OPERATION
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.supports(operation).then_some(120_000)
    }
    fn normalize_parameters(&self, operation: &str, parameters: Value) -> Result<Value, ApiError> {
        serde_json::to_value(parse(operation, parameters)?)
            .map_err(|_| invalid("Invalid bundle parameters"))
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        let request = parse(operation, parameters)?;
        let normalized =
            serde_json::to_value(&request).map_err(|_| invalid("Invalid bundle request"))?;
        let prepared = card_bundle::prepare(self.projects.as_ref(), &CardBundleFiles, request)?;
        let snapshot =
            serde_json::to_string(&prepared).map_err(|_| invalid("Invalid bundle snapshot"))?;
        let sha = digest(snapshot.as_bytes());
        if expected.is_some_and(|s| s != sha) {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Prepared bundle snapshot differs from expected descriptor SHA256",
            ));
        }
        let input = PreparedJobInput {
            operation: OPERATION.into(),
            parameters: normalized,
            source: blobs.put_utf8(&snapshot)?,
            execution_timeout_ms: DecimalU64(30_000),
        };
        input.validate().map_err(invalid)?;
        Ok(input)
    }
    fn execute(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        input.validate().map_err(invalid)?;
        if input.execution_timeout_ms.0 > 120_000 {
            return Err(invalid("Card execution budget exceeds supported limit"));
        }
        let request = parse(&input.operation, input.parameters.clone())?;
        card_bundle::check_control(control)?;
        let snapshot = blobs.read_utf8(&input.source)?;
        if digest(snapshot.as_bytes()) != input.source.sha256
            || snapshot.len() as u64 != input.source.bytes.0
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Stored bundle snapshot changed",
            ));
        }
        let prepared: PreparedCardBundle = serde_json::from_str(&snapshot)
            .map_err(|_| invalid("Invalid captured bundle snapshot"))?;
        // Re-read the same immutable publication and currently pinned inputs; never refresh to latest.
        let current = card_bundle::prepare(self.projects.as_ref(), &CardBundleFiles, request)?;
        if serde_json::to_string(&current).map_err(|_| invalid("Invalid current bundle"))?
            != snapshot
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Bundle inputs changed after admission",
            ));
        }
        card_bundle::execute_bundle(
            &prepared,
            &CardBundleFiles,
            &publication::NativeBundlePublication,
            control,
        )
    }
}
