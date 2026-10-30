use super::{NativeEpisodeEngine, ParentBinding, conflict, internal};
use crate::{
    application::{api::ApiError, playback_session::ClosedEpisodePlayback},
    infrastructure::{
        mpv::{ParentCleanupObservation, ParentCleanupStatus, ParentCleanupTask},
        thread_completion::{CompletionWake, DispatcherHandle},
    },
};
use std::{
    collections::VecDeque,
    fmt,
    num::NonZeroU64,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
};

pub(super) struct Shared {
    pub state: Mutex<State>,
    pub dispatcher: DispatcherHandle,
}
pub(super) struct State {
    pub sequence: u64,
    pub phase: Phase,
}
pub(super) enum Phase {
    Empty,
    Prepared(Route),
    Active(Route),
    Retiring(Arc<RetirementOwner>),
}
pub(super) struct Route {
    pub handle: ParentRouteHandle,
}
#[derive(Clone)]
pub struct ParentRouteHandle {
    router: Weak<Shared>,
    pub(super) epoch: NonZeroU64,
    pub(super) binding: ParentBinding,
}
impl ParentRouteHandle {
    pub(super) fn new(router: &Arc<Shared>, epoch: NonZeroU64, binding: ParentBinding) -> Self {
        Self {
            router: Arc::downgrade(router),
            epoch,
            binding,
        }
    }
    pub fn generation(&self) -> u64 {
        self.epoch.get()
    }
    pub(super) fn belongs(&self, router: &Arc<Shared>) -> bool {
        Weak::ptr_eq(&self.router, &Arc::downgrade(router))
    }
    pub(super) fn same(&self, other: &Self) -> bool {
        self.epoch == other.epoch && Weak::ptr_eq(&self.router, &other.router)
    }
}
impl fmt::Debug for ParentRouteHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParentRouteHandle")
            .field("generation", &self.generation())
            .finish_non_exhaustive()
    }
}
pub struct RoutePreparationFailure {
    pub error: ApiError,
    pub engine: Arc<NativeEpisodeEngine>,
}
impl fmt::Debug for RoutePreparationFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RoutePreparationFailure")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}
#[derive(Clone)]
#[must_use = "retain retirement ownership and observe the actual joined result"]
pub struct RetirementHandle {
    pub(super) owner: Arc<RetirementOwner>,
}
impl fmt::Debug for RetirementHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RetirementHandle")
            .field("route", &self.owner.route)
            .finish_non_exhaustive()
    }
}
#[derive(Debug)]
pub struct RetirementStart {
    pub handle: RetirementHandle,
    pub dispatch_error: Option<Arc<ApiError>>,
}
#[derive(Clone, Debug)]
pub enum RetirementFailure {
    Dispatch(Arc<ApiError>),
    Engine(Arc<ApiError>),
    Incomplete(ClosedEpisodePlayback),
}
#[derive(Clone, Debug)]
pub enum RetirementProgress {
    NotStarted,
    Pending,
    RetryableFailure,
    Complete(Option<ClosedEpisodePlayback>),
}
#[derive(Clone, Debug)]
pub struct RetirementObservation {
    pub progress: RetirementProgress,
    pub retained_failure: Option<RetirementFailure>,
    pub dispatch_error: Option<Arc<ApiError>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouterStatus {
    EmptyNeverReady,
    Prepared { generation: u64 },
    Active { generation: u64 },
    Retiring { generation: u64 },
    JoinedCompleteNoRoute { generation: u64 },
}

type WakeQueue = Arc<Mutex<VecDeque<(u64, CompletionWake)>>>;
struct TaskState {
    task: ParentCleanupTask,
    started: bool,
    attempt: u64,
    wakes: WakeQueue,
    last_failure: Option<RetirementFailure>,
    dispatch_error: Option<Arc<ApiError>>,
}
pub(super) struct RetirementOwner {
    pub route: ParentRouteHandle,
    state: Mutex<TaskState>,
    joined_complete: AtomicBool,
}
impl RetirementOwner {
    pub(super) fn new(route: ParentRouteHandle, dispatcher: DispatcherHandle) -> Self {
        let wakes: WakeQueue = Arc::new(Mutex::new(VecDeque::new()));
        let pending = wakes.clone();
        let wake: CompletionWake = Arc::new(move || {
            let next = pending
                .lock()
                .map_err(|_| "Retirement wake queue poisoned".to_string())?
                .pop_front()
                .ok_or_else(|| "Retirement notification has no admitted attempt".to_string())?
                .1;
            next()
        });
        let task = ParentCleanupTask::new(route.binding.clone(), route.epoch, dispatcher, wake);
        Self {
            route,
            state: Mutex::new(TaskState {
                task,
                started: false,
                attempt: 0,
                wakes,
                last_failure: None,
                dispatch_error: None,
            }),
            joined_complete: AtomicBool::new(false),
        }
    }
    pub(super) fn complete(&self) -> bool {
        self.joined_complete.load(Ordering::Acquire)
    }
    pub(super) fn start_once(
        &self,
        wake: CompletionWake,
    ) -> Result<Option<Arc<ApiError>>, ApiError> {
        let mut state = self.state.lock().map_err(|_| internal())?;
        if state.started {
            return Ok(state.dispatch_error.clone());
        }
        state.start(wake)
    }
    pub(super) fn observe(&self) -> Result<RetirementObservation, ApiError> {
        let mut state = self.state.lock().map_err(|_| internal())?;
        let observation = state.observe();
        if matches!(observation.progress, RetirementProgress::Complete(_)) {
            self.joined_complete.store(true, Ordering::Release);
        }
        Ok(observation)
    }
    pub(super) fn retry(&self, wake: CompletionWake) -> Result<(), ApiError> {
        let mut state = self.state.lock().map_err(|_| internal())?;
        let observed = state.observe();
        match observed.progress {
            RetirementProgress::Complete(_) => {
                self.joined_complete.store(true, Ordering::Release);
                return Err(conflict("Parent cleanup is already complete"));
            }
            RetirementProgress::Pending => {
                return Err(conflict("Original parent cleanup is still in flight"));
            }
            RetirementProgress::NotStarted if !state.started => {
                return Err(conflict("Initial retirement has not started"));
            }
            _ => {}
        }
        match state.start(wake)? {
            Some(error) => Err(copy_error(&error)),
            None => Ok(()),
        }
    }
}
impl TaskState {
    fn start(&mut self, wake: CompletionWake) -> Result<Option<Arc<ApiError>>, ApiError> {
        let attempt = self
            .attempt
            .checked_add(1)
            .ok_or_else(|| conflict("Retirement attempt exhausted"))?;
        self.wakes
            .lock()
            .map_err(|_| internal())?
            .push_back((attempt, wake));
        self.started = true;
        self.attempt = attempt;
        match self.task.start_attempt() {
            Ok(()) => {
                self.dispatch_error = None;
                Ok(None)
            }
            Err(error) => {
                // Rejected submission starts no worker. Remove only this attempt's wake.
                self.wakes
                    .lock()
                    .map_err(|_| internal())?
                    .retain(|(id, _)| *id != attempt);
                let error = Arc::new(error);
                self.dispatch_error = Some(error.clone());
                if self.last_failure.is_none() {
                    self.last_failure = Some(RetirementFailure::Dispatch(error.clone()));
                }
                Ok(Some(error))
            }
        }
    }
    fn observe(&mut self) -> RetirementObservation {
        if let Some(error) = &self.dispatch_error {
            return RetirementObservation {
                progress: RetirementProgress::RetryableFailure,
                retained_failure: self.last_failure.clone(),
                dispatch_error: Some(error.clone()),
            };
        }
        let progress = match self.task.observe() {
            ParentCleanupObservation::NotStarted if self.started => {
                RetirementProgress::RetryableFailure
            }
            ParentCleanupObservation::NotStarted => RetirementProgress::NotStarted,
            ParentCleanupObservation::Pending => RetirementProgress::Pending,
            ParentCleanupObservation::Finished(Ok(ParentCleanupStatus::Complete(receipt))) => {
                RetirementProgress::Complete(receipt.clone())
            }
            ParentCleanupObservation::Finished(Ok(ParentCleanupStatus::Incomplete(receipt))) => {
                self.last_failure = Some(RetirementFailure::Incomplete(receipt.clone()));
                RetirementProgress::RetryableFailure
            }
            ParentCleanupObservation::Finished(Ok(ParentCleanupStatus::Pending)) => {
                self.last_failure = Some(RetirementFailure::Engine(Arc::new(conflict(
                    "Joined cleanup returned pending without future work",
                ))));
                RetirementProgress::RetryableFailure
            }
            ParentCleanupObservation::Finished(Err(error)) => {
                self.last_failure = Some(RetirementFailure::Engine(Arc::new(copy_error(error))));
                RetirementProgress::RetryableFailure
            }
        };
        RetirementObservation {
            progress,
            retained_failure: self.last_failure.clone(),
            dispatch_error: None,
        }
    }
}
fn copy_error(error: &ApiError) -> ApiError {
    ApiError {
        code: error.code,
        message: error.message.clone(),
        retryable: error.retryable,
        details: error.details.clone(),
        remediation: error.remediation.clone(),
        request_id: error.request_id.clone(),
    }
}
