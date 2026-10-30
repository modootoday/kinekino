//! Thin host I/O lifetime driver. SDK requests continue over the original transport.
use super::*;
use crate::application::api::ErrorCode;
use std::{
    io::{Read, Write},
    thread,
    time::{Duration, Instant},
};
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct ReviewStdioCompletion {
    pub io_error: Option<ApiError>,
    pub shutdown: ReviewTransportCompletion,
}
impl ReviewStdioCompletion {
    pub fn first_error(&self) -> Option<&ApiError> {
        self.io_error
            .as_ref()
            .or(self.shutdown.first_error.as_ref())
    }
    pub fn succeeded(&self) -> bool {
        self.first_error().is_none()
    }
}
#[must_use = "Incomplete retains the original transport/session owners; do not replace or drop it as success"]
pub enum ReviewStdioOutcome {
    Complete(ReviewStdioCompletion),
    Incomplete {
        pending: Box<PendingReviewTransport>,
        io_error: Option<ApiError>,
    },
}
fn failed(phase: &str, error: std::io::Error) -> ApiError {
    let mut result = ApiError::new(ErrorCode::EngineFailed, "Review host I/O failed");
    result.details = serde_json::json!({"phase":phase,"io_kind":format!("{:?}",error.kind()),"io_error":error.to_string()});
    result
}
/// Emit one ready record, then retain the host until stdin EOF or I/O failure.
/// Synchronous Read/Write may block; the cleanup observation budget does not
/// promise hard preemption or renew a session's original operation deadline.
pub fn serve_stdio<R: Read + ?Sized, W: Write + ?Sized>(
    host: ReviewTransportHost,
    ready: &serde_json::Value,
    input: &mut R,
    output: &mut W,
) -> ReviewStdioOutcome {
    let mut io_error = (|| {
        serde_json::to_writer(&mut *output, ready).map_err(|error| {
            let kind = error.io_error_kind().unwrap_or(std::io::ErrorKind::Other);
            failed("ready_write", std::io::Error::new(kind, error))
        })?;
        output
            .write_all(b"\n")
            .map_err(|error| failed("ready_write", error))?;
        output
            .flush()
            .map_err(|error| failed("ready_flush", error))?;
        Ok::<_, ApiError>(())
    })()
    .err();
    if io_error.is_none() {
        let mut bytes = [0u8; 4096];
        loop {
            match input.read(&mut bytes) {
                Ok(0) => break,
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(error) => {
                    io_error = Some(failed("input_read", error));
                    break;
                }
            }
        }
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut pending = host.begin_shutdown(deadline);
    loop {
        let observation = pending.poll();
        if observation.complete {
            return match pending.into_completion() {
                Ok(shutdown) => {
                    ReviewStdioOutcome::Complete(ReviewStdioCompletion { io_error, shutdown })
                }
                Err(pending) => ReviewStdioOutcome::Incomplete { pending, io_error },
            };
        }
        if Instant::now() >= deadline {
            return ReviewStdioOutcome::Incomplete {
                pending: Box::new(pending),
                io_error,
            };
        }
        thread::sleep(Duration::from_millis(10));
    }
}
