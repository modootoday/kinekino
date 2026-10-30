use super::*;
use crate::{
    application::catalog::{CaptureMetadata, CatalogArtifacts, RawArtifact},
    domain::request_referer::RequestReferer,
    infrastructure::mpv::{TrustedMpvExecutable, owned_startup},
};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Default)]
struct Store {
    calls: AtomicUsize,
    fail_on: AtomicUsize,
}
impl CatalogArtifacts for Store {
    fn save(&self, raw: &[u8], _: &CaptureMetadata) -> Result<RawArtifact, ApiError> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        if self.fail_on.load(Ordering::SeqCst) == call {
            return Err(ApiError::new(
                ErrorCode::Internal,
                "Injected diagnostic publication failure",
            ));
        }
        let sha = format!("{:x}", Sha256::digest(raw));
        Ok(RawArtifact {
            artifact_id: sha.clone(),
            body_sha256: sha,
            body_bytes: DecimalU64(raw.len() as u64),
            body_path: "synthetic/body".into(),
            metadata_path: "synthetic/metadata".into(),
        })
    }
}

const PROBE: &str = "PLAYER_REPLACEMENT_TEST_PROBE";
#[test]
fn bounded_owned_child_probe() {
    if std::env::var(PROBE).ok().as_deref() == Some("child") {
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
fn command() -> Command {
    let full = format!("{}::bounded_owned_child_probe", module_path!());
    let (_, name) = full.split_once("::").unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact", name]).env(PROBE, "child");
    command
}
fn child() -> Mpv {
    owned_startup::spawn_owned(command(), "unused-replacement-probe".into())
        .unwrap_or_else(|_| panic!("Owned Rust probe must launch"))
}

struct Fixture {
    owner: NativeEpisodeEngine,
    store: Arc<Store>,
    root: PathBuf,
    old_id: DecimalU64,
    old_pid: u32,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "player-replacement-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("mpv.exe");
        let bytes =
            b"Synthetic pinned executable; the internal probe adapter owns real Rust children";
        std::fs::write(&path, bytes).unwrap();
        let trusted =
            TrustedMpvExecutable::open(&path, &format!("{:x}", Sha256::digest(bytes))).unwrap();
        let store = Arc::new(Store::default());
        let owner = NativeEpisodeEngine::new(trusted, None, store.clone()).unwrap();
        let source = EpisodePlaybackSource::synthetic_replacement_source(RequestReferer::Absent);
        let old = Active::from_mpv(child(), &source, store.clone());
        let old_id = DecimalU64(old.id());
        let old_pid = old.mpv.child.id();
        *owner.active.lock().unwrap() = Some(old);
        Self {
            owner,
            store,
            root,
            old_id,
            old_pid,
        }
    }
    fn replace(
        &self,
        runtime: &mut ProbeRuntime,
        referer: RequestReferer,
    ) -> Result<ReplacedEpisode, ApiError> {
        self.owner.replace_using(
            ReplacementInput {
                expected_instance: self.old_id,
                expected_artifact: &"b".repeat(64),
                source: EpisodePlaybackSource::synthetic_replacement_source(referer),
                paused: true,
                volume_percent: 0.0,
            },
            runtime,
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.store.fail_on.store(0, Ordering::SeqCst);
        self.owner
            .shutdown()
            .expect("Every retained probe must be explicitly reaped");
        // Trusted executable holds file guards until the owner itself is dropped.
        let _ = &self.root;
    }
}

enum Launch {
    BeforeSpawn,
    WorkerFailure { cleanup_failure: bool },
    EndHost,
}
struct ProbeRuntime {
    mode: Launch,
    calls: usize,
    new_pid: Option<u32>,
    close_calls: usize,
    fail_close_on: Option<usize>,
}
impl ProbeRuntime {
    fn new(mode: Launch) -> Self {
        Self {
            mode,
            calls: 0,
            new_pid: None,
            close_calls: 0,
            fail_close_on: None,
        }
    }
}
impl ReplacementRuntime for ProbeRuntime {
    fn close(
        &mut self,
        active: &mut Active,
    ) -> Result<crate::application::playback_session::ClosedEpisodePlayback, ApiError> {
        self.close_calls += 1;
        if self.fail_close_on == Some(self.close_calls) {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Injected unconfirmed process shutdown",
            ));
        }
        active.close()
    }
    fn launch(
        &mut self,
        owner: &NativeEpisodeEngine,
        _: &EpisodePlaybackSource,
        _: bool,
        _: f64,
    ) -> Result<Mpv, Box<owned_startup::OwnedStartupFailure>> {
        self.calls += 1;
        match self.mode {
            Launch::BeforeSpawn => Err(owned_startup::OwnedStartupFailure::before_spawn(
                ApiError::new(ErrorCode::EngineFailed, "Injected launch failure"),
            )),
            Launch::WorkerFailure { cleanup_failure } => Err(owned_startup::spawn_failure_probe(
                command(),
                cleanup_failure,
            )),
            Launch::EndHost => {
                let mpv = child();
                self.new_pid = Some(mpv.child.id());
                owner.closed.store(true, Ordering::Release);
                Ok(mpv)
            }
        }
    }
}

#[test]
fn unsupported_preflight_preserves_old_pid_instance_and_does_not_launch() {
    let fixture = Fixture::new();
    let mut runtime = ProbeRuntime::new(Launch::BeforeSpawn);
    assert_eq!(
        fixture
            .replace(&mut runtime, RequestReferer::Empty)
            .err()
            .expect("Injected replacement must fail")
            .code,
        ErrorCode::Unsupported
    );
    assert_eq!(runtime.calls, 0);
    let mut active = fixture.owner.active.lock().unwrap();
    let old = active.as_mut().unwrap();
    assert_eq!(old.id(), fixture.old_id.0);
    assert_eq!(old.mpv.child.id(), fixture.old_pid);
    assert!(old.mpv.child.try_wait().unwrap().is_none());
}

#[test]
fn old_close_diagnostic_failure_retains_same_owner_and_prevents_launch() {
    let fixture = Fixture::new();
    fixture.store.fail_on.store(1, Ordering::SeqCst);
    let mut runtime = ProbeRuntime::new(Launch::BeforeSpawn);
    assert_eq!(
        fixture
            .replace(&mut runtime, RequestReferer::Absent)
            .err()
            .expect("Injected replacement must fail")
            .code,
        ErrorCode::Internal
    );
    assert_eq!(runtime.calls, 0);
    let mut active = fixture.owner.active.lock().unwrap();
    let old = active.as_mut().unwrap();
    assert_eq!(old.id(), fixture.old_id.0);
    assert!(old.mpv.child.try_wait().unwrap().is_some());
    drop(active);
    assert!(fixture.owner.shutdown().unwrap().is_some());
}

#[test]
fn old_process_close_failure_retains_live_child_for_explicit_shutdown() {
    let fixture = Fixture::new();
    let mut runtime = ProbeRuntime::new(Launch::BeforeSpawn);
    runtime.fail_close_on = Some(1);
    let error = fixture
        .replace(&mut runtime, RequestReferer::Absent)
        .err()
        .expect("Injected close must fail");
    assert_eq!(error.code, ErrorCode::EngineFailed);
    assert_eq!(runtime.calls, 0);
    let mut active = fixture.owner.active.lock().unwrap();
    let old = active.as_mut().unwrap();
    assert_eq!(old.id(), fixture.old_id.0);
    assert!(old.mpv.child.try_wait().unwrap().is_none());
    drop(active);
    assert_eq!(
        fixture
            .owner
            .shutdown()
            .unwrap()
            .unwrap()
            .engine_instance_id,
        fixture.old_id
    );
}

#[test]
fn eof_unconfirmed_new_process_close_retains_live_child_and_original_error() {
    let fixture = Fixture::new();
    let mut runtime = ProbeRuntime::new(Launch::EndHost);
    runtime.fail_close_on = Some(2);
    let error = fixture
        .replace(&mut runtime, RequestReferer::Absent)
        .err()
        .expect("EOF cleanup must fail");
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(error.details["new_engine_owned"], true);
    assert_eq!(
        error.details["new_cleanup_error"]["code"],
        serde_json::json!(ErrorCode::EngineFailed)
    );
    let mut active = fixture.owner.active.lock().unwrap();
    let new = active.as_mut().unwrap();
    let new_id = DecimalU64(new.id());
    assert_ne!(new_id, fixture.old_id);
    assert_eq!(Some(new.mpv.child.id()), runtime.new_pid);
    assert!(new.mpv.child.try_wait().unwrap().is_none());
    drop(active);
    let receipt = fixture.owner.shutdown().unwrap().unwrap();
    assert_eq!(receipt.engine_instance_id, new_id);
    assert!(
        receipt.engine_exited
            && receipt.ipc_worker_exited
            && receipt.stderr_worker_exited
            && receipt.stdout_worker_exited
    );
    assert!(fixture.owner.active.lock().unwrap().is_none());
}

#[test]
fn launch_failure_after_old_close_does_not_restore_or_invent_a_new_owner() {
    let fixture = Fixture::new();
    let mut runtime = ProbeRuntime::new(Launch::BeforeSpawn);
    let error = fixture
        .replace(&mut runtime, RequestReferer::Absent)
        .err()
        .expect("Injected replacement must fail");
    assert_eq!(error.details["previous"]["engine_exited"], true);
    assert_eq!(error.details["new_process_created"], false);
    assert_eq!(error.details["new_engine_owned"], false);
    assert!(fixture.owner.active.lock().unwrap().is_none());
    assert_eq!(runtime.calls, 1);
}

#[test]
fn startup_cleanup_failure_reaches_native_owner_and_explicit_shutdown_recovers() {
    let fixture = Fixture::new();
    let mut runtime = ProbeRuntime::new(Launch::WorkerFailure {
        cleanup_failure: true,
    });
    let error = fixture
        .replace(&mut runtime, RequestReferer::Absent)
        .err()
        .expect("Injected replacement must fail");
    assert_eq!(error.details["new_cleanup_confirmed"], false);
    assert_eq!(error.details["new_engine_owned"], true);
    let id = fixture.owner.active.lock().unwrap().as_ref().unwrap().id();
    assert_ne!(id, fixture.old_id.0);
    let receipt = fixture.owner.shutdown().unwrap().unwrap();
    assert_eq!(receipt.engine_instance_id.0, id);
    assert!(
        receipt.engine_exited
            && receipt.ipc_worker_exited
            && receipt.stderr_worker_exited
            && receipt.stdout_worker_exited
    );
    assert!(!receipt.diagnostics.complete);
    assert!(fixture.owner.active.lock().unwrap().is_none());
}

#[test]
fn startup_diagnostic_failure_retains_stopped_new_owner_until_publication_retry() {
    let fixture = Fixture::new();
    fixture.store.fail_on.store(2, Ordering::SeqCst);
    let mut runtime = ProbeRuntime::new(Launch::WorkerFailure {
        cleanup_failure: false,
    });
    let error = fixture
        .replace(&mut runtime, RequestReferer::Absent)
        .err()
        .expect("Injected replacement must fail");
    assert_eq!(error.details["new_cleanup_confirmed"], true);
    assert_eq!(
        error.details["startup"]["diagnostic_capture_confirmed"],
        false
    );
    assert_eq!(error.details["new_engine_owned"], true);
    let mut active = fixture.owner.active.lock().unwrap();
    assert!(
        active
            .as_mut()
            .unwrap()
            .mpv
            .child
            .try_wait()
            .unwrap()
            .is_some()
    );
    drop(active);
    assert!(fixture.owner.shutdown().unwrap().is_some());
}

#[test]
fn eof_during_launch_reaps_new_child_and_permanently_rejects_later_replacement() {
    let fixture = Fixture::new();
    let mut runtime = ProbeRuntime::new(Launch::EndHost);
    let error = fixture
        .replace(&mut runtime, RequestReferer::Absent)
        .err()
        .expect("Injected replacement must fail");
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(error.details["cancelled_new"]["engine_exited"], true);
    assert_eq!(error.details["new_engine_owned"], false);
    assert!(fixture.owner.active.lock().unwrap().is_none());
    assert_eq!(
        fixture
            .replace(&mut runtime, RequestReferer::Absent)
            .err()
            .expect("Injected replacement must fail")
            .code,
        ErrorCode::Cancelled
    );
    assert_eq!(runtime.calls, 1);
}

#[test]
fn eof_new_cleanup_publication_failure_retains_new_owner_for_host_retry() {
    let fixture = Fixture::new();
    fixture.store.fail_on.store(2, Ordering::SeqCst);
    let mut runtime = ProbeRuntime::new(Launch::EndHost);
    let error = fixture
        .replace(&mut runtime, RequestReferer::Absent)
        .err()
        .expect("Injected replacement must fail");
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(error.details["new_engine_owned"], true);
    assert_eq!(
        error.details["new_cleanup_error"]["code"],
        serde_json::json!(ErrorCode::Internal)
    );
    assert_eq!(
        error.details["new_cleanup_error"]["message"],
        "Injected diagnostic publication failure"
    );
    let mut active = fixture.owner.active.lock().unwrap();
    let new = active.as_mut().unwrap();
    assert_ne!(new.id(), fixture.old_id.0);
    assert_eq!(Some(new.mpv.child.id()), runtime.new_pid);
    assert!(new.mpv.child.try_wait().unwrap().is_some());
    drop(active);
    let receipt = fixture.owner.shutdown().unwrap().unwrap();
    assert_eq!(
        receipt.engine_instance_id,
        serde_json::from_value::<DecimalU64>(error.details["new_engine_instance_id"].clone())
            .unwrap()
    );
    assert!(
        receipt.engine_exited
            && receipt.ipc_worker_exited
            && receipt.stderr_worker_exited
            && receipt.stdout_worker_exited
    );
    assert!(fixture.owner.active.lock().unwrap().is_none());
}
