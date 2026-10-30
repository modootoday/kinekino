//! Route pause preserves the parent lifetime and the exact pending resolve owner.
use super::*;
use crate::application::jobs::CancelJob;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct RoutePause(Arc<()>);
pub(super) struct RouteRetirement {
    ticket: RoutePause,
    cancellation_requested: bool,
}
impl TransitionSlot {
    pub(crate) fn pause_route(&self) -> Result<RoutePause, ApiError> {
        self.pause_route_after_mark(|| {})
    }
    fn pause_route_after_mark(&self, after_mark: impl FnOnce()) -> Result<RoutePause, ApiError> {
        // Close admission before waiting for any preparation already holding the state lock.
        self.paused.store(true, Ordering::Release);
        after_mark();
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        self.require_parent_live()?;
        // A resume holding this lock may have cleared the optimistic bit while we waited.
        self.paused.store(true, Ordering::Release);
        let retirement = state.retirement.get_or_insert_with(|| RouteRetirement {
            ticket: RoutePause(Arc::new(())),
            cancellation_requested: false,
        });
        Ok(retirement.ticket.clone())
    }

    pub(crate) fn poll_route_pause(
        &self,
        ticket: &RoutePause,
        jobs: &dyn JobsApi,
    ) -> Result<bool, ApiError> {
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        self.require_parent_live()?;
        require_ticket(&state, ticket)?;
        let Some((_, pending)) = &state.pending else {
            return Ok(true);
        };
        let observed = pending.observe(jobs)?;
        if observed.state.terminal() {
            state.pending = None;
            return Ok(true);
        }
        if !state
            .retirement
            .as_ref()
            .expect("validated original retirement")
            .cancellation_requested
        {
            let cancelled = jobs.cancel(CancelJob {
                job_id: observed.job_id.clone(),
                expected_attempt: observed.attempt,
            })?;
            pending.require_job(&cancelled.job)?;
            state
                .retirement
                .as_mut()
                .expect("validated original retirement")
                .cancellation_requested = true;
            if cancelled.job.state.terminal() {
                state.pending = None;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(crate) fn resume_route(
        &self,
        ticket: &RoutePause,
        verify_fresh_route: impl FnOnce() -> Result<(), ApiError>,
    ) -> Result<(), ApiError> {
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        self.require_parent_live()?;
        require_ticket(&state, ticket)?;
        if state.pending.is_some() {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Original transition cleanup is pending",
            ));
        }
        verify_fresh_route()?;
        self.require_parent_live()?;
        state.retirement = None;
        // This releases only route pause. The independent permanent parent-close bit is untouched.
        self.paused.store(false, Ordering::Release);
        self.require_parent_live()
    }
    fn require_parent_live(&self) -> Result<(), ApiError> {
        if self.closed.load(Ordering::Acquire) {
            Err(ApiError::new(
                ErrorCode::Cancelled,
                "Transition host lifetime has ended",
            ))
        } else {
            Ok(())
        }
    }
}
#[cfg(test)]
mod tests;
fn require_ticket(state: &State, ticket: &RoutePause) -> Result<(), ApiError> {
    match &state.retirement {
        Some(original) if Arc::ptr_eq(&original.ticket.0, &ticket.0) => Ok(()),
        _ => Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Original route pause identity differs",
        )),
    }
}
