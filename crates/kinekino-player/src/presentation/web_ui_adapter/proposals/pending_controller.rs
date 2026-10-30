//! Async review orchestration using the existing ReviewSession state transitions.
pub use super::intent_queue::output_review_intents::{
    DecisionIdSource, OutputReviewAction, OutputReviewField, OutputReviewIntent,
};
use super::pending_host::{
    PendingOutput, PendingRegistryProposalHost, PendingRequest, ProposalAdmission,
};
use super::{MutationReceipt, PendingToken, ReviewScope, ReviewSession, ViewError};
use super::{
    intent_queue::output_review_intents::{self, OutputReviewIntentError, OutputReviewInteraction},
    output_review_state::{OutputReviewState, OutputReviewToken},
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_player::application::proposals::InspectProposal;
use meari::application::plugins::{PluginError, PluginTicket};
mod raw_recovery;
pub use raw_recovery::RawRecovery;
use raw_recovery::RecoveryRead;

#[derive(Clone, Copy)]
enum Phase {
    Save,
    Accept,
    Inspect,
}
struct Active {
    ticket: PluginTicket,
    kind: ActiveKind,
}
#[must_use = "Retain original typed terminal, selection token and real transport identity"]
pub struct RawReviewRetained {
    pub rejected: Box<super::raw_review_state::RawReviewRejected>,
    pub transport: Option<(PluginTicket, bool)>,
}

#[must_use = "Preserve the original output-review token and mismatched typed terminal"]
pub struct OutputReviewScopeMismatch {
    pub token: OutputReviewToken,
    pub outcome: Result<PendingOutput, ApiError>,
}
enum ActiveKind {
    RawRecovery {
        generation: u64,
        phase: RecoveryRead,
        scope: ReviewScope,
    },
    RawReview(super::raw_review_state::RawReviewToken),
    Proposal {
        token: PendingToken,
        phase: Phase,
    },
    OutputReview(OutputReviewToken),
}
#[derive(Debug)]
pub enum PendingControllerError {
    Plugin(PluginError),
    View(ViewError),
}
pub struct PendingReviewController {
    host: PendingRegistryProposalHost,
    session: ReviewSession,
    active: Option<Active>,
    output_review: Option<OutputReviewInteraction>,
    raw_review: Option<super::raw_review_state::RawReviewState>,
    pub unprojected_raw_terminal: Option<Box<RawReviewRetained>>,
    pub output_review_scope_mismatch: Option<Box<OutputReviewScopeMismatch>>,
    raw_recovery: Option<RawRecovery>,
    projection_needed: bool,
    closing: bool,
    last_cancelled: bool,
    close_waiting: bool,
}
impl PendingReviewController {
    pub fn completion_guard_blocked(&self) -> bool {
        self.host.completion_guard_blocked()
    }
    pub fn close_waiting(&self) -> bool {
        self.close_waiting || self.host.completion_guard_blocked()
    }
    pub(super) fn uses_registry(
        &self,
        registry: &std::rc::Rc<std::cell::RefCell<meari::application::plugins::PluginRegistry>>,
    ) -> bool {
        self.host.uses_registry(registry)
    }
    pub(super) fn quiesce_for_parent_close(&mut self) -> Result<(), PluginError> {
        self.closing = true;
        self.invalidate_output_review();
        self.invalidate_raw_review();
        self.close_waiting = false;
        // Drain Core-accepted results first: its bounded history may have evicted
        // this ticket while the provider ACK was awaiting a retry.
        self.poll().map_err(|error| match error {
            PendingControllerError::Plugin(error) => error,
            PendingControllerError::View(error) => PluginError::new(
                meari::application::plugins::PluginErrorCode::Adapter,
                format!("Pending view completion retained for close: {error:?}"),
            ),
        })?;
        self.host.begin_close()?;
        self.close_waiting = !self.host.pending_tickets().is_empty();
        self.host.quiescent()
    }
    pub fn new(
        host: PendingRegistryProposalHost,
        session: ReviewSession,
    ) -> Result<Self, ViewError> {
        if host.scope() != &session.scope().view {
            return Err(ViewError::Stale);
        }
        Ok(Self {
            host,
            session,
            active: None,
            output_review: None,
            raw_review: None,
            unprojected_raw_terminal: None,
            output_review_scope_mismatch: None,
            raw_recovery: None,
            projection_needed: false,
            closing: false,
            last_cancelled: false,
            close_waiting: false,
        })
    }
    pub fn session(&self) -> &ReviewSession {
        &self.session
    }
    pub fn session_mut(&mut self) -> &mut ReviewSession {
        self.invalidate_output_review();
        &mut self.session
    }
    pub fn enable_raw_reviews(
        &mut self,
        next_id: DecisionIdSource,
    ) -> Result<(), PendingControllerError> {
        self.available()?;
        if self.raw_review.is_some() {
            return Err(PendingControllerError::View(ViewError::Busy));
        }
        self.raw_review = Some(super::raw_review_state::RawReviewState::new(next_id));
        self.projection_needed = true;
        Ok(())
    }
    pub fn raw_review(&self) -> Option<&super::raw_review_state::RawReviewState> {
        self.raw_review.as_ref()
    }
    pub fn raw_recovery(&self) -> Option<&RawRecovery> {
        self.raw_recovery.as_ref()
    }
    pub fn invalidate_raw_review(&mut self) {
        if let Some(raw) = &mut self.raw_review {
            raw.invalidate();
            self.projection_needed = true;
        }
    }
    pub fn raw_review_intent(
        &mut self,
        scope: &ReviewScope,
        action: super::raw_review_state::RawReviewAction,
    ) -> Result<(), PendingControllerError> {
        if matches!(
            &action,
            super::raw_review_state::RawReviewAction::InspectRetained
                | super::raw_review_state::RawReviewAction::HistoryRetained
                | super::raw_review_state::RawReviewAction::ReleaseRetained
        ) {
            return self.recovery_intent(scope, action);
        }
        self.available()?;
        if scope != self.session.scope() {
            return Err(PendingControllerError::View(ViewError::Stale));
        }
        let raw = self
            .raw_review
            .as_mut()
            .ok_or(PendingControllerError::View(ViewError::InvalidSelection))?;
        let request = match raw.prepare(action) {
            Ok(request) => request,
            Err(error) => {
                raw.error = Some(error);
                self.projection_needed = true;
                return Ok(());
            }
        };
        self.projection_needed = true;
        if let Some((token, request)) = request {
            match self.host.submit(request) {
                Ok(ProposalAdmission::Pending(ticket)) => {
                    self.active = Some(Active {
                        ticket,
                        kind: ActiveKind::RawReview(token),
                    })
                }
                Ok(ProposalAdmission::Immediate(outcome)) => {
                    self.apply_raw_review(token, *outcome, None)
                }
                Err(error) => self.apply_raw_review(token, Err(error), None),
            }
        }
        Ok(())
    }
    fn apply_raw_review(
        &mut self,
        token: super::raw_review_state::RawReviewToken,
        outcome: Result<PendingOutput, ApiError>,
        transport: Option<(PluginTicket, bool)>,
    ) {
        // The original host already accepted/ACKed this terminal before display correlation.
        self.projection_needed = true;
        let result = match self.raw_review.as_mut() {
            Some(raw) => raw.complete(token, outcome),
            None => Err(Box::new(super::raw_review_state::RawReviewRejected {
                token,
                outcome,
            })),
        };
        if let Err(rejected) = result {
            self.raw_recovery = None;
            // Both callers establish an empty retained slot before admission/host ACK.
            self.unprojected_raw_terminal = Some(Box::new(RawReviewRetained {
                rejected,
                transport,
            }));
        }
    }
    pub fn enable_output_reviews(
        &mut self,
        next_id: DecisionIdSource,
    ) -> Result<(), PendingControllerError> {
        self.available()?;
        if self.output_review.is_some() {
            return Err(PendingControllerError::View(ViewError::Busy));
        }
        self.output_review = Some(OutputReviewInteraction::new(
            self.session.scope().proposal_id.clone(),
            next_id,
        ));
        self.projection_needed = true;
        Ok(())
    }
    pub fn output_review(&self) -> Option<&OutputReviewState> {
        self.output_review
            .as_ref()
            .map(OutputReviewInteraction::state)
    }
    pub fn invalidate_output_review(&mut self) {
        if let Some(review) = &mut self.output_review {
            review.state_mut().invalidate();
            self.projection_needed = true;
        }
    }
    pub fn output_review_intent(
        &mut self,
        scope: &ReviewScope,
        intent: OutputReviewIntent,
    ) -> Result<(), PendingControllerError> {
        self.available()?;
        if scope != self.session.scope() {
            return Err(PendingControllerError::View(ViewError::Stale));
        }
        if matches!(
            &intent,
            OutputReviewIntent::Action(
                OutputReviewAction::Inspect
                    | OutputReviewAction::Select
                    | OutputReviewAction::Record
                    | OutputReviewAction::Retry
            )
        ) && (self.session.dirty() || self.session.pending().is_some())
        {
            return Err(PendingControllerError::View(ViewError::Busy));
        }
        let review = self
            .output_review
            .as_mut()
            .ok_or_else(|| PendingControllerError::Plugin(output_review_intents::unsupported()))?;
        let request = review.prepare(intent).map_err(|error| match error {
            OutputReviewIntentError::Input(error) => {
                PendingControllerError::Plugin(meari::application::plugins::PluginError::new(
                    meari::application::plugins::PluginErrorCode::InvalidPayload,
                    format!("Output review input: {error:?}"),
                ))
            }
            OutputReviewIntentError::Metadata(error) => {
                PendingControllerError::Plugin(output_review_intents::api_error(&error))
            }
        })?;
        self.projection_needed = true;
        if let Some((token, request)) = request {
            match self.host.submit(request) {
                Ok(ProposalAdmission::Pending(ticket)) => {
                    self.active = Some(Active {
                        ticket,
                        kind: ActiveKind::OutputReview(token),
                    });
                }
                Ok(ProposalAdmission::Immediate(outcome)) => {
                    self.apply_output_review(token, *outcome)?
                }
                Err(error) => self.apply_output_review(token, Err(error))?,
            }
        }
        Ok(())
    }
    fn apply_output_review(
        &mut self,
        token: OutputReviewToken,
        outcome: Result<PendingOutput, ApiError>,
    ) -> Result<(), PendingControllerError> {
        self.last_cancelled = outcome
            .as_ref()
            .err()
            .is_some_and(|error| error.code == ErrorCode::Cancelled);
        self.projection_needed = true;
        // The host has already retained/accepted/ACKed this terminal before handing it here.
        // Preserve a mismatched delivered reply as an observation, never replay its mutation.
        let observation = match &outcome {
            Ok(PendingOutput::OutputReviewInspected(value)) => serde_json::to_value(value),
            Ok(PendingOutput::OutputDecisionRecorded(value)) => serde_json::to_value(value),
            Ok(PendingOutput::OutputDecisionsListed(value)) => serde_json::to_value(value),
            Ok(PendingOutput::Created(value) | PendingOutput::Edited(value)) => {
                serde_json::to_value(value)
            }
            Ok(PendingOutput::Inspected(value) | PendingOutput::Imported(value)) => {
                serde_json::to_value(value)
            }
            Ok(PendingOutput::Accepted(value)) => serde_json::to_value(value),
            Ok(
                PendingOutput::RawReviewInspected(_)
                | PendingOutput::RawReviewAsset(_)
                | PendingOutput::RawReviewRecorded(_)
                | PendingOutput::RawReviewHistory(_),
            ) => {
                self.output_review_scope_mismatch =
                    Some(Box::new(OutputReviewScopeMismatch { token, outcome }));
                return Err(PendingControllerError::View(ViewError::InvalidSelection));
            }
            Err(error) => serde_json::to_value(error),
        }
        .map_err(|error| {
            PendingControllerError::Plugin(output_review_intents::api_error(&ApiError::new(
                ErrorCode::Internal,
                error.to_string(),
            )))
        })?;
        let review = self
            .output_review
            .as_mut()
            .ok_or(PendingControllerError::View(ViewError::Stale))?;
        if let Err(error) = review.state_mut().complete(&token, outcome) {
            let mut retained = ApiError::new(
                ErrorCode::Internal,
                format!("Output review terminal does not match its pending view: {error:?}"),
            );
            retained.details = serde_json::json!({"delivered_terminal":observation,"sdk_terminal_already_acknowledged":true});
            review
                .state_mut()
                .outcome_unknown(&token, retained)
                .map_err(|_| PendingControllerError::View(ViewError::Stale))?;
        }
        Ok(())
    }
    pub fn projection_needed(&self) -> bool {
        self.projection_needed
    }
    /// Called only after the document successfully commits its projection.
    pub fn projection_committed(&mut self) {
        self.projection_needed = false;
    }
    pub fn last_terminal_was_cancelled(&self) -> bool {
        self.last_cancelled
    }
    pub fn active_ticket(&self) -> Option<&PluginTicket> {
        self.active.as_ref().map(|active| &active.ticket)
    }
    fn available(&self) -> Result<(), PendingControllerError> {
        if self.closing
            || self.active.is_some()
            || self.unprojected_raw_terminal.is_some()
            || self.output_review_scope_mismatch.is_some()
        {
            Err(PendingControllerError::View(ViewError::Busy))
        } else {
            Ok(())
        }
    }
    pub fn save(&mut self, scope: &ReviewScope) -> Result<(), PendingControllerError> {
        self.available()?;
        let intent = self
            .session
            .begin_save(scope)
            .map_err(PendingControllerError::View)?;
        self.invalidate_output_review();
        self.start(
            PendingRequest::Edit {
                revision: intent.expected_revision,
                input: intent.request,
            },
            intent.token,
            Phase::Save,
        )
    }
    pub fn accept(&mut self, scope: &ReviewScope) -> Result<(), PendingControllerError> {
        self.available()?;
        let intent = self
            .session
            .begin_accept(scope)
            .map_err(PendingControllerError::View)?;
        self.invalidate_output_review();
        self.start(
            PendingRequest::Accept {
                revision: intent.expected_revision,
                input: intent.request,
            },
            intent.token,
            Phase::Accept,
        )
    }
    pub fn refresh(&mut self) -> Result<(), PendingControllerError> {
        self.available()?;
        if self.session.confirmed_mutation().is_none() {
            return Err(PendingControllerError::View(ViewError::InvalidSnapshot));
        }
        let token = self
            .session
            .pending()
            .cloned()
            .ok_or(PendingControllerError::View(ViewError::Stale))?;
        self.inspect(token)
    }
    fn inspect(&mut self, token: PendingToken) -> Result<(), PendingControllerError> {
        self.start(
            PendingRequest::Inspect(InspectProposal {
                proposal_id: self.session.scope().proposal_id.clone(),
            }),
            token,
            Phase::Inspect,
        )
    }
    fn start(
        &mut self,
        input: PendingRequest,
        token: PendingToken,
        phase: Phase,
    ) -> Result<(), PendingControllerError> {
        self.projection_needed = true;
        match self.host.submit(input) {
            Ok(ProposalAdmission::Pending(ticket)) => {
                self.active = Some(Active {
                    ticket,
                    kind: ActiveKind::Proposal { token, phase },
                });
                Ok(())
            }
            Ok(ProposalAdmission::Immediate(outcome)) => self.apply(token, phase, *outcome),
            Err(error) => self.apply(token, phase, Err(error)),
        }
    }
    fn apply(
        &mut self,
        token: PendingToken,
        phase: Phase,
        outcome: Result<PendingOutput, ApiError>,
    ) -> Result<(), PendingControllerError> {
        self.last_cancelled = outcome
            .as_ref()
            .err()
            .is_some_and(|error| error.code == ErrorCode::Cancelled);
        self.projection_needed = true;
        match (phase, outcome) {
            (Phase::Save, Ok(PendingOutput::Edited(document))) => {
                self.session
                    .confirm_mutation(&token, MutationReceipt::Saved(document))
                    .map_err(PendingControllerError::View)?;
                if self.closing {
                    Ok(())
                } else {
                    self.inspect(token)
                }
            }
            (Phase::Accept, Ok(PendingOutput::Accepted(receipt))) => {
                self.session
                    .confirm_mutation(&token, MutationReceipt::Accepted(receipt))
                    .map_err(PendingControllerError::View)?;
                if self.closing {
                    Ok(())
                } else {
                    self.inspect(token)
                }
            }
            (Phase::Inspect, Ok(PendingOutput::Inspected(stored))) => self
                .session
                .complete_refresh(&token, Ok(stored))
                .map_err(PendingControllerError::View),
            (Phase::Inspect, Err(error)) => self
                .session
                .complete_refresh(&token, Err(error))
                .map_err(PendingControllerError::View),
            (_, Err(error)) => self
                .session
                .complete(&token, Err(error))
                .map_err(PendingControllerError::View),
            _ => Err(PendingControllerError::View(ViewError::InvalidSnapshot)),
        }
    }
    /// Call only after dispatch guards end. A provider ACK retry never reissues mutation.
    pub fn poll(&mut self) -> Result<bool, PendingControllerError> {
        let recovering = self
            .active
            .as_ref()
            .is_some_and(|active| matches!(&active.kind, ActiveKind::RawRecovery { .. }));
        if recovering
            && self
                .raw_recovery
                .as_ref()
                .is_some_and(|recovery| recovery.unmatched.is_some())
        {
            return Err(PendingControllerError::View(ViewError::Busy));
        }
        if (self.unprojected_raw_terminal.is_some() && !recovering)
            || self.output_review_scope_mismatch.is_some()
        {
            return if self.active.is_some() {
                Err(PendingControllerError::View(ViewError::Busy))
            } else {
                Ok(false)
            };
        }
        let Some(active) = &self.active else {
            return Ok(false);
        };
        let Some(terminal) = self
            .host
            .poll(&active.ticket)
            .map_err(PendingControllerError::Plugin)?
        else {
            return Ok(false);
        };
        let active = self
            .active
            .take()
            .ok_or(PendingControllerError::View(ViewError::Stale))?;
        match active.kind {
            ActiveKind::RawRecovery {
                generation,
                phase,
                scope,
            } => {
                let recovery = self
                    .raw_recovery
                    .as_mut()
                    .expect("active recovery owns its bounded read state");
                recovery.transport = Some((terminal.ticket, terminal.cancellation_requested));
                if &scope != self.session.scope() {
                    recovery.unmatched = Some(terminal.outcome);
                } else {
                    recovery.complete(generation, phase, terminal.outcome);
                }
                self.projection_needed = true;
            }
            ActiveKind::RawReview(token) => self.apply_raw_review(
                token,
                terminal.outcome,
                Some((terminal.ticket, terminal.cancellation_requested)),
            ),
            ActiveKind::Proposal { token, phase } => self.apply(token, phase, terminal.outcome)?,
            ActiveKind::OutputReview(token) => self.apply_output_review(token, terminal.outcome)?,
        }
        Ok(true)
    }
    pub fn cancel(&mut self) -> Result<(), PendingControllerError> {
        let active = self
            .active
            .as_ref()
            .ok_or(PendingControllerError::View(ViewError::Stale))?;
        self.host
            .cancel(&active.ticket)
            .map_err(PendingControllerError::Plugin)?;
        Ok(())
    }
    /// This closes the SDK registration only, never claims document/Native cleanup.
    pub fn close_sdk(&mut self) -> Result<(), PendingControllerError> {
        self.quiesce_for_parent_close()
            .map_err(PendingControllerError::Plugin)?;
        self.host.close().map_err(PendingControllerError::Plugin)
    }
}

#[cfg(test)]
#[path = "pending_controller/tests.rs"]
mod tests;
