//! One cleanup scheduling loop over an adapter's existing bounded ownership protocol.
use meari_native::infrastructure::native_host::HostRetry;
use std::time::Duration;

pub enum CleanupPoll<S> {
    Pending(HostRetry),
    NeedsTerminalAcceptance,
    Terminal(S),
}
/// Owns all cleanup authority and its budget; poll must preserve the original bounded contract.
/// The driver neither retries failed cleanup steps nor invents SDK/endpoint/OS completion.
pub trait CleanupAdapter {
    type Owners;
    type Evidence;
    type Status;
    type Error;
    fn poll(&mut self) -> CleanupPoll<Self::Status>;
    fn accept_terminal_for_shutdown(&mut self) -> Result<(), Self::Error>;
    fn acceptance_failure_status(&self) -> Self::Status;
    fn into_parts(self) -> (Self::Owners, Self::Evidence);
}
#[must_use = "Return original owners and evidence even on budget exhaustion or acceptance failure"]
pub struct CleanupDriverOutcome<A: CleanupAdapter> {
    pub owners: A::Owners,
    pub evidence: A::Evidence,
    pub status: A::Status,
    pub acceptance_error: Option<A::Error>,
}

pub fn drive_cleanup<A: CleanupAdapter>(
    mut adapter: A,
    on_wake_delay: Duration,
    mut sleep: impl FnMut(Duration),
) -> CleanupDriverOutcome<A> {
    let mut acceptance_error = None;
    let status = loop {
        match adapter.poll() {
            CleanupPoll::Pending(HostRetry::After(delay)) => sleep(delay),
            CleanupPoll::Pending(HostRetry::OnWake) => sleep(on_wake_delay),
            CleanupPoll::NeedsTerminalAcceptance => {
                if let Err(error) = adapter.accept_terminal_for_shutdown() {
                    acceptance_error = Some(error);
                    break adapter.acceptance_failure_status();
                }
            }
            CleanupPoll::Terminal(status) => break status,
        }
    };
    let (owners, evidence) = adapter.into_parts();
    CleanupDriverOutcome {
        owners,
        evidence,
        status,
        acceptance_error,
    }
}

#[cfg(test)]
mod tests;
