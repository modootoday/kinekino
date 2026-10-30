//! Explicit execution host. Incomplete shutdown retains the original owner at exit.
use kinekino_player::{
    application::{
        api::{ApiError, ErrorCode, PlayerSdk, Response},
        sdk_execution::ExecutionBudget,
    },
    infrastructure::local_host::{TransportOptions, execution_host::ExecutionLocalHost},
};
use std::{
    io::{Read, Write},
    sync::Arc,
    time::{Duration, Instant},
};
fn validate(args: &[String]) -> Result<(), ApiError> {
    let mut seen = false;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--serve-execution" {
            if seen {
                return Err(invalid());
            }
            seen = true;
        } else if crate::frame_tools::is_option(&args[index]) {
            index += 1;
            if args.get(index).is_none_or(|value| value.starts_with("--")) {
                return Err(invalid());
            }
        } else {
            return Err(invalid());
        }
        index += 1;
    }
    if !seen {
        return Err(invalid());
    }
    crate::frame_tools::options(args)?.ok_or_else(invalid)?;
    Ok(())
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Execution host requires unique --serve-execution and complete pinned frame tools only",
    )
}
fn report(error: ApiError) -> u8 {
    let code = error.exit_code();
    let response = Response::failure("execution-host".into(), error);
    if let Ok(json) = serde_json::to_string(&response) {
        let _ = writeln!(std::io::stderr().lock(), "{json}");
    }
    code
}
pub(super) fn run(args: &[String]) -> u8 {
    match serve(args) {
        Ok(()) => 0,
        Err(error) => report(error),
    }
}
fn serve(args: &[String]) -> Result<(), ApiError> {
    validate(args)?;
    let frame = crate::frame_tools::configured(args)?.ok_or_else(invalid)?;
    let sdk = PlayerSdk::default().with_frame_capture(frame);
    let host = ExecutionLocalHost::start(
        Arc::new(sdk),
        ExecutionBudget {
            max_operations: 4,
            max_request_bytes: 1_048_576,
            max_total_request_bytes: 4_194_304,
            max_terminal_bytes: 1_048_576,
            max_total_terminal_bytes: 4_194_304,
        },
        TransportOptions::default(),
    )?;
    let ready = serde_json::json!({"host":"execution","endpoint":host.endpoint(),"host_scope":host.host_scope(),"pid":std::process::id(),"commands":kinekino_player::application::sdk_execution::host::schema::COMMANDS,"automatic_retry":false,"quality_approved":false});
    let mut first = if writeln!(std::io::stdout().lock(), "{ready}").is_err() {
        Some(ApiError::new(
            ErrorCode::EngineFailed,
            "Execution ready write failed",
        ))
    } else {
        None
    };
    if first.is_none() {
        let mut byte = [0u8; 1];
        loop {
            match std::io::stdin().lock().read(&mut byte) {
                Ok(0) => break,
                Ok(_) => {}
                Err(_) => {
                    first = Some(ApiError::new(
                        ErrorCode::EngineFailed,
                        "Execution stdin read failed",
                    ));
                    break;
                }
            }
        }
    }
    let mut pending = host.begin_shutdown();
    let cutoff = Instant::now() + Duration::from_secs(3);
    loop {
        match pending.poll() {
            Ok(status) if status.is_complete() => return first.map_or(Ok(()), Err),
            Ok(status) if Instant::now() >= cutoff => {
                let error = first.take().unwrap_or_else(|| {
                    ApiError::new(
                        ErrorCode::Timeout,
                        "Execution host shutdown incomplete; original operation retained",
                    )
                });
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "{}",
                    serde_json::json!({"error":error,"cleanup_complete":false,"observation":format!("{status:?}")})
                );
                // pending and its original SDK/tool/pipe owners are alive here.
                std::process::exit(i32::from(error.exit_code()));
            }
            Err(error) => {
                let code = report(first.take().unwrap_or(error));
                std::process::exit(i32::from(code));
            }
            _ => std::thread::sleep(Duration::from_millis(5)),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn execution_mode_rejects_missing_duplicate_mixed_and_unpinned_before_start() {
        for values in [
            vec![],
            vec!["--serve-execution"],
            vec!["--serve-execution", "--serve-execution"],
            vec!["--serve-execution", "--serve-jobs"],
            vec!["--serve-execution", "--endpoint", "untrusted"],
            vec!["--serve-execution", "--frame-ffmpeg"],
        ] {
            assert!(validate(&values.into_iter().map(str::to_owned).collect::<Vec<_>>()).is_err());
        }
    }
}
