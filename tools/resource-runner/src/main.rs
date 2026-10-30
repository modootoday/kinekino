#[path = "application/child_execution.rs"]
mod child_execution;
#[path = "domain/cpu_selection.rs"]
mod cpu_selection;
#[path = "application/execution_admission.rs"]
mod execution_admission;
#[path = "domain/execution_profile.rs"]
mod execution_profile;
#[path = "presentation/execution_receipt.rs"]
mod execution_receipt;
#[path = "application/launch_stage.rs"]
mod launch_stage;
#[path = "application/topology_diagnostic.rs"]
mod topology_diagnostic;
#[path = "infrastructure/topology_records.rs"]
mod topology_records;
#[path = "presentation/topology_report.rs"]
mod topology_report;
mod windows_budget;
#[path = "infrastructure/windows_resources.rs"]
mod windows_resources;
#[path = "infrastructure/windows_topology.rs"]
mod windows_topology;

use execution_admission::{Admission, QueryFailure};
use execution_profile::Profile;
use execution_receipt::Receipt;
use std::{
    fs::File,
    os::windows::io::AsRawHandle,
    process::{Command, Stdio},
};

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("Resource runner failed: {error}");
            std::process::exit(125);
        }
    }
}
fn run() -> std::io::Result<i32> {
    let mut args = std::env::args_os().skip(1);
    let first = args.next().ok_or_else(invalid)?;
    if first == "--topology" {
        if args.next().is_some() {
            return Err(invalid());
        }
        return topology_diagnostic::run();
    }
    let (requested, stdout) = if first == "--profile" {
        (
            args.next().ok_or_else(invalid)?,
            args.next().ok_or_else(invalid)?,
        )
    } else {
        (std::ffi::OsString::from("conservative2"), first)
    };
    let stderr = args.next().ok_or_else(invalid)?;
    let receipt = args.next().ok_or_else(invalid)?;
    let program = args.next().ok_or_else(invalid)?;
    let requested = requested.to_str().unwrap_or("invalid_unicode_profile");
    let profile = Profile::parse(requested);
    let admission = execution_admission::prepare(profile);
    let receipt_path = std::path::Path::new(&receipt);
    let mut report = Receipt {
        profile,
        requested_profile: requested,
        admission: &admission,
        child: None,
        terminal: true,
        exit: Some((125, 0)),
    };
    if admission.rejection.is_some() {
        report.write(receipt_path)?;
        return Ok(125);
    }
    // Replace any previous terminal receipt before any Job or child can exist.
    report.terminal = false;
    report.exit = None;
    report.write(receipt_path)?;
    let mut launch_sink = execution_receipt::LaunchSink {
        receipt: report,
        path: receipt_path,
    };
    let stdout = launch_stage::run("stdout_create", || File::create(stdout), &mut launch_sink)
        .map_err(std::io::Error::other)?;
    let stderr = launch_stage::run("stderr_create", || File::create(stderr), &mut launch_sink)
        .map_err(std::io::Error::other)?;
    let selected = admission.selected_mask.ok_or_else(invalid)?;
    let budget = match windows_budget::Budget::enter(profile.ok_or_else(invalid)?, selected) {
        Ok(budget) => budget,
        Err(error) => {
            let rejection = Admission {
                selected_mask: admission.selected_mask,
                intervals: admission.intervals.clone(),
                ..Admission::query_failed(QueryFailure {
                    stage: "budget_setup_or_readback",
                    os_code: error.raw_os_error(),
                })
            };
            Receipt {
                admission: &rejection,
                terminal: true,
                exit: Some((125, 0)),
                ..report
            }
            .write(receipt_path)?;
            return Err(error);
        }
    };
    let child = launch_stage::run(
        "child_spawn",
        || {
            Command::new(program)
                .args(args)
                .stdin(Stdio::null())
                .stdout(stdout)
                .stderr(stderr)
                .spawn()
        },
        &mut launch_sink,
    )
    .map_err(std::io::Error::other)?;
    let mut sink = execution_receipt::ExecutionSink::new(report, receipt_path);
    child_execution::supervise(
        child,
        |child| budget.verify(child.as_raw_handle()),
        &mut sink,
    )
    .map_err(std::io::Error::other)
}
fn invalid() -> std::io::Error {
    std::io::Error::other(
        "Expected [--profile conservative2|build4] stdout-path stderr-path receipt-path executable [arguments]",
    )
}
