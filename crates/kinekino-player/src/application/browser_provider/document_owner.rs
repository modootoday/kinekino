//! Original live document lease. Dropping this value is not native cleanup.
use super::*;
use crate::application::api::ErrorCode;
use std::time::Instant;

#[must_use = "Retain the same document owner until explicit release is observed"]
pub struct BrowserDocumentSourceOwner {
    port: Arc<dyn BrowserProviderPort>,
    ticket: BrowserTicket,
    deadline: Option<Instant>,
    closing: bool,
    cancel_admitted: bool,
    released: bool,
}
impl BrowserDocumentSourceOwner {
    pub(super) fn from_session(
        port: Arc<dyn BrowserProviderPort>,
        ticket: BrowserTicket,
        deadline: Instant,
    ) -> Self {
        Self {
            port,
            ticket,
            deadline: Some(deadline),
            closing: false,
            cancel_admitted: false,
            released: false,
        }
    }
    pub fn provider(&self) -> Arc<dyn BrowserProviderPort> {
        self.port.clone()
    }
    pub fn check_origin(
        &self,
        provider: &Arc<dyn BrowserProviderPort>,
        grant: &OwnedMediaReplay,
    ) -> Result<(), ApiError> {
        self.check_ticket(provider, grant.ticket())
    }
    pub fn check_stream(
        &self,
        provider: &Arc<dyn BrowserProviderPort>,
        stream: &OwnedMediaStream,
    ) -> Result<(), ApiError> {
        self.check_ticket(provider, &stream.ticket)
    }
    fn check_ticket(
        &self,
        provider: &Arc<dyn BrowserProviderPort>,
        ticket: &BrowserTicket,
    ) -> Result<(), ApiError> {
        if self.closing || self.released {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original document owner is releasing",
            ));
        }
        if self.deadline.is_some_and(|end| Instant::now() >= end) {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Original document owner expired",
            ));
        }
        if !Arc::ptr_eq(provider, &self.port) || ticket != &self.ticket {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Replay grant differs from original document owner",
            ));
        }
        Ok(())
    }
    fn pending(&self) -> Result<bool, ApiError> {
        if self.deadline.is_some_and(|end| Instant::now() >= end) {
            Err(ApiError::new(
                ErrorCode::Timeout,
                "Original document release remains pending",
            ))
        } else {
            Ok(false)
        }
    }
    /// One bounded admission/snapshot step; never waits, sleeps or starts a browser.
    /// Cleanup may still be observed after expiry, without extending the deadline.
    pub fn poll_release(&mut self, requested: Instant) -> Result<bool, ApiError> {
        if self.released {
            return Ok(true);
        }
        self.deadline = Some(
            self.deadline
                .map_or(requested, |original| original.min(requested)),
        );
        self.closing = true;
        let snapshot = match self.port.poll(&self.ticket) {
            Ok(snapshot) => snapshot,
            Err(error) if transient(&error) => return self.pending(),
            Err(error) => return Err(error),
        };
        if snapshot.ticket != self.ticket {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original document close snapshot changed ticket",
            ));
        }
        if !snapshot.state.terminal() {
            if !self.cancel_admitted {
                match self.port.cancel(&self.ticket) {
                    Ok(_) => self.cancel_admitted = true,
                    Err(error) if transient(&error) => return self.pending(),
                    Err(error) => return Err(error),
                }
            }
            return self.pending();
        }
        match self.port.release(&self.ticket) {
            Ok(()) => {
                self.released = true;
                Ok(true)
            }
            Err(error) if transient(&error) => self.pending(),
            Err(error) => Err(error),
        }
    }
}
fn transient(error: &ApiError) -> bool {
    error.code == ErrorCode::ResourceExhausted
        && matches!(
            error.message.as_str(),
            "Browser state is busy"
                | "Browser command queue is full"
                | "Original browser capture release is pending"
        )
}
#[cfg(test)]
mod tests;
