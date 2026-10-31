//! The same SDK call owns the original family until physical closure and a final receipt.
//! Pending cleanup never detaches a worker or serializes/drops an incomplete owner.
mod launch;
pub(crate) use launch::evidence_base;
pub(crate) mod operation;
mod output_drain;
mod receipt;
#[cfg(test)]
mod tests;
use super::ProcessOutput;
use crate::application::api::{ApiError, ErrorCode};
use kinekino_process::{
    application::supervisor::{Supervisor, SystemClock},
    domain::{deadline::Deadline, observation::FamilyObservation},
    infrastructure::{
        evidence_files::RawFile,
        family::{OwnedProcessFamily, spawn},
    },
};
use std::{
    io::Read,
    process::Command,
    time::{Duration, Instant},
};
const STDERR_LIMIT: usize = 131072;
const CLEANUP_RESERVE: Duration = Duration::from_secs(2);

struct Session {
    owner: Supervisor<OwnedProcessFamily>,
    deadline: Deadline,
    max_stdout: usize,
    first: Option<ApiError>,
    sink: receipt::FileSink,
    receipt_attempted: bool,
    recovery_generation: Option<u8>,
    output_drain: output_drain::OutputDrain,
}

pub(super) fn run(
    command: &Command,
    cutoff: Instant,
    maximum: usize,
    control: Option<&dyn crate::application::review_source::MutationControl>,
    recovery: Option<&dyn crate::application::sdk_execution::ReceiptRecoveryControl>,
) -> Result<ProcessOutput, ApiError> {
    run_with_input(command, cutoff, maximum, control, recovery, None)
}
pub(super) fn run_with_input(
    command: &Command,
    cutoff: Instant,
    maximum: usize,
    control: Option<&dyn crate::application::review_source::MutationControl>,
    recovery: Option<&dyn crate::application::sdk_execution::ReceiptRecoveryControl>,
    input: Option<std::fs::File>,
) -> Result<ProcessOutput, ApiError> {
    let mut operation =
        operation::NativeProcessOperation::prepare(command, cutoff, maximum, control, input)?;
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        operation.launch()?;
        loop {
            if operation.poll(control, recovery) {
                return operation
                    .take_result()
                    .expect("Settled original native operation");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }));
    match outcome {
        Ok(result) => result,
        Err(payload) => {
            struct Closing;
            impl crate::application::review_source::MutationControl for Closing {
                fn checkpoint(&self) -> Result<(), ApiError> {
                    Err(failed("native_callback_unwind"))
                }
            }
            let mut recovery = recovery;
            let mut secondary_panics = Vec::new();
            // Neither operation nor original/secondary panic payloads unwind away
            // while an original child, family or receipt remains unsettled.
            while !operation
                .progress()
                .is_some_and(|progress| progress.may_release())
            {
                let cleanup = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    operation.poll(Some(&Closing), recovery)
                }));
                if let Err(secondary) = cleanup {
                    secondary_panics.push(secondary);
                    // Do not repeatedly invoke a callback that itself panicked.
                    // Sticky receipt failure still retains the original operation.
                    recovery = None;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            drop(secondary_panics);
            std::panic::resume_unwind(payload)
        }
    }
}

impl Session {
    fn remember(&mut self, error: ApiError) {
        if self.first.is_none() {
            self.first = Some(error);
        }
    }
    fn poll(
        &mut self,
        control: Option<&dyn crate::application::review_source::MutationControl>,
        recovery: Option<&dyn crate::application::sdk_execution::ReceiptRecoveryControl>,
    ) -> bool {
        if self.receipt_attempted {
            // A failed immutable receipt is sticky. Do not create unbounded retry files
            // or release this original owner as though persistence had succeeded.
            let settled = self
                .owner
                .cleanup_progress()
                .is_some_and(|progress| progress.settled());
            if settled {
                return self.poll_output_drain();
            }
            if let Some(recovery) = recovery {
                if self.recovery_generation.is_none() {
                    self.recovery_generation = recovery.receipt_failed().ok();
                }
                if let Some(generation) = self.recovery_generation
                    && recovery.take_retry(generation).unwrap_or(false)
                {
                    // Only this explicit original-generation request permits another
                    // immutable sink attempt. Never reset the operation deadline/error.
                    self.recovery_generation = None;
                    if self
                        .owner
                        .record_cleanup(&mut self.sink, &SystemClock)
                        .is_ok()
                    {
                        return self.poll_output_drain();
                    }
                }
            }
            return false;
        }
        if self.first.is_none()
            && let Some(control) = control
            && let Err(error) = control.checkpoint()
        {
            self.remember(error);
        }
        if self.deadline.operation_remaining(Instant::now()).is_zero() {
            self.remember(timeout());
        }
        let paths = self.owner.evidence();
        let limit = bounded_length(&paths.stdout, self.max_stdout)
            .and_then(|()| bounded_length(&paths.stderr, STDERR_LIMIT));
        if let Err(error) = limit {
            self.remember(error);
        }
        let requested = if self.first.as_ref().is_some_and(|error| {
            error.code != ErrorCode::Timeout
                || !self.deadline.operation_remaining(Instant::now()).is_zero()
        }) {
            self.deadline.constrained_end(Instant::now())
        } else {
            self.deadline
        };
        let progress = self.owner.poll_cleanup(requested, &SystemClock);
        let closed = progress.physical_closed();
        let cleanup_error = progress
            .first_error()
            .map(|cause| failed(&format!("cleanup_{:?}", cause.stage)));
        if let Some(error) = cleanup_error {
            self.remember(error);
        }
        if !closed {
            return false;
        }
        self.receipt_attempted = true;
        if self
            .owner
            .record_cleanup(&mut self.sink, &SystemClock)
            .is_err()
        {
            self.remember(failed("receipt"));
            return false;
        }
        self.poll_output_drain()
    }
    fn poll_output_drain(&mut self) -> bool {
        if !self.owner.cleanup_progress().is_some_and(|p| p.settled()) {
            return false;
        }
        match self.output_drain.poll(self.owner.evidence()) {
            Ok(done) => done,
            Err(error) => {
                self.remember(error);
                false
            }
        }
    }
    fn complete(mut self) -> Result<ProcessOutput, ApiError> {
        if self.deadline.operation_remaining(Instant::now()).is_zero() {
            self.remember(timeout());
        }
        let receipt = self.owner.receipt();
        let closed = receipt.child_exit.is_some() && receipt.family == FamilyObservation::Empty;
        if !closed {
            return Err(failed("unsettled_completion"));
        }
        self.first = completion_failure(self.first.take(), receipt.child_exit, receipt.forced);
        if let Some(mut error) = self.first.take() {
            error.details["kinekino_process"] = serde_json::json!({"evidence_root":self.sink.root,"physical_closed":closed,"receipt_recorded":self.owner.receipt_recorded(),"child_exit":receipt.child_exit,"forced":receipt.forced});
            return Err(error);
        }
        let paths = self.owner.evidence();
        let stdout = read(&paths.stdout, self.max_stdout)?;
        let stderr = read(&paths.stderr, STDERR_LIMIT)?;
        if self.deadline.operation_remaining(Instant::now()).is_zero() {
            return Err(timeout());
        }
        // Success leaves nothing to investigate: drop the evidence root now. Failure
        // paths return earlier and keep it, because the error names `evidence_root`.
        let _ = std::fs::remove_dir_all(&self.sink.root);
        Ok(ProcessOutput { stdout, stderr })
    }
}

fn bounded_length(file: &RawFile, limit: usize) -> Result<(), ApiError> {
    let length = file
        .open_read()
        .and_then(|file| file.metadata())
        .map_err(|_| failed("raw_observation"))?
        .len();
    if length > limit as u64 {
        Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Native frame output exceeded its bounded limit",
        ))
    } else {
        Ok(())
    }
}

fn completion_failure(
    first: Option<ApiError>,
    child_exit: Option<u32>,
    forced: bool,
) -> Option<ApiError> {
    if first.is_some() {
        return first;
    }
    if child_exit != Some(0) {
        let mut error = failed("native_exit");
        error.details = serde_json::json!({"native_exit_code":child_exit});
        return Some(error);
    }
    forced.then(|| failed("forced_cleanup"))
}
fn read(file: &RawFile, limit: usize) -> Result<Vec<u8>, ApiError> {
    let mut bytes = Vec::new();
    file.open_read()
        .map_err(|_| failed("raw_read"))?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| failed("raw_read"))?;
    if bytes.len() > limit {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Native frame output exceeded its bounded limit",
        ));
    }
    Ok(bytes)
}
fn timeout() -> ApiError {
    ApiError::new(
        ErrorCode::Timeout,
        "Native frame operation deadline expired; no frame output was published",
    )
}
fn failed(stage: &str) -> ApiError {
    let mut error = ApiError::new(
        ErrorCode::EngineFailed,
        "Native frame process failed; backend output is withheld",
    );
    error.details = serde_json::json!({"stage":stage});
    error
}
