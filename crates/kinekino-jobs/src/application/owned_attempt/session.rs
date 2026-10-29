use super::{
    AttemptBinding, AttemptFailure, AttemptScope, OwnerObservation, RetainedAttempt,
    RetainedAttemptOwner,
};
use crate::{application::SourceBlobStore, domain::PreparedJobInput};
use kinekino_media_contracts::errors::ApiError;
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
    time::Instant,
};

/// The core calls publish only after settlement and its publication CAS. It calls
/// verify only after entering verification; adapters still own ordinary-file,
/// no-overwrite, sync and actual output-byte validation.
pub trait OwnedOutputCandidate: Send {
    /// Nonblocking original-resource cleanup. Pending/error must retain this object.
    /// Called after verification, or while discarding an unpublished/failed output.
    fn poll_release(&mut self, deadline: Instant) -> Result<bool, ApiError>;
    fn publish(&mut self) -> Result<(), ApiError>;
    fn verify(&mut self) -> Result<crate::domain::VerifiedJobOutput, ApiError>;
}
pub type OwnedExecutorPort =
    dyn super::OwnedAttemptExecutor<Candidate = Box<dyn OwnedOutputCandidate>>;

/// A trusted adapter returns this *before* launch. Core retains the same session
/// across run/poll errors and unwinds. Spawn's partial owner must stay in it too.
pub trait OwnedAttemptSession: Send {
    type Candidate: Send + 'static;
    fn run(
        &mut self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> SessionRun<Self::Candidate>;
    fn poll_close(&mut self, deadline: Instant) -> Result<OwnerObservation, ApiError>;
}

/// Untrusted worker output cannot deserialize or mint this parent observation.
/// It is not an ACK; core still requires original CAS and graceful family/EOF proof.
pub struct CooperativeCancellation {
    pub(super) binding: Arc<AttemptBinding>,
}
pub enum SessionRun<T> {
    Candidate(T),
    CooperativeCancelled(CooperativeCancellation),
    Failed(ApiError),
}
pub enum SessionCandidate<T> {
    Output(T),
    CooperativeCancelled(CooperativeCancellation),
}
impl AttemptScope<'_> {
    pub fn observe_cooperative_cancellation(&self) -> Option<CooperativeCancellation> {
        self.cancellation_requested()
            .then(|| CooperativeCancellation {
                binding: Arc::clone(&self.binding),
            })
    }
}

struct SessionOwner<T: Send + 'static> {
    session: Box<dyn OwnedAttemptSession<Candidate = T>>,
}
impl<T: Send + 'static> RetainedAttemptOwner for SessionOwner<T> {
    fn poll_close(&mut self, deadline: Instant) -> Result<OwnerObservation, ApiError> {
        self.session.poll_close(deadline)
    }
}
pub(in crate::application) struct SessionExecution<T> {
    pub retained: RetainedAttempt<SessionCandidate<T>>,
    pub first_error: Option<ApiError>,
    pub panic_payload: Option<Box<dyn Any + Send>>,
}

/// Session is held outside catch_unwind; run can borrow it but cannot unwind its
/// parent-owned Box. Opaque panic payload remains with the original attempt too.
pub(in crate::application) fn run_prepared<T: Send + 'static>(
    mut session: Box<dyn OwnedAttemptSession<Candidate = T>>,
    input: &PreparedJobInput,
    blobs: &dyn SourceBlobStore,
    scope: AttemptScope<'_>,
) -> SessionExecution<T> {
    let mut first_error = None;
    let mut panic_payload = None;
    let result = match catch_unwind(AssertUnwindSafe(|| session.run(input, blobs, &scope))) {
        Ok(SessionRun::Candidate(candidate)) => Ok(SessionCandidate::Output(candidate)),
        Ok(SessionRun::CooperativeCancelled(marker)) => {
            Ok(SessionCandidate::CooperativeCancelled(marker))
        }
        Ok(SessionRun::Failed(error)) => {
            let copy = AttemptFailure::capture(&error);
            first_error = Some(error);
            Err(ApiError::new(copy.code(), copy.message()))
        }
        Err(payload) => {
            panic_payload = Some(payload);
            let error = ApiError::new(
                kinekino_media_contracts::errors::ErrorCode::Internal,
                "Original owned session run panicked",
            );
            first_error = Some(ApiError::new(error.code, &error.message));
            Err(error)
        }
    };
    SessionExecution {
        retained: scope.retain(Box::new(SessionOwner { session }), result),
        first_error,
        panic_payload,
    }
}
