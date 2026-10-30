//! One original UI-thread timer owner; no thread, SDK execution, or successful HostRetry.
use super::super::viewer_callbacks::progression::Continuation;
use meari_native::infrastructure::{
    native_host::{HostAccessError, NativeDocumentHandle},
    window::{NativeHostTimer, NativeHostTimerToken, NativeWindowWake},
};
use std::time::Instant;

pub(super) struct AutomaticWake {
    timer: NativeHostTimer,
    armed: Option<(Continuation, NativeHostTimerToken, Instant)>,
}
impl AutomaticWake {
    pub(super) fn new(
        wake: &NativeWindowWake,
        document: &NativeDocumentHandle,
    ) -> Result<Self, HostAccessError> {
        Ok(Self {
            timer: wake.timer(document)?,
            armed: None,
        })
    }
    pub(super) fn reconcile(&mut self, next: Option<Continuation>) -> Result<(), HostAccessError> {
        if self.armed.as_ref().map(|(plan, _, _)| plan) == next.as_ref() {
            return Ok(()); // Unrelated completions must not keep postponing this same operation.
        }
        if let Some(next) = next {
            let delay = next.action.delay();
            let deadline = Instant::now() + delay;
            let token = self.timer.schedule_after(delay)?;
            self.armed = Some((next, token, deadline));
        } else {
            if let Some((_, token, _)) = &self.armed {
                self.timer.cancel(token)?;
            }
            self.armed = None;
        }
        Ok(())
    }
    /// Completion is shared with SDK events; only consume our own due plan, once.
    pub(super) fn take_due(
        &mut self,
        now: Instant,
    ) -> Result<Option<Continuation>, HostAccessError> {
        if !self
            .armed
            .as_ref()
            .is_some_and(|(_, _, deadline)| now >= *deadline)
        {
            return Ok(None);
        }
        let Some((_, token, _)) = self.armed.as_ref() else {
            return Ok(None);
        };
        self.timer.cancel(token)?;
        Ok(self.armed.take().map(|(plan, _, _)| plan))
    }
}
