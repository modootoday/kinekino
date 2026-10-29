use crate::application::{RunControl, WorkerTransport};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::protocol::{
    application::decode_worker_outcome,
    domain::{ExpectedWorkerContext, WorkerOutcomeCandidate},
};
use std::time::Instant;

/// Parent session retains this state across failed reads and run unwinds.
pub(super) struct Driver {
    pub transport: Box<dyn WorkerTransport>,
    request: Vec<u8>,
    context: ExpectedWorkerContext,
    started: bool,
    pub observation: Option<WorkerOutcomeCandidate>,
}
impl Driver {
    pub fn new(
        transport: Box<dyn WorkerTransport>,
        request: Vec<u8>,
        context: ExpectedWorkerContext,
    ) -> Self {
        Self {
            transport,
            request,
            context,
            started: false,
            observation: None,
        }
    }
    pub fn run(&mut self, control: &dyn RunControl) -> Result<(), ApiError> {
        if self.started {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original VAD session cannot launch twice",
            ));
        }
        if control.cancellation_requested() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "VAD cancelled before launch; no ACK",
            ));
        }
        if Instant::now() >= control.deadline() {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "VAD deadline expired before launch",
            ));
        }
        self.started = true;
        let raw = self.transport.launch_read(&self.request, control)?;
        self.observation = Some(decode_worker_outcome(
            raw.stdout,
            raw.declared_bytes,
            &self.context,
        )?);
        if control.cancellation_requested() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "VAD cancelled after read; original owner retained",
            ));
        }
        if Instant::now() >= control.deadline() {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Original VAD deadline expired during read",
            ));
        }
        Ok(())
    }
}
