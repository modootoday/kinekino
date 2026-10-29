use super::*;
use kinekino_process::application::{raw_records::RecordPoll, supervisor::SystemClock};
impl ParentHost {
    /// Always retain the same process/IPC after error or a receipt sink panic.
    /// At most one raw record per channel per call; no blocking finish/Drop loop.
    pub fn poll_close(
        &mut self,
        requested: Deadline,
        sink: &mut impl ParentReceiptSink,
    ) -> HostCloseFacts {
        self.closing = true;
        self.deadline = self.deadline.constrained_by(requested);
        let (process, process_error) = match &mut self.process {
            None => self.poll_unassigned(),
            Some(owner) => {
                let progress = owner.poll_cleanup(self.deadline, &SystemClock);
                let error = if !self.process_error_seen {
                    progress.first_error().map(|cause| {
                        io::Error::new(
                            cause.error.kind(),
                            "original cleanup error retained by supervisor",
                        )
                    })
                } else {
                    None
                };
                (
                    if progress.physical_closed() {
                        ProcessClosure::Closed
                    } else {
                        ProcessClosure::Pending
                    },
                    error,
                )
            }
        };
        if let Some(error) =
            process_error.filter(|_| !self.process_error_seen || self.has_unassigned_owner())
        {
            self.process_error_seen = true;
            self.remember(CloseStage::Process, error);
        }
        let ipc = self.ipc.poll_close();
        if ipc.had_error && !self.ipc_error_seen {
            self.ipc_error_seen = true;
            self.remember(
                CloseStage::Ipc,
                io::Error::other("original IPC errors retained by SourceIpc"),
            );
        }
        if process != ProcessClosure::Pending {
            for index in 0..2 {
                if self.raw_eof[index] || self.raw_failed[index] {
                    continue;
                }
                let raw = if index == 0 {
                    &self.evidence.stdout
                } else {
                    &self.evidence.stderr
                };
                match self.cursors[index].poll(raw, 4096, true) {
                    Ok(RecordPoll::Pending) => self.raw_eof[index] = true,
                    Ok(RecordPoll::Record { .. }) => {}
                    Err(error) => {
                        self.raw_failed[index] = true;
                        self.remember(
                            if index == 0 {
                                CloseStage::RawStdout
                            } else {
                                CloseStage::RawStderr
                            },
                            error,
                        );
                    }
                }
            }
        }
        if matches!(
            process,
            ProcessClosure::Closed | ProcessClosure::UnassignedClosed
        ) && ipc.io_reaped
            && ipc.local_endpoints_closed
            && self.raw_eof == [true; 2]
            && !self.receipt_attempted
        {
            // Mark before calling external sink; unwind/failed writes never retry.
            self.receipt_attempted = true;
            if let Some(owner) = &mut self.process
                && let Err(error) = owner.record_cleanup(sink, &SystemClock)
            {
                self.remember(CloseStage::Receipt, error);
            }
            if let Some(receipt) = &self.unassigned_receipt {
                match sink.record_unassigned(receipt) {
                    Ok(()) => self.unassigned_recorded = true,
                    Err(error) => self.remember(CloseStage::Receipt, error),
                }
            }
        }
        let receipt = match &self.process {
            None if self.has_unassigned_owner() => {
                if self.unassigned_recorded {
                    ReceiptState::Recorded
                } else {
                    ReceiptState::Unrecorded
                }
            }
            None => ReceiptState::NotApplicable,
            Some(owner)
                if owner
                    .cleanup_progress()
                    .is_some_and(|progress| progress.final_receipt_recorded()) =>
            {
                ReceiptState::Recorded
            }
            Some(_) => ReceiptState::Unrecorded,
        };
        HostCloseFacts {
            ipc,
            process,
            receipt,
            stdout_eof: self.raw_eof[0],
            stderr_eof: self.raw_eof[1],
        }
    }
}
