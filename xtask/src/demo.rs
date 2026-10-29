//! demo-browser: launches the CDP Chromium adapter in a visible window for manual review.
use std::process::{Command, ExitCode};

pub fn browser(args: &[String]) -> ExitCode {
    let status = Command::new(env!("CARGO"))
        .args([
            "run",
            "-j",
            "1",
            "-p",
            "kinekino-browser",
            "--example",
            "demo_browser",
            "--",
        ])
        .args(args)
        .status();
    match status {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}
