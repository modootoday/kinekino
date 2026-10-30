//! Private integration candidate. No default issuer or product registration.
#[path = "process_source/ports.rs"]
mod ports;
use crate::application::{
    api::{ApiError, ErrorCode},
    download::{
        DownloadSession, MediaIdentity, OwnedTransferControl, OwnedTransferError, SourceCustody,
        SourceOpener, TransferEvidence,
    },
    jobs::ExecutionControl,
};
use kinekino_process::domain::deadline::Deadline;
use kinekino_source_host::{
    parent_host::ParentReceiptSink,
    parent_launch::{LaunchCloseFacts, ParentLaunch},
    parent_stream::ReadStatus,
    parent_transfer::{ParentTransfer, TransferError},
};
pub use ports::SourceReceiptSink;
use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

/// Implemented only by the adapter retaining the original browser request/attempt.
/// Implementations must check the exact original execution identity and currentness.
/// There is deliberately no implementation that treats request bytes as authority.
pub trait ReplayAdmission: Send {
    /// Called only after the original opener is in the existing core slot.
    fn prepare(&mut self) -> Result<(), ApiError> {
        Ok(())
    }
    fn poll_release(&mut self, _deadline: Instant) -> Result<bool, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Original replay owner release is unavailable",
        ))
    }
    fn check_request(
        &self,
        _request: &kinekino_source_host::request::HttpBootstrap,
    ) -> Result<(), ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Original replay request binding is unavailable",
        ))
    }
    fn check(&self, control: &dyn ExecutionControl) -> Result<(), ApiError>;
    fn consume(&mut self, control: &dyn ExecutionControl) -> Result<(), ApiError>;
    /// Checks retained original source/attempt provenance after transport close.
    /// Publication controls need not expose an active worker execution identity.
    fn check_origin(&self, control: &dyn ExecutionControl) -> Result<(), ApiError>;
}
struct State<G: Send, A: ReplayAdmission, S: SourceReceiptSink> {
    transfer: Option<ParentTransfer<G>>,
    pending_launch: Option<ParentLaunch<G>>,
    admission: A,
    sink: S,
    deadline: Deadline,
    spent: bool,
    closing: bool,
    facts_attempted: bool,
    closed: bool,
    first: Option<ApiError>,
    cleanup: Option<ApiError>,
}
type Shared<G, A, S> = Arc<Mutex<State<G, A, S>>>;
#[cfg(test)]
mod tests;
pub struct ProcessSourceOpener<G: Send, A: ReplayAdmission, S: SourceReceiptSink> {
    shared: Shared<G, A, S>,
    identity: MediaIdentity,
}
struct ProcessSession<G: Send, A: ReplayAdmission, S: SourceReceiptSink> {
    shared: Shared<G, A, S>,
    evidence: TransferEvidence,
    body_ended: bool,
}
fn failed(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, message)
}
pub(super) fn copy(error: &ApiError) -> ApiError {
    ApiError {
        code: error.code,
        message: error.message.clone(),
        retryable: error.retryable,
        details: error.details.clone(),
        remediation: error.remediation.clone(),
        request_id: error.request_id.clone(),
    }
}
pub(super) fn control_check(control: &dyn ExecutionControl) -> Result<(), ApiError> {
    if control.cancellation_requested() {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Source operation cancelled",
        ));
    }
    if Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Source operation deadline expired",
        ));
    }
    Ok(())
}
impl<G: Send, A: ReplayAdmission, S: SourceReceiptSink> State<G, A, S> {
    fn check(&self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        if self.closing {
            return Err(failed("Original source is closing"));
        }
        if let Some(error) = &self.first {
            return Err(copy(error));
        }
        control_check(control)?;
        self.admission.check(control)
    }
    fn remember<T>(&mut self, result: Result<T, ApiError>) -> Result<T, ApiError> {
        if let Err(error) = &result
            && self.first.is_none()
        {
            self.first = Some(copy(error));
        }
        result
    }
    fn with_current<T>(
        &mut self,
        control: &dyn ExecutionControl,
        operation: impl FnOnce(
            &mut ParentTransfer<G>,
            &mut dyn FnMut() -> Result<(), TransferError>,
        ) -> Result<T, TransferError>,
    ) -> Result<T, ApiError> {
        self.check(control)?;
        let admission = &self.admission;
        let mut original = None;
        let transfer = self
            .transfer
            .as_mut()
            .ok_or_else(|| failed("Original source transfer is unprepared"))?;
        let result = operation(transfer, &mut || {
            let result = control_check(control).and_then(|()| admission.check(control));
            if let Err(error) = result {
                original = Some(error);
                Err(TransferError::Current)
            } else {
                Ok(())
            }
        })
        .map_err(|_| original.unwrap_or_else(|| failed("Original source IO failed")));
        self.remember(result)
    }
    fn poll_io(
        &mut self,
        control: &dyn ExecutionControl,
    ) -> Result<kinekino_source_host::parent_transfer::TransferProgress, ApiError> {
        self.with_current(control, |transfer, check| transfer.poll_io(check))
    }
    fn close(&mut self, requested: Option<Instant>) -> Result<bool, ApiError> {
        self.closing = true;
        // Never extend either original cutoff. Requested is the core's close cutoff.
        if let Some(requested) = requested {
            self.deadline = self.deadline.constrained_end(requested);
        }
        if self.closed {
            return Ok(true);
        }
        let facts = if let Some(transfer) = &mut self.transfer {
            transfer.poll_close(self.deadline, &mut self.sink)
        } else if let Some(launch) = &mut self.pending_launch {
            launch.poll_close(self.deadline, &mut self.sink)
        } else {
            return Err(failed("Original source preparation owner is absent"));
        };
        if !facts.ready_to_ack() {
            return Ok(false);
        }
        if self.facts_attempted {
            return Err(self
                .cleanup
                .as_ref()
                .map(copy)
                .unwrap_or_else(|| failed("Original transport receipt remains unconfirmed")));
        }
        self.facts_attempted = true;
        let recorded = if let Some(transfer) = &self.transfer {
            self.sink.record_transport(&facts, transfer)
        } else {
            self.sink.record_prepared_transport(
                &facts,
                self.pending_launch.as_ref().expect("retained preparation"),
            )
        };
        if let Err(error) = recorded {
            self.cleanup = Some(copy(&error));
            return Err(error);
        }
        self.closed = true;
        Ok(true)
    }
}
impl<G: Send, A: ReplayAdmission, S: SourceReceiptSink> ProcessSourceOpener<G, A, S> {
    /// Infallible launch-free custody, owned by the existing SDK before open.
    pub fn new(
        transfer: ParentTransfer<G>,
        admission: A,
        sink: S,
        deadline: Deadline,
        identity: MediaIdentity,
    ) -> Self {
        Self {
            shared: Arc::new(Mutex::new(State {
                transfer: Some(transfer),
                pending_launch: None,
                admission,
                sink,
                deadline,
                spent: false,
                closing: false,
                facts_attempted: false,
                closed: false,
                first: None,
                cleanup: None,
            })),
            identity,
        }
    }
}
fn lock<G: Send, A: ReplayAdmission, S: SourceReceiptSink>(
    shared: &Shared<G, A, S>,
) -> Result<MutexGuard<'_, State<G, A, S>>, ApiError> {
    shared
        .lock()
        .map_err(|_| failed("Original source owner was interrupted"))
}
impl<G: Send + 'static, A: ReplayAdmission + 'static, S: SourceReceiptSink + 'static>
    ProcessSourceOpener<G, A, S>
{
    /// Original core slot owns admission and unstarted IPC before request setup.
    /// The callback cannot move either original owner out of the retained state.
    pub fn adopt_in_session(
        custody: SourceCustody<'_>,
        launch: ParentLaunch<G>,
        admission: A,
        sink: S,
        deadline: Deadline,
        identity: MediaIdentity,
        request: impl FnOnce(&A) -> Result<kinekino_source_host::request::HttpBootstrap, ApiError>,
    ) -> Result<(), ApiError> {
        let source = Self {
            shared: Arc::new(Mutex::new(State {
                transfer: None,
                pending_launch: Some(launch),
                admission,
                sink,
                deadline,
                spent: false,
                closing: false,
                facts_attempted: false,
                closed: false,
                first: None,
                cleanup: None,
            })),
            identity,
        };
        let original = Arc::clone(&source.shared);
        custody.adopt(Box::new(source));
        let mut state = lock(&original)?;
        let admitted = state.admission.prepare();
        state.remember(admitted)?;
        let prepared = request(&state.admission)
            .and_then(|request| state.admission.check_request(&request).map(|()| request));
        let prepared = state.remember(prepared)?;
        let launch = state
            .pending_launch
            .take()
            .expect("original launch retained");
        state.transfer = Some(ParentTransfer::new(launch, prepared));
        Ok(())
    }
}
impl<G: Send + 'static, A: ReplayAdmission + 'static, S: SourceReceiptSink + 'static> SourceOpener
    for ProcessSourceOpener<G, A, S>
{
    fn poll_release(&mut self, deadline: Instant) -> Result<bool, ApiError> {
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        // Cleanup already began; a transport receipt failure must not prevent
        // cancelling/releasing the original failed attempt's browser origin.
        // This does not acknowledge transport closure or discard its error.
        if !state.closing {
            return Ok(false);
        }
        state.admission.poll_release(deadline)
    }
    fn owner_observation(&self) -> crate::application::jobs::owned_attempt::OwnerObservation {
        use crate::application::jobs::owned_attempt::OwnerObservation;
        let state = self
            .shared
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if !state.closed {
            return OwnerObservation::Pending;
        }
        let Some(host) = state.transfer.as_ref().and_then(ParentTransfer::host) else {
            return OwnerObservation::NotStarted;
        };
        let Some(receipt) = host.receipt() else {
            return OwnerObservation::Pending;
        };
        // Preserve the exact Windows DWORD bits in the core's signed status.
        // Raw u32 remains in the original receipt; only zero denotes success.
        let Some(exit_code) = receipt
            .child_exit
            .map(|exit| i32::from_ne_bytes(exit.to_ne_bytes()))
        else {
            return OwnerObservation::Pending;
        };
        OwnerObservation::Exited {
            exit_code,
            family_empty: receipt.family
                == kinekino_process::domain::observation::FamilyObservation::Empty,
            streams_eof: true,
            forced: receipt.forced,
        }
    }
    fn open(
        &mut self,
        control: &dyn ExecutionControl,
    ) -> Result<Box<dyn DownloadSession + Send>, ApiError> {
        let mut state = lock(&self.shared)?;
        if state.spent {
            return Err(failed("Original source open was already attempted"));
        }
        state.spent = true;
        let result = (|| {
            state.check(control)?;
            state.admission.check_request(
                state
                    .transfer
                    .as_ref()
                    .ok_or_else(|| failed("Original transfer is unprepared"))?
                    .request(),
            )?;
            loop {
                state.check(control)?;
                if state
                    .transfer
                    .as_mut()
                    .expect("checked transfer")
                    .poll_prepared()
                    .map_err(|_| failed("Original source connection failed"))?
                {
                    break;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            let deadline = state.deadline;
            state.with_current(control, |transfer, check| transfer.start(deadline, check))?;
            state.check(control)?;
            loop {
                if state.poll_io(control)?.bootstrap_sent {
                    break;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            // Consumed once, immediately before queuing Open, never from URL bytes.
            state.admission.consume(control)?;
            state.check(control)?;
            state.with_current(control, |transfer, check| transfer.queue_open(check))?;
            loop {
                if state.poll_io(control)?.metadata_ready {
                    break;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            state.check(control)?;
            let length = state
                .transfer
                .as_ref()
                .expect("opened transfer")
                .metadata()
                .ok_or_else(|| failed("Source metadata missing"))?
                .response_length();
            Ok(length)
        })();
        let length = state.remember(result)?;
        Ok(Box::new(ProcessSession {
            shared: Arc::clone(&self.shared),
            body_ended: false,
            evidence: TransferEvidence {
                identity: self.identity.clone(),
                response_length: length,
                trusted_sha256: None,
            },
        }))
    }
    fn poll_close(&mut self, deadline: Instant) -> Result<bool, ApiError> {
        // A panic poisons normal use, but cannot discard the original cleanup owner.
        self.shared
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .close(Some(deadline))
    }
}
impl<G: Send, A: ReplayAdmission, S: SourceReceiptSink> DownloadSession
    for ProcessSession<G, A, S>
{
    fn poll_release(&mut self, deadline: Instant) -> Result<bool, ApiError> {
        let mut state = lock(&self.shared)?;
        if !state.closed {
            return Ok(false);
        }
        state.admission.poll_release(deadline)
    }
    fn evidence(&self) -> &TransferEvidence {
        &self.evidence
    }
    fn check_current(&self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        lock(&self.shared)?.check(control)
    }
    fn check_origin(&self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        let state = lock(&self.shared)?;
        control_check(control)?;
        if let Some(error) = state.first.as_ref().or(state.cleanup.as_ref()) {
            return Err(copy(error));
        }
        if !self.body_ended || !state.closed {
            return Err(failed("Source body and physical close are not confirmed"));
        }
        state.admission.check_origin(control)
    }
    fn close_owned(&mut self) -> Result<(), ApiError> {
        loop {
            let mut state = self
                .shared
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            if state.close(None)? {
                return Ok(());
            }
            if state.deadline.remaining(Instant::now()).is_zero() {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "Original source close remains pending",
                ));
            }
            drop(state);
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    fn read(
        &mut self,
        buffer: &mut [u8],
        control: &dyn ExecutionControl,
    ) -> Result<usize, ApiError> {
        self.receive(buffer, control, || control_check(control))
    }
    fn read_owned(
        &mut self,
        buffer: &mut [u8],
        control: &dyn OwnedTransferControl,
    ) -> Result<usize, OwnedTransferError> {
        self.receive(buffer, control, || control.checkpoint())
    }
}
impl<G: Send, A: ReplayAdmission, S: SourceReceiptSink> ProcessSession<G, A, S> {
    fn receive<E: From<ApiError>>(
        &mut self,
        buffer: &mut [u8],
        control: &dyn ExecutionControl,
        mut checkpoint: impl FnMut() -> Result<(), E>,
    ) -> Result<usize, E> {
        if buffer.is_empty() {
            return Err(
                ApiError::new(ErrorCode::InvalidArgument, "Source read buffer is empty").into(),
            );
        }
        loop {
            checkpoint()?;
            let mut state = lock(&self.shared)?;
            state.check(control)?;
            state.poll_io(control)?;
            let read =
                state.with_current(control, |transfer, check| transfer.read(buffer, check))?;
            state.check(control)?;
            match read {
                ReadStatus::Data(count) => return Ok(count),
                ReadStatus::End => {
                    self.body_ended = true;
                    return Ok(0);
                }
                ReadStatus::Pending => {}
            }
            let transfer = state
                .transfer
                .as_ref()
                .ok_or_else(|| failed("Original transfer is absent"))?;
            let credit = transfer.available_credit();
            if credit != 0 && !transfer.control_pending() {
                state.with_current(control, |transfer, check| {
                    transfer.queue_credit(credit, check)
                })?;
            }
            drop(state);
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
