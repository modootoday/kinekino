//! Source-only ownership policy. No OS adapter or second lifecycle cleanup driver.
pub mod probe_host;
#[cfg(test)]
mod probe_host_tests;
mod registry;
pub use registry::{
    ConstructionFailure, CreationFailure, KeyError, RecoveryKey, RecoveryRegistry,
    ReleaseObservation, ReleaseState, ReservationError, ReservationFailure,
};
#[cfg(test)]
mod tests;
