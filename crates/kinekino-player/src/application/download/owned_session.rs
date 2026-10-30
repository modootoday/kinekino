//! Uses the existing jobs core's pre-owned session and retained closeout. No new
//! public pending owner Box, publication gate, process registry or blocking Drop.
use super::{
    AdmittedTransfer, DownloadPreparation, DownloadReceipt, DownloadSession, OwnedTransferControl,
    OwnedTransferError, OwnedVerification, PartialOutput, PendingMediaVerification,
    PreparedDownloadResources, SourceCustody, SourceOpener, VerificationPoll,
    transfer::{admit, final_source, receive_owned},
};
use crate::application::{
    api::{ApiError, ErrorCode},
    jobs::{
        ExecutionControl, JobAttemptIdentityV1, SourceBlobStore, VerifiedJobOutput,
        owned_attempt::{
            AttemptScope, OwnedAttemptExecutor, OwnedAttemptSession, OwnedOutputCandidate,
            OwnerObservation, SessionRun,
        },
    },
};
use kinekino_jobs::domain::PreparedJobInput;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub(crate) struct DownloadOwnedExecutor {
    pub preparation: Arc<dyn DownloadPreparation>,
}
struct DownloadCoreSession {
    candidate_returned: bool,
    preparation: Arc<dyn DownloadPreparation>,
    prepared: Option<PreparedDownloadResources>,
    source_opener: Option<Box<dyn SourceOpener>>,
    binding: JobAttemptIdentityV1,
    deadline: Instant,
    source: Option<Box<dyn DownloadSession + Send>>,
    partial: Option<Box<dyn PartialOutput + Send>>,
    probe: Option<Box<dyn PendingMediaVerification>>,
    admitted: Option<AdmittedTransfer>,
    receipt: Option<DownloadReceipt>,
    first: Option<ApiError>,
    last_observation: OwnerObservation,
    cancel_sent: bool,
    ran: bool,
}
impl OwnedAttemptExecutor for DownloadOwnedExecutor {
    type Candidate = Box<dyn OwnedOutputCandidate>;
    fn prepare_owned(
        &self,
        input: &PreparedJobInput,
        _blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> Result<Box<dyn OwnedAttemptSession<Candidate = Self::Candidate>>, ApiError> {
        if input.operation != "production.download_media" {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Owned download operation differs",
            ));
        }
        Ok(Box::new(DownloadCoreSession {
            candidate_returned: false,
            preparation: self.preparation.clone(),
            prepared: None,
            source_opener: None,
            binding: scope.identity().clone(),
            deadline: scope.deadline(),
            source: None,
            partial: None,
            probe: None,
            admitted: None,
            receipt: None,
            first: None,
            last_observation: OwnerObservation::NotStarted,
            cancel_sent: false,
            ran: false,
        }))
    }
}
struct ScopeControl<'a, 's>(&'a AttemptScope<'s>);
impl ExecutionControl for ScopeControl<'_, '_> {
    fn execution_identity(&self) -> Option<&JobAttemptIdentityV1> {
        Some(self.0.identity())
    }
    fn cancellation_requested(&self) -> bool {
        self.0.cancellation_requested()
    }
    fn deadline(&self) -> Instant {
        self.0.deadline()
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        Err(internal("Only core settlement may reserve publication"))
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        Err(internal("Only core may enter verification"))
    }
}
impl OwnedTransferControl for ScopeControl<'_, '_> {
    fn checkpoint(&self) -> Result<(), OwnedTransferError> {
        if let Some(marker) = self.0.observe_cooperative_cancellation() {
            return Err(OwnedTransferError::LocalCancellation(marker));
        }
        if Instant::now() >= self.0.deadline() {
            return Err(
                ApiError::new(ErrorCode::Timeout, "Original download deadline expired").into(),
            );
        }
        Ok(())
    }
}
struct ClosedControl {
    deadline: Instant,
}
impl ExecutionControl for ClosedControl {
    fn cancellation_requested(&self) -> bool {
        true
    }
    fn deadline(&self) -> Instant {
        self.deadline
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        Err(internal("Closeout cannot publish"))
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        Err(internal("Closeout cannot verify"))
    }
}
impl DownloadCoreSession {
    fn run_inner(
        &mut self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> Result<Box<dyn OwnedOutputCandidate>, OwnedTransferError> {
        if self.ran
            || scope.identity() != &self.binding
            || scope.deadline() != self.deadline
            || input
                .sha256()
                .map_err(|_| internal("Invalid prepared download input"))?
                != self.binding.input_sha256()
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Download original attempt binding differs",
            )
            .into());
        }
        self.ran = true;
        let control = ScopeControl(scope);
        control.checkpoint()?;
        let preparation = self.preparation.clone();
        let resources = preparation.prepare_in_session(
            input,
            blobs,
            scope,
            SourceCustody::new(&mut self.source_opener)?,
        )?;
        let output = resources.output.clone();
        let verifier = resources.verifier.clone();
        let maximum_bytes = resources.maximum_bytes;
        self.prepared = Some(resources);
        control.checkpoint()?;
        self.source = Some(
            self.source_opener
                .as_mut()
                .ok_or_else(|| internal("Preparation did not adopt an original source"))?
                .open(&control)?,
        );
        self.admitted = Some(admit(
            self.source.as_ref().unwrap().as_ref(),
            &control,
            maximum_bytes,
        )?);
        self.partial = Some(output.create_owned()?);
        self.receipt = Some(receive_owned(
            self.source.as_mut().unwrap().as_mut(),
            self.partial.as_mut().unwrap().as_mut(),
            &control,
            self.admitted.as_ref().unwrap(),
            maximum_bytes,
        )?);
        control.checkpoint()?;
        final_source(
            self.source.as_ref().unwrap().as_ref(),
            &control,
            self.admitted.as_ref().unwrap(),
        )?;
        let lease = self.partial.as_ref().unwrap().verification_lease()?;
        lease.check_current()?;
        match verifier.begin_owned(
            lease,
            self.admitted.as_ref().unwrap().identity.clone(),
            &control,
        )? {
            OwnedVerification::Complete(result) => result?,
            // Preparation starts no process. Store BEFORE any poll/start callback.
            OwnedVerification::Pending(probe) => self.probe = Some(probe),
        }
        while let Some(probe) = self.probe.as_mut() {
            control.checkpoint()?;
            match probe.poll(&control) {
                VerificationPoll::Pending => {
                    if let Some(error) = probe.first_error() {
                        return Err(copy_error(error).into());
                    }
                    std::thread::sleep(
                        Duration::from_millis(2)
                            .min(self.deadline.saturating_duration_since(Instant::now())),
                    );
                }
                VerificationPoll::Closed(result) => {
                    let observation = probe.owner_observation();
                    if !closed(observation) {
                        return Err(
                            internal("Probe closure lacks original owner observation").into()
                        );
                    }
                    self.last_observation = observation;
                    self.probe.take();
                    result?;
                }
            }
        }
        control.checkpoint()?;
        final_source(
            self.source.as_ref().unwrap().as_ref(),
            &control,
            self.admitted.as_ref().unwrap(),
        )?;
        self.source.as_mut().unwrap().close_owned()?;
        // No CAS/publish here. Core retains owner and candidate across unwinds,
        // observes settlement, then invokes its one publication gate.
        let candidate = Box::new(DownloadCandidate {
            partial: self.partial.take().unwrap(),
            source: self.source.take().unwrap(),
            receipt: self.receipt.take().unwrap(),
            deadline: self.deadline,
            published: false,
        });
        self.candidate_returned = true;
        Ok(candidate)
    }
}
impl OwnedAttemptSession for DownloadCoreSession {
    type Candidate = Box<dyn OwnedOutputCandidate>;
    fn run(
        &mut self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> SessionRun<Self::Candidate> {
        match self.run_inner(input, blobs, scope) {
            Ok(candidate) => SessionRun::Candidate(candidate),
            Err(OwnedTransferError::LocalCancellation(marker)) => {
                SessionRun::CooperativeCancelled(marker)
            }
            Err(OwnedTransferError::Failure(error)) => {
                if self.first.is_none() {
                    self.first = Some(copy_error(&error));
                }
                SessionRun::Failed(error)
            }
        }
    }
    fn poll_close(&mut self, deadline: Instant) -> Result<OwnerObservation, ApiError> {
        self.deadline = self.deadline.min(deadline);
        let mut close_error = None;
        // Drive the ORIGINAL source independently of a pending/failed probe.
        let mut source_closed = false;
        match self
            .source_opener
            .as_mut()
            .map_or(Ok(true), |source| source.poll_close(self.deadline))
        {
            Ok(true) => {
                source_closed = true;
            }
            Ok(false) => {}
            Err(error) => retain_close_error(&mut self.first, &mut close_error, error),
        }
        let source_observation = if source_closed {
            self.source_opener
                .as_ref()
                .map_or(OwnerObservation::NotStarted, |source| {
                    source.owner_observation()
                })
        } else {
            OwnerObservation::Pending
        };
        let mut probe_closed = self.probe.is_none();
        if let Some(probe) = self.probe.as_mut() {
            if !self.cancel_sent {
                self.cancel_sent = true;
                if let Err(error) = probe.request_cancel() {
                    retain_close_error(&mut self.first, &mut close_error, error);
                }
            }
            match probe.poll(&ClosedControl {
                deadline: self.deadline,
            }) {
                VerificationPoll::Pending => {}
                VerificationPoll::Closed(result) => {
                    let observation = probe.owner_observation();
                    if let Err(error) = result {
                        retain_close_error(&mut self.first, &mut close_error, error);
                    }
                    if closed(observation) {
                        self.last_observation = observation;
                        self.probe.take();
                        probe_closed = true;
                    } else {
                        retain_close_error(
                            &mut self.first,
                            &mut close_error,
                            internal("Original probe closeout remains unobserved"),
                        );
                    }
                }
            }
        }
        // A failed run cannot publish. Drive its original origin cancellation
        // independently of source receipt failure or a still-pending probe.
        // Retain session/partial/leases until ALL original cleanup gates close.
        let mut origin_released = self.candidate_returned;
        if !self.candidate_returned {
            match self
                .source_opener
                .as_mut()
                .map_or(Ok(true), |source| source.poll_release(self.deadline))
            {
                Ok(released) => origin_released = released,
                Err(error) => retain_close_error(&mut self.first, &mut close_error, error),
            }
        }
        if let Some(error) = close_error {
            return Err(error);
        }
        if !probe_closed || !source_closed || !closed(source_observation) || !origin_released {
            return Ok(OwnerObservation::Pending);
        }
        // Keep the original source session/guards alongside the probe lease.
        // Host IPC/process close above must not release these guards early.
        if let Some(source) = self.source.as_mut()
            && let Err(error) = source.close_owned()
        {
            if self.first.is_none() {
                self.first = Some(copy_error(&error));
            }
            return Err(error);
        }
        self.last_observation = merge_observations(self.last_observation, source_observation);
        if let Some(partial) = self.partial.as_mut() {
            partial.abort_owned()?;
        }
        self.partial.take();
        self.source.take();
        Ok(self.last_observation)
    }
}
/// At most four close errors per poll. Preserve first run failure and every
/// parallel close error before returning one structured diagnostic to the core.
fn retain_close_error(
    first: &mut Option<ApiError>,
    pending: &mut Option<ApiError>,
    error: ApiError,
) {
    if first.is_none() {
        *first = Some(copy_error(&error));
    }
    if let Some(primary) = pending {
        let details = std::mem::take(&mut primary.details);
        primary.details =
            serde_json::json!({"primary_details":details,"parallel_close_failure":error});
    } else {
        *pending = Some(error);
    }
}
fn merge_observations(left: OwnerObservation, right: OwnerObservation) -> OwnerObservation {
    use OwnerObservation::*;
    match (left, right) {
        (Pending, _) | (_, Pending) => Pending,
        (NotStarted, observation) | (observation, NotStarted) => observation,
        (
            Exited {
                exit_code: left_code,
                family_empty: left_empty,
                streams_eof: left_eof,
                forced: left_forced,
            },
            Exited {
                exit_code: right_code,
                family_empty: right_empty,
                streams_eof: right_eof,
                forced: right_forced,
            },
        ) => Exited {
            exit_code: if left_code != 0 {
                left_code
            } else {
                right_code
            },
            family_empty: left_empty && right_empty,
            streams_eof: left_eof && right_eof,
            forced: left_forced || right_forced,
        },
    }
}
struct PublicationControl {
    deadline: Instant,
}
impl ExecutionControl for PublicationControl {
    fn cancellation_requested(&self) -> bool {
        false
    } // Core publication CAS already won.
    fn deadline(&self) -> Instant {
        self.deadline
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        Err(internal("Candidate cannot reserve a second publication"))
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        Err(internal("Candidate cannot change core state"))
    }
}
struct DownloadCandidate {
    partial: Box<dyn PartialOutput + Send>,
    source: Box<dyn DownloadSession + Send>,
    receipt: DownloadReceipt,
    deadline: Instant,
    published: bool,
}
impl OwnedOutputCandidate for DownloadCandidate {
    fn poll_release(&mut self, deadline: Instant) -> Result<bool, ApiError> {
        self.source.poll_release(self.deadline.min(deadline))
    }
    fn publish(&mut self) -> Result<(), ApiError> {
        if self.published {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Download candidate was already published",
            ));
        }
        self.source.check_origin(&PublicationControl {
            deadline: self.deadline,
        })?;
        if let Err(mut primary) = self.partial.publish_owned() {
            if let Err(cleanup) = self.partial.abort_owned() {
                primary.details = serde_json::json!({"primary_details":primary.details,"cleanup_failure":cleanup});
            }
            return Err(primary);
        }
        self.published = true;
        Ok(())
    }
    fn verify(&mut self) -> Result<VerifiedJobOutput, ApiError> {
        if !self.published {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Core has not published this download",
            ));
        }
        self.partial.verify_published(
            &self.receipt,
            &PublicationControl {
                deadline: self.deadline,
            },
        )
    }
}
fn closed(observation: OwnerObservation) -> bool {
    matches!(
        observation,
        OwnerObservation::NotStarted
            | OwnerObservation::Exited {
                family_empty: true,
                streams_eof: true,
                ..
            }
    )
}
fn internal(message: &str) -> ApiError {
    ApiError::new(ErrorCode::Internal, message)
}
fn copy_error(error: &ApiError) -> ApiError {
    ApiError {
        code: error.code,
        message: error.message.clone(),
        retryable: error.retryable,
        details: error.details.clone(),
        remediation: error.remediation.clone(),
        request_id: error.request_id.clone(),
    }
}
#[cfg(test)]
#[path = "owned_session/tests.rs"]
mod owner_observation_tests;
