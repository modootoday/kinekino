//! SDK runner and OS host processes (playback/native/provider/job/review). Adapter crate:
//! platform `cfg` lives here; composed by `kinekino-app`.
// Linux build: Windows-only adapters are cfg-gated, so their callees look unused here. Removed once kinekino-platform split lands.
#![cfg_attr(
    not(windows),
    allow(dead_code, unused_imports, unused_variables, unreachable_code)
)]
#[cfg(windows)]
#[path = "execution_host.rs"]
mod execution_host;
#[path = "frame_tools.rs"]
mod frame_tools;
#[path = "host_shutdown/mod.rs"]
mod host_shutdown;
#[path = "job_host.rs"]
mod job_host;
#[cfg(windows)]
#[path = "native_host.rs"]
mod native_host;
#[cfg(windows)]
#[path = "playback_host.rs"]
mod playback_host;
#[cfg(windows)]
#[path = "provider_host.rs"]
mod provider_host;
#[cfg(windows)]
#[path = "review_host.rs"]
mod review_host;
#[path = "runner.rs"]
mod runner;

pub mod fixtures;
#[path = "player_gui/mod.rs"]
pub mod player_gui;
pub use runner::{adblock, execute_value, run};
