#[path = "supervisor/ports.rs"]
mod ports;
use super::raw_records::{RecordCursor, RecordPoll};
pub use ports::ReceiptSink;
mod cleanup;
mod record_observation;
use crate::{
    domain::{deadline::Deadline, identity::ProcessIdentity, observation::*},
    infrastructure::evidence_files::EvidencePaths,
};
pub use cleanup::CleanupProgress;
pub use record_observation::{PhaseObservation, RecordKind, RecordObservation, RecordWaitExit};
use std::{
    io, thread,
    time::{Duration, Instant},
};

pub trait Clock {
    fn now(&self) -> Instant;
    fn pause(&self, duration: Duration);
}
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
    fn pause(&self, duration: Duration) {
        thread::sleep(duration);
    }
}
/// Implementations must poll owned handles; never search or reopen a PID to terminate it.
pub trait ProcessFamily {
    fn deadline(&self) -> Deadline;
    fn identity(&self) -> &ProcessIdentity;
    fn close_stdin(&mut self) -> io::Result<()>;
    fn poll_child(&mut self) -> io::Result<Option<u32>>;
    fn poll_family(&mut self) -> io::Result<FamilyObservation>;
    fn terminate_owned_family(&mut self) -> io::Result<()>;
}
#[must_use = "Explicitly finish or retain this original process-family owner"]
pub struct Supervisor<P> {
    deadline: Deadline,
    process: P,
    evidence: EvidencePaths,
    raw_limit: u64,
    receipt: ProcessReceipt,
    receipt_recorded: bool,
    cleanup: Option<cleanup::CleanupState>,
}
#[must_use = "Incomplete retains original handles and errors; it is not a cleanup ACK"]
pub enum FinishOutcome<P> {
    Complete(Completion),
    Incomplete(Box<Incomplete<P>>),
}
pub struct Completion {
    receipt: ProcessReceipt,
}
impl Completion {
    pub fn receipt(&self) -> &ProcessReceipt {
        &self.receipt
    }
}
pub struct Incomplete<P> {
    pub owner: Supervisor<P>,
    pub cause: FailureCause,
    pub termination_error: Option<io::Error>,
    pub receipt_error: Option<io::Error>,
}
impl<P: ProcessFamily> Supervisor<P> {
    pub fn new(process: P, evidence: EvidencePaths, raw_limit: u64) -> Self {
        // No owner is consumed for a fallible validation: limits are checked by spawn callers.
        let receipt = ProcessReceipt {
            identity: process.identity().clone(),
            child_exit: None,
            family: FamilyObservation::Unknown,
            forced: false,
            stdin_eof_requested: false,
        };
        Self {
            deadline: process.deadline(),
            process,
            evidence,
            raw_limit,
            receipt,
            receipt_recorded: false,
            cleanup: None,
        }
    }
    pub fn process(&self) -> &P {
        &self.process
    }
    pub fn evidence(&self) -> &EvidencePaths {
        &self.evidence
    }
    pub fn receipt(&self) -> &ProcessReceipt {
        &self.receipt
    }
    /// At least one sink call returned Ok; this is not a deadline or storage-atomicity claim.
    pub fn receipt_recorded(&self) -> bool {
        self.receipt_recorded
    }
    pub fn poll_state(&mut self) -> Result<&ProcessReceipt, FailureCause> {
        self.receipt.child_exit = self.process.poll_child().map_err(|error| FailureCause {
            stage: FailureStage::ChildObservation,
            error,
        })?;
        self.receipt.family = FamilyObservation::Unknown;
        self.receipt.family = self.process.poll_family().map_err(|error| FailureCause {
            stage: FailureStage::FamilyObservation,
            error,
        })?;
        Ok(&self.receipt)
    }
    /// Byte-bounded regular-file polling, not a hard deadline on synchronous filesystem calls.
    /// The caller retains this same supervisor on all errors.
    pub fn wait_stdout_record(
        &mut self,
        cursor: &mut RecordCursor,
        maximum: usize,
        deadline: Deadline,
        clock: &impl Clock,
    ) -> Result<RecordPoll, FailureCause> {
        self.wait_stdout_record_inner(cursor, maximum, deadline, clock, None)
    }
    /// Explicit diagnostics using only this original owner. No sink runs in the wait loop.
    pub fn wait_stdout_record_observed(
        &mut self,
        cursor: &mut RecordCursor,
        maximum: usize,
        deadline: Deadline,
        clock: &impl Clock,
        observation: &mut RecordObservation,
    ) -> Result<RecordPoll, FailureCause> {
        *observation = RecordObservation {
            offset_before: cursor.offset(),
            ..RecordObservation::default()
        };
        let started = Instant::now();
        let result = self.wait_stdout_record_inner(
            cursor,
            maximum,
            deadline,
            clock,
            Some(&mut *observation),
        );
        observation.complete(cursor.offset(), started.elapsed());
        result
    }
    fn wait_stdout_record_inner(
        &mut self,
        cursor: &mut RecordCursor,
        maximum: usize,
        deadline: Deadline,
        clock: &impl Clock,
        mut observation: Option<&mut RecordObservation>,
    ) -> Result<RecordPoll, FailureCause> {
        let deadline = self.deadline.constrained_by(deadline);
        loop {
            if let Some(value) = observation.as_deref_mut() {
                value.iteration();
            }
            if deadline.operation_remaining(clock.now()).is_zero() {
                if let Some(value) = observation.as_deref_mut() {
                    value.exit = RecordWaitExit::PreCutoff;
                }
                return Err(FailureCause {
                    stage: FailureStage::Deadline,
                    error: io::Error::new(
                        io::ErrorKind::TimedOut,
                        "record observation budget exhausted",
                    ),
                });
            }
            let started = observation.as_ref().map(|_| Instant::now());
            let checked = self.evidence.check_limit(self.raw_limit);
            if let (Some(value), Some(started)) = (observation.as_deref_mut(), started) {
                value.limit_check.record(started.elapsed());
            }
            checked.map_err(|error| {
                if let Some(value) = observation.as_deref_mut() {
                    value.exit = RecordWaitExit::LimitError;
                }
                FailureCause {
                    stage: FailureStage::RawOutput,
                    error,
                }
            })?;
            let started = observation.as_ref().map(|_| Instant::now());
            let polled = cursor.poll(&self.evidence.stdout, maximum, false);
            if let (Some(value), Some(started)) = (observation.as_deref_mut(), started) {
                value.record_poll.record(started.elapsed());
                value.last_record = match &polled {
                    Ok(RecordPoll::Pending) => RecordKind::Pending,
                    Ok(RecordPoll::Record { .. }) => RecordKind::Record,
                    Err(_) => RecordKind::Error,
                };
            }
            let record = polled.map_err(|error| {
                if let Some(value) = observation.as_deref_mut() {
                    value.exit = RecordWaitExit::PollError;
                }
                FailureCause {
                    stage: FailureStage::RawOutput,
                    error,
                }
            })?;
            if deadline.operation_remaining(clock.now()).is_zero() {
                if let Some(value) = observation.as_deref_mut() {
                    value.exit = RecordWaitExit::PostCutoff;
                }
                return Err(FailureCause {
                    stage: FailureStage::Deadline,
                    error: io::Error::new(
                        io::ErrorKind::TimedOut,
                        "record read returned after operation cutoff",
                    ),
                });
            }
            if matches!(record, RecordPoll::Record { .. }) {
                if let Some(value) = observation.as_deref_mut() {
                    value.exit = RecordWaitExit::Record;
                }
                return Ok(record);
            }
            let started = observation.as_ref().map(|_| Instant::now());
            let child = self.process.poll_child();
            if let (Some(value), Some(started)) = (observation.as_deref_mut(), started) {
                value.child_poll.record(started.elapsed());
            }
            self.receipt.child_exit = child.map_err(|error| {
                if let Some(value) = observation.as_deref_mut() {
                    value.exit = RecordWaitExit::ChildError;
                }
                FailureCause {
                    stage: FailureStage::ChildObservation,
                    error,
                }
            })?;
            if self.receipt.child_exit.is_some() {
                if let Some(value) = observation.as_deref_mut() {
                    value.exit = RecordWaitExit::ChildExit;
                }
                return Err(FailureCause {
                    stage: FailureStage::ChildObservation,
                    error: io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "child exited before complete ready record",
                    ),
                });
            }
            clock.pause(Duration::from_millis(10).min(deadline.operation_remaining(clock.now())));
        }
    }
    fn incomplete(
        mut self,
        stage: FailureStage,
        error: io::Error,
        sink: &mut impl ReceiptSink,
    ) -> FinishOutcome<P> {
        let termination_error = if !self.receipt.forced
            && (self.receipt.child_exit.is_none()
                || self.receipt.family != FamilyObservation::Empty)
        {
            self.receipt.forced = true;
            self.process.terminate_owned_family().err()
        } else {
            None
        };
        let receipt_error = if self.receipt_recorded {
            None
        } else {
            sink.record(&self.receipt).err()
        };
        self.receipt_recorded |= receipt_error.is_none();
        FinishOutcome::Incomplete(Box::new(Incomplete {
            owner: self,
            cause: FailureCause { stage, error },
            termination_error,
            receipt_error,
        }))
    }
    pub fn finish(
        mut self,
        deadline: Deadline,
        clock: &impl Clock,
        sink: &mut impl ReceiptSink,
    ) -> FinishOutcome<P> {
        let deadline = self.deadline.constrained_by(deadline);
        if let Err(error) = self.process.close_stdin() {
            return self.incomplete(FailureStage::StdinEof, error, sink);
        }
        self.receipt.stdin_eof_requested = true;
        loop {
            self.receipt.child_exit = match self.process.poll_child() {
                Ok(exit) => exit,
                Err(error) => return self.incomplete(FailureStage::ChildObservation, error, sink),
            };
            self.receipt.family = FamilyObservation::Unknown;
            self.receipt.family = match self.process.poll_family() {
                Ok(family) => family,
                Err(error) => return self.incomplete(FailureStage::FamilyObservation, error, sink),
            };
            if let Err(error) = self.evidence.check_limit(self.raw_limit) {
                return self.incomplete(FailureStage::RawOutput, error, sink);
            }
            if deadline.remaining(clock.now()).is_zero() {
                return self.incomplete(
                    FailureStage::Deadline,
                    io::Error::new(
                        io::ErrorKind::TimedOut,
                        "observation completed after deadline",
                    ),
                    sink,
                );
            }
            if self.receipt.child_exit.is_some() && self.receipt.family == FamilyObservation::Empty
            {
                if let Err(error) = sink.record(&self.receipt) {
                    return self.incomplete(FailureStage::Receipt, error, sink);
                }
                self.receipt_recorded = true;
                if deadline.remaining(clock.now()).is_zero() {
                    return self.incomplete(
                        FailureStage::Deadline,
                        io::Error::new(io::ErrorKind::TimedOut, "receipt returned after deadline"),
                        sink,
                    );
                }
                return FinishOutcome::Complete(Completion {
                    receipt: self.receipt,
                });
            }
            if deadline.remaining(clock.now()).is_zero() {
                return self.incomplete(
                    FailureStage::Deadline,
                    io::Error::new(io::ErrorKind::TimedOut, "owned family still incomplete"),
                    sink,
                );
            }
            if !self.receipt.forced && deadline.operation_remaining(clock.now()).is_zero() {
                self.receipt.forced = true;
                if let Err(error) = self.process.terminate_owned_family() {
                    return self.incomplete(FailureStage::Termination, error, sink);
                }
            }
            clock.pause(Duration::from_millis(10).min(deadline.remaining(clock.now())));
        }
    }
}
