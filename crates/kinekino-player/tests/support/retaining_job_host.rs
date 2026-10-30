//! Shared test closeout of the original Rust job host. Uses production begin/poll;
//! it never treats carrier Drop, timeout or a stored job error as cleanup proof.
use kinekino_jobs::infrastructure::{
    PendingWorkerHost, RetainingWorkerHost, WorkerShutdownComplete, WorkerShutdownOutcome,
};
use std::{
    any::Any,
    io::{self, Write},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    thread,
    time::{Duration, Instant},
};

pub(crate) struct CloseFailure {
    pending: Option<Box<PendingWorkerHost>>,
    completed: Option<Box<WorkerShutdownComplete>>,
    diagnostics: Vec<DiagnosticFailure>,
}
enum DiagnosticFailure {
    Io(io::Error),
    Panic(Box<dyn Any + Send>),
}
impl DiagnosticFailure {
    fn kind(&self) -> &'static str {
        match self {
            Self::Io(error) => match error.kind() {
                io::ErrorKind::BrokenPipe => "broken_pipe",
                _ => "io_error",
            },
            Self::Panic(payload) => {
                let _original_payload = payload.as_ref();
                "panic"
            }
        }
    }
}
fn diagnose(carrier: &mut CloseFailure, write: impl FnOnce(&CloseFailure) -> io::Result<()>) {
    // Carrier already contains the original owner. Diagnostics cannot unwind it
    // or replace the primary action/cleanup error; retain even panic payloads.
    let result = catch_unwind(AssertUnwindSafe(|| write(carrier)));
    match result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => carrier.diagnostics.push(DiagnosticFailure::Io(error)),
        Err(payload) => carrier.diagnostics.push(DiagnosticFailure::Panic(payload)),
    }
}
fn write_complete(carrier: &CloseFailure, writer: &mut dyn Write) -> io::Result<()> {
    let result = carrier
        .completed
        .as_ref()
        .expect("original completed carrier");
    writeln!(
        writer,
        "original_host_closeout={}",
        serde_json::json!({
            "receipt":result.receipt,"first_error":result.errors.first(),
            "secondary_overflow":result.errors.overflow()
        })
    )?;
    writer.flush()
}
pub(crate) fn close(host: RetainingWorkerHost) -> Result<(), CloseFailure> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut owner = host.begin_shutdown(deadline);
    loop {
        match owner.poll(deadline) {
            WorkerShutdownOutcome::Complete(result) => {
                let mut carrier = CloseFailure {
                    pending: None,
                    completed: Some(Box::new(result)),
                    diagnostics: Vec::new(),
                };
                diagnose(&mut carrier, |value| {
                    write_complete(value, &mut io::stderr().lock())
                });
                let result = carrier.completed.as_ref().unwrap();
                if result.receipt.is_some()
                    && result.errors.first().is_none()
                    && carrier.diagnostics.is_empty()
                {
                    return Ok(());
                }
                return Err(carrier);
            }
            WorkerShutdownOutcome::Pending(pending) => owner = pending,
        }
        if Instant::now() >= deadline {
            return Err(CloseFailure {
                pending: Some(Box::new(owner)),
                completed: None,
                diagnostics: Vec::new(),
            });
        }
        thread::sleep(Duration::from_millis(2));
    }
}
pub(crate) fn conclude(action: thread::Result<()>, close_failure: Option<CloseFailure>) {
    let failed = close_failure.is_some();
    if let Some(error) = close_failure {
        // Explicit incomplete test-process boundary BEFORE any diagnostic work.
        // Preserve the whole original carrier, including completed diagnostics,
        // and never report this retention as cleanup completion.
        let retained = Box::leak(Box::new(error));
        diagnose(retained, |error| {
            let mut writer = io::stderr().lock();
            writeln!(
                writer,
                "original_case_closeout_failed pending={} first={:?}",
                error.pending.is_some(),
                error
                    .pending
                    .as_ref()
                    .and_then(|owner| owner.errors().first())
                    .or_else(|| error
                        .completed
                        .as_ref()
                        .and_then(|value| value.errors.first()))
            )?;
            for diagnostic in &error.diagnostics {
                writeln!(writer, "secondary_diagnostic_failure={}", diagnostic.kind())?;
            }
            writer.flush()
        });
    }
    if let Err(payload) = action {
        resume_unwind(payload);
    }
    assert!(
        !failed,
        "Original host closeout failed; inspect preserved fixture evidence"
    );
}

#[cfg(test)]
mod diagnostic_tests {
    use super::*;
    use kinekino_media_contracts::errors::{ApiError, ErrorCode};

    enum Fault {
        Write,
        Flush,
        Panic,
    }
    struct Writer(Fault);
    impl Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            match self.0 {
                Fault::Write => Err(io::Error::new(io::ErrorKind::BrokenPipe, "injected write")),
                Fault::Panic => panic!("injected diagnostic writer panic"),
                Fault::Flush => Ok(bytes.len()),
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("injected flush"))
        }
    }
    fn fixture() -> CloseFailure {
        let mut result = WorkerShutdownComplete {
            receipt: None,
            errors: Default::default(),
        };
        result
            .errors
            .record(ApiError::new(ErrorCode::Timeout, "original cleanup error"));
        CloseFailure {
            pending: None,
            completed: Some(Box::new(result)),
            diagnostics: Vec::new(),
        }
    }
    fn exercise(fault: Fault, expected: &str) {
        // Direct diagnostic injection, not a fabricated live-process observation.
        let mut carrier = fixture();
        let address = carrier.completed.as_deref().unwrap() as *const WorkerShutdownComplete;
        diagnose(&mut carrier, |value| {
            write_complete(value, &mut Writer(fault))
        });
        assert_eq!(carrier.diagnostics.len(), 1);
        assert_eq!(carrier.diagnostics[0].kind(), expected);
        let result = carrier.completed.as_deref().unwrap();
        assert_eq!(result as *const WorkerShutdownComplete, address);
        assert_eq!(result.errors.first().unwrap().code, ErrorCode::Timeout);
        assert_eq!(
            result.errors.first().unwrap().message,
            "original cleanup error"
        );
        assert!(result.receipt.is_none());
        assert!(carrier.pending.is_none());
    }
    #[test]
    fn write_failure_keeps_completed_carrier_and_original_cleanup_error() {
        kinekino_platform::install();
        exercise(Fault::Write, "broken_pipe");
    }
    #[test]
    fn flush_failure_keeps_completed_carrier_and_original_cleanup_error() {
        kinekino_platform::install();
        exercise(Fault::Flush, "io_error");
    }
    #[test]
    fn diagnostic_panic_keeps_completed_carrier_and_original_cleanup_error() {
        kinekino_platform::install();
        exercise(Fault::Panic, "panic");
    }
    #[test]
    fn conclude_preserves_original_action_panic_over_cleanup_failure() {
        kinekino_platform::install();
        let result = catch_unwind(AssertUnwindSafe(|| {
            conclude(Err(Box::new(37_u32)), Some(fixture()));
        }));
        assert_eq!(*result.unwrap_err().downcast::<u32>().unwrap(), 37);
    }
}
