//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait BatchApi: Send + Sync {
    fn create(&self, input: CreateBatch) -> Result<BatchReceipt, ApiError>;
    fn inspect(&self, input: InspectBatch, expected: Option<u64>) -> Result<BatchPage, ApiError>;
    fn execute(
        &self,
        input: BatchIdentity,
        expected: u64,
        action: BatchAction,
    ) -> Result<BatchReceipt, ApiError>;
}
