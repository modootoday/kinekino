//! Retained Viewing observation over library-issued commit/submission proof only.
use super::*;
use crate::presentation::web_ui_adapter::player_shell::document::product_frame::StudioTask;
use crate::presentation::web_ui_adapter::player_shell::{NavigationError, RouteId};
use meari_native::infrastructure::{
    native_host::{NativeDocumentHandle, RetainedCommitProof},
    window::NativeFrameObservation,
};

pub(super) struct RetainedFrameExpectation {
    ticket: RouteTicket,
    route: RouteId,
    document: NativeDocumentHandle,
    proof: RetainedCommitProof,
}
impl StartRouteObservation {
    /// Original receipt/proof are captured before this fallible observer registration.
    pub(crate) fn arm_retained_viewing_route(
        &mut self,
        ticket: RouteTicket,
        document: NativeDocumentHandle,
        proof: RetainedCommitProof,
    ) -> Result<(), NavigationError> {
        self.arm_retained_route(ticket, document, proof, "view.start")
    }

    pub(crate) fn arm_retained_studio_route(
        &mut self,
        ticket: RouteTicket,
        document: NativeDocumentHandle,
        proof: RetainedCommitProof,
        task: StudioTask,
    ) -> Result<(), NavigationError> {
        self.arm_retained_route(ticket, document, proof, task.route_id())
    }

    fn arm_retained_route(
        &mut self,
        ticket: RouteTicket,
        document: NativeDocumentHandle,
        proof: RetainedCommitProof,
        expected_route: &'static str,
    ) -> Result<(), NavigationError> {
        let current = document
            .validate_retained_commit(&proof)
            .map_err(|_| NavigationError::StaleObservation)?;
        if self.terminal_window || !self.committed || self.frame_context.is_none() {
            return Err(NavigationError::RecoveryBlocked);
        }
        let route = self.state.route_target(&ticket)?.clone();
        if route.as_str() != expected_route {
            return Err(NavigationError::RouteMismatch);
        }
        if self
            .pending_retained
            .as_ref()
            .is_some_and(|pending| self.state.route_target(&pending.ticket).is_err())
        {
            self.pending_retained = None;
        }
        if let Some(pending) = &self.pending_retained {
            if pending.ticket.generation() == ticket.generation()
                && pending.route == route
                && pending.proof.receipt() == proof.receipt()
                && pending.document.validate_retained_commit(&proof).is_ok()
            {
                return Ok(());
            }
            return Err(NavigationError::Busy);
        }
        if self.pending_card.is_some() {
            return Err(NavigationError::Busy);
        }
        let committed = proof.receipt().after();
        let observed = self
            .observed_document
            .as_ref()
            .ok_or(NavigationError::RecoveryBlocked)?;
        if committed.host_id != observed.host_id
            || committed.owner != observed.owner
            || committed.definition_revision != observed.definition_revision
            || current.host_id != observed.host_id
            || current.owner != observed.owner
            || current.definition_revision != observed.definition_revision
        {
            return Err(NavigationError::ForeignObservation);
        }
        if self
            .last_card_commit
            .as_ref()
            .is_some_and(|previous| committed.document_revision <= previous.document_revision)
        {
            return Err(NavigationError::StaleObservation);
        }
        self.last_card_commit = Some(committed.clone());
        self.pending_retained = Some(RetainedFrameExpectation {
            ticket,
            route,
            document,
            proof,
        });
        Ok(())
    }

    pub(super) fn observe_retained_submission(
        &mut self,
        frame: &NativeFrameObservation,
        delivered_current: &HostStamp,
    ) {
        let Some(pending) = &self.pending_retained else {
            return;
        };
        if self.state.route_target(&pending.ticket).ok() != Some(&pending.route) {
            self.pending_retained = None;
            return;
        }
        let Some(frame_proof) = &frame.retained else {
            return;
        };
        // The notice and its proof describe the same actual submission. Delivery can
        // happen after a normal input/turn; the library validates the live lineage.
        if frame_proof.stamp() != &frame.document
            || frame_proof.ticket() != &frame.submission.ticket
        {
            return;
        }
        let Ok(current) = pending.document.validate_retained_commit(&pending.proof) else {
            return;
        };
        if current.host_id != delivered_current.host_id
            || current.owner != delivered_current.owner
            || current.definition_revision != delivered_current.definition_revision
            || delivered_current.document_revision > current.document_revision
            || delivered_current.turn_sequence > current.turn_sequence
            || frame.document.host_id != current.host_id
            || frame.document.owner != current.owner
            || frame.document.definition_revision != current.definition_revision
            || frame.document.document_revision > current.document_revision
            || frame.document.turn_sequence > current.turn_sequence
            || pending
                .document
                .validate_retained_frame(&pending.proof, frame_proof)
                .is_err()
        {
            return;
        }
        if !self.accept_ticket(&frame.submission.ticket) {
            return;
        }
        let pending = self
            .pending_retained
            .take()
            .expect("validated retained expectation");
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
            self.pending_retained = Some(pending);
        } else {
            self.observed_document = Some(current);
        }
    }
}

#[cfg(test)]
mod tests;
