//! Test synchronization only; never renews the original operation budget.
use super::fixture_data::{io, ordinary_paths};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
    time::Duration,
};
pub const STARTUP_READY: &str = "fixture.startup.ready";
pub const STARTUP_RELEASE: &str = "fixture.startup.release";
pub const ENGINE_READY: &str = "fixture.engine.ready";
pub const ENGINE_RELEASE: &str = "fixture.engine.release";
pub fn publish(root: &Path, name: &str, bytes: &[u8]) -> Result<File, ApiError> {
    if ![STARTUP_READY, STARTUP_RELEASE, ENGINE_READY, ENGINE_RELEASE].contains(&name) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Unknown fixture barrier",
        ));
    }
    let pending = root.join(format!("{name}.pending"));
    let path = root.join(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .map_err(io)?;
    file.write_all(bytes).map_err(io)?;
    file.sync_all().map_err(io)?;
    drop(file);
    fs::hard_link(pending, &path).map_err(io)?;
    ordinary_paths::file(&path)
}
pub fn released(root: &Path, name: &str) -> Result<bool, ApiError> {
    if ![STARTUP_RELEASE, ENGINE_RELEASE].contains(&name) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Unknown fixture release",
        ));
    }
    let mut file = match ordinary_paths::file(&root.join(name)) {
        Ok(file) => file,
        Err(error) if error.code == ErrorCode::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(9)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    if bytes != b"release\n" {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Fixture release malformed",
        ));
    }
    Ok(true)
}
pub fn wait_startup(root: &Path) -> Result<(), ApiError> {
    let _ready = publish(root, STARTUP_READY, b"ready\n")?;
    // The original parent Supervisor/Deadline is the only forced lifetime owner.
    // Deliberately no child-started duration or fresh timeout for this test hook.
    while !released(root, STARTUP_RELEASE)? {
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}
