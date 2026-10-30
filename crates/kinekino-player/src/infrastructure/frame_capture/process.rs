use crate::application::api::ApiError;
#[cfg(test)]
use crate::application::api::ErrorCode;
pub(crate) mod owned;
#[cfg(test)]
use std::time::Duration;
use std::{process::Command, time::Instant};

/// Native tool output. Every run goes through the OS process-ownership port (ADR 0006), so the
/// same bounded/owned family semantics apply on every OS (capabilities differ, not code paths).
pub struct ProcessOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
pub fn run(
    command: &mut Command,
    deadline: Instant,
    max_stdout: usize,
) -> Result<ProcessOutput, ApiError> {
    owned::run(command, deadline, max_stdout, None, None)
}

pub fn run_with_recovery(
    command: &mut Command,
    deadline: Instant,
    max_stdout: usize,
    control: Option<&dyn crate::application::review_source::MutationControl>,
    recovery: Option<&dyn crate::application::sdk_execution::ReceiptRecoveryControl>,
) -> Result<ProcessOutput, ApiError> {
    owned::run(command, deadline, max_stdout, control, recovery)
}
/// The caller retains its original input owner until this original family is physically
/// closed and receipted. Input is a read-only regular handle, never a worker-selected path.
pub(crate) fn run_with_input(
    command: &mut Command,
    deadline: Instant,
    max_stdout: usize,
    control: Option<&dyn crate::application::review_source::MutationControl>,
    input: std::fs::File,
) -> Result<ProcessOutput, ApiError> {
    owned::run_with_input(command, deadline, max_stdout, control, None, Some(input))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    #[ignore = "Subprocess fixture invoked by deadline_kills_and_reaps_child"]
    fn held_file_child() {
        let Some(path) = std::env::var_os("FRAME_CAPTURE_PROCESS_TEST_FILE") else {
            return;
        };
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        crate::infrastructure::os_policy::apply_share_mode(
            &mut options,
            crate::infrastructure::os_policy::ShareMode::Exclusive,
        );
        let mut file = options.open(path).unwrap();
        file.write_all(b"started").unwrap();
        file.flush().unwrap();
        std::thread::sleep(Duration::from_secs(30));
        file.write_all(b"unexpected completion").unwrap();
    }

    #[test]
    #[ignore = "Owned descendant fixture; invoked only by the explicit central deadline test"]
    fn held_descendant_parent() {
        let mut descendant = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "infrastructure::frame_capture::process::tests::held_file_child",
                "--ignored",
            ])
            .spawn()
            .unwrap();
        std::thread::sleep(Duration::from_secs(30));
        descendant.wait().unwrap();
    }

    #[test]
    #[ignore = "Central allocation: real inherited-pipe descendant fault; preserves the first failure"]
    fn deadline_with_inherited_pipe_descendant_remains_bounded() {
        let path =
            std::env::temp_dir().join(format!("capture-descendant-{}.lock", std::process::id()));
        let start = Instant::now();
        let error = run(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "infrastructure::frame_capture::process::tests::held_descendant_parent",
                    "--ignored",
                ])
                .env("FRAME_CAPTURE_PROCESS_TEST_FILE", &path),
            start + Duration::from_secs(2),
            4096,
        )
        .err()
        .expect("held parent must exceed its original deadline");
        assert_eq!(error.code, ErrorCode::Timeout);
        assert!(
            path.exists(),
            "the real descendant must have reached the held file"
        );
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "inherited readers extended the original 2s deadline: {:?}",
            start.elapsed()
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn deadline_kills_and_reaps_child() {
        let path = std::env::temp_dir().join(format!("capture-child-{}.lock", std::process::id()));
        let start = Instant::now();
        let error = run(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "infrastructure::frame_capture::process::tests::held_file_child",
                    "--ignored",
                ])
                .env("FRAME_CAPTURE_PROCESS_TEST_FILE", &path),
            start + Duration::from_secs(2),
            4096,
        )
        .err()
        .expect("child must exceed deadline");
        assert_eq!(error.code, ErrorCode::Timeout);
        assert!(start.elapsed() < Duration::from_secs(5));
        // Windows exclusive handle is released only once the child has exited.
        let mut options = std::fs::OpenOptions::new();
        options.read(true).write(true);
        crate::infrastructure::os_policy::apply_share_mode(
            &mut options,
            crate::infrastructure::os_policy::ShareMode::Exclusive,
        );
        let mut file = options.open(&path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "started");
        drop(file);
        std::fs::remove_file(path).unwrap();
    }
}
