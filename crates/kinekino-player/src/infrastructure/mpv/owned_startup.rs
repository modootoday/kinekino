//! Owned startup for trusted fixed launch arguments.
//! Caller must retain recovery in the engine owner before returning its public error.
use super::{Mpv, MpvControlHandle, control, ipc, process_output};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        native_playback::NativePlayback,
    },
    domain::api_values::DecimalU64,
};
use std::{
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum Stage {
    Control,
    Child,
    StderrPipe,
    StderrWorker,
    StdoutPipe,
    StdoutWorker,
    IpcWorker,
}

#[derive(Default)]
struct FaultPlan {
    #[cfg(test)]
    stage: Option<Stage>,
    #[cfg(test)]
    cleanup_failure: bool,
}
impl FaultPlan {
    fn check(&self, stage: Stage) -> Result<(), Stage> {
        #[cfg(test)]
        if self.stage == Some(stage) {
            return Err(stage);
        }
        let _ = (self, stage);
        Ok(())
    }
    fn cleanup(&self, engine: &mut Mpv) -> bool {
        #[cfg(test)]
        if self.cleanup_failure {
            engine.control.state.close();
            return false;
        }
        let _ = self;
        engine.shutdown().is_ok()
    }
}

#[must_use = "Retain the recovery object in the native owner; do not discard an unconfirmed child"]
pub(super) struct OwnedStartupFailure {
    error: ApiError,
    evidence: Option<Box<Mpv>>,
}
impl OwnedStartupFailure {
    pub(super) fn before_spawn(mut error: ApiError) -> Box<Self> {
        error.details = serde_json::json!({"stage":"preflight","process_created":false,"engine_instance_id":null,"startup_completed":false,"cleanup_confirmed":true,"recovery_required":false});
        Box::new(Self {
            error,
            evidence: None,
        })
    }
    pub(super) fn into_parts(self) -> (ApiError, Option<Box<Mpv>>) {
        (self.error, self.evidence)
    }
}

#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "Preserve disabled standalone startup adapter and failure probes"
    )
)]
pub(super) fn spawn_owned(command: Command, pipe: String) -> Result<Mpv, Box<OwnedStartupFailure>> {
    spawn_with_plan(command, pipe, FaultPlan::default())
}
pub(super) fn spawn_owned_scoped(
    command: Command,
    pipe: String,
    scope: Arc<super::guarded_command::HostScope>,
) -> Result<Mpv, Box<OwnedStartupFailure>> {
    spawn_scoped_with_plan(command, pipe, FaultPlan::default(), Some(scope))
}
#[cfg(test)]
pub(super) fn spawn_failure_probe(
    command: Command,
    cleanup_failure: bool,
) -> Box<OwnedStartupFailure> {
    spawn_with_plan(
        command,
        "unused-failure-probe".into(),
        FaultPlan {
            stage: Some(Stage::StdoutWorker),
            cleanup_failure,
        },
    )
    .err()
    .expect("The injected worker failure must reject startup")
}
fn spawn_with_plan(
    command: Command,
    pipe: String,
    faults: FaultPlan,
) -> Result<Mpv, Box<OwnedStartupFailure>> {
    spawn_scoped_with_plan(command, pipe, faults, None)
}
fn spawn_scoped_with_plan(
    mut command: Command,
    pipe: String,
    faults: FaultPlan,
    scope: Option<Arc<super::guarded_command::HostScope>>,
) -> Result<Mpv, Box<OwnedStartupFailure>> {
    faults
        .check(Stage::Control)
        .map_err(|stage| failure(stage, None, &faults))?;
    let (commands, receiver) = tokio::sync::mpsc::channel(control::COMMAND_CAPACITY);
    let control = match scope {
        Some(scope) => MpvControlHandle::new_scoped(commands, scope),
        None => MpvControlHandle::new(commands),
    }
    .map_err(|_| failure(Stage::Control, None, &faults))?;
    faults.check(Stage::Child).map_err(|stage| {
        control.state.close();
        failure(stage, None, &faults)
    })?;
    crate::infrastructure::os_policy::hide_console_window(&mut command);
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| {
            control.state.close();
            failure(Stage::Child, None, &faults)
        })?;
    // Own the child immediately, before any fallible reader/thread construction.
    let mut owned = Mpv {
        child,
        control,
        updates: Arc::new(Mutex::new(NativePlayback::default())),
        snapshot: NativePlayback::default(),
        ipc_worker: None,
        stderr_worker: None,
        stderr: Arc::new(Mutex::new(process_output::OutputSnapshot::default())),
        stdout: Arc::new(Mutex::new(process_output::OutputSnapshot::default())),
        stdout_worker: None,
        pipe,
    };
    let configured = (|| -> Result<(), Stage> {
        faults.check(Stage::StderrPipe)?;
        let stderr = owned.child.stderr.take().ok_or(Stage::StderrPipe)?;
        faults.check(Stage::StderrWorker)?;
        owned.stderr_worker = Some(
            process_output::collect(stderr, owned.stderr.clone(), "player-mpv-stderr")
                .map_err(|_| Stage::StderrWorker)?,
        );
        faults.check(Stage::StdoutPipe)?;
        let stdout = owned.child.stdout.take().ok_or(Stage::StdoutPipe)?;
        faults.check(Stage::StdoutWorker)?;
        owned.stdout_worker = Some(
            process_output::collect(stdout, owned.stdout.clone(), "player-mpv-stdout")
                .map_err(|_| Stage::StdoutWorker)?,
        );
        faults.check(Stage::IpcWorker)?;
        owned.ipc_worker = Some(
            ipc::connect(
                owned.pipe.clone(),
                receiver,
                owned.updates.clone(),
                owned.control.state.clone(),
            )
            .map_err(|_| Stage::IpcWorker)?,
        );
        Ok(())
    })();
    match configured {
        Ok(()) => Ok(owned),
        Err(stage) => Err(failure(stage, Some(owned), &faults)),
    }
}

fn failure(stage: Stage, mut owned: Option<Mpv>, faults: &FaultPlan) -> Box<OwnedStartupFailure> {
    let created = owned.is_some();
    let instance = owned
        .as_ref()
        .map(|engine| DecimalU64(engine.control_handle().instance_id()));
    if let Some(engine) = owned.as_mut() {
        engine.snapshot.error = Some("Native startup configuration failed".into());
        if let Ok(mut latest) = engine.updates.lock() {
            latest.error = engine.snapshot.error.clone();
        }
    }
    let cleanup_confirmed = owned.as_mut().is_none_or(|engine| faults.cleanup(engine));
    // Even a successfully stopped process can contain observed failure output to persist.
    let evidence = owned.take().map(Box::new);
    let mut error = ApiError::new(ErrorCode::EngineFailed, "Owned native startup failed");
    error.details = serde_json::json!({
        "stage":stage,"engine_instance_id":instance,"process_created":created,
        "startup_completed":false,"cleanup_confirmed":cleanup_confirmed,
        "recovery_required":created && !cleanup_confirmed,
        "output_evidence_retained":evidence.is_some(),
    });
    Box::new(OwnedStartupFailure { error, evidence })
}

#[cfg(test)]
mod tests {
    use super::*;
    const PROBE: &str = "PLAYER_OWNED_STARTUP_TEST_CHILD";
    fn child_probe() -> bool {
        if std::env::var(PROBE).ok().as_deref() == Some("probe") {
            // Bounded fallback if the parent fails before it can kill/reap this owned child.
            std::thread::sleep(std::time::Duration::from_secs(2));
            true
        } else {
            false
        }
    }
    fn command_for(test: &str) -> Command {
        let full = format!("{}::{test}", module_path!());
        let (_, filter) = full.split_once("::").unwrap();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", filter, "--nocapture"])
            .env(PROBE, "probe");
        command
    }
    #[test]
    fn worker_failure_confirms_owned_child_and_existing_reader_cleanup() {
        if child_probe() {
            return;
        }
        let failure = spawn_with_plan(
            command_for("worker_failure_confirms_owned_child_and_existing_reader_cleanup"),
            "unused-test-pipe".into(),
            FaultPlan {
                stage: Some(Stage::StdoutWorker),
                cleanup_failure: false,
            },
        )
        .err()
        .expect("Injected worker failure must reject startup");
        let (error, recovery) = (*failure).into_parts();
        assert_eq!(error.details["stage"], "stdout_worker");
        assert_eq!(error.details["process_created"], true);
        assert_eq!(error.details["cleanup_confirmed"], true);
        assert_eq!(error.details["recovery_required"], false);
        let mut stopped =
            recovery.expect("Observed failure output must reach the diagnostics owner");
        assert!(stopped.child.try_wait().unwrap().is_some());
        assert!(!stopped.stdout.lock().unwrap().collector_started);
    }
    #[test]
    fn unconfirmed_cleanup_retains_owned_child_for_explicit_retry() {
        if child_probe() {
            return;
        }
        let failure = spawn_with_plan(
            command_for("unconfirmed_cleanup_retains_owned_child_for_explicit_retry"),
            "unused-test-pipe".into(),
            FaultPlan {
                stage: Some(Stage::StdoutWorker),
                cleanup_failure: true,
            },
        )
        .err()
        .expect("Injected worker failure must reject startup");
        let (error, recovery) = (*failure).into_parts();
        assert_eq!(error.details["cleanup_confirmed"], false);
        assert_eq!(error.details["recovery_required"], true);
        let mut engine = recovery.expect("Unconfirmed process ownership must not be discarded");
        assert!(engine.stderr_worker.is_some());
        assert!(engine.snapshot.error.is_some());
        engine.shutdown().unwrap();
        assert!(engine.child.try_wait().unwrap().is_some());
        let missing_stdout = engine.stdout.lock().unwrap();
        assert!(!missing_stdout.collector_started && !missing_stdout.eof_observed);
        drop(missing_stdout);
        assert!(
            engine.stderr_worker.is_none()
                && engine.stdout_worker.is_none()
                && engine.ipc_worker.is_none()
        );
    }
}
