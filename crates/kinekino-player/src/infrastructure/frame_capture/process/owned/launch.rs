//! Adapt the private frame command to the existing suspended/assigned family launcher.
use crate::application::api::{ApiError, ErrorCode};
use kinekino_process::infrastructure::{evidence_files::PreparedEvidence, family::SpawnSpec};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
/// Overrides the parent of owned-process evidence roots (default: `<temp>/kinekino-frame-owned`).
pub(crate) const EVIDENCE_DIR_ENV: &str = "KINEKINO_PROCESS_EVIDENCE_DIR";

/// Dedicated parent for per-process evidence roots, never bare `temp_dir()`.
pub(crate) fn evidence_base() -> PathBuf {
    std::env::var_os(EVIDENCE_DIR_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("kinekino-frame-owned"))
}

pub(super) fn prepare(
    command: &Command,
) -> Result<(SpawnSpec, PreparedEvidence, PathBuf), ApiError> {
    let cwd = std::env::current_dir().map_err(error)?;
    let cwd = match command.get_current_dir() {
        Some(path) => cwd.join(path),
        None => cwd,
    };
    let executable = PathBuf::from(command.get_program());
    if !executable.is_absolute() {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Frame process requires an explicit absolute executable",
        ));
    }
    // Private frame callers inherit the environment and may override/remove named variables;
    // this is not a general adapter for arbitrary Command/env_clear configurations.
    let environment = if command.get_envs().next().is_none() {
        None
    } else {
        let mut values = BTreeMap::<String, OsString>::new();
        for (key, value) in std::env::vars_os() {
            let key = key
                .into_string()
                .map_err(|_| error(std::io::Error::other("Non-Unicode environment name")))?;
            values.insert(key.to_ascii_uppercase(), value);
        }
        for (key, value) in command.get_envs() {
            let key = key
                .to_str()
                .ok_or_else(|| error(std::io::Error::other("Non-Unicode environment name")))?
                .to_ascii_uppercase();
            if let Some(value) = value {
                values.insert(key, value.to_owned());
            } else {
                values.remove(&key);
            }
        }
        Some(values)
    };
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| error(std::io::Error::other("Clock before epoch")))?
        .as_nanos();
    let base = evidence_base();
    std::fs::create_dir_all(&base).map_err(error)?;
    let root = base.join(format!(
        "frame-owned-{}-{stamp}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).map_err(error)?;
    let evidence = PreparedEvidence::create(&root, None, 0).map_err(error)?;
    Ok((
        SpawnSpec {
            executable,
            cwd,
            arguments: command.get_args().map(OsString::from).collect(),
            environment,
        },
        evidence,
        root,
    ))
}
pub(super) fn prepare_with_input(
    command: &Command,
    input: std::fs::File,
) -> Result<(SpawnSpec, PreparedEvidence, PathBuf), ApiError> {
    let (spec, evidence, root) = prepare(command)?;
    let evidence = evidence.with_input_file(input).map_err(error)?;
    Ok((spec, evidence, root))
}

fn error(_: std::io::Error) -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        "Cannot prepare private native frame process evidence",
    )
}
