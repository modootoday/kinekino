use super::*;
mod live;
use kinekino_process::{
    application::supervisor::ReceiptSink,
    domain::observation::ProcessReceipt,
    infrastructure::{evidence_files::PreparedEvidence, family::SpawnSpec, ipc::PreparedSourceIpc},
};
use kinekino_source_host::{
    parent_host::UnassignedCloseReceipt, parent_launch::ParentLaunch, request::HttpBootstrap,
};
use std::{
    fs, io,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Control(Instant);
impl ExecutionControl for Control {
    fn cancellation_requested(&self) -> bool {
        false
    }
    fn deadline(&self) -> Instant {
        self.0
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        panic!("source cannot publish")
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        panic!("source cannot verify")
    }
}
struct Admission(
    Arc<AtomicUsize>,
    Arc<std::sync::atomic::AtomicBool>,
    bool,
    Arc<AtomicUsize>,
);
impl ReplayAdmission for Admission {
    fn poll_release(&mut self, _: Instant) -> Result<bool, ApiError> {
        self.3.fetch_add(1, Ordering::SeqCst);
        Ok(true)
    }
    fn check_request(&self, _: &HttpBootstrap) -> Result<(), ApiError> {
        Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Fixture request differs from original grant",
        ))
    }
    fn check_origin(&self, _: &dyn ExecutionControl) -> Result<(), ApiError> {
        if self.1.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(ApiError::new(
                ErrorCode::SourceExpired,
                "fixture original source revoked",
            ))
        }
    }
    fn check(&self, _: &dyn ExecutionControl) -> Result<(), ApiError> {
        if self.2 {
            return Ok(());
        }
        Err(ApiError::new(
            ErrorCode::PermissionDenied,
            "fixture rejects original replay",
        ))
    }
    fn consume(&mut self, _: &dyn ExecutionControl) -> Result<(), ApiError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        panic!("rejected replay cannot be consumed")
    }
}
struct Guard(Arc<AtomicUsize>);
impl Drop for Guard {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
struct Sink {
    calls: Arc<AtomicUsize>,
    fail: bool,
}
impl ReceiptSink for Sink {
    fn record(&mut self, _: &ProcessReceipt) -> io::Result<()> {
        panic!("no process was created")
    }
}
impl ParentReceiptSink for Sink {
    fn record_unassigned(&mut self, _: &UnassignedCloseReceipt) -> io::Result<()> {
        panic!("no process was created")
    }
}
impl SourceReceiptSink for Sink {
    fn record_transport<G: Send>(
        &mut self,
        facts: &LaunchCloseFacts,
        original: &ParentTransfer<G>,
    ) -> Result<(), ApiError> {
        assert!(facts.ready_to_ack());
        assert!(facts.host.is_none() && original.host().is_none());
        assert!(original.io_errors().is_empty());
        assert!(!original.io_overflow());
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail {
            let mut error = failed("fixture transport sink failure");
            error.request_id = "original-sink-error".into();
            Err(error)
        } else {
            Ok(())
        }
    }
}
type Opener = ProcessSourceOpener<Guard, Admission, Sink>;
struct Fixture {
    opener: Opener,
    control: Control,
    calls: Arc<AtomicUsize>,
    consumed: Arc<AtomicUsize>,
    drops: Arc<AtomicUsize>,
    origin_valid: Arc<std::sync::atomic::AtomicBool>,
    releases: Arc<AtomicUsize>,
}
fn fixture(label: &str, fail: bool) -> Fixture {
    let root = std::env::temp_dir().join(format!(
        "sdk-source-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    eprintln!("sdk_source_evidence={}", root.display());
    let start = Instant::now();
    let deadline = Deadline::new(start, Duration::from_secs(8), Duration::from_secs(2)).unwrap();
    let evidence = PreparedEvidence::create(&root, None, 64).unwrap();
    let mut ipc = PreparedSourceIpc::new(deadline).unwrap();
    while !ipc.poll_connected().unwrap() {
        assert!(!deadline.operation_remaining(Instant::now()).is_zero());
        std::thread::sleep(Duration::from_millis(1));
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let consumed = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let launch = ParentLaunch::new(
        SpawnSpec {
            executable: root.join("must-not-launch.exe"),
            cwd: root,
            arguments: Vec::new(),
            environment: None,
        },
        evidence,
        deadline,
        ipc,
        65536,
        Guard(drops.clone()),
    );
    let transfer = ParentTransfer::new(
        launch,
        HttpBootstrap::new([8; 32], 1000, "https://example.invalid/fixture", None).unwrap(),
    );
    let origin_valid = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let releases = Arc::new(AtomicUsize::new(0));
    let opener = ProcessSourceOpener::new(
        transfer,
        Admission(
            consumed.clone(),
            origin_valid.clone(),
            false,
            releases.clone(),
        ),
        Sink {
            calls: calls.clone(),
            fail,
        },
        deadline,
        MediaIdentity {
            media: "fixture-media".into(),
            tracks: "fixture-tracks".into(),
        },
    );
    Fixture {
        opener,
        control: Control(start + Duration::from_secs(6)),
        calls,
        consumed,
        drops,
        origin_valid,
        releases,
    }
}
#[test]
#[ignore = "actual original Windows IPC handles; centrally assigned"]
fn rejected_sdk_open_is_spent_and_session_close_drives_original_no_child_cleanup() {
    let mut f = fixture("rejected-open", false);
    let error = f.opener.open(&f.control).err().expect("rejected replay");
    assert_eq!(error.code, ErrorCode::PermissionDenied);
    let error = f.opener.open(&f.control).err().expect("spent opener");
    assert_eq!(error.message, "Original source open was already attempted");
    // Cleanup-only fixture; no session was admitted and no metadata/GET is claimed.
    let mut session = ProcessSession {
        body_ended: false,
        shared: f.opener.shared.clone(),
        evidence: TransferEvidence {
            identity: f.opener.identity.clone(),
            response_length: None,
            trusted_sha256: None,
        },
    };
    session.close_owned().unwrap();
    assert!(f.opener.poll_close(f.control.0).unwrap());
    assert_eq!(f.calls.load(Ordering::SeqCst), 1);
    assert_eq!(f.consumed.load(Ordering::SeqCst), 0);
    assert_eq!(f.drops.load(Ordering::SeqCst), 0);
    drop(session);
    assert_eq!(f.drops.load(Ordering::SeqCst), 0);
    drop(f.opener);
    assert_eq!(f.drops.load(Ordering::SeqCst), 1);
}
#[test]
#[ignore = "original prepared Windows handles; request mismatch must not launch a child"]
fn request_mismatch_is_rejected_before_launch_and_keeps_original_cleanup() {
    let mut f = fixture("request-mismatch", false);
    f.opener.shared.lock().unwrap().admission.2 = true;
    assert_eq!(
        f.opener.open(&f.control).err().unwrap().code,
        ErrorCode::RevisionConflict
    );
    assert!(
        f.opener
            .shared
            .lock()
            .unwrap()
            .transfer
            .as_ref()
            .unwrap()
            .host()
            .is_none()
    );
    assert_eq!(f.consumed.load(Ordering::SeqCst), 0);
    assert!(f.opener.open(&f.control).is_err());
    while !f.opener.poll_close(f.control.0).unwrap() {
        assert!(Instant::now() < f.control.0);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(f.calls.load(Ordering::SeqCst), 1);
    assert_eq!(f.drops.load(Ordering::SeqCst), 0);
    drop(f.opener);
    assert_eq!(f.drops.load(Ordering::SeqCst), 1);
}
#[test]
#[ignore = "actual original Windows IPC handles; centrally assigned"]
fn closed_source_origin_uses_retained_admission_and_rejects_revocation() {
    let f = fixture("closed-origin", false);
    // BodyEnd here is an explicit fixture, never a real HTTP/admission success.
    let mut session = ProcessSession {
        shared: f.opener.shared.clone(),
        body_ended: false,
        evidence: TransferEvidence {
            identity: f.opener.identity.clone(),
            response_length: None,
            trusted_sha256: None,
        },
    };
    assert!(session.check_origin(&f.control).is_err());
    session.close_owned().unwrap();
    assert!(session.check_origin(&f.control).is_err());
    session.body_ended = true;
    assert!(session.check_current(&f.control).is_err());
    session.check_origin(&f.control).unwrap();
    f.origin_valid.store(false, Ordering::SeqCst);
    assert_eq!(
        session.check_origin(&f.control).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    assert_eq!(f.consumed.load(Ordering::SeqCst), 0);
    assert_eq!(f.calls.load(Ordering::SeqCst), 1);
}
#[test]
#[ignore = "actual original Windows IPC handles; centrally assigned"]
fn transport_receipt_failure_is_not_retried_or_acknowledged() {
    let mut f = fixture("sink-failure", true);
    let error = loop {
        match f.opener.poll_close(f.control.0) {
            Ok(false) => {
                assert!(Instant::now() < f.control.0);
                std::thread::sleep(Duration::from_millis(1));
            }
            Ok(true) => panic!("sink failure cannot ACK"),
            Err(error) => break error,
        }
    };
    assert_eq!(error.request_id, "original-sink-error");
    assert_eq!(
        f.opener.owner_observation(),
        crate::application::jobs::owned_attempt::OwnerObservation::Pending
    );
    assert!(f.opener.poll_release(f.control.0).unwrap());
    assert_eq!(f.releases.load(Ordering::SeqCst), 1);
    assert_eq!(
        f.opener.poll_close(f.control.0).unwrap_err().request_id,
        "original-sink-error"
    );
    assert_eq!(f.calls.load(Ordering::SeqCst), 1);
    assert_eq!(f.drops.load(Ordering::SeqCst), 0);
    drop(f.opener);
    assert_eq!(f.drops.load(Ordering::SeqCst), 1);
}
