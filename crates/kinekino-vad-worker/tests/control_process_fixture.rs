//! Explicit harness=false test artifact. Never included in the production main.
#![cfg(test)]
#[path = "support/fixture_barrier.rs"]
pub mod fixture_barrier;
#[path = "support/fixture_data.rs"]
pub mod fixture_data;
#[path = "support/fixture_engine.rs"]
pub mod fixture_engine;
#[path = "support/fixture_executable_path.rs"]
pub mod fixture_executable_path;
#[path = "support/fixture_payload.rs"]
pub mod fixture_payload;
#[path = "support/infrastructure.rs"]
pub mod infrastructure;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::io::Write;

enum FixtureOutput<W> {
    Real(W),
    FailFlush(Vec<u8>),
}
impl<W: Write> Write for FixtureOutput<W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Real(writer) => writer.write(bytes),
            Self::FailFlush(buffer) => {
                buffer.extend_from_slice(bytes);
                Ok(bytes.len())
            }
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Real(writer) => writer.flush(),
            Self::FailFlush(_) => Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe)),
        }
    }
}
fn run() -> Result<(), ApiError> {
    use fixture_data::{FixtureAdmission, FixtureData};
    use kinekino_vad_worker::{
        control::serve_controlled_request,
        entry::WorkerLaunch,
        staged_files::{WorkerArtifactFiles, WorkerRequestFiles},
    };
    let launch = WorkerLaunch::parse(std::env::args_os().skip(1))?;
    let files = WorkerRequestFiles::open(&launch.request)?;
    let raw_executable = std::env::current_exe().map_err(fixture_data::io)?;
    let original_executable = fixture_executable_path::open_current_executable(&raw_executable)?;
    let executable = &original_executable.path;
    let data = FixtureData::load(executable)?;
    let original_request = kinekino_vad::protocol::application::decode_worker_request(
        std::fs::File::open(&launch.request).map_err(fixture_data::io)?,
        files.declared_bytes(),
        files.descriptor_raw(),
    )?;
    if original_request.expected().request() != files.original_request()
        || original_request.expected().profile().runtime() != data.profile.runtime()
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Fixture ordinary path differs from original request/runtime pin",
        ));
    }
    if launch.manifest != data.manifest_pin {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Fixture launch is not the fixed synthetic release",
        ));
    }
    let root = launch
        .request
        .parent()
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Fixture root absent"))?;
    let mode = fixture_engine::FixtureMode::parse(
        &std::env::var("PLAYER_VAD_TEST_FIXTURE_MODE").map_err(|_| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Explicit test fixture mode required",
            )
        })?,
    )?;
    match std::env::var("PLAYER_VAD_TEST_STARTUP_BARRIER").as_deref() {
        Ok("1") => fixture_barrier::wait_startup(root)?,
        Err(std::env::VarError::NotPresent) => {}
        _ => {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid fixture startup barrier",
            ));
        }
    }
    let output = match std::env::var("PLAYER_VAD_TEST_OUTPUT").as_deref() {
        Ok("fail-flush") => FixtureOutput::FailFlush(Vec::new()),
        Err(std::env::VarError::NotPresent) => FixtureOutput::Real(std::io::stdout().lock()),
        _ => {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid explicit fixture output fault",
            ));
        }
    };
    serve_controlled_request(
        std::io::stdin().lock(),
        &files,
        &launch,
        output,
        |request, control| {
            let reader = WorkerArtifactFiles::open(
                root,
                executable,
                request.expected().descriptor(),
                &data.profile,
            )?;
            let engine = fixture_engine::FixtureEngine {
                mode,
                root,
                data: &data,
            };
            kinekino_vad_worker::run::run_request(
                request,
                &FixtureAdmission(&data),
                &reader,
                &engine,
                control,
            )
        },
    )
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            let code = error.exit_code();
            let _ = writeln!(std::io::stderr().lock(), "{}", error.message);
            std::process::ExitCode::from(code)
        }
    }
}
