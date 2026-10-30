//! Explicit application shutdown acceptance over the UI's bounded ownership carrier.
use super::native_cleanup_driver::{CleanupAdapter, CleanupPoll, drive_cleanup};
use crate::{
    application::api::ApiError, presentation::web_ui_adapter::proposals::window_cleanup::*,
};
use std::time::Duration;

pub struct ProductCleanupOutcome {
    pub owners: ProductWindowOwners,
    pub evidence: CleanupEvidence,
    pub status: CleanupStatus,
    pub acceptance_error: Option<ApiError>,
}

pub fn finish_owned_product(owners: ProductWindowOwners) -> ProductCleanupOutcome {
    let limits = CleanupLimits::new(Duration::from_secs(2), 400, Duration::from_millis(5))
        .expect("fixed positive product cleanup limits");
    finish_owned_product_with_limits(owners, limits)
}

pub fn finish_owned_product_with_limits(
    owners: ProductWindowOwners,
    limits: CleanupLimits,
) -> ProductCleanupOutcome {
    let outcome = drive_cleanup(
        ProposalCleanupAdapter(OwnedProductCleanup::new(owners, limits)),
        Duration::from_millis(5),
        std::thread::sleep,
    );
    ProductCleanupOutcome {
        owners: outcome.owners,
        evidence: outcome.evidence,
        status: outcome.status,
        acceptance_error: outcome.acceptance_error,
    }
}

struct ProposalCleanupAdapter(OwnedProductCleanup);
impl CleanupAdapter for ProposalCleanupAdapter {
    type Owners = ProductWindowOwners;
    type Evidence = CleanupEvidence;
    type Status = CleanupStatus;
    type Error = ApiError;
    fn poll(&mut self) -> CleanupPoll<CleanupStatus> {
        match self.0.poll() {
            CleanupStatus::Pending(retry) => CleanupPoll::Pending(retry),
            CleanupStatus::NeedsTerminalAcceptance => CleanupPoll::NeedsTerminalAcceptance,
            terminal => CleanupPoll::Terminal(terminal),
        }
    }
    fn accept_terminal_for_shutdown(&mut self) -> Result<(), ApiError> {
        // Existing explicit proposal quarantine policy, never a viewer acknowledgement.
        self.0.accept_initial_for_shutdown().map(|_| ())
    }
    fn acceptance_failure_status(&self) -> CleanupStatus {
        CleanupStatus::Faulted
    }
    fn into_parts(self) -> (ProductWindowOwners, CleanupEvidence) {
        self.0.into_parts()
    }
}

#[cfg(test)]
mod tests;
