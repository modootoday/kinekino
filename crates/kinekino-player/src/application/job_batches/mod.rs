//! Durable intent/observation contract for batches using the original Jobs API.
#[path = "ports.rs"]
mod ports;
pub use ports::BatchStore;
mod document;
mod orchestration;
pub mod service;
mod succession;
#[cfg(test)]
mod tests;
use crate::application::api::ApiError;
pub use document::*;
pub use orchestration::{BatchAction, run};
pub use succession::{validate_initial, validate_successor};

/// Original writer lease covers load, durable intent, Jobs call, and observation.
/// Publish must check CAS, validate_successor and preserve immutable event history.
/// A failed publish must never expose a partial record as the current revision.
pub trait BatchLease {
    fn load(&self) -> Result<BatchDocument, ApiError>;
    fn publish(&mut self, expected_revision: u64, document: &BatchDocument)
    -> Result<(), ApiError>;
}
