use crate::application::{
    api::{ApiError, ErrorCode},
    jobs::ExecutionControl,
};
use std::{path::Path, sync::Arc};

/// Only the original attempt can issue the opaque local cancellation marker.
/// Adapter errors, including Cancelled, always remain failures.
pub enum OwnedTransferError {
    LocalCancellation(crate::application::jobs::owned_attempt::CooperativeCancellation),
    Failure(ApiError),
}
impl From<ApiError> for OwnedTransferError {
    fn from(error: ApiError) -> Self {
        Self::Failure(error)
    }
}
pub trait OwnedTransferControl: ExecutionControl {
    fn checkpoint(&self) -> Result<(), OwnedTransferError>;
}

/// Keeps the SAME sealed writer/file/directory guards alive until physical probe close.
/// It is a read/retention lease, never permission to append, rename or delete.
pub trait VerificationLease: Send + Sync {
    fn path(&self) -> &Path;
    fn check_current(&self) -> Result<(), ApiError>;
}

#[must_use = "Pending owns the original probe and sealed-output guards"]
pub enum OwnedVerification {
    Complete(Result<(), ApiError>),
    Pending(Box<dyn PendingMediaVerification>),
}
pub enum VerificationPoll {
    Pending,
    /// Physical child/family/stream close and its receipt have been observed.
    Closed(Result<(), ApiError>),
}
pub trait PendingMediaVerification: Send {
    fn owner_observation(&self) -> crate::application::jobs::owned_attempt::OwnerObservation {
        crate::application::jobs::owned_attempt::OwnerObservation::Pending
    }
    fn request_cancel(&mut self) -> Result<(), ApiError>;
    fn poll(&mut self, control: &dyn ExecutionControl) -> VerificationPoll;
    fn first_error(&self) -> Option<&ApiError>;
}

#[must_use = "Retain Pending in the original job owner until physical close"]
#[cfg(test)]
pub enum OwnedDownload {
    Complete(DownloadReceipt),
    Pending(Box<PendingDownload>),
}
#[cfg(test)]
pub enum DownloadPoll {
    Pending { cleanup_pending: bool },
    Finished(Result<DownloadReceipt, ApiError>),
}
pub(super) struct AdmittedTransfer {
    pub identity: MediaIdentity,
    pub length: Option<u64>,
    pub digest: Option<[u8; 32]>,
}
#[must_use = "Dropping an unsettled transfer is not physical probe cleanup"]
#[cfg(test)]
pub struct PendingDownload {
    pub(super) session: Box<dyn DownloadSession + Send>,
    pub(super) partial: Option<Box<dyn PartialOutput + Send>>,
    pub(super) probe: Option<Box<dyn PendingMediaVerification>>,
    pub(super) control: Arc<dyn ExecutionControl>,
    pub(super) admitted: AdmittedTransfer,
    pub(super) receipt: Option<DownloadReceipt>,
    pub(super) primary: Option<ApiError>,
    pub(super) cleanup_error: Option<ApiError>,
    pub(super) cancellation_sent: bool,
    pub(super) finished: bool,
}

/// Nonsensitive identity selected by trusted provider resolution, never a signed URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaIdentity {
    pub media: String,
    pub tracks: String,
}

/// Expectations describe their origin; an observed length is not a content digest.
pub struct TransferEvidence {
    pub identity: MediaIdentity,
    pub response_length: Option<u64>,
    /// Only an independently trusted digest may populate this field.
    pub trusted_sha256: Option<[u8; 32]>,
}

/// Issued by the provider adapter retaining actual request headers/session lifetime.
/// Reads must respect control deadlines and cancellation while blocked on transport.
/// No URL/cookie serialization, Debug or caller-constructible session token contract.
pub trait DownloadSession {
    fn poll_release(&mut self, _: std::time::Instant) -> Result<bool, ApiError> {
        self.close_owned().map(|()| true)
    }
    fn read_owned(
        &mut self,
        buffer: &mut [u8],
        control: &dyn OwnedTransferControl,
    ) -> Result<usize, OwnedTransferError> {
        self.read(buffer, control)
            .map_err(OwnedTransferError::Failure)
    }
    fn check_origin(&self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        self.check_current(control)
    }
    fn close_owned(&mut self) -> Result<(), ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Owned provider request close is unavailable",
        ))
    }
    fn evidence(&self) -> &TransferEvidence;
    fn check_current(&self, control: &dyn ExecutionControl) -> Result<(), ApiError>;
    fn read(
        &mut self,
        buffer: &mut [u8],
        control: &dyn ExecutionControl,
    ) -> Result<usize, ApiError>;
}

pub trait PartialOutput {
    fn publish_owned(&mut self) -> Result<(), ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Retained output publication is unavailable",
        ))
    }
    fn abort_owned(&mut self) -> Result<(), ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Retained output cleanup is unavailable",
        ))
    }
    fn verify_published(
        &mut self,
        _receipt: &DownloadReceipt,
        _control: &dyn ExecutionControl,
    ) -> Result<crate::application::jobs::VerifiedJobOutput, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Original-handle published verification is unavailable",
        ))
    }
    fn verification_lease(&self) -> Result<Arc<dyn VerificationLease>, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Owned sealed-output verification is unavailable",
        ))
    }
    fn append(&mut self, bytes: &[u8]) -> Result<(), ApiError>;
    fn seal(&mut self) -> Result<(), ApiError>;
    fn path(&self) -> &Path;
    fn abort(self: Box<Self>) -> Result<(), ApiError>;
    /// Must recheck original directory/file guards and never replace an existing target.
    fn publish(self: Box<Self>) -> Result<(), ApiError>;
}

pub trait DownloadOutput {
    fn create_owned(&self) -> Result<Box<dyn PartialOutput + Send>, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Owned partial output is unavailable",
        ))
    }
    /// The returned owner removes only its own partial file on drop before publication.
    fn create(&self) -> Result<Box<dyn PartialOutput>, ApiError>;
}

/// Concrete adapter probes tracks, duration and file readability, not just extensions.
pub trait DownloadVerifier {
    /// Preparation MUST NOT launch. Core stores Pending in its pre-owned session
    /// before poll may launch. Path-only default verification is not owned support.
    fn begin_owned(
        &self,
        _lease: Arc<dyn VerificationLease>,
        _expected: MediaIdentity,
        _control: &dyn ExecutionControl,
    ) -> Result<OwnedVerification, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Owned media verification is unavailable",
        ))
    }
    fn verify(
        &self,
        path: &Path,
        expected: &MediaIdentity,
        control: &dyn ExecutionControl,
    ) -> Result<(), ApiError>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct DownloadReceipt {
    pub identity: MediaIdentity,
    pub bytes: u64,
    pub sha256: [u8; 32],
}
