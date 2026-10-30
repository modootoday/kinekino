//! Synthetic original provider only; no native browser or process is started.
use super::*;
use crate::application::jobs::ExecutionControl;
use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};
struct Port {
    work: Mutex<Option<BrowserWork>>,
    cancel_calls: AtomicUsize,
    release_calls: AtomicUsize,
    queue_full_once: AtomicBool,
    missing: AtomicBool,
    wrong_ticket: AtomicBool,
}
impl Port {
    fn finish_cancel(&self) {
        let mut slot = self.work.lock().unwrap();
        let work = slot.as_mut().unwrap();
        let ticket = work.snapshot().ticket;
        work.apply(&ticket, BrowserEvent::Cancelled);
    }
    fn native_released(&self) {
        self.work
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .record_native_capture_released()
            .unwrap();
    }
}
fn absent() -> ApiError {
    ApiError::new(ErrorCode::NotFound, "Original fixture ticket absent")
}
impl BrowserProviderPort for Port {
    fn try_start(&self, request: ValidatedBrowserRequest) -> Result<BrowserTicket, ApiError> {
        let slot = self.work.lock().unwrap();
        let work = slot.as_ref().ok_or_else(absent)?;
        assert_eq!(
            request.request().request_id,
            work.request().request().request_id
        );
        Ok(work.snapshot().ticket)
    }
    fn poll(&self, _: &BrowserTicket) -> Result<BrowserSnapshot, ApiError> {
        if self.missing.load(Ordering::SeqCst) {
            return Err(absent());
        }
        let mut snapshot = self
            .work
            .lock()
            .unwrap()
            .as_ref()
            .ok_or_else(absent)?
            .snapshot();
        if self.wrong_ticket.load(Ordering::SeqCst) {
            snapshot.ticket = BrowserTicket::new("other".into(), "other".into(), 2).unwrap();
        }
        Ok(snapshot)
    }
    fn cancel(&self, _: &BrowserTicket) -> Result<CancelDisposition, ApiError> {
        self.cancel_calls.fetch_add(1, Ordering::SeqCst);
        if self.queue_full_once.swap(false, Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Browser command queue is full",
            ));
        }
        Ok(self
            .work
            .lock()
            .unwrap()
            .as_mut()
            .ok_or_else(absent)?
            .cancel())
    }
    fn navigate(&self, _: &BrowserTicket, _: u64, _: &str) -> Result<(), ApiError> {
        Err(absent())
    }
    fn release(&self, _: &BrowserTicket) -> Result<(), ApiError> {
        self.release_calls.fetch_add(1, Ordering::SeqCst);
        let mut slot = self.work.lock().unwrap();
        let original = slot.as_ref().ok_or_else(absent)?;
        if !original.snapshot().state.terminal() || !original.native_capture_released() {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Original browser capture release is pending",
            ));
        }
        slot.take();
        Ok(())
    }
}
struct Host;
impl BrowserProviderOwner for Host {
    fn request_shutdown(&self) -> Result<(), ApiError> {
        panic!("document owner must not stop shared host")
    }
    fn shutdown_snapshot(&self) -> Result<BrowserHostSnapshot, ApiError> {
        Ok(BrowserHostSnapshot {
            state: BrowserHostState::Ready,
            worker_exited: false,
            failure: None,
        })
    }
}
struct Control(Instant);
impl ExecutionControl for Control {
    fn deadline(&self) -> Instant {
        self.0
    }
    fn cancellation_requested(&self) -> bool {
        false
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        panic!("document cannot publish")
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        panic!("document cannot verify")
    }
}
fn fixture() -> (Arc<Port>, BrowserDocumentSourceOwner) {
    let request = ValidatedBrowserRequest::new(BrowserRequest {
        request_id: "source-owner".into(),
        operation: BrowserOperation::ReadCatalog,
        url: "https://catalog.example.test/방영중".into(),
        timeout_ms: DecimalU64(1000),
        blocking_enabled: true,
    })
    .unwrap();
    let ticket = BrowserTicket::new("source-owner".into(), "source-owner".into(), 1).unwrap();
    let mut work = BrowserWork::new(ticket.clone(), request.clone()).unwrap();
    work.apply(
        &ticket,
        BrowserEvent::Started {
            engine_version: "synthetic".into(),
            blocking_installed: true,
        },
    );
    let port = Arc::new(Port {
        work: Mutex::new(Some(work)),
        cancel_calls: AtomicUsize::new(0),
        release_calls: AtomicUsize::new(0),
        queue_full_once: AtomicBool::new(false),
        missing: AtomicBool::new(false),
        wrong_ticket: AtomicBool::new(false),
    });
    let session =
        BrowserDocumentSession::start(port.clone(), &Host, request, &Control(end())).unwrap();
    let owner = session.into_source_owner();
    assert_eq!(port.cancel_calls.load(Ordering::SeqCst), 0);
    assert_eq!(port.release_calls.load(Ordering::SeqCst), 0);
    (port, owner)
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(5)
}
#[test]
fn handoff_preserves_original_document_until_cancel_native_release_and_ticket_release() {
    let (port, mut owner) = fixture();
    let deadline = end();
    assert!(!owner.poll_release(deadline).unwrap());
    assert!(!owner.poll_release(deadline).unwrap());
    assert_eq!(port.cancel_calls.load(Ordering::SeqCst), 1);
    port.finish_cancel();
    assert!(!owner.poll_release(deadline).unwrap());
    assert!(!owner.released && port.work.lock().unwrap().is_some());
    port.native_released();
    assert!(owner.poll_release(deadline).unwrap());
    assert!(port.work.lock().unwrap().is_none());
    let calls = port.release_calls.load(Ordering::SeqCst);
    assert!(owner.poll_release(deadline).unwrap());
    assert_eq!(port.release_calls.load(Ordering::SeqCst), calls);
}
#[test]
fn queue_pressure_retries_only_unadmitted_cancel() {
    let (port, mut owner) = fixture();
    port.queue_full_once.store(true, Ordering::SeqCst);
    assert!(!owner.poll_release(end()).unwrap());
    assert!(!owner.cancel_admitted);
    assert!(!owner.poll_release(end()).unwrap());
    assert!(owner.cancel_admitted);
    assert!(!owner.poll_release(end()).unwrap());
    assert_eq!(port.cancel_calls.load(Ordering::SeqCst), 2);
}
#[test]
fn missing_ticket_is_an_error_not_release_observation() {
    let (port, mut owner) = fixture();
    port.missing.store(true, Ordering::SeqCst);
    assert_eq!(
        owner.poll_release(end()).unwrap_err().code,
        ErrorCode::NotFound
    );
    assert!(!owner.released);
    assert_eq!(port.cancel_calls.load(Ordering::SeqCst), 0);
}
#[test]
fn foreign_snapshot_is_not_used_to_cancel_or_release() {
    let (port, mut owner) = fixture();
    port.wrong_ticket.store(true, Ordering::SeqCst);
    assert_eq!(
        owner.poll_release(end()).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(port.cancel_calls.load(Ordering::SeqCst), 0);
    assert_eq!(port.release_calls.load(Ordering::SeqCst), 0);
}
#[test]
fn original_deadline_is_never_extended_but_actual_cleanup_can_still_be_observed() {
    let (port, mut owner) = fixture();
    let expired = Instant::now() - Duration::from_millis(1);
    assert_eq!(
        owner.poll_release(expired).unwrap_err().code,
        ErrorCode::Timeout
    );
    assert_eq!(
        owner.poll_release(end()).unwrap_err().code,
        ErrorCode::Timeout
    );
    assert_eq!(owner.deadline, Some(expired));
    assert_eq!(port.cancel_calls.load(Ordering::SeqCst), 1);
    port.finish_cancel();
    port.native_released();
    assert!(owner.poll_release(end()).unwrap());
    assert_eq!(owner.deadline, Some(expired));
}
#[cfg(feature = "source-adapter-test-support")]
#[test]
fn owner_requires_original_provider_arc_and_grant_ticket() {
    let (port, grant, owner) =
        fixture_grant_for_request("https://example.invalid/offline-sdk-body", end());
    owner.check_origin(&port, &grant).unwrap();
    let (foreign, _, _) =
        fixture_grant_for_request("https://example.invalid/offline-sdk-body", end());
    assert_eq!(
        owner.check_origin(&foreign, &grant).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let (_, different_ticket_owner) = fixture();
    assert_eq!(
        different_ticket_owner
            .check_origin(&different_ticket_owner.provider(), &grant)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
}
#[cfg(feature = "source-adapter-test-support")]
#[test]
fn expired_original_session_cannot_be_renewed_by_a_new_source_context() {
    let (port, grant, owner) = fixture_grant_for_request(
        "https://example.invalid/offline-sdk-body",
        Instant::now() - Duration::from_millis(1),
    );
    assert_eq!(
        owner.check_origin(&port, &grant).unwrap_err().code,
        ErrorCode::Timeout
    );
}
