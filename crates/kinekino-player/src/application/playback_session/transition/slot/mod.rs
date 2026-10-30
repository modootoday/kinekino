//! Host-owned single transition slot. No detached task or automatic retry.
use super::{
    resolve_job::PendingAdjacentResolution,
    service::{EpisodeReplacementOwner, ReplacedEpisode},
};
use crate::application::{
    api::{ApiError, ErrorCode},
    episode_resolve::EpisodeResolutionResults,
    jobs::{JobId, JobRecord, JobState, JobsApi},
};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
pub struct TransitionSlot {
    closed: AtomicBool,
    paused: AtomicBool,
    state: Mutex<State>,
}
#[derive(Default)]
struct State {
    next: u64,
    pending: Option<(u64, PendingAdjacentResolution)>,
    #[cfg(any(test, feature = "native-proposal-window"))]
    retirement: Option<RouteRetirement>,
}
#[cfg(any(test, feature = "native-proposal-window"))]
mod retirement;
#[cfg(feature = "native-proposal-window")]
pub(crate) use retirement::RoutePause;
#[cfg(any(test, feature = "native-proposal-window"))]
use retirement::RouteRetirement;
impl TransitionSlot {
    /// Stop new begin/observe/commit admission without waiting on an in-flight slot lock.
    /// The original pending owner remains in state until end drains it after native cleanup.
    pub fn close_admission(&self) {
        self.closed.store(true, Ordering::Release);
    }
    /// Original admission observation, including revocation through another alias of this slot.
    pub fn admission_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
    /// Reserve before submitting a job: concurrent starts cannot create hidden extra jobs.
    /// The closure must only call the shared prepare/submit use case.
    pub fn begin(
        &self,
        prepare: impl FnOnce() -> Result<PendingAdjacentResolution, ApiError>,
    ) -> Result<(u64, JobId), ApiError> {
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        self.require_open()?;
        if state.pending.is_some() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "A transition is already owned",
            ));
        }
        let sequence = state.next.checked_add(1).ok_or_else(|| {
            ApiError::new(ErrorCode::ResourceExhausted, "Transition handle exhausted")
        })?;
        let pending = prepare()?;
        let job_id = pending.job_id().clone();
        // Retain any admitted job even if EOF raced; end() must return it for host cleanup.
        state.next = sequence;
        state.pending = Some((sequence, pending));
        self.require_open()?;
        Ok((sequence, job_id))
    }
    pub fn observe(&self, handle: u64, jobs: &dyn JobsApi) -> Result<JobRecord, ApiError> {
        let state = self.state.lock().map_err(|_| unavailable())?;
        self.require_open()?;
        require_pending(&state, handle)?.observe(jobs)
    }
    pub fn commit(
        &self,
        handle: u64,
        jobs: &dyn JobsApi,
        resolutions: &dyn EpisodeResolutionResults,
        owner: &dyn EpisodeReplacementOwner,
    ) -> Result<ReplacedEpisode, ApiError> {
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        self.require_open()?;
        let pending = require_pending(&state, handle)?;
        let observed = pending.observe(jobs)?;
        if observed.state != JobState::Completed {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Transition resolve is not completed",
            ));
        }
        self.require_open()?;
        let result = pending.commit(jobs, resolutions, owner);
        // One commit attempt only, including an observed engine failure. Never replay effects.
        state.pending = None;
        result
    }
    /// Forgetting a preparation does not claim that its durable job was cancelled.
    /// The caller can explicitly cancel this job using the existing jobs API.
    pub fn release(&self, handle: u64) -> Result<JobId, ApiError> {
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        self.require_open()?;
        let job_id = require_pending(&state, handle)?.job_id().clone();
        state.pending = None;
        Ok(job_id)
    }
    /// Assembly calls native permanent shutdown first, then drains this slot and jobs host.
    /// Return the admitted job to the owner even when begin raced with EOF.
    pub fn end(&self) -> Result<Option<JobId>, ApiError> {
        self.close_admission();
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        Ok(state
            .pending
            .take()
            .map(|(_, pending)| pending.job_id().clone()))
    }
    fn require_open(&self) -> Result<(), ApiError> {
        if self.closed.load(Ordering::Acquire) {
            Err(ApiError::new(
                ErrorCode::Cancelled,
                "Transition host lifetime has ended",
            ))
        } else if self.paused.load(Ordering::Acquire) {
            Err(ApiError::new(
                ErrorCode::Cancelled,
                "Original Viewing route is retiring",
            ))
        } else {
            Ok(())
        }
    }
}
fn require_pending(state: &State, handle: u64) -> Result<&PendingAdjacentResolution, ApiError> {
    state
        .pending
        .as_ref()
        .filter(|(id, _)| *id == handle)
        .map(|(_, pending)| pending)
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::NotFound,
                "Transition handle is not owned by this host",
            )
        })
}
fn unavailable() -> ApiError {
    ApiError::new(ErrorCode::Internal, "Transition ownership lock unavailable")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closing_admission_never_waits_for_the_in_flight_slot_lock_or_reopens_it() {
        let slot = TransitionSlot::default();
        let held = slot.state.lock().unwrap();
        slot.close_admission();
        assert!(slot.closed.load(Ordering::Acquire));
        drop(held);
        assert_eq!(
            slot.begin(|| panic!("closed admission must not submit another job"))
                .unwrap_err()
                .code,
            ErrorCode::Cancelled
        );
        assert!(slot.end().unwrap().is_none());
        slot.close_admission();
        assert!(slot.end().unwrap().is_none());
    }
    #[test]
    fn ended_host_never_submits_or_resurrects_a_transition() {
        let slot = TransitionSlot::default();
        assert!(slot.end().unwrap().is_none());
        assert!(slot.end().unwrap().is_none());
        let error = slot
            .begin(|| panic!("ended host must not prepare or submit a job"))
            .err()
            .unwrap();
        assert_eq!(error.code, ErrorCode::Cancelled);
        assert_eq!(slot.release(1).unwrap_err().code, ErrorCode::Cancelled);
    }
    #[test]
    fn exhausted_handles_reject_before_preparation_and_do_not_wrap() {
        let slot = TransitionSlot {
            closed: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            state: Mutex::new(State {
                next: u64::MAX,
                pending: None,
                retirement: None,
            }),
        };
        let error = slot
            .begin(|| panic!("exhaustion must not submit a hidden job"))
            .err()
            .unwrap();
        assert_eq!(error.code, ErrorCode::ResourceExhausted);
        assert_eq!(slot.state.lock().unwrap().next, u64::MAX);
        assert_eq!(slot.release(0).unwrap_err().code, ErrorCode::NotFound);
    }
}
