use super::{
    AdmittedTransfer, DownloadOutput, DownloadReceipt, DownloadSession, DownloadVerifier,
    OwnedTransferControl, OwnedTransferError, PartialOutput,
};
#[cfg(test)]
use super::{DownloadPoll, OwnedDownload, OwnedVerification, PendingDownload, VerificationPoll};
use crate::application::{
    api::{ApiError, ErrorCode},
    jobs::ExecutionControl,
};
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::sync::Arc;

fn check(control: &dyn ExecutionControl) -> Result<(), ApiError> {
    if control.cancellation_requested() {
        return Err(ApiError::new(ErrorCode::Cancelled, "Download cancelled"));
    }
    if std::time::Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Download deadline expired",
        ));
    }
    Ok(())
}
pub(super) fn admit(
    session: &dyn DownloadSession,
    control: &dyn ExecutionControl,
    maximum: u64,
) -> Result<AdmittedTransfer, ApiError> {
    if maximum == 0 {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Download byte limit must be positive",
        ));
    }
    check(control)?;
    session.check_current(control)?;
    let evidence = session.evidence();
    if evidence
        .response_length
        .is_some_and(|length| length > maximum)
    {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Download exceeds byte limit",
        ));
    }
    Ok(AdmittedTransfer {
        identity: evidence.identity.clone(),
        length: evidence.response_length,
        digest: evidence.trusted_sha256,
    })
}
pub(super) fn receive(
    session: &mut dyn DownloadSession,
    partial: &mut dyn PartialOutput,
    control: &dyn ExecutionControl,
    admitted: &AdmittedTransfer,
    maximum: u64,
) -> Result<DownloadReceipt, ApiError> {
    receive_with(
        session,
        partial,
        admitted,
        maximum,
        || check(control),
        |session, buffer| {
            session.check_current(control)?;
            session.read(buffer, control)
        },
    )
}
#[allow(
    dead_code,
    reason = "Owned executor is path-mounted until production host admission"
)]
pub(super) fn receive_owned(
    session: &mut dyn DownloadSession,
    partial: &mut dyn PartialOutput,
    control: &dyn OwnedTransferControl,
    admitted: &AdmittedTransfer,
    maximum: u64,
) -> Result<DownloadReceipt, OwnedTransferError> {
    receive_with(
        session,
        partial,
        admitted,
        maximum,
        || control.checkpoint(),
        |session, buffer| {
            session.check_current(control)?;
            session.read_owned(buffer, control)
        },
    )
}
fn receive_with<E: From<ApiError>>(
    session: &mut dyn DownloadSession,
    partial: &mut dyn PartialOutput,
    admitted: &AdmittedTransfer,
    maximum: u64,
    checkpoint: impl Fn() -> Result<(), E>,
    mut read: impl FnMut(&mut dyn DownloadSession, &mut [u8]) -> Result<usize, E>,
) -> Result<DownloadReceipt, E> {
    let mut buffer = [0_u8; 64 * 1024];
    let mut digest = Sha256::new();
    let mut bytes = 0_u64;
    loop {
        checkpoint()?;
        let count = read(session, &mut buffer)?;
        checkpoint()?;
        if count > buffer.len() {
            return Err(ApiError::new(
                ErrorCode::Internal,
                "Transport returned an invalid byte count",
            )
            .into());
        }
        if count == 0 {
            break;
        }
        bytes = bytes.checked_add(count as u64).ok_or_else(|| {
            ApiError::new(ErrorCode::ResourceExhausted, "Download byte count overflow")
        })?;
        if bytes > maximum || admitted.length.is_some_and(|length| bytes > length) {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Download exceeded admitted length",
            )
            .into());
        }
        partial.append(&buffer[..count])?;
        digest.update(&buffer[..count]);
    }
    let sha256: [u8; 32] = digest.finalize().into();
    if bytes == 0
        || admitted.length.is_some_and(|length| bytes != length)
        || admitted.digest.is_some_and(|expected| sha256 != expected)
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Downloaded bytes differ from admitted evidence",
        )
        .into());
    }
    partial.seal()?;
    Ok(DownloadReceipt {
        identity: admitted.identity.clone(),
        bytes,
        sha256,
    })
}
pub(super) fn final_source(
    session: &dyn DownloadSession,
    control: &dyn ExecutionControl,
    admitted: &AdmittedTransfer,
) -> Result<(), ApiError> {
    check(control)?;
    session.check_current(control)?;
    let evidence = session.evidence();
    if evidence.identity != admitted.identity
        || evidence.response_length != admitted.length
        || evidence.trusted_sha256 != admitted.digest
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Download session evidence changed",
        ));
    }
    Ok(())
}
fn publication(
    session: &dyn DownloadSession,
    control: &dyn ExecutionControl,
    admitted: &AdmittedTransfer,
) -> Result<(), ApiError> {
    final_source(session, control, admitted)?;
    control.reserve_publication()?;
    control.begin_verification()
}
fn abort(partial: Box<dyn PartialOutput>, mut primary: ApiError) -> ApiError {
    if let Err(cleanup) = partial.abort() {
        primary.details =
            serde_json::json!({"primary_details":primary.details,"cleanup_failure":cleanup});
    }
    primary
}
/// Accepted synchronous path. It does not advertise owned external probe support.
pub fn transfer(
    session: &mut dyn DownloadSession,
    output: &dyn DownloadOutput,
    verifier: &dyn DownloadVerifier,
    control: &dyn ExecutionControl,
    maximum_bytes: u64,
) -> Result<DownloadReceipt, ApiError> {
    let admitted = admit(session, control, maximum_bytes)?;
    let mut partial = output.create()?;
    let result = (|| {
        let receipt = receive(session, partial.as_mut(), control, &admitted, maximum_bytes)?;
        check(control)?;
        verifier.verify(partial.path(), &admitted.identity, control)?;
        publication(session, control, &admitted)?;
        Ok(receipt)
    })();
    match result {
        Ok(receipt) => {
            partial.publish()?;
            Ok(receipt)
        }
        Err(primary) => Err(abort(partial, primary)),
    }
}

/// Owned verifier path. No retry, new deadline, publication or cleanup ACK is
/// inferred from Pending. The job coordinator must retain and poll this carrier.
#[cfg(test)]
pub(super) fn transfer_owned(
    mut session: Box<dyn DownloadSession + Send>,
    output: &dyn DownloadOutput,
    verifier: &dyn DownloadVerifier,
    control: Arc<dyn ExecutionControl>,
    maximum_bytes: u64,
) -> Result<OwnedDownload, ApiError> {
    let admitted = admit(session.as_ref(), control.as_ref(), maximum_bytes)?;
    let mut partial = output.create_owned()?;
    let prepared = (|| {
        let receipt = receive(
            session.as_mut(),
            partial.as_mut(),
            control.as_ref(),
            &admitted,
            maximum_bytes,
        )?;
        final_source(session.as_ref(), control.as_ref(), &admitted)?;
        let lease = partial.verification_lease()?;
        lease.check_current()?;
        let verification =
            verifier.begin_owned(lease, admitted.identity.clone(), control.as_ref())?;
        Ok((receipt, verification))
    })();
    let (receipt, verification) = match prepared {
        Ok(value) => value,
        Err(primary) => return Err(abort(partial, primary)),
    };
    match verification {
        OwnedVerification::Complete(result) => {
            if let Err(primary) =
                result.and_then(|()| publication(session.as_ref(), control.as_ref(), &admitted))
            {
                return Err(abort(partial, primary));
            }
            partial.publish()?;
            Ok(OwnedDownload::Complete(receipt))
        }
        OwnedVerification::Pending(probe) => {
            Ok(OwnedDownload::Pending(Box::new(PendingDownload {
                session,
                partial: Some(partial),
                probe: Some(probe),
                control,
                admitted,
                receipt: Some(receipt),
                primary: None,
                cleanup_error: None,
                cancellation_sent: false,
                finished: false,
            })))
        }
    }
}
#[cfg(test)]
impl PendingDownload {
    pub fn first_error(&self) -> Option<&ApiError> {
        self.primary
            .as_ref()
            .or_else(|| self.probe.as_ref().and_then(|probe| probe.first_error()))
    }
    pub fn poll(&mut self) -> DownloadPoll {
        if self.finished {
            return DownloadPoll::Finished(Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Owned transfer already settled",
            )));
        }
        let probe = self
            .probe
            .as_mut()
            .expect("unsettled transfer retains probe");
        if self.primary.is_none() {
            self.primary = probe.first_error().map(copy_error);
        }
        if self.primary.is_none()
            && let Err(error) =
                final_source(self.session.as_ref(), self.control.as_ref(), &self.admitted)
        {
            self.primary = Some(error);
        }
        if self.primary.is_some() && !self.cancellation_sent {
            self.cancellation_sent = true;
            if let Err(error) = probe.request_cancel() {
                self.cleanup_error = Some(error);
            }
        }
        let observation = probe.poll(self.control.as_ref());
        if self.primary.is_none() {
            self.primary = probe.first_error().map(copy_error);
        }
        let result = match observation {
            VerificationPoll::Pending => {
                return DownloadPoll::Pending {
                    cleanup_pending: self.primary.is_some(),
                };
            }
            VerificationPoll::Closed(result) => result,
        };
        // Only Closed permits releasing original probe/lease and touching output.
        self.probe.take();
        if let Err(error) = result {
            if self.primary.is_none() {
                self.primary = Some(error);
            } else if self.cleanup_error.is_none() {
                self.cleanup_error = Some(error);
            } else if let Some(primary) = self.primary.as_mut() {
                primary.details = serde_json::json!({"primary_details":primary.details,"probe_terminal_failure":error});
            }
        }
        let partial = self
            .partial
            .take()
            .expect("unsettled transfer retains partial");
        self.finished = true;
        if let Some(mut primary) = self.primary.take() {
            if let Some(cleanup) = self.cleanup_error.take() {
                primary.details = serde_json::json!({"primary_details":primary.details,"probe_cleanup_failure":cleanup});
            }
            return DownloadPoll::Finished(Err(abort(partial, primary)));
        }
        if let Err(primary) =
            publication(self.session.as_ref(), self.control.as_ref(), &self.admitted)
        {
            return DownloadPoll::Finished(Err(abort(partial, primary)));
        }
        match partial.publish() {
            Ok(()) => DownloadPoll::Finished(Ok(self
                .receipt
                .take()
                .expect("receipt retained until settlement"))),
            Err(error) => DownloadPoll::Finished(Err(error)),
        }
    }
}
#[cfg(test)]
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
mod owned_tests {
    use super::*;
    use crate::application::download::{
        MediaIdentity, PendingMediaVerification, TransferEvidence, VerificationLease,
    };
    use std::{
        path::Path,
        sync::{
            Mutex,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        time::{Duration, Instant},
    };

    #[derive(Default)]
    struct State {
        leases: usize,
        sealed: bool,
        published: bool,
        aborted: bool,
        dropped_with_lease: bool,
    }
    struct Lease(Arc<Mutex<State>>);
    impl VerificationLease for Lease {
        fn path(&self) -> &Path {
            Path::new("owned-fixture.part")
        }
        fn check_current(&self) -> Result<(), ApiError> {
            assert!(self.0.lock().unwrap().sealed);
            Ok(())
        }
    }
    impl Drop for Lease {
        fn drop(&mut self) {
            self.0.lock().unwrap().leases -= 1;
        }
    }
    struct Partial(Arc<Mutex<State>>);
    impl PartialOutput for Partial {
        fn verification_lease(&self) -> Result<Arc<dyn VerificationLease>, ApiError> {
            let mut state = self.0.lock().unwrap();
            assert!(state.sealed);
            state.leases += 1;
            Ok(Arc::new(Lease(self.0.clone())))
        }
        fn append(&mut self, _: &[u8]) -> Result<(), ApiError> {
            assert!(!self.0.lock().unwrap().sealed);
            Ok(())
        }
        fn seal(&mut self) -> Result<(), ApiError> {
            self.0.lock().unwrap().sealed = true;
            Ok(())
        }
        fn path(&self) -> &Path {
            Path::new("owned-fixture.part")
        }
        fn abort(self: Box<Self>) -> Result<(), ApiError> {
            let mut state = self.0.lock().unwrap();
            assert_eq!(state.leases, 0);
            state.aborted = true;
            Ok(())
        }
        fn publish(self: Box<Self>) -> Result<(), ApiError> {
            let mut state = self.0.lock().unwrap();
            assert_eq!(state.leases, 0);
            state.published = true;
            Ok(())
        }
    }
    impl Drop for Partial {
        fn drop(&mut self) {
            let mut state = self.0.lock().unwrap();
            state.dropped_with_lease = state.leases != 0;
        }
    }
    struct Output(Arc<Mutex<State>>);
    impl DownloadOutput for Output {
        fn create(&self) -> Result<Box<dyn PartialOutput>, ApiError> {
            Ok(Box::new(Partial(self.0.clone())))
        }
        fn create_owned(&self) -> Result<Box<dyn PartialOutput + Send>, ApiError> {
            Ok(Box::new(Partial(self.0.clone())))
        }
    }
    struct Source {
        evidence: TransferEvidence,
        done: bool,
        alive: Arc<AtomicBool>,
    }
    impl DownloadSession for Source {
        fn evidence(&self) -> &TransferEvidence {
            &self.evidence
        }
        fn check_current(&self, _: &dyn ExecutionControl) -> Result<(), ApiError> {
            if self.alive.load(Ordering::Acquire) {
                Ok(())
            } else {
                Err(ApiError::new(
                    ErrorCode::SourceExpired,
                    "original source expired",
                ))
            }
        }
        fn read(&mut self, buffer: &mut [u8], _: &dyn ExecutionControl) -> Result<usize, ApiError> {
            if self.done {
                Ok(0)
            } else {
                self.done = true;
                buffer[0] = 7;
                Ok(1)
            }
        }
    }
    struct Control {
        cancel: AtomicBool,
        reserves: AtomicUsize,
        deadline: Instant,
    }
    impl ExecutionControl for Control {
        fn cancellation_requested(&self) -> bool {
            self.cancel.load(Ordering::Acquire)
        }
        fn deadline(&self) -> Instant {
            self.deadline
        }
        fn reserve_publication(&self) -> Result<(), ApiError> {
            assert!(!self.cancellation_requested());
            self.reserves.fetch_add(1, Ordering::AcqRel);
            Ok(())
        }
        fn begin_verification(&self) -> Result<(), ApiError> {
            assert_eq!(self.reserves.load(Ordering::Acquire), 1);
            Ok(())
        }
    }
    #[derive(Default)]
    struct ProbeState {
        closed: AtomicBool,
        cancels: AtomicUsize,
        cancel_error: AtomicBool,
        terminal_error: AtomicBool,
    }
    struct Probe {
        lease: Option<Arc<dyn VerificationLease>>,
        state: Arc<ProbeState>,
    }
    impl PendingMediaVerification for Probe {
        fn request_cancel(&mut self) -> Result<(), ApiError> {
            self.state.cancels.fetch_add(1, Ordering::AcqRel);
            if self.state.cancel_error.load(Ordering::Acquire) {
                Err(ApiError::new(
                    ErrorCode::PermissionDenied,
                    "probe cancel IO",
                ))
            } else {
                Ok(())
            }
        }
        fn first_error(&self) -> Option<&ApiError> {
            None
        }
        fn poll(&mut self, _: &dyn ExecutionControl) -> VerificationPoll {
            if !self.state.closed.load(Ordering::Acquire) {
                return VerificationPoll::Pending;
            }
            self.lease.take();
            VerificationPoll::Closed(if self.state.terminal_error.load(Ordering::Acquire) {
                Err(ApiError::new(ErrorCode::EngineFailed, "probe close error"))
            } else {
                Ok(())
            })
        }
    }
    struct Verifier(Arc<ProbeState>);
    impl DownloadVerifier for Verifier {
        fn verify(
            &self,
            _: &Path,
            _: &MediaIdentity,
            _: &dyn ExecutionControl,
        ) -> Result<(), ApiError> {
            panic!("owned caller cannot use Path-only fallback")
        }
        fn begin_owned(
            &self,
            lease: Arc<dyn VerificationLease>,
            _: MediaIdentity,
            _: &dyn ExecutionControl,
        ) -> Result<OwnedVerification, ApiError> {
            Ok(OwnedVerification::Pending(Box::new(Probe {
                lease: Some(lease),
                state: self.0.clone(),
            })))
        }
    }
    struct Fixture {
        output: Arc<Mutex<State>>,
        probe: Arc<ProbeState>,
        control: Arc<Control>,
        alive: Arc<AtomicBool>,
    }
    impl Fixture {
        fn new() -> Self {
            Self {
                output: Arc::new(Mutex::new(State::default())),
                probe: Arc::new(ProbeState::default()),
                control: Arc::new(Control {
                    cancel: AtomicBool::new(false),
                    reserves: AtomicUsize::new(0),
                    deadline: Instant::now() + Duration::from_secs(30),
                }),
                alive: Arc::new(AtomicBool::new(true)),
            }
        }
        fn start(&self) -> Box<PendingDownload> {
            let source = Source {
                evidence: TransferEvidence {
                    identity: MediaIdentity {
                        media: "fixture".into(),
                        tracks: "video+audio".into(),
                    },
                    response_length: Some(1),
                    trusted_sha256: None,
                },
                done: false,
                alive: self.alive.clone(),
            };
            match transfer_owned(
                Box::new(source),
                &Output(self.output.clone()),
                &Verifier(self.probe.clone()),
                self.control.clone(),
                8,
            )
            .unwrap()
            {
                OwnedDownload::Pending(pending) => pending,
                OwnedDownload::Complete(_) => panic!("fixture must retain original pending probe"),
            }
        }
    }
    #[test]
    fn pending_probe_retains_partial_and_prevents_early_publication() {
        let fixture = Fixture::new();
        let mut pending = fixture.start();
        assert!(matches!(
            pending.poll(),
            DownloadPoll::Pending {
                cleanup_pending: false
            }
        ));
        assert_eq!(fixture.output.lock().unwrap().leases, 1);
        assert_eq!(fixture.control.reserves.load(Ordering::Acquire), 0);
        fixture.probe.closed.store(true, Ordering::Release);
        assert!(matches!(pending.poll(), DownloadPoll::Finished(Ok(_))));
        let state = fixture.output.lock().unwrap();
        assert!(state.published && !state.aborted && !state.dropped_with_lease);
    }
    #[test]
    fn cancellation_is_pending_until_original_probe_close_then_aborts() {
        let fixture = Fixture::new();
        let mut pending = fixture.start();
        fixture.control.cancel.store(true, Ordering::Release);
        for _ in 0..2 {
            assert!(matches!(
                pending.poll(),
                DownloadPoll::Pending {
                    cleanup_pending: true
                }
            ));
        }
        assert_eq!(fixture.probe.cancels.load(Ordering::Acquire), 1);
        assert!(!fixture.output.lock().unwrap().aborted);
        fixture.probe.closed.store(true, Ordering::Release);
        match pending.poll() {
            DownloadPoll::Finished(Err(error)) => assert_eq!(error.code, ErrorCode::Cancelled),
            _ => panic!("expected original cancellation"),
        }
        let state = fixture.output.lock().unwrap();
        assert!(state.aborted && !state.published && !state.dropped_with_lease);
    }
    #[test]
    fn primary_cancel_and_both_cleanup_failures_are_preserved() {
        let fixture = Fixture::new();
        let mut pending = fixture.start();
        fixture.control.cancel.store(true, Ordering::Release);
        fixture.probe.cancel_error.store(true, Ordering::Release);
        assert!(matches!(
            pending.poll(),
            DownloadPoll::Pending {
                cleanup_pending: true
            }
        ));
        fixture.probe.terminal_error.store(true, Ordering::Release);
        fixture.probe.closed.store(true, Ordering::Release);
        match pending.poll() {
            DownloadPoll::Finished(Err(error)) => {
                assert_eq!(error.code, ErrorCode::Cancelled);
                assert_eq!(
                    error.details["probe_cleanup_failure"]["code"],
                    "permission_denied"
                );
                assert_eq!(
                    error.details["primary_details"]["probe_terminal_failure"]["code"],
                    "engine_failed"
                );
            }
            _ => panic!("expected retained errors"),
        }
    }
    #[test]
    fn source_expiry_remains_first_error_when_cancel_arrives_later() {
        let fixture = Fixture::new();
        let mut pending = fixture.start();
        fixture.alive.store(false, Ordering::Release);
        assert!(matches!(
            pending.poll(),
            DownloadPoll::Pending {
                cleanup_pending: true
            }
        ));
        fixture.control.cancel.store(true, Ordering::Release);
        fixture.probe.closed.store(true, Ordering::Release);
        match pending.poll() {
            DownloadPoll::Finished(Err(error)) => assert_eq!(error.code, ErrorCode::SourceExpired),
            _ => panic!("source expiry must remain first"),
        }
        assert_eq!(fixture.control.reserves.load(Ordering::Acquire), 0);
    }
}
