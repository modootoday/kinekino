#[path = "ports.rs"]
mod ports;
use kinekino_media_contracts::errors::ApiError;
use kinekino_player::{
    application::proposals::{AcceptProposal, EditProposal, InspectProposal},
    domain::{api_values::DecimalU64, proposals::*},
};
pub use ports::ReviewHost;

use super::{MutationReceipt, ReviewScope, ReviewSession, ViewError};

/// Synchronous interaction orchestration. No retries, IO or SDK policy decisions.
pub struct ReviewController<H> {
    host: H,
    session: ReviewSession,
}

impl<H: ReviewHost> ReviewController<H> {
    pub(super) fn host(&self) -> &H {
        &self.host
    }
    pub fn new(host: H, session: ReviewSession) -> Self {
        Self { host, session }
    }

    pub fn session(&self) -> &ReviewSession {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut ReviewSession {
        &mut self.session
    }

    pub fn into_parts(self) -> (H, ReviewSession) {
        (self.host, self.session)
    }

    pub fn save(&mut self, scope: &ReviewScope) -> Result<(), ViewError> {
        let intent = self.session.begin_save(scope)?;
        match self.host.edit(intent.expected_revision, intent.request) {
            Ok(document) => {
                self.session
                    .confirm_mutation(&intent.token, MutationReceipt::Saved(document))?;
                self.refresh()
            }
            Err(error) => self.session.complete(&intent.token, Err(error)),
        }
    }

    pub fn accept(&mut self, scope: &ReviewScope) -> Result<(), ViewError> {
        let intent = self.session.begin_accept(scope)?;
        match self.host.accept(intent.expected_revision, intent.request) {
            Ok(receipt) => {
                self.session
                    .confirm_mutation(&intent.token, MutationReceipt::Accepted(receipt))?;
                self.refresh()
            }
            Err(error) => self.session.complete(&intent.token, Err(error)),
        }
    }

    /// Initial follow-up inspect and explicit refresh share this path; no mutation is repeated.
    pub fn refresh(&mut self) -> Result<(), ViewError> {
        if self.session.confirmed_mutation().is_none() {
            return Err(ViewError::InvalidSnapshot);
        }
        let token = self.session.pending().cloned().ok_or(ViewError::Stale)?;
        let inspected = self.host.inspect(InspectProposal {
            proposal_id: self.session.scope().proposal_id.clone(),
        });
        self.session.complete_refresh(&token, inspected)
    }
}
