//! Public request binding. Actual profile/audio admission and execution are separate.
mod prepared;
mod product_host;
pub mod product_release;
pub use product_host::{ProductVadHostOptions, ProductVadHostPlan};
mod request;
pub use request::{VAD_JOB_OPERATION, VadJobParameters, VadSourceRequest, normalize_parameters};
#[cfg(test)]
mod tests;
