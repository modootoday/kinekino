use crate::application::{MAX_RELEASE_MANIFEST_BYTES, SnapshotLease};
use crate::application::{RunControl, WorkerRead, WorkerTransport, WorkerTransportFactory};
use kinekino_jobs::application::owned_attempt::{AttemptFailure, OwnerObservation};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::domain::ArtifactPin;
use kinekino_vad::protocol::application::{MAX_DESCRIPTOR_BYTES, MAX_OUTCOME_BYTES};
mod cleanup;
use kinekino_process::{
    application::supervisor::{ReceiptSink, Supervisor},
    domain::{
        deadline::Deadline,
        observation::{FamilyObservation, ProcessReceipt},
    },
    infrastructure::{
        evidence_files::PreparedEvidence,
        family::{OwnedProcessFamily, SpawnSpec, spawn},
    },
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

/// Existing immutable path adapter supplies an ordinary, non-reparse executable
/// with its original directory guards and denies replacement while held open.
pub trait ExecutableGuard: Send + Sync {
    fn path(&self) -> &Path;
    fn open_guarded(&self) -> Result<File, ApiError>;
}
/// Product adapter retains ancestor handles and the original exclusive writer
/// lease. The session holds the same guard even if its factory is dropped.
pub trait EvidenceRootGuard: Send + Sync {
    fn path(&self) -> &Path;
}

/// Trusted per-attempt producer. This port borrows the original control and
/// request; it cannot create an SDK scope/identity/deadline or approve JSON.
pub trait WorkerPayloadPreparation: Send + Sync {
    fn stage(
        &self,
        request: &[u8],
        owned_root: &Path,
        original: &dyn RunControl,
    ) -> Result<StagedWorkerInput, ApiError>;
}
pub struct StagedWorkerInput {
    manifest: ArtifactPin,
    child_configuration: Option<ArtifactPin>,
    _lease: Arc<dyn SnapshotLease>,
}
impl StagedWorkerInput {
    pub fn new(manifest: ArtifactPin, lease: Arc<dyn SnapshotLease>) -> Self {
        Self {
            manifest,
            child_configuration: None,
            _lease: lease,
        }
    }
    /// Host-created immutable sidecar; not caller request configuration.
    pub fn with_child_configuration(mut self, pin: ArtifactPin) -> Result<Self, ApiError> {
        if pin.id() != "parent-release-configuration"
            || pin.bytes() > crate::application::MAX_CHILD_CONFIGURATION_BYTES
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid parent configuration pin",
            ));
        }
        self.child_configuration = Some(pin);
        Ok(self)
    }
}
#[derive(Clone)]
struct PayloadBinding {
    manifest: ArtifactPin,
    producer: Arc<dyn WorkerPayloadPreparation>,
}

/// Trusted per-attempt composition, never deserialized from a CLI payload.
/// Original operation+cleanup cutoffs are supplied before prepare/launch.
pub struct NativeTransportFactory {
    pub executable: Arc<dyn ExecutableGuard>,
    pub executable_pin: ArtifactPin,
    pub cwd: PathBuf,
    pub evidence_parent: Arc<dyn EvidenceRootGuard>,
    pub original: Deadline,
    pub environment: BTreeMap<String, OsString>,
    next: AtomicU64,
    payload: Option<PayloadBinding>,
}
impl NativeTransportFactory {
    pub fn new(
        executable: Arc<dyn ExecutableGuard>,
        executable_pin: ArtifactPin,
        cwd: PathBuf,
        evidence_parent: Arc<dyn EvidenceRootGuard>,
        original: Deadline,
        environment: BTreeMap<String, OsString>,
    ) -> Self {
        Self {
            executable,
            executable_pin,
            cwd,
            evidence_parent,
            original,
            environment,
            next: AtomicU64::new(0),
            payload: None,
        }
    }
    /// Explicit opt-in for a trusted parent release. Existing inert factories
    /// and request encoders retain their previous behavior.
    pub fn with_payload(
        mut self,
        expected_manifest: ArtifactPin,
        producer: Arc<dyn WorkerPayloadPreparation>,
    ) -> Result<Self, ApiError> {
        if expected_manifest.bytes() > MAX_RELEASE_MANIFEST_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Worker release manifest exceeds bound",
            ));
        }
        self.payload = Some(PayloadBinding {
            manifest: expected_manifest,
            producer,
        });
        Ok(self)
    }
}
impl WorkerTransportFactory for NativeTransportFactory {
    fn prepare(&self) -> Result<Box<dyn WorkerTransport>, ApiError> {
        if !self.executable.path().is_absolute()
            || !self.cwd.is_absolute()
            || !self.evidence_parent.path().is_absolute()
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Absolute trusted VAD paths required",
            ));
        };
        // Inert allocation only; raw files and processes are created in run.
        let root = self.evidence_parent.path().join(format!(
            "vad-{}-{}",
            std::process::id(),
            self.next.fetch_add(1, Ordering::Relaxed)
        ));
        Ok(Box::new(NativeTransport {
            executable: self.executable.clone(),
            binary_guard: None,
            pin: self.executable_pin.clone(),
            cwd: self.cwd.clone(),
            environment: self.environment.clone(),
            root,
            _evidence_guard: self.evidence_parent.clone(),
            original: self.original,
            owner: None,
            started: false,
            no_child_confirmed: true,
            cancel_signal_sent: false,
            secondary: Vec::new(),
            secondary_overflow: false,
            cleanup: cleanup::CleanupTracking::default(),
            payload: self.payload.clone(),
            staged_input: None,
            _worker_control: None,
            _cancel_guard: None,
        }))
    }
}
struct NativeTransport {
    executable: Arc<dyn ExecutableGuard>,
    binary_guard: Option<File>,
    pin: ArtifactPin,
    cwd: PathBuf,
    environment: BTreeMap<String, OsString>,
    root: PathBuf,
    _evidence_guard: Arc<dyn EvidenceRootGuard>,
    original: Deadline,
    owner: Option<Supervisor<OwnedProcessFamily>>,
    started: bool,
    no_child_confirmed: bool,
    cancel_signal_sent: bool,
    secondary: Vec<AttemptFailure>,
    secondary_overflow: bool,
    cleanup: cleanup::CleanupTracking,
    payload: Option<PayloadBinding>,
    staged_input: Option<StagedWorkerInput>,
    _worker_control: Option<super::worker_control::PreparedWorkerControl>,
    _cancel_guard: Option<File>,
}
fn io(error: std::io::Error) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, error.to_string())
}
fn payload_control(control: &dyn RunControl) -> Result<(), ApiError> {
    if control.cancellation_requested() {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Original VAD payload preparation stopped; no cleanup ACK",
        ));
    }
    if Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Original VAD payload preparation cutoff expired",
        ));
    }
    Ok(())
}
/// Read-only cap of the original caller and already-established operation end.
/// This creates no Deadline, SDK scope or renewed duration, and keeps cleanup
/// reserve unavailable to binary admission/payload staging.
struct PreparationControl<'a> {
    caller: &'a dyn RunControl,
    operation_end: Instant,
}
impl<'a> PreparationControl<'a> {
    fn new(caller: &'a dyn RunControl, original: Deadline) -> Result<Self, ApiError> {
        payload_control(caller)?;
        let now = Instant::now();
        let remaining = original.operation_remaining(now);
        if remaining.is_zero() {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Original operation cutoff already expired",
            ));
        }
        let operation_end = now.checked_add(remaining).ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Original operation cutoff cannot be represented",
            )
        })?;
        Ok(Self {
            caller,
            operation_end,
        })
    }
}
impl RunControl for PreparationControl<'_> {
    fn deadline(&self) -> Instant {
        self.caller.deadline().min(self.operation_end)
    }
    fn cancellation_requested(&self) -> bool {
        self.caller.cancellation_requested()
    }
}
fn remember_secondary(errors: &mut Vec<AttemptFailure>, overflow: &mut bool, error: ApiError) {
    if errors.len() < 8 {
        errors.push(AttemptFailure::capture(&error));
    } else {
        *overflow = true;
    }
}
fn signal(root: &std::path::Path) -> Result<File, ApiError> {
    super::worker_control::publish_cancellation(root)
}
struct Sink(PathBuf);
impl ReceiptSink for Sink {
    fn record(&mut self, r: &ProcessReceipt) -> std::io::Result<()> {
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.0)?;
        let bytes=serde_json::to_vec(&serde_json::json!({"pid":r.identity.pid,"creation_filetime":r.identity.creation_filetime,"executable":r.identity.executable,"child_exit":r.child_exit,"family":format!("{:?}",r.family),"forced":r.forced,"stdin_eof_requested":r.stdin_eof_requested})).map_err(std::io::Error::other)?;
        f.write_all(&bytes)?;
        f.sync_all()
    }
}
impl WorkerTransport for NativeTransport {
    fn cancellation_signal_sent(&self) -> bool {
        self.cancel_signal_sent
    }
    fn launch_read(
        &mut self,
        request: &[u8],
        control: &dyn RunControl,
    ) -> Result<WorkerRead, ApiError> {
        if self.started {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native VAD already started",
            ));
        };
        self.started = true;
        let preparation = PreparationControl::new(control, self.original)?;
        if self.pin.bytes() > 1024 * 1024 * 1024 {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "VAD executable pin exceeds bound",
            ));
        };
        self.binary_guard = Some(self.executable.open_guarded()?);
        let mut binary = self
            .binary_guard
            .as_mut()
            .expect("Original executable guard")
            .take(self.pin.bytes() + 1);
        let mut digest = Sha256::new();
        let mut count = 0u64;
        let mut chunk = [0u8; 8192];
        loop {
            if preparation.cancellation_requested() {
                return Err(ApiError::new(
                    ErrorCode::Cancelled,
                    "VAD launch admission stopped; no ACK",
                ));
            };
            if Instant::now() >= preparation.deadline() {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "Original VAD executable admission cutoff",
                ));
            };
            let n = binary.read(&mut chunk).map_err(io)?;
            if n == 0 {
                break;
            };
            count += n as u64;
            if count > self.pin.bytes() {
                return Err(ApiError::new(
                    ErrorCode::SourceExpired,
                    "VAD executable exceeds pin",
                ));
            };
            digest.update(&chunk[..n]);
        }
        if count != self.pin.bytes() || format!("{:x}", digest.finalize()) != self.pin.sha256() {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "VAD executable pin mismatch",
            ));
        };
        fs::create_dir(&self.root).map_err(io)?;
        let evidence =
            PreparedEvidence::create(&self.root, Some(request), MAX_DESCRIPTOR_BYTES as usize)
                .map_err(io)?;
        let input = evidence
            .paths
            .input
            .as_ref()
            .ok_or_else(|| ApiError::new(ErrorCode::Internal, "VAD request file absent"))?
            .clone();
        payload_control(&preparation)?;
        if let Some(binding) = &self.payload {
            let staged = binding.producer.stage(request, &self.root, &preparation)?;
            if staged.manifest != binding.manifest {
                return Err(ApiError::new(
                    ErrorCode::SourceExpired,
                    "Staged release differs from the original admitted manifest",
                ));
            }
            // Store before any child creation. Err/unwind/Pending after spawn
            // keep the same producer, admission and staged guards in self.
            self.staged_input = Some(staged);
        }
        payload_control(&preparation)?;
        let mut arguments = vec![
            "--vad-request-v1".into(),
            input.into_os_string(),
            "--cancel-file".into(),
            self.root.join("cancel.signal").into_os_string(),
        ];
        if let Some(staged) = &self.staged_input {
            arguments.extend([
                "--release-manifest-id".into(),
                staged.manifest.id().into(),
                "--release-manifest-sha256".into(),
                staged.manifest.sha256().into(),
                "--release-manifest-bytes".into(),
                staged.manifest.bytes().to_string().into(),
            ]);
        }
        let worker_control =
            super::worker_control::prepare(&self.root, self.original, &preparation, request)?;
        arguments.extend(worker_control.arguments());
        self._worker_control = Some(worker_control);
        if let Some(pin) = self
            .staged_input
            .as_ref()
            .and_then(|s| s.child_configuration.as_ref())
        {
            arguments.extend([
                "--parent-release-id".into(),
                pin.id().into(),
                "--parent-release-sha256".into(),
                pin.sha256().into(),
                "--parent-release-bytes".into(),
                pin.bytes().to_string().into(),
            ]);
        }
        let spec = SpawnSpec {
            executable: self.executable.path().to_owned(),
            cwd: self.cwd.clone(),
            environment: Some(self.environment.clone()),
            arguments,
        };
        payload_control(&preparation)?;
        // An unwind during the original spawn is not a no-child observation.
        self.no_child_confirmed = false;
        match spawn(spec, evidence, self.original) {
            Ok((process, evidence)) => {
                self.owner = Some(Supervisor::new(process, evidence, MAX_OUTCOME_BYTES))
            }
            Err(failure) => {
                let failure = *failure;
                if let Some(owner) = failure.owner {
                    self.owner = Some(Supervisor::new(*owner, failure.evidence, MAX_OUTCOME_BYTES))
                } else {
                    self.no_child_confirmed = true;
                }
                if let Some(error) = failure.termination_error {
                    remember_secondary(&mut self.secondary, &mut self.secondary_overflow, io(error))
                };
                return Err(io(failure.error));
            }
        }
        loop {
            if control.cancellation_requested() && !self.cancel_signal_sent {
                self._cancel_guard = Some(signal(&self.root)?);
                self.cancel_signal_sent = true;
                // Continue bounded observation of the same original owner. Only
                // a verified Partial can accompany a parent cooperative marker.
            };
            if Instant::now() >= control.deadline()
                || self.original.operation_remaining(Instant::now()).is_zero()
            {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "Original VAD operation cutoff; owner retained",
                ));
            };
            let owner = self
                .owner
                .as_mut()
                .expect("Original owner stored before observation");
            owner
                .evidence()
                .check_limit(MAX_OUTCOME_BYTES)
                .map_err(io)?;
            let r = owner.poll_state().map_err(|failure| io(failure.error))?;
            if r.child_exit.is_some() && r.family == FamilyObservation::Empty {
                let stdout = owner.evidence().stdout.open_read().map_err(io)?;
                let declared_bytes = stdout.metadata().map_err(io)?.len();
                return Ok(WorkerRead {
                    declared_bytes,
                    stdout: Box::new(stdout),
                });
            }
            std::thread::sleep(
                Duration::from_millis(10).min(self.original.operation_remaining(Instant::now())),
            );
        }
    }
    fn poll_close(&mut self, deadline: Instant) -> Result<OwnerObservation, ApiError> {
        cleanup::poll(self, deadline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{OnceLock, atomic::AtomicUsize};
    /// Absolute on every host; never opened by the inert fixtures.
    fn fixture(name: &str) -> &'static Path {
        static ROOT: OnceLock<[PathBuf; 3]> = OnceLock::new();
        let [root, worker, evidence] = ROOT.get_or_init(|| {
            let root = std::env::temp_dir().join("fixture");
            [root.clone(), root.join("worker.exe"), root.join("evidence")]
        });
        match name {
            "worker.exe" => worker,
            "evidence" => evidence,
            _ => root,
        }
    }
    struct Guard(AtomicUsize);
    impl ExecutableGuard for Guard {
        fn path(&self) -> &Path {
            fixture("worker.exe")
        }
        fn open_guarded(&self) -> Result<File, ApiError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Err(ApiError::new(
                ErrorCode::Internal,
                "No OS access in inert fixture",
            ))
        }
    }
    impl EvidenceRootGuard for Guard {
        fn path(&self) -> &Path {
            fixture("evidence")
        }
    }
    struct PayloadOwner(Arc<AtomicUsize>);
    impl Drop for PayloadOwner {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    impl SnapshotLease for PayloadOwner {}
    struct NeverStage {
        _owner: Arc<PayloadOwner>,
        calls: Arc<AtomicUsize>,
    }
    impl WorkerPayloadPreparation for NeverStage {
        fn stage(
            &self,
            _: &[u8],
            _: &Path,
            _: &dyn RunControl,
        ) -> Result<StagedWorkerInput, ApiError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            panic!("Payload staging must not run during inert prepare/close");
        }
    }
    fn payload_factory(guard: Arc<Guard>) -> NativeTransportFactory {
        kinekino_platform::install();
        NativeTransportFactory::new(
            guard.clone(),
            ArtifactPin::new(
                "worker".into(),
                "a".repeat(64),
                kinekino_media_contracts::api_values::DecimalU64(1),
            )
            .unwrap(),
            fixture("").into(),
            guard,
            Deadline::new(
                Instant::now(),
                Duration::from_secs(5),
                Duration::from_secs(1),
            )
            .unwrap(),
            BTreeMap::new(),
        )
    }
    #[test]
    fn preparation_view_preserves_original_operation_cutoff_and_cancellation() {
        struct Caller {
            end: Instant,
            cancel: std::sync::atomic::AtomicBool,
        }
        impl RunControl for Caller {
            fn deadline(&self) -> Instant {
                self.end
            }
            fn cancellation_requested(&self) -> bool {
                self.cancel.load(Ordering::SeqCst)
            }
        }
        let start = Instant::now();
        let original =
            Deadline::new(start, Duration::from_secs(5), Duration::from_secs(1)).unwrap();
        let caller = Caller {
            end: start + Duration::from_secs(10),
            cancel: false.into(),
        };
        let view = PreparationControl::new(&caller, original).unwrap();
        assert_eq!(view.deadline(), start + Duration::from_secs(4));
        caller.cancel.store(true, Ordering::SeqCst);
        assert!(view.cancellation_requested());
        assert_eq!(view.deadline(), start + Duration::from_secs(4));
        assert_eq!(
            payload_control(&view).unwrap_err().code,
            ErrorCode::Cancelled
        );
        let caller = Caller {
            end: start + Duration::from_secs(2),
            cancel: false.into(),
        };
        assert_eq!(
            PreparationControl::new(&caller, original)
                .unwrap()
                .deadline(),
            caller.end
        );
    }
    #[test]
    fn inert_payload_session_retains_original_source_after_factory_drop_without_staging() {
        let guard = Arc::new(Guard(AtomicUsize::new(0)));
        let released = Arc::new(AtomicUsize::new(0));
        let calls = Arc::new(AtomicUsize::new(0));
        let owner = Arc::new(PayloadOwner(released.clone()));
        let producer = Arc::new(NeverStage {
            _owner: owner.clone(),
            calls: calls.clone(),
        });
        let manifest = ArtifactPin::new(
            "release".into(),
            "b".repeat(64),
            kinekino_media_contracts::api_values::DecimalU64(1),
        )
        .unwrap();
        let factory = payload_factory(guard.clone())
            .with_payload(manifest, producer)
            .unwrap();
        let mut session = factory.prepare().unwrap();
        drop(factory);
        drop(owner);
        assert_eq!(
            session.poll_close(Instant::now()).unwrap(),
            OwnerObservation::NotStarted
        );
        assert_eq!(guard.0.load(Ordering::SeqCst), 0);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(released.load(Ordering::SeqCst), 0);
        drop(session);
        assert_eq!(released.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn oversized_payload_manifest_rejects_before_any_source_or_executable_access() {
        let guard = Arc::new(Guard(AtomicUsize::new(0)));
        let released = Arc::new(AtomicUsize::new(0));
        let calls = Arc::new(AtomicUsize::new(0));
        let producer = Arc::new(NeverStage {
            _owner: Arc::new(PayloadOwner(released.clone())),
            calls: calls.clone(),
        });
        let manifest = ArtifactPin::new(
            "release".into(),
            "b".repeat(64),
            kinekino_media_contracts::api_values::DecimalU64(MAX_RELEASE_MANIFEST_BYTES + 1),
        )
        .unwrap();
        let error = payload_factory(guard.clone())
            .with_payload(manifest, producer)
            .err()
            .unwrap();
        assert_eq!(error.code, ErrorCode::ResourceExhausted);
        assert_eq!(guard.0.load(Ordering::SeqCst), 0);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(released.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn native_factory_prepares_send_session_without_opening_or_launching() {
        fn send<T: Send>() {}
        kinekino_platform::install();
        send::<NativeTransport>();
        let guard = Arc::new(Guard(AtomicUsize::new(0)));
        let pin = ArtifactPin::new(
            "worker".into(),
            "a".repeat(64),
            kinekino_media_contracts::api_values::DecimalU64(1),
        )
        .unwrap();
        let factory = NativeTransportFactory::new(
            guard.clone(),
            pin,
            fixture("").into(),
            guard.clone(),
            Deadline::new(
                Instant::now(),
                Duration::from_secs(5),
                Duration::from_secs(1),
            )
            .unwrap(),
            BTreeMap::new(),
        );
        let mut session = factory.prepare().unwrap();
        assert_eq!(guard.0.load(Ordering::SeqCst), 0);
        assert_eq!(
            session.poll_close(Instant::now()).unwrap(),
            OwnerObservation::NotStarted
        );
        drop(session);
    }
    #[test]
    fn secondary_native_errors_reuse_common_bounded_capture_without_replacing_first() {
        let mut errors = Vec::new();
        let mut overflow = false;
        for i in 0..12 {
            remember_secondary(
                &mut errors,
                &mut overflow,
                ApiError::new(
                    ErrorCode::EngineFailed,
                    format!("secondary {i}: {}", "x".repeat(5000)),
                ),
            );
        }
        assert_eq!(errors.len(), 8);
        assert!(overflow);
        assert!(errors[0].message().starts_with("secondary 0:"));
        assert!(errors.iter().all(|e| e.message().len() <= 4096));
    }
}
