//! Root navigation and editor actions share one revocable export, with distinct typed targets.
use super::*;
use meari::application::host_callbacks::{
    LocalActionInvocation, LocalActionKey, LocalDispatchError,
};

impl ProposalDocumentSession {
    pub fn new_retaining_with_local_callbacks(
        registry: Rc<RefCell<PluginRegistry>>,
        controller: ReviewController<RegistryProposalHost>,
        bindings: ReviewBindings,
        local_signatures: BTreeMap<String, CallbackSignature>,
    ) -> Result<Self, Box<SessionConstructionFailure>> {
        Self::new_composed_retaining_with_local(
            registry,
            DocumentController::Immediate(Box::new(controller)),
            bindings,
            local_signatures,
        )
    }
    pub fn new_pending_retaining_with_local_callbacks(
        registry: Rc<RefCell<PluginRegistry>>,
        controller: PendingReviewController,
        bindings: ReviewBindings,
        local_signatures: BTreeMap<String, CallbackSignature>,
    ) -> Result<Self, Box<SessionConstructionFailure>> {
        Self::new_composed_retaining_with_local(
            registry,
            DocumentController::Pending(Box::new(controller)),
            bindings,
            local_signatures,
        )
    }
    /// Keys are from the current committed binding set. Stage failures leave these unchanged.
    /// A prop name selects metadata only; dispatch must validate the original opaque handle.
    pub fn local_action_key(&self, prop: &str) -> Result<&LocalActionKey, PluginError> {
        let mounted = self.mounted.as_ref().ok_or_else(closed)?;
        if mounted.closed.load(Ordering::Acquire) {
            return Err(closed());
        }
        mounted
            .local_keys
            .get(prop)
            .ok_or_else(|| PluginError::new(PluginErrorCode::Stale, "Unknown frame local action"))
    }
    /// Called after the real renderer's node/disabled guard and outside document borrows.
    /// Only NotLocal permits the caller to try the original plugin dispatch; Invalid does not.
    pub fn dispatch_local_after_renderer_guard(
        &self,
        handle: &CallbackHandle,
        arguments: CallbackArguments,
        current_scope: &PluginScope,
    ) -> Result<LocalActionInvocation, LocalDispatchError> {
        self.mounted
            .as_ref()
            .ok_or_else(closed)?
            .callbacks
            .dispatch_local(handle, arguments, current_scope)
    }
}

#[cfg(all(test, windows, feature = "native-proposal-window"))]
#[path = "local_composition/tests.rs"]
mod tests;
