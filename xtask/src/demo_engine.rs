//! demo-engine: runs one embedded engine adapter on the desktop with the port contract.
//! `cef` needs KINEKINO_CEF_DIR (an unpacked CEF distribution; never committed).
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const CEF_DIR_ENV: &str = "KINEKINO_CEF_DIR";
// Build-only CEF SDK entries that the runtime does not need next to the executable.
const SDK_ONLY: [&str; 7] = [
    "include",
    "cmake",
    "libcef_dll",
    "CMakeLists.txt",
    "CREDITS.html",
    "archive.json",
    "LICENSE.txt",
];

pub fn run(args: &[String]) -> ExitCode {
    let rest = args.get(1..).unwrap_or_default();
    let result = match args.first().map(String::as_str) {
        Some("chrome") => cargo_run("demo_engine_chrome", &[], rest),
        Some("cef") => cef(rest),
        _ => Err("usage: cargo xtask demo-engine <cef|chrome> [--hold]".into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("demo-engine: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cargo(example: &str, features: &[&str], sub: &str) -> Command {
    let mut c = Command::new(env!("CARGO"));
    c.args([
        sub,
        "-j",
        "1",
        "-p",
        "kinekino-browser",
        "--example",
        example,
    ]);
    for f in features {
        c.args(["--features", f]);
    }
    c
}

fn cargo_run(example: &str, features: &[&str], args: &[String]) -> Result<(), String> {
    let status = cargo(example, features, "run")
        .arg("--")
        .args(args)
        .status();
    check(status)
}

fn check(status: std::io::Result<std::process::ExitStatus>) -> Result<(), String> {
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("exited with {s}")),
        Err(e) => Err(e.to_string()),
    }
}

/// CEF looks for resources next to its executable, so the example runs from a staging
/// directory under runtime/ that links the distribution files (no copies of libcef).
fn cef(args: &[String]) -> Result<(), String> {
    let dist = std::env::var_os(CEF_DIR_ENV)
        .map(PathBuf::from)
        .ok_or(format!(
            "{CEF_DIR_ENV} must point to an unpacked CEF distribution"
        ))?;
    check(
        cargo("demo_engine_cef", &["cef"], "build")
            .env("CEF_PATH", &dist)
            .status(),
    )?;
    let target = crate::workspace::target_dir(&crate::workspace::root()?);
    let exe = target.join("debug/examples/demo_engine_cef");
    let stage = PathBuf::from("runtime/cef-run");
    stage_dist(&dist, &stage).map_err(|e| e.to_string())?;
    let staged = stage.join("demo_engine_cef");
    std::fs::copy(&exe, &staged).map_err(|e| format!("{}: {e}", exe.display()))?;
    let cache = std::fs::canonicalize(&stage)
        .map_err(|e| e.to_string())?
        .join("cache");
    check(
        Command::new(&staged)
            .args(args)
            .env("LD_LIBRARY_PATH", &dist)
            .env(CEF_DIR_ENV, &dist)
            .env("KINEKINO_CEF_CACHE", cache)
            .status(),
    )
}

fn stage_dist(dist: &Path, stage: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(stage)?;
    for entry in std::fs::read_dir(dist)? {
        let entry = entry?;
        let name = entry.file_name();
        if SDK_ONLY.iter().any(|s| name == *s) {
            continue;
        }
        let link = stage.join(&name);
        if link.symlink_metadata().is_err() {
            link_file(&entry.path(), &link)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn link_file(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

#[cfg(not(unix))]
fn link_file(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::copy(src, dst).map(|_| ())
}
