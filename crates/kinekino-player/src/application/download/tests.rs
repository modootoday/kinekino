use super::*;
use crate::application::{
    api::{ApiError, ErrorCode},
    jobs::ExecutionControl,
};
use std::{
    cell::Cell,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

struct Control {
    cancelled: AtomicBool,
    reserved: AtomicBool,
    deadline: Instant,
}
impl Control {
    fn new() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            reserved: AtomicBool::new(false),
            deadline: Instant::now() + Duration::from_secs(30),
        }
    }
}
impl ExecutionControl for Control {
    fn cancellation_requested(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
    fn deadline(&self) -> Instant {
        self.deadline
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        if self.cancellation_requested() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "fixture CAS cancellation winner",
            ));
        }
        assert!(!self.reserved.swap(true, Ordering::SeqCst));
        Ok(())
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        assert!(
            self.reserved.load(Ordering::SeqCst),
            "jobs verification requires publication reservation"
        );
        Ok(())
    }
}
struct Session {
    evidence: TransferEvidence,
    data: Vec<u8>,
    offset: usize,
    checks: Cell<usize>,
    expires_at_check: Option<usize>,
}
impl Session {
    fn new(length: Option<u64>) -> Self {
        Self {
            evidence: TransferEvidence {
                identity: MediaIdentity {
                    media: "fixture-media".into(),
                    tracks: "video+audio".into(),
                },
                response_length: length,
                trusted_sha256: None,
            },
            data: vec![7; 5],
            offset: 0,
            checks: Cell::new(0),
            expires_at_check: None,
        }
    }
}
impl DownloadSession for Session {
    fn evidence(&self) -> &TransferEvidence {
        &self.evidence
    }
    fn check_current(&self, _: &dyn ExecutionControl) -> Result<(), ApiError> {
        let count = self.checks.get() + 1;
        self.checks.set(count);
        if self.expires_at_check == Some(count) {
            return Err(ApiError::new(ErrorCode::SourceExpired, "fixture expired"));
        }
        Ok(())
    }
    fn read(&mut self, buffer: &mut [u8], _: &dyn ExecutionControl) -> Result<usize, ApiError> {
        let count = (self.data.len() - self.offset).min(buffer.len()).min(2);
        buffer[..count].copy_from_slice(&self.data[self.offset..self.offset + count]);
        self.offset += count;
        Ok(count)
    }
}
#[derive(Default)]
struct State {
    active: bool,
    published: bool,
    bytes: Vec<u8>,
    final_bytes: Vec<u8>,
    cleanup_failure: bool,
}
#[derive(Default)]
struct Output(Arc<Mutex<State>>);
struct Partial {
    state: Arc<Mutex<State>>,
    path: PathBuf,
    sealed: bool,
}
impl DownloadOutput for Output {
    fn create(&self) -> Result<Box<dyn PartialOutput>, ApiError> {
        let mut state = self.0.lock().unwrap();
        assert!(!state.active);
        state.active = true;
        state.bytes.clear();
        Ok(Box::new(Partial {
            state: self.0.clone(),
            path: PathBuf::from("fixture.part"),
            sealed: false,
        }))
    }
}
impl PartialOutput for Partial {
    fn append(&mut self, bytes: &[u8]) -> Result<(), ApiError> {
        assert!(!self.sealed);
        self.state.lock().unwrap().bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn seal(&mut self) -> Result<(), ApiError> {
        self.sealed = true;
        Ok(())
    }
    fn path(&self) -> &Path {
        assert!(self.sealed);
        &self.path
    }
    fn abort(self: Box<Self>) -> Result<(), ApiError> {
        if self.state.lock().unwrap().cleanup_failure {
            return Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "fixture cleanup denied",
            ));
        }
        Ok(())
    }
    fn publish(self: Box<Self>) -> Result<(), ApiError> {
        let mut state = self.state.lock().unwrap();
        if state.published {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "fixture target exists",
            ));
        }
        state.final_bytes = state.bytes.clone();
        state.published = true;
        Ok(())
    }
}
impl Drop for Partial {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.active = false;
        state.bytes.clear();
    }
}
struct Verifier<'a> {
    cancel: Option<&'a Control>,
}
impl DownloadVerifier for Verifier<'_> {
    fn verify(
        &self,
        _: &Path,
        _: &MediaIdentity,
        _: &dyn ExecutionControl,
    ) -> Result<(), ApiError> {
        if let Some(control) = self.cancel {
            control.cancelled.store(true, Ordering::SeqCst);
        }
        Ok(())
    }
}
#[test]
fn streamed_bytes_publish_after_verification() {
    let output = Output::default();
    let control = Control::new();
    let receipt = transfer(
        &mut Session::new(Some(5)),
        &output,
        &Verifier { cancel: None },
        &control,
        10,
    )
    .unwrap();
    assert_eq!(receipt.bytes, 5);
    let state = output.0.lock().unwrap();
    assert!(state.published && !state.active && control.reserved.load(Ordering::SeqCst));
    assert_eq!(state.final_bytes, vec![7; 5]);
}
#[test]
fn truncated_response_cleans_partial_and_allows_reacquisition() {
    let output = Output::default();
    let error = transfer(
        &mut Session::new(Some(6)),
        &output,
        &Verifier { cancel: None },
        &Control::new(),
        10,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::SourceExpired);
    {
        let state = output.0.lock().unwrap();
        assert!(!state.active && !state.published && state.bytes.is_empty());
    }
    drop(output.create().unwrap());
    assert!(!output.0.lock().unwrap().active);
}
#[test]
fn cancellation_during_verification_never_reserves_publication() {
    let output = Output::default();
    let control = Control::new();
    let error = transfer(
        &mut Session::new(Some(5)),
        &output,
        &Verifier {
            cancel: Some(&control),
        },
        &control,
        10,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert!(!control.reserved.load(Ordering::SeqCst));
    assert!(!output.0.lock().unwrap().active);
}
#[test]
fn expiration_immediately_before_publication_preserves_no_output() {
    let output = Output::default();
    let control = Control::new();
    let mut session = Session::new(Some(5));
    // Initial admission + three chunks + EOF + final guard.
    session.expires_at_check = Some(6);
    let error = transfer(
        &mut session,
        &output,
        &Verifier { cancel: None },
        &control,
        10,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert!(!control.reserved.load(Ordering::SeqCst));
    let state = output.0.lock().unwrap();
    assert!(!state.active && !state.published && state.bytes.is_empty());
}

#[test]
fn unknown_length_still_enforces_actual_byte_limit() {
    let output = Output::default();
    let control = Control::new();
    let error = transfer(
        &mut Session::new(None),
        &output,
        &Verifier { cancel: None },
        &control,
        4,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::ResourceExhausted);
    assert!(!control.reserved.load(Ordering::SeqCst));
    let state = output.0.lock().unwrap();
    assert!(!state.active && !state.published && state.bytes.is_empty());
}

#[test]
fn trusted_digest_mismatch_cannot_reach_publication() {
    let output = Output::default();
    let control = Control::new();
    let mut session = Session::new(Some(5));
    session.evidence.trusted_sha256 = Some([0; 32]);
    let error = transfer(
        &mut session,
        &output,
        &Verifier { cancel: None },
        &control,
        10,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert!(!control.reserved.load(Ordering::SeqCst));
    assert!(!output.0.lock().unwrap().active);
}

#[test]
fn expired_deadline_does_not_acquire_output() {
    let output = Output::default();
    let mut control = Control::new();
    control.deadline = Instant::now();
    let mut session = Session::new(Some(5));
    let error = transfer(
        &mut session,
        &output,
        &Verifier { cancel: None },
        &control,
        10,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::Timeout);
    assert_eq!(session.checks.get(), 0);
    assert!(!output.0.lock().unwrap().active);
}

struct RejectVerifier;
impl DownloadVerifier for RejectVerifier {
    fn verify(
        &self,
        _: &Path,
        _: &MediaIdentity,
        _: &dyn ExecutionControl,
    ) -> Result<(), ApiError> {
        Err(ApiError::new(
            ErrorCode::EngineFailed,
            "fixture invalid media tracks",
        ))
    }
}
#[test]
fn invalid_media_never_reserves_publication() {
    let output = Output::default();
    let control = Control::new();
    let error = transfer(
        &mut Session::new(Some(5)),
        &output,
        &RejectVerifier,
        &control,
        10,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::EngineFailed);
    assert!(!control.reserved.load(Ordering::SeqCst));
    let state = output.0.lock().unwrap();
    assert!(!state.active && !state.published && state.bytes.is_empty());
}

#[test]
fn primary_failure_and_explicit_cleanup_failure_are_both_preserved() {
    let output = Output::default();
    output.0.lock().unwrap().cleanup_failure = true;
    let error = transfer(
        &mut Session::new(Some(6)),
        &output,
        &Verifier { cancel: None },
        &Control::new(),
        10,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert_eq!(
        error.details["cleanup_failure"]["code"],
        "permission_denied"
    );
}

#[test]
fn publication_collision_keeps_completed_bytes() {
    let output = Output::default();
    {
        let mut state = output.0.lock().unwrap();
        state.published = true;
        state.final_bytes = b"existing media".to_vec();
    }
    let error = transfer(
        &mut Session::new(Some(5)),
        &output,
        &Verifier { cancel: None },
        &Control::new(),
        10,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    let state = output.0.lock().unwrap();
    assert_eq!(state.final_bytes, b"existing media");
    assert!(!state.active && state.bytes.is_empty());
}
