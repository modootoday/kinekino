//! One isolated child loop. Parent must already own replay admission and Job.
mod drive;
mod outcome;
use crate::{
    domain::{Cutoffs, ProtocolError},
    http_send::SendError,
    worker_ipc::{IpcError, WorkerIpc},
    worker_session::{WorkerError, WorkerSession},
};
use kinekino_process::{domain::source_clock::SourceCutoffs, infrastructure::ipc::uptime_millis};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopError {
    Ipc(IpcError),
    Worker(WorkerError),
    Cutoff,
    PeerEof,
    Order,
    Panic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopStatus {
    Closed,
    Incomplete,
}
/// Same IPC/session remain owned across errors/unwind and explicit close.
pub struct WorkerLoop {
    ipc: WorkerIpc,
    session: Option<WorkerSession>,
    cutoffs: SourceCutoffs,
    outcome: outcome::FailureLedger,
    closing: bool,
    draining_failure: bool,
}
impl WorkerLoop {
    pub fn new(ipc: WorkerIpc, cutoffs: SourceCutoffs) -> Self {
        Self {
            ipc,
            session: None,
            cutoffs,
            outcome: outcome::FailureLedger::default(),
            closing: false,
            draining_failure: false,
        }
    }
    pub fn first_error(&self) -> Option<LoopError> {
        self.outcome.first
    }
    pub fn secondary_errors(&self) -> &[LoopError] {
        &self.outcome.secondary
    }
    pub fn secondary_overflow(&self) -> bool {
        self.outcome.overflow
    }
    fn current_cutoffs(&self) -> Cutoffs {
        self.session.as_ref().map_or(
            Cutoffs {
                operation: self.cutoffs.operation(),
                end: self.cutoffs.end(),
            },
            WorkerSession::cutoffs,
        )
    }
    fn fail(&mut self, error: LoopError, now: u64) {
        self.outcome.remember(error);
        let operation = self.current_cutoffs().operation;
        let primary = match self.outcome.first.expect("recorded first error") {
            LoopError::Worker(error) => error,
            LoopError::Cutoff => WorkerError::Protocol(ProtocolError::Cutoff),
            _ => WorkerError::Protocol(ProtocolError::Io),
        };
        // B does not admit fresh writes after operation cutoff or a transport error.
        self.draining_failure = false;
        if now < operation
            && self.ipc.first_error().is_none()
            && let Some(session) = &mut self.session
        {
            match session.begin_failure(primary, now) {
                Ok(queued) => self.draining_failure = queued,
                Err(secondary) => self.outcome.remember(LoopError::Worker(secondary)),
            }
        }
        self.closing = !self.draining_failure;
    }
    /// Explicit unwind recovery keeps the original owners. Caller retains the
    /// panic separately; this marker does not replace an earlier primary error.
    pub fn begin_unwind_close(&mut self, panic: Box<dyn std::any::Any + Send>) {
        self.outcome.retain_panic(panic);
        self.closing = true;
        if let Some(session) = &mut self.session {
            session.abort();
        }
    }
    /// Reap original IPC. Closed describes local IO only; parent must still
    /// observe actual child/family exit and persist its original receipt.
    pub async fn close(&mut self) -> LoopStatus {
        self.closing = true;
        if let Some(session) = &mut self.session {
            session.close_http();
        }
        loop {
            let close = self.ipc.poll_close();
            if close.had_error {
                self.outcome.remember(LoopError::Ipc(IpcError::Io));
            }
            if close.io_reaped && close.local_endpoints_closed {
                return LoopStatus::Closed;
            }
            if uptime_millis() >= self.current_cutoffs().end {
                return LoopStatus::Incomplete;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }
}
fn check_cutoff(cutoffs: Cutoffs) -> Result<(), SendError> {
    if uptime_millis() >= cutoffs.operation {
        Err(SendError::Invalidated)
    } else {
        Ok(())
    }
}
