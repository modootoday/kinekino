use super::*;
impl WorkerSession {
    pub fn first_failure(&self) -> Option<WorkerError> {
        self.failure
    }

    /// Best-effort wire failure at a frame boundary only. False means no frame
    /// was queued: outer owner must close original IPC/HTTP/process resources.
    /// Err is a secondary encoding/protocol error; first_failure is unchanged.
    pub fn begin_failure(&mut self, primary: WorkerError, now: u64) -> Result<bool, WorkerError> {
        let primary = *self.failure.get_or_insert(primary);
        if let Some(send) = &mut self.send {
            send.abort();
        }
        if self.outgoing.is_some()
            || now >= self.exchange.cutoffs().end
            || matches!(
                self.exchange.status(),
                BodyStatus::AwaitingOpen
                    | BodyStatus::Ended
                    | BodyStatus::Failed
                    | BodyStatus::Aborted
            )
        {
            return Ok(false);
        }
        // Fixed, bounded text only. Never format reqwest errors, URLs or headers.
        let message: &[u8] = match primary {
            WorkerError::Http(SendError::Invalidated) => b"source invalidated",
            WorkerError::Http(SendError::Cancelled) => b"source cancelled",
            WorkerError::Http(SendError::Setup) => b"source transport setup failed",
            WorkerError::Http(SendError::Transport) => b"source transport failed",
            WorkerError::Http(SendError::Response(_)) => b"source response rejected",
            WorkerError::Protocol(ProtocolError::Cutoff) => b"source deadline expired",
            WorkerError::Protocol(_) => b"source protocol failed",
            WorkerError::Http(SendError::Order) | WorkerError::Order => b"source order invalid",
        };
        self.queue(Kind::Failure, message.to_vec(), now)?;
        Ok(true)
    }
    pub(super) fn writable(&self) -> Result<(), WorkerError> {
        if self.failure_written() {
            return Ok(());
        }
        if self
            .outgoing
            .as_ref()
            .is_some_and(|frame| frame.kind == Kind::Failure)
        {
            return Ok(());
        }
        self.healthy()
    }
    /// Only serialized Failure bytes finished. Never a successful job or a
    /// physical-close receipt, and the original primary error is still present.
    pub fn failure_written(&self) -> bool {
        self.failure.is_some()
            && self.exchange.status() == BodyStatus::Failed
            && self.outgoing.is_none()
    }
}
