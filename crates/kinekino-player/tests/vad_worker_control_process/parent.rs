//! Parent test consumer reuses the exact SDK adapter and original Supervisor.
#[path = "configured.rs"]
mod configured;
#[path = "configured_payload.rs"]
mod configured_payload;
#[path = "scenarios.rs"]
mod scenarios;
#[path = "support.rs"]
mod support;
#[path = "../../../kinekino-vad-executor/src/infrastructure/worker_control.rs"]
mod worker_control;
use crate::{fixture_barrier, fixture_data::FixtureData, fixture_payload::FixturePayload};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
/// `fixture_data` compiles the product storage-path sources by path; they resolve the OS policy here.
pub(crate) use kinekino_player::infrastructure::os_policy;
use kinekino_process::{
    application::supervisor::{ReceiptSink, Supervisor, SystemClock},
    domain::observation::{FamilyObservation, ProcessReceipt},
    infrastructure::{
        evidence_files::PreparedEvidence,
        family::{LaunchFailure, OwnedProcessFamily, SpawnSpec, spawn},
    },
};
use kinekino_vad_executor::{
    Deadline,
    application::{RunControl, WorkerTransport, WorkerTransportFactory},
    infrastructure::{
        EvidenceRootGuard, ExecutableGuard, NativeTransportFactory, StagedWorkerInput,
        WorkerPayloadPreparation,
    },
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, File},
    io::{Read, Write},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug)]
pub(super) enum CaseKind {
    Complete,
    RunningCancel,
    BeforeEngineCancel,
    DelayedStart,
    RunningExpiry,
    SignalFault,
    BindingMismatch,
    FlushFailure,
    ConfiguredComplete,
    ConfiguredCancel,
    ConfiguredRequestMismatch,
    ConfiguredBytesMismatch,
    ConfiguredManifestMismatch,
    ConfiguredWorkerMismatch,
}
struct Original {
    end: Instant,
}
impl RunControl for Original {
    fn deadline(&self) -> Instant {
        self.end
    }
    fn cancellation_requested(&self) -> bool {
        false
    }
}
struct JsonReceipt(PathBuf);
impl ReceiptSink for JsonReceipt {
    fn record(&mut self, receipt: &ProcessReceipt) -> std::io::Result<()> {
        let bytes = serde_json::to_vec_pretty(&json!({"identity":{"pid":receipt.identity.pid,"creation_filetime":receipt.identity.creation_filetime,"executable":receipt.identity.executable},"child_exit":receipt.child_exit,"family":format!("{:?}",receipt.family),"forced":receipt.forced,"stdin_eof_requested":receipt.stdin_eof_requested})).map_err(std::io::Error::other)?;
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.0)?;
        file.write_all(&bytes)?;
        file.sync_all()
    }
}
struct Case {
    kind: CaseKind,
    root: PathBuf,
    data: Arc<FixtureData>,
    deadline: Deadline,
    operation_end: Instant,
    end: Instant,
    owner: Option<Supervisor<OwnedProcessFamily>>,
    launch_failure: Option<Box<LaunchFailure>>,
    native: Option<Box<dyn WorkerTransport>>,
    _control: Option<worker_control::PreparedWorkerControl>,
    _payload: Option<StagedWorkerInput>,
    guards: Vec<File>,
    terminal: Option<ProcessReceipt>,
    exported: bool,
    close_error: Option<ApiError>,
    cleanup_panic: Option<Box<dyn std::any::Any + Send>>,
}
impl Case {
    fn new(kind: CaseKind) -> Self {
        let base = PathBuf::from(
            std::env::var_os("VAD_CONTROL_PROCESS_EVIDENCE_ROOT")
                .expect("Explicit evidence root required"),
        );
        assert!(base.is_absolute());
        fs::create_dir_all(&base).unwrap();
        let root = base.join(format!("{kind:?}"));
        fs::create_dir(&root).unwrap();
        let executable = PathBuf::from(
            std::env::var_os("VAD_CONTROL_PROCESS_FIXTURE_EXE")
                .expect("Fixed compiled fixture required"),
        );
        // One initial budget only. Operation4s + original cleanup4s; child
        // startup/staging/waits consume it. No phase starts another timeout.
        let start = Instant::now();
        let end = start + Duration::from_secs(8);
        let operation_end = start + Duration::from_secs(4);
        let deadline =
            Deadline::new(start, Duration::from_secs(8), Duration::from_secs(4)).unwrap();
        let data = Arc::new(FixtureData::load(&executable).unwrap());
        assert_eq!(
            data.profile.runtime().sha256(),
            std::env::var("VAD_CONTROL_PROCESS_FIXTURE_SHA256")
                .expect("Actual fixture SHA required")
        );
        Self {
            kind,
            root,
            data,
            deadline,
            operation_end,
            end,
            owner: None,
            launch_failure: None,
            native: None,
            _control: None,
            _payload: None,
            guards: Vec::new(),
            terminal: None,
            exported: false,
            close_error: None,
            cleanup_panic: None,
        }
    }
    fn environment(&self, mode: &str) -> BTreeMap<String, OsString> {
        let mut env = BTreeMap::new();
        if let Some(system) = std::env::var_os("SystemRoot") {
            env.insert("SystemRoot".into(), system);
        }
        env.insert("PLAYER_VAD_TEST_FIXTURE_MODE".into(), mode.into());
        env
    }
    fn spawn(&mut self, mode: &str, startup: bool, flush: bool, mismatch: bool) {
        let payload = FixturePayload::new(self.data.clone());
        self.spawn_prepared(mode, startup, flush, mismatch, &payload, &[]);
    }
    fn spawn_prepared(
        &mut self,
        mode: &str,
        startup: bool,
        flush: bool,
        mismatch: bool,
        payload: &dyn WorkerPayloadPreparation,
        suffix: &[OsString],
    ) {
        let prepared = PreparedEvidence::create(
            &self.root,
            Some(&self.data.request),
            kinekino_vad::protocol::application::MAX_DESCRIPTOR_BYTES as usize,
        )
        .unwrap();
        let original = Original {
            end: self.operation_end,
        };
        self._payload = Some(
            payload
                .stage(&self.data.request, &self.root, &original)
                .unwrap(),
        );
        let control =
            worker_control::prepare(&self.root, self.deadline, &original, &self.data.request)
                .unwrap();
        let mut arguments: Vec<OsString> = vec![
            "--vad-request-v1".into(),
            self.root.join("stdin.raw").into_os_string(),
            "--cancel-file".into(),
            self.root.join("cancel.signal").into_os_string(),
            "--release-manifest-id".into(),
            self.data.manifest_pin.id().into(),
            "--release-manifest-sha256".into(),
            self.data.manifest_pin.sha256().into(),
            "--release-manifest-bytes".into(),
            self.data.manifest_pin.bytes().to_string().into(),
        ];
        let mut control_args = control.arguments();
        if mismatch {
            control_args[3] = "0".repeat(64).into();
        }
        arguments.extend(control_args);
        arguments.extend_from_slice(suffix);
        self._control = Some(control);
        let mut environment = self.environment(mode);
        if startup {
            environment.insert("PLAYER_VAD_TEST_STARTUP_BARRIER".into(), "1".into());
        }
        if flush {
            environment.insert("PLAYER_VAD_TEST_OUTPUT".into(), "fail-flush".into());
        }
        let spec = SpawnSpec {
            executable: self.data.executable.clone(),
            cwd: self.root.clone(),
            environment: Some(environment),
            arguments,
        };
        match spawn(spec, prepared, self.deadline) {
            Ok((process, paths)) => {
                self.owner = Some(Supervisor::new(
                    process,
                    paths,
                    kinekino_vad::protocol::application::MAX_OUTCOME_BYTES + 1,
                ));
                JsonReceipt(self.root.join("spawn.json"))
                    .record(self.owner.as_ref().unwrap().receipt())
                    .unwrap();
            }
            Err(mut failure) => {
                if let Some(process) = failure.owner.take() {
                    self.owner = Some(Supervisor::new(
                        *process,
                        failure.evidence.clone(),
                        kinekino_vad::protocol::application::MAX_OUTCOME_BYTES + 1,
                    ));
                }
                let diagnostic = failure.error.to_string();
                self.launch_failure = Some(failure);
                panic!("Original launch failed and retained: {diagnostic}");
            }
        }
    }
}
pub(super) fn exercise(kind: CaseKind) {
    let mut case = Box::new(Case::new(kind));
    let result = catch_unwind(AssertUnwindSafe(|| scenarios::run(&mut case)));
    let closed = catch_unwind(AssertUnwindSafe(|| {
        let closed = case.close();
        case.export();
        closed
    }));
    match (result, closed) {
        (Err(original), closed) => {
            if let Err(secondary) = closed {
                case.cleanup_panic = Some(secondary);
            }
            let _retained = Box::leak(case);
            resume_unwind(original);
        }
        (Ok(()), Err(close)) => {
            let _retained = Box::leak(case);
            resume_unwind(close);
        }
        (Ok(()), Ok(false)) => {
            let root = case.root.clone();
            let _retained = Box::leak(case);
            panic!("Original owner incomplete and retained: {}", root.display());
        }
        (Ok(()), Ok(true)) => {
            if let Some(receipt) = &case.terminal {
                assert_eq!(receipt.family, FamilyObservation::Empty);
                assert!(receipt.stdin_eof_requested);
                assert!(!receipt.forced);
            }
        }
    }
}
