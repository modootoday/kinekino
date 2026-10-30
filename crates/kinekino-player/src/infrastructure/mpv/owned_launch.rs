//! Fixed trusted launch arguments with no public raw command input.
use super::{
    Mpv, TrustedMpvExecutable, network_plan,
    owned_startup::{self, OwnedStartupFailure},
};
use crate::application::{
    api::{ApiError, ErrorCode},
    playback_session::EpisodePlaybackSource,
};
use std::process::Command;

impl Mpv {
    #[allow(
        dead_code,
        reason = "Preserve disabled standalone launch adapter compatibility"
    )]
    pub(super) fn launch_episode_owned(
        executable: &TrustedMpvExecutable,
        source: &EpisodePlaybackSource,
        parent: Option<u32>,
        paused: bool,
        volume_percent: f64,
    ) -> Result<Self, Box<OwnedStartupFailure>> {
        Self::launch_episode_with_scope(executable, source, parent, paused, volume_percent, None)
    }
    pub(super) fn launch_episode_owned_scoped(
        executable: &TrustedMpvExecutable,
        source: &EpisodePlaybackSource,
        parent: Option<u32>,
        paused: bool,
        volume_percent: f64,
        scope: std::sync::Arc<super::guarded_command::HostScope>,
    ) -> Result<Self, Box<OwnedStartupFailure>> {
        Self::launch_episode_with_scope(
            executable,
            source,
            parent,
            paused,
            volume_percent,
            Some(scope),
        )
    }
    fn launch_episode_with_scope(
        executable: &TrustedMpvExecutable,
        source: &EpisodePlaybackSource,
        parent: Option<u32>,
        paused: bool,
        volume_percent: f64,
        scope: Option<std::sync::Arc<super::guarded_command::HostScope>>,
    ) -> Result<Self, Box<OwnedStartupFailure>> {
        let arguments = network_plan::arguments(source, paused, volume_percent)
            .map_err(OwnedStartupFailure::before_spawn)?;
        let pipe = super::ipc_address("kinekino-mpv").map_err(|_| {
            OwnedStartupFailure::before_spawn(ApiError::new(
                ErrorCode::EngineFailed,
                "Native IPC address unavailable",
            ))
        })?;
        let mut command = Command::new(executable.path());
        command.args(arguments);
        if let Some(parent) = parent {
            if parent == 0 {
                return Err(OwnedStartupFailure::before_spawn(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Invalid native parent window",
                )));
            }
            command.arg(format!("--wid={parent}"));
        }
        command
            .arg(format!("--input-ipc-server={pipe}"))
            .arg("--")
            .arg(source.url());
        match scope {
            Some(scope) => owned_startup::spawn_owned_scoped(command, pipe, scope),
            None => owned_startup::spawn_owned(command, pipe),
        }
    }
}
