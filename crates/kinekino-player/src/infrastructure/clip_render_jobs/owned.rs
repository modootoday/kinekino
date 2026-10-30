//! Core holds this original session before any native child starts.
use super::*;
use crate::application::review_source::{NativeProcessLedger, NativeProcessObservation};
use kinekino_jobs::application::owned_attempt::{
    AttemptScope, OwnedAttemptExecutor, OwnedAttemptSession, OwnedOutputCandidate,
    OwnerObservation, SessionRun,
};

struct ScopeControl<'a, 'b> {
    scope: &'a AttemptScope<'b>,
    native: &'a NativeProcessLedger,
}
impl MutationControl for ScopeControl<'_, '_> {
    fn native_processes(&self) -> Option<&NativeProcessLedger> {
        Some(self.native)
    }
    fn checkpoint(&self) -> Result<(), ApiError> {
        if self.scope.cancellation_requested() {
            Err(error(
                ErrorCode::Cancelled,
                "Original owned clip attempt cancelled",
            ))
        } else if Instant::now() >= self.scope.deadline() {
            Err(error(
                ErrorCode::Timeout,
                "Original owned clip deadline elapsed",
            ))
        } else {
            Ok(())
        }
    }
}
struct Session {
    executor: ClipRenderJobExecutor,
    native: NativeProcessLedger,
    candidate: Option<clip_renderer::ClipOwnedOutputCandidate>,
    attempted: bool,
    identity: kinekino_jobs::application::JobAttemptIdentityV1,
    deadline: Instant,
}
fn validate(input: &PreparedJobInput, scope: &AttemptScope<'_>) -> Result<(), ApiError> {
    input
        .validate()
        .map_err(|message| error(ErrorCode::InvalidArgument, message))?;
    if input.operation != RENDER_OPERATION || input.execution_timeout_ms.0 > 120_000 {
        return Err(error(
            ErrorCode::InvalidArgument,
            "Unsupported owned clip input or budget",
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
            "Original owned clip input binding changed",
        ));
    }
    Ok(())
}
impl OwnedAttemptExecutor for ClipRenderJobExecutor {
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
            native: Default::default(),
            candidate: None,
            attempted: false,
            identity: scope.identity().clone(),
            deadline: scope.deadline(),
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
        if self.attempted || &self.identity != scope.identity() || self.deadline != scope.deadline()
        {
            return SessionRun::Failed(error(
                ErrorCode::RevisionConflict,
                "Original owned clip session already consumed or rebound",
            ));
        }
        self.attempted = true;
        let result: Result<clip_renderer::ClipOwnedOutputCandidate, ApiError> = (|| {
            validate(input, scope)?;
            let control = ScopeControl {
                scope,
                native: &self.native,
            };
            control.checkpoint()?;
            let render = self.executor.render_retained(input, blobs, &control)?;
            self.candidate = Some(clip_renderer::ClipOwnedOutputCandidate::new(
                self.executor.artifacts.clone(),
                render,
                scope,
                self.executor.clone(),
            ));
            self.candidate
                .as_mut()
                .expect("Original candidate")
                .stage(scope)?;
            Ok(self.candidate.take().expect("Staged original candidate"))
        })();
        match result {
            Ok(candidate) => SessionRun::Candidate(Box::new(candidate)),
            Err(error) if error.code == ErrorCode::Cancelled => {
                match scope.observe_cooperative_cancellation() {
                    Some(marker) => SessionRun::CooperativeCancelled(marker),
                    None => SessionRun::Failed(error),
                }
            }
            Err(error) => SessionRun::Failed(error),
        }
    }
    fn poll_close(&mut self, deadline: Instant) -> Result<OwnerObservation, ApiError> {
        if let Some(candidate) = &mut self.candidate
            && !candidate.poll_release(deadline)?
        {
            return Ok(OwnerObservation::Pending);
        }
        let observations = self.native.observations();
        if observations.iter().any(|observation| !observation.closed()) {
            return Ok(OwnerObservation::Pending);
        }
        let mut exit = None;
        let mut forced = false;
        for observation in observations {
            if let NativeProcessObservation::Observed {
                child_exit: Some(code),
                forced: was_forced,
                ..
            } = observation
            {
                if exit.is_none() || code != 0 {
                    exit = Some(code as i32);
                }
                forced |= was_forced;
            }
        }
        Ok(match exit {
            None => OwnerObservation::NotStarted,
            Some(exit_code) => OwnerObservation::Exited {
                exit_code,
                family_empty: true,
                streams_eof: true,
                forced,
            },
        })
    }
}
