//! One original slot and its exact explicit end result; no copied admission permission or Drop hook.
use crate::application::{
    api::ApiError, jobs::JobId, playback_session::transition::TransitionSlot,
};
use std::sync::Arc;

pub(crate) struct TransitionLifetime {
    slot: Arc<TransitionSlot>,
    ended: Option<Result<Option<JobId>, ApiError>>,
}
impl TransitionLifetime {
    pub(super) fn new() -> Self {
        Self {
            slot: Arc::new(TransitionSlot::default()),
            ended: None,
        }
    }
    pub(super) fn slot(&self) -> Arc<TransitionSlot> {
        self.slot.clone()
    }
    pub(super) fn close_admission(&self) {
        self.slot.close_admission();
    }
    /// Only after original native/execution cleanup (or a pre-window setup that never ran).
    pub(super) fn finish_once(&mut self) {
        if self.ended.is_none() {
            self.ended = Some(self.slot.end());
        }
    }
    pub(crate) fn admission_closed(&self) -> bool {
        self.slot.admission_closed()
    }
    pub(crate) fn end_result(&self) -> Option<&Result<Option<JobId>, ApiError>> {
        self.ended.as_ref()
    }
    pub(super) fn ended_successfully(&self) -> bool {
        matches!(self.ended, Some(Ok(_)))
    }
}

#[cfg(test)]
mod tests;
