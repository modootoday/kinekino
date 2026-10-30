use super::{Mpv, TrustedMpvExecutable, network_plan};
use crate::application::{
    api::{ApiError, ErrorCode},
    playback_session::EpisodePlaybackSource,
};
use std::process::Command;

impl Mpv {
    /// Explicit transfer attempt from historical verified bytes, not a live-session assertion.
    pub fn launch_episode(
        executable: &TrustedMpvExecutable,
        source: &EpisodePlaybackSource,
        parent: Option<u32>,
        paused: bool,
        volume_percent: f64,
    ) -> Result<Self, ApiError> {
        let arguments = network_plan::arguments(source, paused, volume_percent)?;
        let pipe = super::ipc_address("kinekino-mpv").map_err(|_| failed())?;
        let mut command = Command::new(executable.path());
        command.args(arguments);
        if let Some(parent) = parent {
            if parent == 0 {
                return Err(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Invalid native parent window",
                ));
            }
            command.arg(format!("--wid={parent}"));
        }
        command
            .arg(format!("--input-ipc-server={pipe}"))
            .arg("--")
            .arg(source.url());
        Self::spawn_process(command, pipe).map_err(|_| failed())
    }
}

fn failed() -> ApiError {
    // Do not expose argv, URL/query, pipe name or engine stderr in the public API.
    ApiError::new(
        ErrorCode::EngineFailed,
        "Native process could not be started",
    )
}
