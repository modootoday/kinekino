//! A Card props receipt arms an expectation; only a later checked frame reports the route.
use super::*;
use crate::presentation::web_ui_adapter::player_shell::{NavigationError, RouteId};
use meari_native::infrastructure::native_host::HostCommitReceipt;

pub(super) struct CardFrameExpectation {
    ticket: RouteTicket,
    route: RouteId,
    committed: HostStamp,
}
impl StartRouteObservation {
    pub(crate) fn arm_jobs_route(
        &mut self,
        ticket: RouteTicket,
        receipt: &HostCommitReceipt,
        current: &HostStamp,
    ) -> Result<(), NavigationError> {
        self.arm_workspace_route(ticket, receipt, current, "studio.jobs")
    }
    pub(crate) fn arm_card_route(
        &mut self,
        ticket: RouteTicket,
        receipt: &HostCommitReceipt,
        current: &HostStamp,
    ) -> Result<(), NavigationError> {
        self.arm_workspace_route(ticket, receipt, current, "studio.cards")
    }
    pub(crate) fn arm_viewing_route(
        &mut self,
        ticket: RouteTicket,
        receipt: &HostCommitReceipt,
        current: &HostStamp,
    ) -> Result<(), NavigationError> {
        self.arm_workspace_route(ticket, receipt, current, "view.start")
    }
    fn arm_workspace_route(
        &mut self,
        ticket: RouteTicket,
        receipt: &HostCommitReceipt,
        current: &HostStamp,
        expected_route: &str,
    ) -> Result<(), NavigationError> {
        if self.terminal_window || !self.committed || self.frame_context.is_none() {
            return Err(NavigationError::RecoveryBlocked);
        }
        if self
            .pending_card
            .as_ref()
            .is_some_and(|pending| self.state.route_target(&pending.ticket).is_err())
        {
            self.pending_card = None;
        }
        if self.pending_card.is_some() || self.pending_retained.is_some() {
            return Err(NavigationError::Busy);
        }
        let route = self.state.route_target(&ticket)?.clone();
        if route.as_str() != expected_route {
            return Err(NavigationError::RouteMismatch);
        }
        let before = receipt.before();
        let after = receipt.after();
        let observed = self
            .observed_document
            .as_ref()
            .ok_or(NavigationError::RecoveryBlocked)?;
        if after.host_id != observed.host_id
            || after.owner != observed.owner
            || after.definition_revision != observed.definition_revision
        {
            return Err(NavigationError::ForeignObservation);
        }
        if self
            .last_card_commit
            .as_ref()
            .is_some_and(|previous| after.document_revision <= previous.document_revision)
        {
            return Err(NavigationError::StaleObservation);
        }
        if after != current
            || before.host_id != after.host_id
            || before.owner != after.owner
            || before.definition_revision != after.definition_revision
            || before.document_revision.checked_add(1) != Some(after.document_revision)
        {
            return Err(NavigationError::StaleObservation);
        }
        self.pending_card = Some(CardFrameExpectation {
            ticket,
            route,
            committed: after.clone(),
        });
        self.last_card_commit = Some(after.clone());
        Ok(())
    }
    pub(super) fn observe_card_submission(&mut self, current: &HostStamp, frame: &FrameTicket) {
        let Some(pending) = &self.pending_card else {
            return;
        };
        if self.state.route_target(&pending.ticket).ok() != Some(&pending.route) {
            self.pending_card = None;
            return;
        }
        let committed = &pending.committed;
        if current.host_id != committed.host_id
            || current.owner != committed.owner
            || current.definition_revision != committed.definition_revision
            || current.document_revision < committed.document_revision
        {
            return;
        }
        if !self.accept_ticket(frame) {
            return;
        }
        let Some(pending) = self.pending_card.take() else {
            return;
        };
        if self
            .state
            .observe_route(
                &pending.ticket,
                RouteHostObservation::Reported {
                    route: pending.route.clone(),
                },
            )
            .is_err()
        {
            self.pending_card = Some(pending);
        } else {
            self.observed_document = Some(current.clone());
        }
    }
}

#[cfg(test)]
#[path = "card/tests.rs"]
mod tests;
