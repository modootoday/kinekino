//! Document interaction composition. Execution and SDK policies stay in the injected host.
use super::pending_controller::{PendingControllerError, PendingReviewController};
use super::{
    RegistryProposalHost, ReviewController, ReviewHost, ReviewScope, ReviewSession, ViewError,
};
use super::{
    intent_queue::output_review_intents::{self, OutputReviewIntent},
    output_review_state::OutputReviewState,
};
use meari::application::plugins::{PluginError, PluginRegistry};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub enum InteractionError {
    View(ViewError),
    Plugin(PluginError),
}
impl From<PendingControllerError> for InteractionError {
    fn from(error: PendingControllerError) -> Self {
        match error {
            PendingControllerError::View(error) => Self::View(error),
            PendingControllerError::Plugin(error) => Self::Plugin(error),
        }
    }
}
pub trait InteractionController {
    fn raw_review(
        &mut self,
        _scope: &ReviewScope,
        _action: super::raw_review_state::RawReviewAction,
    ) -> Result<(), InteractionError> {
        Err(InteractionError::Plugin(PluginError::new(
            meari::application::plugins::PluginErrorCode::MissingCapability,
            "Raw review capability is unavailable",
        )))
    }
    fn session(&self) -> &ReviewSession;
    fn session_mut(&mut self) -> &mut ReviewSession;
    fn save(&mut self, scope: &ReviewScope) -> Result<(), InteractionError>;
    fn accept(&mut self, scope: &ReviewScope) -> Result<(), InteractionError>;
    fn refresh(&mut self) -> Result<(), InteractionError>;
    fn output_review(
        &mut self,
        _scope: &ReviewScope,
        _intent: OutputReviewIntent,
    ) -> Result<(), InteractionError> {
        Err(InteractionError::Plugin(
            output_review_intents::unsupported(),
        ))
    }
    fn output_review_error(&self) -> bool {
        false
    }
}
impl<H: ReviewHost> InteractionController for ReviewController<H> {
    fn session(&self) -> &ReviewSession {
        ReviewController::session(self)
    }
    fn session_mut(&mut self) -> &mut ReviewSession {
        ReviewController::session_mut(self)
    }
    fn save(&mut self, scope: &ReviewScope) -> Result<(), InteractionError> {
        ReviewController::save(self, scope).map_err(InteractionError::View)
    }
    fn accept(&mut self, scope: &ReviewScope) -> Result<(), InteractionError> {
        ReviewController::accept(self, scope).map_err(InteractionError::View)
    }
    fn refresh(&mut self) -> Result<(), InteractionError> {
        ReviewController::refresh(self).map_err(InteractionError::View)
    }
}
pub enum DocumentController {
    Immediate(Box<ReviewController<RegistryProposalHost>>),
    Pending(Box<PendingReviewController>),
}
impl DocumentController {
    pub fn raw_recovery(&self) -> Option<&super::pending_controller::RawRecovery> {
        match self {
            Self::Pending(controller) => controller.raw_recovery(),
            Self::Immediate(_) => None,
        }
    }
    pub fn raw_review(&self) -> Option<&super::raw_review_state::RawReviewState> {
        match self {
            Self::Pending(controller) => controller.raw_review(),
            Self::Immediate(_) => None,
        }
    }
    pub fn raw_review_unprojected(&self) -> bool {
        matches!(self, Self::Pending(controller) if controller.unprojected_raw_terminal.is_some())
    }
    pub fn raw_review_retained(&self) -> Option<&super::pending_controller::RawReviewRetained> {
        match self {
            Self::Pending(controller) => controller.unprojected_raw_terminal.as_deref(),
            Self::Immediate(_) => None,
        }
    }
    pub fn output_review(&self) -> Option<&OutputReviewState> {
        match self {
            Self::Pending(controller) => controller.output_review(),
            Self::Immediate(_) => None,
        }
    }
    pub fn completion_guard_blocked(&self) -> bool {
        matches!(self, Self::Pending(controller) if controller.completion_guard_blocked())
    }
    pub fn close_waiting(&self) -> bool {
        matches!(self, Self::Pending(controller) if controller.close_waiting())
    }
    pub fn uses_registry(&self, registry: &Rc<RefCell<PluginRegistry>>) -> bool {
        match self {
            Self::Immediate(controller) => controller.host().uses_registry(registry),
            Self::Pending(controller) => controller.uses_registry(registry),
        }
    }
    pub fn poll(&mut self) -> Result<bool, InteractionError> {
        match self {
            Self::Immediate(_) => Ok(false),
            Self::Pending(controller) => controller.poll().map_err(Into::into),
        }
    }
    pub fn projection_needed(&self) -> bool {
        matches!(self,Self::Pending(controller) if controller.projection_needed())
    }
    pub fn operation_pending(&self) -> bool {
        matches!(self,Self::Pending(controller) if controller.active_ticket().is_some())
    }
    pub fn projection_committed(&mut self) {
        if let Self::Pending(controller) = self {
            controller.projection_committed();
        }
    }
    /// Parent lease remains the one authority that unregisters the SDK adapter/worker.
    pub fn quiesce_for_parent_close(&mut self) -> Result<(), PluginError> {
        match self {
            Self::Immediate(_) => Ok(()),
            Self::Pending(controller) => controller.quiesce_for_parent_close(),
        }
    }
}
impl InteractionController for DocumentController {
    fn raw_review(
        &mut self,
        scope: &ReviewScope,
        action: super::raw_review_state::RawReviewAction,
    ) -> Result<(), InteractionError> {
        match self {
            Self::Pending(controller) => controller
                .raw_review_intent(scope, action)
                .map_err(Into::into),
            Self::Immediate(_) => Err(InteractionError::Plugin(PluginError::new(
                meari::application::plugins::PluginErrorCode::MissingCapability,
                "Raw review capability is unavailable",
            ))),
        }
    }
    fn output_review(
        &mut self,
        scope: &ReviewScope,
        intent: OutputReviewIntent,
    ) -> Result<(), InteractionError> {
        match self {
            Self::Pending(controller) => controller
                .output_review_intent(scope, intent)
                .map_err(Into::into),
            Self::Immediate(_) => Err(InteractionError::Plugin(
                output_review_intents::unsupported(),
            )),
        }
    }
    fn output_review_error(&self) -> bool {
        DocumentController::output_review(self).is_some_and(|state| state.error().is_some())
    }
    fn session(&self) -> &ReviewSession {
        match self {
            Self::Immediate(controller) => controller.session(),
            Self::Pending(controller) => controller.session(),
        }
    }
    fn session_mut(&mut self) -> &mut ReviewSession {
        match self {
            Self::Immediate(controller) => controller.session_mut(),
            Self::Pending(controller) => controller.session_mut(),
        }
    }
    fn save(&mut self, scope: &ReviewScope) -> Result<(), InteractionError> {
        match self {
            Self::Immediate(controller) => controller.save(scope).map_err(InteractionError::View),
            Self::Pending(controller) => controller.save(scope).map_err(Into::into),
        }
    }
    fn accept(&mut self, scope: &ReviewScope) -> Result<(), InteractionError> {
        match self {
            Self::Immediate(controller) => controller.accept(scope).map_err(InteractionError::View),
            Self::Pending(controller) => controller.accept(scope).map_err(Into::into),
        }
    }
    fn refresh(&mut self) -> Result<(), InteractionError> {
        match self {
            Self::Immediate(controller) => controller.refresh().map_err(InteractionError::View),
            Self::Pending(controller) => controller.refresh().map_err(Into::into),
        }
    }
}
