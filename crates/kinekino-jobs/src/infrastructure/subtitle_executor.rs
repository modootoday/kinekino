mod operation;
mod snapshot;
#[cfg(all(test, windows))]
mod tests;

use super::RootedJobPaths;
use crate::{
    application::{ExecutionControl, JobExecutor, SourceBlobStore},
    domain::{PreparedJobInput, VerifiedJobOutput, valid_sha256},
};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use operation::Request;
use serde_json::Value;
use sha2::{Digest, Sha256};
use snapshot::{SnapshotStorage, open_source, read_source, selected_path};
use std::{cell::RefCell, sync::Arc};

pub struct SubtitleJobExecutor {
    paths: Arc<dyn RootedJobPaths>,
}
impl SubtitleJobExecutor {
    pub fn new(paths: Arc<dyn RootedJobPaths>) -> Self {
        Self { paths }
    }
}
fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn expired(control: &dyn ExecutionControl) -> Result<(), ApiError> {
    if std::time::Instant::now() >= control.deadline() {
        Err(ApiError::new(
            ErrorCode::Timeout,
            "Job execution deadline expired",
        ))
    } else {
        Ok(())
    }
}
impl JobExecutor for SubtitleJobExecutor {
    fn supports_batch_capture(&self, operation: &str) -> bool {
        self.supports(operation)
    }
    fn operations(&self) -> Vec<String> {
        operation::OPERATIONS
            .iter()
            .map(|name| (*name).into())
            .collect()
    }
    fn supports(&self, operation: &str) -> bool {
        operation::OPERATIONS.contains(&operation)
    }
    fn normalize_parameters(&self, operation: &str, parameters: Value) -> Result<Value, ApiError> {
        Request::parse(operation, parameters)?.parameters()
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.supports(operation).then_some(120000)
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        if !self.supports(operation) {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Job executor supports only SRT/VTT/transcript subtitle operations",
            ));
        }
        if expected.is_some_and(|sha| !valid_sha256(sha)) {
            return Err(invalid("Invalid expected source SHA256"));
        }
        let request = Request::parse(operation, parameters)?;
        let mut selected = open_source(self.paths.as_ref(), request.input_path())?;
        let source = read_source(&mut selected.file)?;
        let sha = digest(source.as_bytes());
        if expected.is_some_and(|expected| expected != sha) {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Selected source differs from the expected SHA256",
            ));
        }
        let (output, _guards) = selected_path(self.paths.as_ref(), request.output_path())?;
        if selected.path == output || request.input_path() == request.output_path() {
            return Err(invalid("Input and output must differ"));
        }
        let input = PreparedJobInput {
            operation: operation.into(),
            parameters: request.parameters()?,
            source: blobs.put_utf8(&source)?,
            execution_timeout_ms: DecimalU64(30000),
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
        if !self.supports(&input.operation) || input.execution_timeout_ms.0 > 120000 {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Unsupported task or SRT/VTT execution budget",
            ));
        }
        if control.cancellation_requested() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Cancellation requested before conversion",
            ));
        }
        expired(control)?;
        let request = Request::parse(&input.operation, input.parameters.clone()).map_err(|_| {
            invalid(match input.operation.as_str() {
                "subtitles.convert" => "Invalid saved conversion parameters",
                "subtitles.export_range" => "Invalid saved range export parameters",
                _ => "Invalid saved transcript subtitle parameters",
            })
        })?;
        let mut selected = open_source(self.paths.as_ref(), request.input_path())?;
        let current = read_source(&mut selected.file)?;
        if digest(current.as_bytes()) != input.source.sha256 {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Source file changed after admission; the captured snapshot remains preserved",
            ));
        }
        let snapshot = blobs.read_utf8(&input.source)?;
        expired(control)?;
        let (_output, _output_guards) = selected_path(self.paths.as_ref(), request.output_path())?;
        let storage = SnapshotStorage {
            source_path: request.input_path(),
            output_path: request.output_path(),
            snapshot: &snapshot,
            control,
            paths: self.paths.as_ref(),
            verified: RefCell::new(None),
        };
        let result = request.execute(&storage)?;
        let mut verified = storage.verified.borrow_mut().take().ok_or_else(|| {
            ApiError::new(
                ErrorCode::EngineFailed,
                "Conversion returned without verified publication",
            )
        })?;
        verified.receipt = result;
        verified.validate().map_err(invalid)?;
        Ok(verified)
    }
}
