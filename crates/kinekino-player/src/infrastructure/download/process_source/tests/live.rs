//! Actual IPC byte flow with a synthetic admission/body, never browser/HTTP proof.
use super::*;
#[cfg(feature = "source-adapter-test-support")]
mod jobs;
use std::io::Write;
struct LiveAdmission {
    valid: Arc<std::sync::atomic::AtomicBool>,
    consumed: Arc<AtomicUsize>,
}
impl ReplayAdmission for LiveAdmission {
    fn poll_release(&mut self, _: Instant) -> Result<bool, ApiError> {
        Ok(true)
    }
    fn check_request(
        &self,
        request: &kinekino_source_host::request::HttpBootstrap,
    ) -> Result<(), ApiError> {
        if request.target().host_str() == Some("example.invalid")
            && request.target().path().starts_with("/offline-sdk-")
            && request.referer().is_none()
        {
            Ok(())
        } else {
            Err(failed("Offline fixture request differs"))
        }
    }
    fn check(&self, _: &dyn ExecutionControl) -> Result<(), ApiError> {
        if self.valid.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(ApiError::new(
                ErrorCode::SourceExpired,
                "fixture source expired",
            ))
        }
    }
    fn consume(&mut self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        self.check(control)?;
        if self.consumed.fetch_add(1, Ordering::SeqCst) != 0 {
            return Err(failed("fixture replay already consumed"));
        }
        Ok(())
    }
    fn check_origin(&self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        self.check(control)
    }
}
struct LiveSink {
    expected_exit: u32,
    file: fs::File,
    transport: Arc<AtomicUsize>,
}
impl ReceiptSink for LiveSink {
    fn record(&mut self, receipt: &ProcessReceipt) -> io::Result<()> {
        writeln!(self.file, "{receipt:?}")?;
        self.file.flush()?;
        self.file.sync_all()
    }
}
impl ParentReceiptSink for LiveSink {
    fn record_unassigned(&mut self, _: &UnassignedCloseReceipt) -> io::Result<()> {
        Err(io::Error::other("unexpected unassigned fixture child"))
    }
}
impl SourceReceiptSink for LiveSink {
    fn record_prepared_transport<G: Send>(
        &mut self,
        facts: &LaunchCloseFacts,
        original: &ParentLaunch<G>,
    ) -> Result<(), ApiError> {
        assert!(facts.ready_to_ack());
        assert!(facts.host.is_none() && facts.prepared_ipc.is_some());
        assert!(original.host().is_none());
        assert!(original.io_errors().is_empty() && !original.io_overflow());
        writeln!(self.file, "transport-prepared-no-child={facts:?}")
            .and_then(|()| self.file.flush())
            .and_then(|()| self.file.sync_all())
            .map_err(|_| failed("fixture prepared transport persistence failed"))?;
        self.transport.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn record_transport<G: Send>(
        &mut self,
        facts: &LaunchCloseFacts,
        original: &ParentTransfer<G>,
    ) -> Result<(), ApiError> {
        assert!(facts.ready_to_ack());
        assert!(original.io_errors().is_empty());
        assert!(!original.io_overflow());
        let Some(host) = original.host() else {
            assert!(facts.host.is_none() && facts.prepared_ipc.is_some());
            writeln!(self.file, "transport-no-child={facts:?}")
                .and_then(|()| self.file.flush())
                .and_then(|()| self.file.sync_all())
                .map_err(|_| failed("fixture no-child transport persistence failed"))?;
            self.transport.fetch_add(1, Ordering::SeqCst);
            return Ok(());
        };
        assert!(host.first_close_error().is_none());
        let receipt = host.receipt().unwrap();
        assert_eq!(receipt.child_exit, Some(self.expected_exit));
        assert!(!receipt.forced);
        assert_eq!(
            receipt.family,
            kinekino_process::domain::observation::FamilyObservation::Empty
        );
        writeln!(self.file, "transport={facts:?}")
            .and_then(|()| self.file.flush())
            .and_then(|()| self.file.sync_all())
            .map_err(|_| failed("fixture transport persistence failed"))?;
        self.transport.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}
fn exercise_source_exit(expected_exit: u32) {
    let worker = std::path::PathBuf::from(
        std::env::var_os("SOURCE_SDK_FIXTURE_BIN").expect("fixed fixture worker"),
    );
    assert!(worker.is_absolute());
    let _artifact = fs::File::open(&worker).unwrap();
    let root = std::env::temp_dir().join(format!(
        "sdk-source-live-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    eprintln!("sdk_source_evidence={}", root.display());
    let start = Instant::now();
    let deadline = Deadline::new(start, Duration::from_secs(10), Duration::from_secs(3)).unwrap();
    let control = Control(start + Duration::from_secs(7));
    let evidence = PreparedEvidence::create(&root, None, 64).unwrap();
    let mut ipc = PreparedSourceIpc::new(deadline).unwrap();
    while !ipc.poll_connected().unwrap() {
        assert!(Instant::now() < control.0);
        std::thread::sleep(Duration::from_millis(1));
    }
    let drops = Arc::new(AtomicUsize::new(0));
    let consumed = Arc::new(AtomicUsize::new(0));
    let valid = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let transport = Arc::new(AtomicUsize::new(0));
    let launch = ParentLaunch::new(
        SpawnSpec {
            executable: worker,
            cwd: root.clone(),
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
        HttpBootstrap::new(
            [7; 32],
            1000,
            if expected_exit == 0 {
                "https://example.invalid/offline-sdk-body"
            } else {
                "https://example.invalid/offline-sdk-high-exit"
            },
            None,
        )
        .unwrap(),
    );
    let sink = LiveSink {
        expected_exit,
        file: fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(root.join("source-receipts.txt"))
            .unwrap(),
        transport: transport.clone(),
    };
    let identity = MediaIdentity {
        media: "fixture-media".into(),
        tracks: "fixture-tracks".into(),
    };
    let mut opener = ProcessSourceOpener::new(
        transfer,
        LiveAdmission {
            valid: valid.clone(),
            consumed: consumed.clone(),
        },
        sink,
        deadline,
        identity.clone(),
    );
    let mut session = opener.open(&control).unwrap();
    assert_eq!(consumed.load(Ordering::SeqCst), 1);
    assert_eq!(session.evidence().identity, identity);
    assert_eq!(
        session.evidence().response_length,
        Some(b"offline-sdk-body".len() as u64)
    );
    assert_eq!(session.evidence().trusted_sha256, None);
    let mut body = Vec::new();
    let mut buffer = [0; 3];
    loop {
        let n = session.read(&mut buffer, &control).unwrap();
        if n == 0 {
            break;
        }
        body.extend_from_slice(&buffer[..n]);
    }
    assert_eq!(body, b"offline-sdk-body");
    assert!(session.check_origin(&control).is_err());
    session.close_owned().unwrap();
    let observation = opener.owner_observation();
    let crate::application::jobs::owned_attempt::OwnerObservation::Exited {
        exit_code,
        family_empty: true,
        streams_eof: true,
        forced: false,
    } = observation
    else {
        panic!("Original DWORD exit is not represented: {observation:?}");
    };
    assert_eq!(u32::from_ne_bytes(exit_code.to_ne_bytes()), expected_exit);
    session.check_origin(&control).unwrap();
    assert!(opener.poll_close(control.0).unwrap());
    assert_eq!(transport.load(Ordering::SeqCst), 1);
    assert_eq!(consumed.load(Ordering::SeqCst), 1);
    valid.store(false, Ordering::SeqCst);
    assert_eq!(
        session.check_origin(&control).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    drop(session);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(opener);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
#[test]
#[ignore = "actual pinned fixture child and original Windows IPC; centrally assigned"]
fn sdk_open_metadata_credit_body_end_and_physical_close_use_same_original_owner() {
    exercise_source_exit(0);
}
#[test]
#[ignore = "actual pinned controlled high-bit Windows exit; centrally assigned"]
fn high_bit_windows_exit_preserves_original_status_after_physical_close() {
    exercise_source_exit(0x8000_0001);
}
