use super::{ExecutionControl, JobExecutor, SourceBlobStore};
use crate::domain::{PreparedJobInput, VerifiedJobOutput};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};

pub struct ExecutorRegistry {
    operations: BTreeMap<String, Arc<dyn JobExecutor>>,
}
impl ExecutorRegistry {
    pub fn new(executors: Vec<Arc<dyn JobExecutor>>) -> Result<Self, ApiError> {
        let mut operations = BTreeMap::new();
        for executor in executors {
            for operation in executor.operations() {
                if operation.is_empty()
                    || !executor.supports(&operation)
                    || operations.insert(operation, executor.clone()).is_some()
                {
                    return Err(ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Job executor operations must be explicit and unique",
                    ));
                }
            }
        }
        Ok(Self { operations })
    }
    fn executor(&self, operation: &str) -> Result<&dyn JobExecutor, ApiError> {
        self.operations
            .get(operation)
            .map(Arc::as_ref)
            .ok_or_else(|| ApiError::new(ErrorCode::Unsupported, "Job operation is unavailable"))
    }
}
impl JobExecutor for ExecutorRegistry {
    fn supports_batch_capture(&self, operation: &str) -> bool {
        self.executor(operation)
            .is_ok_and(|executor| executor.supports_batch_capture(operation))
    }
    fn owned_executor(&self, operation: &str) -> Option<&super::owned_attempt::OwnedExecutorPort> {
        self.operations.get(operation)?.owned_executor(operation)
    }
    fn operations(&self) -> Vec<String> {
        self.operations.keys().cloned().collect()
    }
    fn supports(&self, operation: &str) -> bool {
        self.operations.contains_key(operation)
    }
    fn normalize_parameters(&self, operation: &str, parameters: Value) -> Result<Value, ApiError> {
        self.executor(operation)?
            .normalize_parameters(operation, parameters)
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.executor(operation)
            .ok()?
            .execution_timeout_limit_ms(operation)
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        self.executor(operation)?
            .prepare(operation, parameters, expected, blobs)
    }
    fn execute(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        self.executor(&input.operation)?
            .execute(input, blobs, control)
    }
}
