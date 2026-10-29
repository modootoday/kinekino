//! Inert production entry body: no fixture engine and no native admission fallback.
//! The composition root (`kinekino-app`, bin `kinekino-vad-worker`) installs the OS policy first.
use kinekino_media_contracts::errors::ApiError;
use std::io::Write;

fn run() -> Result<(), ApiError> {
    use crate::{
        control::serve_controlled_request,
        entry::{WorkerLaunch, unsupported_execution},
        staged_files::WorkerRequestFiles,
    };
    let arguments: Vec<_> = std::env::args_os().skip(1).take(23).collect();
    if arguments.len() == 22 {
        use crate::parent_configuration::{
            ConfiguredWorkerLaunch, serve_parent_configured_native_request,
        };
        let launch = ConfiguredWorkerLaunch::parse(arguments)?;
        let files = WorkerRequestFiles::open(&launch.worker.request)?;
        return serve_parent_configured_native_request(
            std::io::stdin().lock(),
            &files,
            &launch,
            std::io::stdout().lock(),
        );
    }
    let launch = WorkerLaunch::parse(arguments)?;
    let files = WorkerRequestFiles::open(&launch.request)?;
    serve_controlled_request(
        std::io::stdin().lock(),
        &files,
        &launch,
        std::io::stdout().lock(),
        |request, control| {
            if control.checkpoint()? {
                return Err(ApiError::new(
                    kinekino_media_contracts::errors::ErrorCode::Cancelled,
                    "Original worker cancellation observed; no cleanup ACK",
                ));
            }
            unsupported_execution(request)
        },
    )
}
/// Runs the worker process body; the OS policy must already be installed.
pub fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            let (exit_code, _diagnostic_result) = report_failure(&error, std::io::stderr().lock());
            std::process::ExitCode::from(exit_code)
        }
    }
}

fn report_failure(error: &ApiError, mut diagnostics: impl Write) -> (u8, std::io::Result<()>) {
    let exit_code = error.exit_code();
    let result = writeln!(diagnostics, "{}", error.message).and_then(|()| diagnostics.flush());
    (exit_code, result)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct BrokenPipe;
    impl Write for BrokenPipe {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn diagnostic_broken_pipe_preserves_original_mapped_exit() {
        let error = ApiError::new(
            kinekino_media_contracts::errors::ErrorCode::Cancelled,
            "original cancellation",
        );
        let (exit_code, diagnostic) = report_failure(&error, BrokenPipe);
        assert_eq!(exit_code, error.exit_code());
        assert_eq!(exit_code, 9);
        assert_eq!(
            diagnostic.unwrap_err().kind(),
            std::io::ErrorKind::BrokenPipe
        );
    }
}
