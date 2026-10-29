//! Separate harness=false technical child; never a production engine selector.
#![cfg(test)]
#[path = "support/fixture_barrier.rs"]
pub mod fixture_barrier;
#[path = "support/fixture_data.rs"]
pub mod fixture_data;
#[path = "support/fixture_engine.rs"]
pub mod fixture_engine;
#[path = "support/fixture_executable_path.rs"]
pub mod fixture_executable_path;
#[path = "support/infrastructure.rs"]
pub mod infrastructure;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::io::Write;

fn run() -> Result<(), ApiError> {
    use kinekino_vad_worker::{
        parent_configuration::{ConfiguredWorkerLaunch, serve_parent_configured_request},
        staged_files::WorkerRequestFiles,
    };
    let launch = ConfiguredWorkerLaunch::parse(std::env::args_os().skip(1))?;
    let files = WorkerRequestFiles::open(&launch.worker.request)?;
    let original = fixture_executable_path::open_current_executable(
        &std::env::current_exe().map_err(fixture_data::io)?,
    )?;
    let data = fixture_data::FixtureData::load(&original.path)?;
    let request = kinekino_vad::protocol::application::decode_worker_request(
        std::fs::File::open(&launch.worker.request).map_err(fixture_data::io)?,
        files.declared_bytes(),
        files.descriptor_raw(),
    )?;
    if request.expected().request() != files.original_request()
        || request.expected().profile().runtime() != data.profile.runtime()
        || launch.worker.manifest != data.manifest_pin
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Configured fixture differs from the fixed executable/release",
        ));
    }
    let root = launch.worker.request.parent().ok_or_else(|| {
        ApiError::new(ErrorCode::InvalidArgument, "Configured fixture root absent")
    })?;
    let mode = fixture_engine::FixtureMode::parse(
        &std::env::var("PLAYER_VAD_TEST_FIXTURE_MODE").map_err(|_| {
            ApiError::new(ErrorCode::InvalidArgument, "Explicit fixture mode required")
        })?,
    )?;
    let engine = fixture_engine::FixtureEngine {
        mode,
        root,
        data: &data,
    };
    serve_parent_configured_request(
        std::io::stdin().lock(),
        &files,
        &launch,
        &original.path,
        std::io::stdout().lock(),
        &engine,
    )
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(std::io::stderr().lock(), "{}", error.message);
            std::process::ExitCode::from(error.exit_code())
        }
    }
}
