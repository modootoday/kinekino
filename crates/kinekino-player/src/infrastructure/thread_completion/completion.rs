use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

pub type CompletionWake = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinFailure {
    Spawn(String),
    WorkerPanicked,
    DispatcherPanicked,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationState {
    Pending,
    Calling,
    Returned,
    Failed(String),
}

pub(super) struct Job {
    pub run: Box<dyn FnOnce() -> bool + Send>,
    pub fault: Arc<AtomicBool>,
    pub wake: CompletionWake,
    pub notification: Arc<Mutex<NotificationState>>,
}
struct State<T> {
    result: Mutex<Option<Result<T, JoinFailure>>>,
    fault: Arc<AtomicBool>,
    notification: Arc<Mutex<NotificationState>>,
}
/// A queued or rejected request still owns the work closure. No child has started yet.
pub struct JoinRequest<T> {
    pub(super) job: Job,
    pub(super) ticket: JoinTicket<T>,
}
pub struct JoinTicket<T> {
    state: Arc<State<T>>,
    consumed: bool,
}
impl<T> std::fmt::Debug for JoinTicket<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JoinTicket")
            .field("consumed", &self.consumed)
            .finish_non_exhaustive()
    }
}

pub(super) fn notify(wake: &CompletionWake, state: &Mutex<NotificationState>) -> bool {
    {
        let mut status = state.lock().unwrap_or_else(|error| error.into_inner());
        match &*status {
            NotificationState::Returned => return true,
            NotificationState::Failed(_) | NotificationState::Calling => return false,
            NotificationState::Pending => *status = NotificationState::Calling,
        }
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| wake()));
    let status = match result {
        Ok(Ok(())) => NotificationState::Returned,
        Ok(Err(error)) => NotificationState::Failed(error),
        Err(_) => NotificationState::Failed("Completion notification panicked".into()),
    };
    let returned = status == NotificationState::Returned;
    *state.lock().unwrap_or_else(|error| error.into_inner()) = status;
    returned
}

impl<T: Send + 'static> JoinRequest<T> {
    pub fn new(work: impl FnOnce() -> T + Send + 'static, wake: CompletionWake) -> Self {
        let state = Arc::new(State {
            result: Mutex::new(None),
            fault: Arc::new(AtomicBool::new(false)),
            notification: Arc::new(Mutex::new(NotificationState::Pending)),
        });
        let result_state = state.clone();
        let work_wake = wake.clone();
        let job = Job {
            fault: state.fault.clone(),
            wake,
            notification: state.notification.clone(),
            run: Box::new(move || {
                let result = match std::thread::Builder::new()
                    .name("owned-completion-worker".into())
                    .spawn(work)
                {
                    Ok(worker) => worker.join().map_err(|_| JoinFailure::WorkerPanicked),
                    Err(error) => Err(JoinFailure::Spawn(error.to_string())),
                };
                // Only this internal metadata is locked; work and notification run outside it.
                *result_state
                    .result
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()) = Some(result);
                notify(&work_wake, &result_state.notification)
            }),
        };
        Self {
            job,
            ticket: JoinTicket {
                state,
                consumed: false,
            },
        }
    }
}
impl<T> JoinTicket<T> {
    /// Some means the child was joined, failed to spawn, or the dispatcher faulted.
    /// A dispatcher fault never upgrades an absent result into successful completion.
    pub fn try_take(&mut self) -> Option<Result<T, JoinFailure>> {
        if self.consumed {
            return None;
        }
        let result = self
            .state
            .result
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        if result.is_some() {
            self.consumed = true;
            return result;
        }
        if self.state.fault.load(Ordering::Acquire) {
            self.consumed = true;
            return Some(Err(JoinFailure::DispatcherPanicked));
        }
        None
    }
    /// Returned is callback return, not proof that a GUI consumed the event.
    pub fn notification(&self) -> NotificationState {
        self.state
            .notification
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
}
