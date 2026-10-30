use super::*;
use crate::application::{api::ErrorCode, jobs::ExecutionControl};
use std::time::{Duration, Instant};

pub(super) fn check(control: &dyn ExecutionControl) -> Result<(), ApiError> {
    if control.cancellation_requested() {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Episode resolution cancellation requested",
        ));
    }
    if Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Episode resolution budget expired",
        ));
    }
    Ok(())
}
fn busy(error: &ApiError) -> bool {
    error.code == ErrorCode::ResourceExhausted && error.message == "Browser state is busy"
}

pub struct BrowserDocumentSession {
    pub(super) port: Arc<dyn BrowserProviderPort>,
    pub(super) ticket: BrowserTicket,
    transferred: bool,
    deadline: Instant,
}
impl BrowserDocumentSession {
    pub fn into_source_owner(mut self) -> BrowserDocumentSourceOwner {
        let owner = BrowserDocumentSourceOwner::from_session(
            self.port.clone(),
            self.ticket.clone(),
            self.deadline,
        );
        self.transferred = true;
        owner
    }
    #[cfg(feature = "source-adapter-test-support")]
    pub(super) fn fixture_admitted(
        port: Arc<dyn BrowserProviderPort>,
        ticket: BrowserTicket,
        deadline: Instant,
    ) -> Self {
        Self {
            port,
            ticket,
            transferred: false,
            deadline,
        }
    }
    pub fn start(
        port: Arc<dyn BrowserProviderPort>,
        owner: &dyn BrowserProviderOwner,
        request: ValidatedBrowserRequest,
        control: &dyn ExecutionControl,
    ) -> Result<Self, ApiError> {
        loop {
            check(control)?;
            match owner.shutdown_snapshot() {
                Ok(state) if state.state == BrowserHostState::Ready => break,
                Ok(state) if state.state == BrowserHostState::Starting => {}
                Ok(_) => {
                    return Err(ApiError::new(
                        ErrorCode::EngineFailed,
                        "Browser owner is unavailable",
                    ));
                }
                Err(error) if busy(&error) => {}
                Err(error) => return Err(error),
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let ticket = loop {
            check(control)?;
            match port.try_start(request.clone()) {
                Ok(ticket) => break ticket,
                Err(error) if busy(&error) => std::thread::sleep(Duration::from_millis(10)),
                Err(error) => return Err(error),
            }
        };
        Ok(Self {
            port,
            ticket,
            transferred: false,
            deadline: control.deadline(),
        })
    }
    pub fn snapshot(&self, control: &dyn ExecutionControl) -> Result<BrowserSnapshot, ApiError> {
        loop {
            check(control)?;
            match self.port.poll(&self.ticket) {
                Err(error) if busy(&error) => std::thread::sleep(Duration::from_millis(10)),
                Ok(snapshot) if snapshot.ticket != self.ticket => {
                    return Err(ApiError::new(
                        ErrorCode::EngineFailed,
                        "Browser snapshot ticket changed",
                    ));
                }
                result => return result,
            }
        }
    }
    pub fn until(
        &self,
        control: &dyn ExecutionControl,
        ready: impl Fn(&BrowserSnapshot) -> bool,
    ) -> Result<BrowserSnapshot, ApiError> {
        loop {
            let snapshot = self.snapshot(control)?;
            if matches!(
                snapshot.state,
                BrowserWorkState::Failed | BrowserWorkState::Cancelled
            ) {
                return Err(snapshot
                    .failure
                    .as_deref()
                    .map(copy_error)
                    .unwrap_or_else(|| {
                        ApiError::new(
                            ErrorCode::Cancelled,
                            "Browser work stopped before resolution",
                        )
                    }));
            }
            if ready(&snapshot) {
                return Ok(snapshot);
            }
            if snapshot.state.terminal() {
                return Err(ApiError::new(
                    ErrorCode::NeedsInput,
                    "Browser work ended without the required episode evidence",
                ));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    pub fn activate(
        &self,
        generation: u64,
        control: &dyn ExecutionControl,
    ) -> Result<(), ApiError> {
        loop {
            check(control)?;
            match self.port.activate_episode(&self.ticket, generation) {
                Err(error) if busy(&error) => std::thread::sleep(Duration::from_millis(10)),
                result => return result,
            }
        }
    }
}
impl Drop for BrowserDocumentSession {
    fn drop(&mut self) {
        if self.transferred {
            return;
        }
        for _ in 0..100 {
            match self.port.poll(&self.ticket) {
                Ok(snapshot) if snapshot.state.terminal() => {
                    match self.port.release(&self.ticket) {
                        Ok(()) => return,
                        Err(error)
                            if busy(&error)
                                || (error.code == ErrorCode::ResourceExhausted
                                    && error.message
                                        == "Original browser capture release is pending") => {}
                        Err(_) => return,
                    }
                }
                Ok(_) => {
                    let _ = self.port.cancel(&self.ticket);
                }
                Err(error) if busy(&error) => {}
                Err(_) => return,
            }
            std::thread::sleep(Duration::from_millis(10));
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
