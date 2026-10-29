use super::*;

pub(super) struct CleanupState {
    deadline: Deadline,
    eof_attempted: bool,
    physical_closed: bool,
    final_recorded: bool,
    deadline_expired: bool,
    contradictory_exit: bool,
    first: Option<FailureCause>,
    secondary: Vec<FailureCause>,
    secondary_overflow: bool,
}
impl CleanupState {
    fn record(&mut self, stage: FailureStage, error: io::Error) {
        let cause = FailureCause { stage, error };
        if self.first.is_none() {
            self.first = Some(cause);
        } else if self.secondary.len() < 8 {
            self.secondary.push(cause);
        } else {
            self.secondary_overflow = true;
        }
    }
}

/// Borrowed observations of this original owner. Physical closure, receipt
/// persistence, deadline and success are deliberately separate facts.
pub struct CleanupProgress<'a> {
    receipt: &'a ProcessReceipt,
    state: &'a CleanupState,
}
impl CleanupProgress<'_> {
    pub fn receipt(&self) -> &ProcessReceipt {
        self.receipt
    }
    pub fn physical_closed(&self) -> bool {
        self.state.physical_closed
    }
    pub fn final_receipt_recorded(&self) -> bool {
        self.state.final_recorded
    }
    pub fn settled(&self) -> bool {
        self.state.physical_closed && self.state.final_recorded
    }
    pub fn deadline_expired(&self) -> bool {
        self.state.deadline_expired
    }
    pub fn first_error(&self) -> Option<&FailureCause> {
        self.state.first.as_ref()
    }
    pub fn secondary_errors(&self) -> &[FailureCause] {
        &self.state.secondary
    }
    pub fn secondary_overflow(&self) -> bool {
        self.state.secondary_overflow
    }
    pub fn operation_remaining(&self, now: Instant) -> Duration {
        self.state.deadline.operation_remaining(now)
    }
    pub fn remaining(&self, now: Instant) -> Duration {
        self.state.deadline.remaining(now)
    }
}

impl<P: ProcessFamily> Supervisor<P> {
    /// Pure setup; all later requests can only shorten these original cutoffs.
    pub fn begin_cleanup(&mut self, requested: Deadline) {
        if let Some(state) = &mut self.cleanup {
            state.deadline = state.deadline.constrained_by(requested);
        } else {
            self.cleanup = Some(CleanupState {
                deadline: self.deadline.constrained_by(requested),
                eof_attempted: false,
                physical_closed: false,
                final_recorded: false,
                deadline_expired: false,
                contradictory_exit: false,
                first: None,
                secondary: Vec::new(),
                secondary_overflow: false,
            });
        }
    }
    pub fn cleanup_progress(&self) -> Option<CleanupProgress<'_>> {
        self.cleanup.as_ref().map(|state| CleanupProgress {
            receipt: &self.receipt,
            state,
        })
    }
    /// One original EOF/child/family/raw-limit/force observation step. Never
    /// sleeps, loops, consumes the owner, opens a PID or starts another worker.
    /// OS/filesystem calls themselves are synchronous, not hard real-time.
    pub fn poll_cleanup(&mut self, requested: Deadline, clock: &impl Clock) -> CleanupProgress<'_> {
        self.begin_cleanup(requested);
        let state = self.cleanup.as_mut().expect("initialized cleanup");
        if state.physical_closed {
            return CleanupProgress {
                receipt: &self.receipt,
                state,
            };
        }
        if !state.eof_attempted {
            state.eof_attempted = true;
            match self.process.close_stdin() {
                Ok(()) => self.receipt.stdin_eof_requested = true,
                Err(error) => state.record(FailureStage::StdinEof, error),
            }
        }
        match self.process.poll_child() {
            Ok(exit) => {
                if let Some(original) = self.receipt.child_exit {
                    if exit != Some(original) {
                        state.contradictory_exit = true;
                        state.record(
                            FailureStage::ChildObservation,
                            io::Error::other("Original child exit observation changed"),
                        );
                    }
                } else {
                    self.receipt.child_exit = exit;
                }
            }
            Err(error) => state.record(FailureStage::ChildObservation, error),
        }
        self.receipt.family = FamilyObservation::Unknown;
        match self.process.poll_family() {
            Ok(family) => self.receipt.family = family,
            Err(error) => state.record(FailureStage::FamilyObservation, error),
        }
        if let Err(error) = self.evidence.check_limit(self.raw_limit) {
            state.record(FailureStage::RawOutput, error);
        }
        let now = clock.now();
        if state.deadline.remaining(now).is_zero() && !state.deadline_expired {
            state.deadline_expired = true;
            state.record(
                FailureStage::Deadline,
                io::Error::new(io::ErrorKind::TimedOut, "Original cleanup cutoff expired"),
            );
        }
        state.physical_closed = !state.contradictory_exit
            && self.receipt.child_exit.is_some()
            && self.receipt.family == FamilyObservation::Empty;
        if !state.physical_closed
            && !self.receipt.forced
            && (state.first.is_some() || state.deadline.operation_remaining(now).is_zero())
        {
            self.receipt.forced = true;
            if let Err(error) = self.process.terminate_owned_family() {
                state.record(FailureStage::Termination, error);
            }
            // Terminate success does not imply exit/Empty. Another original poll
            // must observe both; a failed terminate is sticky and not auto-retried.
        }
        CleanupProgress {
            receipt: &self.receipt,
            state,
        }
    }
    /// Persist this original snapshot. A partial snapshot is diagnostic only;
    /// final_recorded becomes true only after physical closure and a successful
    /// sink call. Use fresh immutable receipt destinations for distinct snapshots.
    pub fn record_cleanup(
        &mut self,
        sink: &mut impl ReceiptSink,
        clock: &impl Clock,
    ) -> io::Result<()> {
        let Some(state) = &mut self.cleanup else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cleanup has not begun",
            ));
        };
        if state.final_recorded {
            return Ok(());
        }
        match sink.record(&self.receipt) {
            Ok(()) => {
                self.receipt_recorded = true;
                state.final_recorded = state.physical_closed;
                if state.deadline.remaining(clock.now()).is_zero() && !state.deadline_expired {
                    state.deadline_expired = true;
                    state.record(
                        FailureStage::Deadline,
                        io::Error::new(
                            io::ErrorKind::TimedOut,
                            "Original receipt write returned after cleanup cutoff",
                        ),
                    );
                }
                Ok(())
            }
            Err(error) => {
                let returned = match error.raw_os_error() {
                    Some(code) => io::Error::from_raw_os_error(code),
                    None => io::Error::new(error.kind(), error.to_string()),
                };
                state.record(FailureStage::Receipt, error);
                Err(returned)
            }
        }
    }
}
