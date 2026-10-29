use super::LoopError;
#[derive(Default)]
pub(super) struct FailureLedger {
    pub first: Option<LoopError>,
    pub secondary: Vec<LoopError>,
    pub overflow: bool,
    panic: Option<Box<dyn std::any::Any + Send>>,
}
impl FailureLedger {
    /// Repeated observation of the same fact is not a fresh secondary event.
    pub fn remember(&mut self, error: LoopError) {
        if self.first.is_none() {
            self.first = Some(error);
        } else if self.first != Some(error) && !self.secondary.contains(&error) {
            if self.secondary.len() < 8 {
                self.secondary.push(error);
            } else {
                self.overflow = true;
            }
        }
    }
    pub fn retain_panic(&mut self, panic: Box<dyn std::any::Any + Send>) {
        self.remember(LoopError::Panic);
        if self.panic.is_none() {
            self.panic = Some(panic);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{http_send::SendError, worker_ipc::IpcError, worker_session::WorkerError};
    #[test]
    fn cleanup_and_original_panic_do_not_replace_first_failure_or_grow_without_bound() {
        let mut ledger = FailureLedger::default();
        ledger.remember(LoopError::Order);
        ledger.retain_panic(Box::new("original fixture panic"));
        for error in [
            LoopError::Cutoff,
            LoopError::PeerEof,
            LoopError::Ipc(IpcError::Io),
            LoopError::Ipc(IpcError::Order),
            LoopError::Ipc(IpcError::Eof),
            LoopError::Worker(WorkerError::Order),
            LoopError::Worker(WorkerError::Http(SendError::Setup)),
            LoopError::Worker(WorkerError::Http(SendError::Transport)),
            LoopError::Worker(WorkerError::Http(SendError::Cancelled)),
        ] {
            ledger.remember(error);
        }
        for _ in 0..100 {
            ledger.remember(LoopError::Ipc(IpcError::Io));
        }
        assert_eq!(ledger.first, Some(LoopError::Order));
        assert_eq!(ledger.secondary.len(), 8);
        assert!(ledger.overflow);
        assert_eq!(
            ledger.panic.as_ref().unwrap().downcast_ref::<&str>(),
            Some(&"original fixture panic")
        );
    }
}
