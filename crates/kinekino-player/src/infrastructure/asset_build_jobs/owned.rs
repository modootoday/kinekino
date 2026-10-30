use super::*;
use kinekino_jobs::application::{
    JobAttemptIdentityV1,
    owned_attempt::{
        AttemptScope, OwnedAttemptExecutor, OwnedAttemptSession, OwnedOutputCandidate,
        OwnerObservation, SessionRun,
    },
};
use std::time::Instant;
pub(super) struct ScopeControl<'a, 'b>(pub &'a AttemptScope<'b>);
impl MutationControl for ScopeControl<'_, '_> {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if self.0.cancellation_requested() {
            return Err(error(
                ErrorCode::Cancelled,
                "Asset attempt cancelled before publication",
            ));
        }
        deadline(self.0.deadline())
    }
}
pub(super) fn deadline(at: Instant) -> Result<(), ApiError> {
    if Instant::now() >= at {
        Err(error(
            ErrorCode::Timeout,
            "Original asset attempt deadline elapsed",
        ))
    } else {
        Ok(())
    }
}
fn validate(input: &PreparedJobInput, scope: &AttemptScope<'_>) -> Result<(), ApiError> {
    input
        .validate()
        .map_err(|message| error(ErrorCode::InvalidArgument, message))?;
    if input.operation != BUILD_OPERATION || input.execution_timeout_ms.0 > 120_000 {
        return Err(error(
            ErrorCode::InvalidArgument,
            "Unsupported asset operation/budget",
        ));
    }
    if scope.identity().source_blob_sha256() != input.source.sha256
        || scope.identity().input_sha256()
            != input
                .sha256()
                .map_err(|message| error(ErrorCode::InvalidArgument, message))?
    {
        return Err(error(
            ErrorCode::SourceExpired,
            "Original asset attempt/input binding changed",
        ));
    }
    Ok(())
}
struct Session {
    executor: AssetBuildJobExecutor,
    identity: JobAttemptIdentityV1,
    deadline: Instant,
    attempted: bool,
    candidate: Option<publication::AssetCandidate>,
}
impl OwnedAttemptExecutor for AssetBuildJobExecutor {
    type Candidate = Box<dyn OwnedOutputCandidate>;
    fn prepare_owned(
        &self,
        input: &PreparedJobInput,
        _blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> Result<Box<dyn OwnedAttemptSession<Candidate = Self::Candidate>>, ApiError> {
        validate(input, scope)?;
        Ok(Box::new(Session {
            executor: self.clone(),
            identity: scope.identity().clone(),
            deadline: scope.deadline(),
            attempted: false,
            candidate: None,
        }))
    }
}
impl OwnedAttemptSession for Session {
    type Candidate = Box<dyn OwnedOutputCandidate>;
    fn run(
        &mut self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> SessionRun<Self::Candidate> {
        if self.attempted || self.identity != *scope.identity() || self.deadline != scope.deadline()
        {
            return SessionRun::Failed(error(
                ErrorCode::RevisionConflict,
                "Asset session consumed or rebound",
            ));
        }
        self.attempted = true;
        let result: Result<publication::AssetCandidate, ApiError> = (|| {
            validate(input, scope)?;
            let control = ScopeControl(scope);
            control.checkpoint()?;
            let output = self.executor.render(input, blobs, &control)?;
            self.candidate = Some(publication::AssetCandidate::new(
                self.executor.artifacts.clone(),
                output,
                scope,
            )?);
            self.candidate
                .as_mut()
                .expect("Original asset candidate")
                .stage(scope)?;
            Ok(self.candidate.take().expect("Staged original asset"))
        })();
        match result {
            Ok(candidate) => SessionRun::Candidate(Box::new(candidate)),
            Err(failure) if failure.code == ErrorCode::Cancelled => {
                match scope.observe_cooperative_cancellation() {
                    Some(marker) => SessionRun::CooperativeCancelled(marker),
                    None => SessionRun::Failed(failure),
                }
            }
            Err(failure) => SessionRun::Failed(failure),
        }
    }
    fn poll_close(&mut self, deadline: Instant) -> Result<OwnerObservation, ApiError> {
        if let Some(candidate) = &mut self.candidate
            && !candidate.poll_release(deadline)?
        {
            return Ok(OwnerObservation::Pending);
        }
        // CPU/ordinary-file path never starts a native process; not an engine ACK.
        Ok(OwnerObservation::NotStarted)
    }
}
