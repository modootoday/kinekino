//! Preparation contracts and borrowed custody in the original core session.
use super::{DownloadOutput, DownloadSession, DownloadVerifier};
use crate::application::{
    api::{ApiError, ErrorCode},
    jobs::{
        ExecutionControl, SourceBlobStore,
        owned_attempt::{AttemptScope, OwnerObservation},
    },
};
use kinekino_jobs::domain::PreparedJobInput;
use std::{sync::Arc, time::Instant};

pub trait SourceOpener: Send {
    fn poll_release(&mut self, deadline: Instant) -> Result<bool, ApiError> {
        self.poll_close(deadline)
    }
    fn owner_observation(&self) -> OwnerObservation {
        OwnerObservation::Pending
    }
    fn open(
        &mut self,
        control: &dyn ExecutionControl,
    ) -> Result<Box<dyn DownloadSession + Send>, ApiError>;
    fn poll_close(&mut self, deadline: Instant) -> Result<bool, ApiError>;
}
pub struct PreparedDownload {
    pub source: Box<dyn SourceOpener>,
    pub output: Arc<dyn DownloadOutput + Send + Sync>,
    pub verifier: Arc<dyn DownloadVerifier + Send + Sync>,
    pub maximum_bytes: u64,
}
pub struct PreparedDownloadResources {
    pub output: Arc<dyn DownloadOutput + Send + Sync>,
    pub verifier: Arc<dyn DownloadVerifier + Send + Sync>,
    pub maximum_bytes: u64,
}
/// Non-Clone/non-Serialize borrowed context, never a new pending owner carrier.
pub struct SourceCustody<'a> {
    source: &'a mut Option<Box<dyn SourceOpener>>,
}
impl<'a> SourceCustody<'a> {
    pub(super) fn new(source: &'a mut Option<Box<dyn SourceOpener>>) -> Result<Self, ApiError> {
        if source.is_some() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original source custody is already assigned",
            ));
        }
        Ok(Self { source })
    }
    /// Consumed exactly once; core already owns this slot before the callback.
    /// Adopt before any fallible callback after accepting a live source owner.
    pub fn adopt(self, source: Box<dyn SourceOpener>) {
        *self.source = Some(source);
    }
}
pub trait DownloadPreparation: Send + Sync {
    /// Legacy launch-free/file-only path; may not take a live owner's custody.
    fn prepare(
        &self,
        _: &PreparedJobInput,
        _: &dyn SourceBlobStore,
        _: &AttemptScope<'_>,
    ) -> Result<PreparedDownload, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Source preparation requires original session custody",
        ))
    }
    fn prepare_in_session(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
        custody: SourceCustody<'_>,
    ) -> Result<PreparedDownloadResources, ApiError> {
        let prepared = self.prepare(input, blobs, scope)?;
        custody.adopt(prepared.source);
        Ok(PreparedDownloadResources {
            output: prepared.output,
            verifier: prepared.verifier,
            maximum_bytes: prepared.maximum_bytes,
        })
    }
}
