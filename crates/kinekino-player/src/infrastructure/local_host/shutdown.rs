use super::executor::{DispatchState, Executor};
use std::thread::JoinHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerJoin {
    Pending,
    Joined,
    Panicked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalHostShutdownStatus {
    pub pipe: WorkerJoin,
    pub sdk: WorkerJoin,
    /// None means the dispatch state is poisoned, not that work completed.
    pub dispatch: Option<DispatchState>,
}
impl LocalHostShutdownStatus {
    pub fn is_quiescent(&self) -> bool {
        self.pipe == WorkerJoin::Joined && self.sdk == WorkerJoin::Joined
    }
}

/// Keep this original owner until both real joins are observed.
/// Drop detaches unfinished handles; it cannot prove shutdown or cancel accepted SDK work.
#[must_use = "Retain and poll this owner until both worker joins are observed"]
pub struct PendingLocalHost {
    executor: Executor,
    pipe: Option<JoinHandle<()>>,
    sdk: Option<JoinHandle<()>>,
    pipe_result: WorkerJoin,
    sdk_result: WorkerJoin,
}
impl PendingLocalHost {
    pub(super) fn new(
        executor: Executor,
        pipe: Option<JoinHandle<()>>,
        sdk: Option<JoinHandle<()>>,
    ) -> Self {
        Self {
            executor,
            pipe,
            sdk,
            pipe_result: WorkerJoin::Pending,
            sdk_result: WorkerJoin::Pending,
        }
    }
    /// Never waits for an unfinished worker; independent joins are sticky.
    pub fn poll(&mut self) -> LocalHostShutdownStatus {
        join_finished(&mut self.pipe, &mut self.pipe_result);
        join_finished(&mut self.sdk, &mut self.sdk_result);
        LocalHostShutdownStatus {
            pipe: self.pipe_result,
            sdk: self.sdk_result,
            dispatch: self.executor.state(),
        }
    }
}
fn join_finished(thread: &mut Option<JoinHandle<()>>, result: &mut WorkerJoin) {
    if thread.as_ref().is_some_and(JoinHandle::is_finished) {
        *result = match thread.take().expect("finished worker handle").join() {
            Ok(()) => WorkerJoin::Joined,
            Err(_) => WorkerJoin::Panicked,
        };
    }
}
