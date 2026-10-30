use std::{fmt, io};

#[derive(Debug)]
pub struct Failure {
    pub stage: &'static str,
    pub original_error: io::Error,
    pub receipt_error: Option<String>,
}
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pre-child launch failed at {}: {}; failure_receipt_error={:?}",
            self.stage, self.original_error, self.receipt_error
        )
    }
}
impl std::error::Error for Failure {}
pub trait Sink {
    fn launch_failed(&mut self, failure: &Failure) -> io::Result<()>;
}
/// Only use before spawn succeeds: no child identity or cleanup claim exists yet.
pub fn run<T>(
    stage: &'static str,
    operation: impl FnOnce() -> io::Result<T>,
    sink: &mut impl Sink,
) -> Result<T, Failure> {
    operation().map_err(|original_error| {
        let mut failure = Failure {
            stage,
            original_error,
            receipt_error: None,
        };
        if let Err(error) = sink.launch_failed(&failure) {
            failure.receipt_error = Some(error.to_string());
        }
        failure
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Recorder {
        fail_write: bool,
        stage: Option<&'static str>,
        original: Option<String>,
    }
    impl Sink for Recorder {
        fn launch_failed(&mut self, failure: &Failure) -> io::Result<()> {
            self.stage = Some(failure.stage);
            self.original = Some(failure.original_error.to_string());
            if self.fail_write {
                Err(io::Error::other("receipt unavailable"))
            } else {
                Ok(())
            }
        }
    }
    fn rejects(stage: &'static str) {
        let mut sink = Recorder::default();
        let error = run::<()>(
            stage,
            || Err(io::Error::other("injected launch failure")),
            &mut sink,
        )
        .unwrap_err();
        assert_eq!(error.stage, stage);
        assert_eq!(sink.stage, Some(stage));
        assert_eq!(sink.original.as_deref(), Some("injected launch failure"));
        assert!(error.receipt_error.is_none());
    }
    #[test]
    fn stdout_creation_failure_is_reported() {
        rejects("stdout_create");
    }
    #[test]
    fn stderr_creation_failure_is_reported() {
        rejects("stderr_create");
    }
    #[test]
    fn spawn_failure_is_reported() {
        rejects("child_spawn");
    }
    #[test]
    fn receipt_write_failure_preserves_both_errors_for_stderr() {
        let mut sink = Recorder {
            fail_write: true,
            ..Recorder::default()
        };
        let error = run::<()>(
            "child_spawn",
            || Err(io::Error::other("executable missing")),
            &mut sink,
        )
        .unwrap_err();
        assert!(error.to_string().contains("executable missing"));
        assert_eq!(error.receipt_error.as_deref(), Some("receipt unavailable"));
    }
}
