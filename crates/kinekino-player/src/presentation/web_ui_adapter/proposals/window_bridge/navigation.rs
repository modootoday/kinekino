//! Navigation reads the active editor owner; it never clears a draft or completes work.
use super::*;
use crate::presentation::web_ui_adapter::player_shell::NavigationGuards;

impl<L: ParentProposalLease> ProposalWindowBridge<L> {
    pub fn navigation_guards(&self) -> NavigationGuards {
        NavigationGuards {
            dirty: self.review().dirty(),
            pending: self.operation_pending()
                || self.review().pending().is_some()
                || self.needs_drain
                || self.unexpected_ticket.is_some()
                || self.owners.session.close_waiting(),
            recovery_blocked: self.review().confirmed_mutation().is_some()
                || self.projection_pending()
                || self.staged_frame.is_some()
                || self.retired_cleanup_pending
                || self.owners.session.completion_guard_blocked()
                || self.owned_session_closed
                || self.owned_parent_closed,
        }
    }

    pub(super) fn ensure_selection_navigation(&self) -> Result<(), ViewError> {
        let guards = self.navigation_guards();
        if guards.recovery_blocked {
            return Err(ViewError::RefreshRequired);
        }
        if guards.pending {
            return Err(ViewError::Busy);
        }
        if guards.dirty {
            return Err(ViewError::SaveRequired);
        }
        Ok(())
    }
}
