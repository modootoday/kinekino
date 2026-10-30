//! Two-phase UI selection: the active inspector changes only after owner handoff/DOM commit.
use super::list::JobsListProjection;
use crate::application::jobs::JobId;
use std::num::NonZeroU64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionStamp {
    route: NonZeroU64,
    generation: NonZeroU64,
    job: Option<JobId>,
}
impl SelectionStamp {
    pub fn job(&self) -> Option<&JobId> {
        self.job.as_ref()
    }
    pub fn route(&self) -> NonZeroU64 {
        self.route
    }
    pub fn generation(&self) -> NonZeroU64 {
        self.generation
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionError {
    Busy,
    Closing,
    Stale,
    ForeignRoute,
    UnknownRow,
    CounterExhausted,
}
#[derive(Clone)]
pub struct JobsSelection {
    route: NonZeroU64,
    sequence: u64,
    result_floor: u64,
    active: Option<SelectionStamp>,
    pending: Option<SelectionStamp>,
    closing: bool,
}
impl JobsSelection {
    pub fn new(route: NonZeroU64) -> Self {
        Self {
            route,
            sequence: 0,
            result_floor: 0,
            active: None,
            pending: None,
            closing: false,
        }
    }
    pub fn request(
        &mut self,
        page: &JobsListProjection,
        job: Option<JobId>,
    ) -> Result<SelectionStamp, SelectionError> {
        if self.closing {
            return Err(SelectionError::Closing);
        }
        if self.pending.is_some() || page.disabled() {
            return Err(SelectionError::Busy);
        }
        if self.route != page.route() {
            return Err(SelectionError::ForeignRoute);
        }
        if job.as_ref().is_some_and(|job| !page.contains(job)) {
            return Err(SelectionError::UnknownRow);
        }
        let next = self
            .sequence
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .ok_or(SelectionError::CounterExhausted)?;
        let stamp = SelectionStamp {
            route: self.route,
            generation: next,
            job,
        };
        self.sequence = next.get();
        // Aborting a failed handoff may keep the old visual snapshot, never revive its callbacks.
        self.result_floor = next.get();
        self.pending = Some(stamp.clone());
        Ok(stamp)
    }
    pub fn commit(&mut self, stamp: &SelectionStamp) -> Result<(), SelectionError> {
        if self.closing || self.pending.as_ref() != Some(stamp) {
            return Err(SelectionError::Stale);
        }
        self.active = self.pending.take();
        Ok(())
    }
    pub fn abort(&mut self, stamp: &SelectionStamp) -> Result<(), SelectionError> {
        if self.pending.as_ref() != Some(stamp) {
            return Err(SelectionError::Stale);
        }
        self.pending = None;
        Ok(())
    }
    pub fn active(&self) -> Option<&SelectionStamp> {
        self.active.as_ref()
    }
    pub fn pending(&self) -> Option<&SelectionStamp> {
        self.pending.as_ref()
    }
    pub fn accepts_result(&self, stamp: &SelectionStamp) -> bool {
        !self.closing
            && self.pending.is_none()
            && self.active.as_ref() == Some(stamp)
            && stamp.generation.get() >= self.result_floor
    }
    pub fn begin_close(&mut self) {
        self.closing = true;
    }
}
