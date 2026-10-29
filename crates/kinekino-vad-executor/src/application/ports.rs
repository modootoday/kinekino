use kinekino_jobs::{
    application::{JobAttemptIdentityV1, owned_attempt::OwnerObservation},
    domain::VerifiedJobOutput,
};
use kinekino_media_contracts::errors::ApiError;
use kinekino_vad::protocol::domain::{DecodedDescriptor, WorkerOutcomeCandidate};
use std::{io::Read, time::Instant};

/// Read-only adapter view. It cannot mint a jobs binding, marker or permit.
pub trait RunControl {
    fn deadline(&self) -> Instant;
    fn cancellation_requested(&self) -> bool;
}
pub struct WorkerRead {
    pub declared_bytes: u64,
    pub stdout: Box<dyn Read + Send>,
}
/// Trusted native implementations own original handles *before* launch_read.
/// Every partial launch owner stays in self, including on Err or unwind.
/// Reader EOF is raw I/O evidence, not family settlement.
/// Persist original stdout/stderr before exposing the bounded reader; decode
/// failures must leave the same raw files and handles in this transport.
pub trait WorkerTransport: Send {
    /// Parent successfully created and synced this original session's signal.
    /// Worker JSON/exit labels cannot supply this observation.
    fn cancellation_signal_sent(&self) -> bool {
        false
    }
    fn launch_read(
        &mut self,
        request: &[u8],
        control: &dyn RunControl,
    ) -> Result<WorkerRead, ApiError>;
    /// Constrain this caller cap by the already-owned supervisor's original
    /// cleanup end; never replace it with a renewed duration or execution cutoff.
    fn poll_close(&mut self, deadline: Instant) -> Result<OwnerObservation, ApiError>;
}
/// Preparation allocates an inert session only: no process, thread or OS worker.
pub trait WorkerTransportFactory: Send + Sync {
    fn prepare(&self) -> Result<Box<dyn WorkerTransport>, ApiError>;
}
pub struct PreparedResources {
    pub encoder: std::sync::Arc<dyn RequestEncoder>,
    pub transport: Box<dyn WorkerTransport>,
    pub publication: std::sync::Arc<dyn OutputPublication>,
}
/// Trusted host composition creates fresh attempt resources without launching.
/// Identity/end are original parent data, not authority minted by this factory.
pub trait AttemptResourcesFactory: Send + Sync {
    fn prepare(
        &self,
        identity: &JobAttemptIdentityV1,
        original_end: Instant,
    ) -> Result<PreparedResources, ApiError>;
}
/// Trusted encoder chooses no executable, shell, argv or environment from JSON.
pub trait RequestEncoder: Send + Sync {
    fn encode(
        &self,
        identity: &JobAttemptIdentityV1,
        descriptor: &DecodedDescriptor,
    ) -> Result<Vec<u8>, ApiError>;
}
/// Core alone schedules publication after settlement/CAS. Implementations reuse
/// guarded no-overwrite/sync storage and verify actual bytes before returning.
pub trait OutputPublication: Send + Sync {
    fn publish(&self, output: &WorkerOutcomeCandidate, deadline: Instant) -> Result<(), ApiError>;
    fn verify(
        &self,
        output: &WorkerOutcomeCandidate,
        deadline: Instant,
    ) -> Result<VerifiedJobOutput, ApiError>;
}
