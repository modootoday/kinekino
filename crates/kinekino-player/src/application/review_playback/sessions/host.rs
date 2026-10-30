//! Trusted host shutdown is independent of disappearing case/media authorization.
use super::*;
use std::sync::atomic::Ordering;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ShutdownObservation {
    pub owners: usize,
    pub physically_finished: usize,
    pub complete: bool,
}
impl ReviewPlaybackSessions {
    /// Linearizes against open/control under the same entries lock. Requesting
    /// close is not physical completion; keep this service until observation.
    pub fn begin_close(&self) -> Result<(), ApiError> {
        let mut entries = self.entries.lock().map_err(|_| poisoned())?;
        self.closing.store(true, Ordering::Release);
        let mut first = None;
        for entry in entries.iter_mut() {
            if entry.terminal.is_none() && !entry.close_requested {
                match entry.owner.request_close() {
                    Ok(()) => entry.close_requested = true,
                    Err(error) => {
                        first.get_or_insert_with(|| entry.retained_error(error));
                    }
                }
            }
        }
        first.map_or(Ok(()), Err)
    }

    /// No case lookup: a revoked or missing input must not prevent cleanup of
    /// the original owner. Observation failure preserves that owner and error.
    pub fn observe_shutdown(&self) -> Result<ShutdownObservation, ApiError> {
        let mut entries = self.entries.lock().map_err(|_| poisoned())?;
        if !self.closing.load(Ordering::Acquire) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Begin host close before observing shutdown",
            ));
        }
        let mut physically_finished = 0;
        let mut first = None;
        for entry in entries.iter_mut() {
            match entry.view() {
                Ok(view) if view.observation.quiescent() => physically_finished += 1,
                Ok(_) => {}
                Err(error) => {
                    first.get_or_insert(error);
                }
            }
        }
        if let Some(error) = first {
            return Err(error);
        }
        Ok(ShutdownObservation {
            owners: entries.len(),
            physically_finished,
            complete: physically_finished == entries.len(),
        })
    }
}
