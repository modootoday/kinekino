mod capture;
mod control;
mod deferred_engine;
mod diagnostics;
mod events;
#[expect(
    dead_code,
    reason = "Guarded protocol owner and application issuer integration is assigned separately to C"
)]
mod guarded_command;
mod host;
mod ipc;
mod launch;
mod message_reader;
mod native_observation;
mod network_plan;
mod option_projection;
mod options_state;
mod owned_launch;
mod owned_startup;
mod review_worker;
pub use review_worker::{
    ReviewPlaybackWorker, ReviewWorkerControl, ReviewWorkerLimits, ReviewWorkerSnapshot,
    ReviewWorkerStatusReader,
};
mod parent_binding;
#[cfg(feature = "native-proposal-window")]
mod parent_surface;
#[cfg(feature = "native-proposal-window")]
pub(crate) use parent_surface::{
    CheckedMediaParent, ParentMediaFailure, ParentMediaSurface, ParentMediaSurfaceError,
};
#[cfg(feature = "native-proposal-window")]
pub(crate) use parent_surface::{ChildInputError, DisabledChildProof};
#[cfg(feature = "native-proposal-window")]
pub use parent_surface::{
    HiddenSurfaceProbe, NativeSurfaceInput, ProbeError, ProbeFailure, ProbeObservation,
    ProbeRelease, SurfaceInputError, SurfaceInputFailure,
};
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Trusted assembly connection deferred to the assigned NativeChild stage; remove at integration"
    )
)]
mod parent_router;
pub use parent_router::ParentEngineRouter;
#[cfg(feature = "native-proposal-window")]
pub(crate) use parent_router::{
    ParentRouteHandle, RetirementFailure, RetirementHandle, RetirementProgress,
};
mod pointer_policy;
mod process_output;
mod session_engine;
mod shutdown_observation;
mod track_projection;
mod trusted_executable;
use crate::application::native_playback::NativePlayback;
use anyhow::{Context, Result};
pub use control::MpvControlHandle;
pub use deferred_engine::DeferredEpisodeEngine;
pub use host::NativePlaybackHost;
pub use parent_binding::{
    ParentBindFailure, ParentBinding, ParentCleanupObservation, ParentCleanupStatus,
    ParentCleanupTask, ParentOwnedEngine,
};
use serde_json::Value;
pub use session_engine::NativeEpisodeEngine;
use std::{
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
};
pub use trusted_executable::TrustedMpvExecutable;

pub struct Mpv {
    child: Child,
    control: MpvControlHandle,
    updates: ipc::LatestPlayback,
    snapshot: NativePlayback,
    ipc_worker: Option<std::thread::JoinHandle<()>>,
    stderr_worker: Option<std::thread::JoinHandle<()>>,
    stderr: process_output::LatestOutput,
    stdout: process_output::LatestOutput,
    stdout_worker: Option<std::thread::JoinHandle<()>>,
    pipe: String,
}

impl Mpv {
    pub fn launch(
        source: &crate::domain::media::MediaSource,
        parent: u32,
        start: f64,
        volume: f64,
    ) -> Result<Self> {
        anyhow::ensure!(
            source.hls || crate::domain::navigation::media_candidate(&source.url),
            "No transferable media URL detected"
        );
        Self::spawn(
            std::ffi::OsStr::new(&source.url),
            Some(&source.referer),
            source.hls,
            parent,
            start,
            volume,
        )
    }

    pub fn launch_local(
        source: &crate::domain::local_video::LocalVideo,
        parent: u32,
        volume: f64,
    ) -> Result<Self> {
        anyhow::ensure!(source.path().is_file(), "Local video no longer exists");
        Self::spawn(source.path().as_os_str(), None, false, parent, 0., volume)
    }

    fn spawn(
        source: &std::ffi::OsStr,
        referer: Option<&str>,
        hls: bool,
        parent: u32,
        start: f64,
        volume: f64,
    ) -> Result<Self> {
        let pipe = ipc_address("kinekino-mpv")?;
        let executable = std::env::current_exe()?
            .parent()
            .context("Missing application directory")?
            .join("mpv.exe");
        anyhow::ensure!(
            executable.is_file(),
            "Place a trusted mpv.exe beside kinekino-player.exe before using native mode"
        );
        let mut command = Command::new(executable);
        command.args([
            "--no-config",
            "--idle=yes",
            "--force-window=yes",
            "--hwdec=auto-safe",
            "--osc=no",
            "--mute=yes",
            "--input-default-bindings=no",
            "--input-vo-keyboard=no",
        ]);
        command.args(pointer_policy::ARGUMENTS);
        command.arg(if referer.is_some() {
            "--demuxer-lavf-o=protocol_whitelist=%20%https,tls,tcp,crypto"
        } else {
            "--demuxer-lavf-o=protocol_whitelist=file"
        });
        if hls {
            command.args([
                "--demuxer=lavf",
                "--demuxer-lavf-format=hls",
                "--demuxer-lavf-o-append=allowed_extensions=ALL",
                "--demuxer-lavf-o-append=extension_picky=0",
            ]);
        }
        command
            .arg(format!("--wid={parent}"))
            .arg(format!("--start={}", start.max(0.)))
            .arg(format!("--volume={}", volume.clamp(0., 100.)));
        if let Some(referer) = referer {
            command.arg(format!("--referrer={referer}"));
        }
        command
            .arg(format!("--input-ipc-server={pipe}"))
            .arg("--")
            .arg(source);
        Self::spawn_process(command, pipe)
    }

    fn spawn_process(mut command: Command, pipe: String) -> Result<Self> {
        let (commands, receiver) = tokio::sync::mpsc::channel(control::COMMAND_CAPACITY);
        let control = MpvControlHandle::new(commands)?;
        crate::infrastructure::os_policy::hide_console_window(&mut command);
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Could not start mpv")?;
        let updates = Arc::new(Mutex::new(NativePlayback::default()));
        let stderr = Arc::new(Mutex::new(process_output::OutputSnapshot::default()));
        let stdout = Arc::new(Mutex::new(process_output::OutputSnapshot::default()));
        let Some(reader) = child.stderr.take() else {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("Missing native stderr");
        };
        let stderr_worker =
            match process_output::collect(reader, stderr.clone(), "player-mpv-stderr") {
                Ok(worker) => worker,
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(error.into());
                }
            };
        let Some(reader) = child.stdout.take() else {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stderr_worker.join();
            anyhow::bail!("Missing native stdout");
        };
        let stdout_worker =
            match process_output::collect(reader, stdout.clone(), "player-mpv-stdout") {
                Ok(worker) => worker,
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stderr_worker.join();
                    return Err(error.into());
                }
            };
        let ipc_worker = match ipc::connect(
            pipe.clone(),
            receiver,
            updates.clone(),
            control.state.clone(),
        ) {
            Ok(worker) => worker,
            Err(error) => {
                control.state.close();
                let _ = child.kill();
                let _ = child.wait();
                let _ = stderr_worker.join();
                let _ = stdout_worker.join();
                return Err(error.into());
            }
        };
        Ok(Self {
            child,
            control,
            updates,
            snapshot: NativePlayback::default(),
            ipc_worker: Some(ipc_worker),
            stderr_worker: Some(stderr_worker),
            stderr,
            stdout,
            stdout_worker: Some(stdout_worker),
            pipe,
        })
    }

    pub fn send(&self, command: Value) -> Result<()> {
        self.control.send_legacy(command)
    }
    pub fn control_handle(&self) -> MpvControlHandle {
        self.control.clone()
    }

    /// Stops and waits only for the process and IPC worker created by this instance.
    pub fn shutdown(&mut self) -> Result<()> {
        self.control.state.close();
        if self.child.try_wait()?.is_none() {
            self.child.kill()?;
        }
        self.child.wait()?;
        if let Some(worker) = self.ipc_worker.take() {
            worker
                .join()
                .map_err(|_| anyhow::anyhow!("mpv IPC worker failed during shutdown"))?;
        }
        if let Some(worker) = self.stderr_worker.take() {
            worker
                .join()
                .map_err(|_| anyhow::anyhow!("mpv stderr worker failed during shutdown"))?;
        }
        if let Some(worker) = self.stdout_worker.take() {
            worker
                .join()
                .map_err(|_| anyhow::anyhow!("mpv stdout worker failed during shutdown"))?;
        }
        Ok(())
    }

    pub fn snapshot(&mut self) -> &NativePlayback {
        match self.updates.lock() {
            Ok(update) => self.snapshot = update.clone(),
            Err(_) => {
                self.snapshot.connected = false;
                self.snapshot.error = Some("Native observation unavailable".into());
            }
        }
        if let Ok(Some(status)) = self.child.try_wait() {
            self.control.state.close();
            self.snapshot.connected = false;
            self.snapshot.error = Some(format!("mpv exited: {status}"));
        }
        &self.snapshot
    }
}

impl Drop for Mpv {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Fresh private mpv `--input-ipc-server` address from the OS IPC port (ADR 0007): a named
/// pipe on Windows, a socket in the owner-only runtime dir on Unix.
pub(super) fn ipc_address(product: &str) -> std::io::Result<String> {
    let ipc = crate::infrastructure::os_policy::ipc();
    ipc.address(&ipc.fresh_endpoint(product)?)
}
