use super::*;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Condvar, Mutex},
    thread::JoinHandle,
};
pub(super) enum Phase {
    Queued,
    Running,
    Terminal(Arc<ExecutionTerminal>),
}
pub(super) struct Entry {
    pub recovery: recovery::RecoveryState,
    pub request: Arc<Request>,
    pub request_bytes: usize,
    pub cancelling: bool,
    pub phase: Phase,
}
pub(super) struct State {
    pub next: u64,
    pub entries: BTreeMap<u64, Entry>,
    pub queue: VecDeque<u64>,
    pub acknowledged: VecDeque<u64>,
    pub request_bytes: usize,
    pub terminal_reserved: usize,
    pub closing: bool,
    pub worker_finished: bool,
    pub worker_joined: bool,
    pub faulted: bool,
    pub close_fault: Option<Arc<ApiError>>,
    pub join_failure: Option<Arc<ApiError>>,
    pub notification_error: Option<Arc<ApiError>>,
    pub attachment: Option<Arc<()>>,
}
pub(super) struct Shared {
    #[cfg(test)]
    pub join_attempts: std::sync::atomic::AtomicUsize,
    pub owner: Arc<()>,
    pub budget: ExecutionBudget,
    pub client: crate::application::api::ApplicationClientHandle,
    pub notifier: Arc<dyn CompletionNotifier>,
    pub state: Mutex<State>,
    pub changed: Condvar,
    pub worker: Mutex<Option<JoinHandle<()>>>,
}
impl Shared {
    pub fn lock(&self) -> Result<std::sync::MutexGuard<'_, State>, ApiError> {
        self.state
            .lock()
            .map_err(|_| internal("SDK execution state is unavailable"))
    }
    pub fn check(&self, id: &OperationId) -> Result<(), ApiError> {
        if !Arc::ptr_eq(&self.owner, &id.owner) {
            return Err(stale());
        }
        Ok(())
    }
    pub fn notify(&self) {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.notifier.notify()))
                .unwrap_or_else(|_| Err(internal("SDK completion notifier panicked")));
        if let Err(error) = result
            && let Ok(mut state) = self.lock()
        {
            state.notification_error = Some(Arc::new(error));
        }
    }
}
impl State {
    pub fn authorize(&self, authority: Option<&Arc<()>>) -> Result<(), ApiError> {
        match (&self.attachment, authority) {
            (None, None) => Ok(()),
            (Some(active), Some(authority)) if Arc::ptr_eq(active, authority) => Ok(()),
            _ => Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "SDK execution consumer authority is inactive",
            )),
        }
    }
    pub fn new() -> Self {
        Self {
            next: 1,
            entries: BTreeMap::new(),
            queue: VecDeque::new(),
            acknowledged: VecDeque::new(),
            request_bytes: 0,
            terminal_reserved: 0,
            closing: false,
            worker_finished: false,
            worker_joined: false,
            faulted: false,
            close_fault: None,
            join_failure: None,
            notification_error: None,
            attachment: None,
        }
    }
}
