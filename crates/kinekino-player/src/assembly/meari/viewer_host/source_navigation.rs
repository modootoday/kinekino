//! Accepted source navigation intents, never an SDK request or durable callback grant.
use super::*;
use crate::presentation::web_ui_adapter::{
    player_shell::document::product_frame::StudioTask,
    proposals::window_bridge::{PanelBridgeError, PanelLocalIntentPort},
};
use meari::application::{
    host_callbacks::{LocalActionInvocation, LocalActionKey},
    plugins::{PluginError, PluginErrorCode},
};
use meari::domain::host_callbacks::CallbackArguments;
use std::cell::Cell;

pub(super) const NAVIGATION_BINDING: &str = "onNavigate";
struct ReturnIntent {
    key: LocalActionKey,
    stamp: HostStamp,
    task: StudioTask,
}
#[derive(Default)]
pub(super) struct NavigationIntents {
    pending: Option<ReturnIntent>,
    blocked: bool,
    closed: bool,
    first_error: Option<ViewerHostError>,
}
fn rejected(code: PluginErrorCode, message: &str) -> IntentAdmission<PanelBridgeError> {
    IntentAdmission::Rejected(PanelBridgeError::Plugin(PluginError::new(code, message)))
}
fn same_projection(a: &HostStamp, b: &HostStamp) -> bool {
    a.host_id == b.host_id
        && a.owner == b.owner
        && a.definition_revision == b.definition_revision
        && a.document_revision == b.document_revision
}
impl NavigationIntents {
    #[cfg(test)]
    pub(super) fn pending_count(&self) -> usize {
        usize::from(self.pending.is_some())
    }
    fn admit(
        &mut self,
        invocation: LocalActionInvocation,
        current: &LocalActionKey,
        stamp: HostStamp,
        task: StudioTask,
    ) -> IntentAdmission<PanelBridgeError> {
        if self.closed {
            return rejected(PluginErrorCode::Closing, "Navigation owner is closing");
        }
        if !invocation.matches_key(current) || invocation.scope() != &stamp.owner {
            return rejected(PluginErrorCode::Stale, "Navigation source identity differs");
        }
        if !matches!(invocation.arguments(), CallbackArguments::Text(value) if value == "VIEW") {
            return rejected(
                PluginErrorCode::UnsupportedTarget,
                "This active editor supports explicit Viewing return only",
            );
        }
        if let Some(pending) = &self.pending {
            if pending.key != *current
                || pending.task != task
                || !same_projection(&pending.stamp, &stamp)
            {
                return rejected(
                    PluginErrorCode::Busy,
                    "Another navigation intent remains owned",
                );
            }
            // A new authenticated click is the only retry of an unchanged blocked intent.
            self.blocked = false;
            return IntentAdmission::Accepted;
        }
        self.pending = Some(ReturnIntent {
            key: invocation.key().clone(),
            stamp,
            task,
        });
        self.blocked = false;
        #[cfg(test)]
        if let Some(probe) = super::super::viewer_native_probe::current() {
            probe.record(
                super::super::viewer_native_probe::Kind::LiveSourceNavigationAdmitted,
                &true,
            );
        }
        IntentAdmission::Accepted
    }
    pub(super) fn close(&mut self) {
        self.closed = true;
        self.pending = None;
    }
    fn failed(&mut self, error: ViewerHostError) -> ViewerHostError {
        self.blocked = true;
        self.first_error.get_or_insert_with(|| error.clone());
        error
    }
    fn request(
        &mut self,
        request: impl FnOnce() -> Result<(), ViewerHostError>,
    ) -> Result<(), ViewerHostError> {
        if self.pending.is_none() || self.blocked {
            return Err(ViewerHostError::UnsupportedCallback);
        }
        if let Err(error) = request() {
            return Err(self.failed(error));
        }
        // The caller has stored the original accepted ticket before returning Ok.
        self.pending = None;
        self.blocked = false;
        Ok(())
    }
}
pub(super) fn bridge_port(
    mailbox: Rc<RefCell<NavigationIntents>>,
    document: NativeDocumentHandle,
    task: StudioTask,
    gate: Option<Rc<Cell<bool>>>,
    mut selection: Option<PanelLocalIntentPort>,
) -> PanelLocalIntentPort {
    Box::new(move |session, invocation| {
        let key = match session.local_action_key(NAVIGATION_BINDING) {
            Ok(key) if invocation.matches_key(key) => key,
            _ => {
                return match &mut selection {
                    Some(port) => port(session, invocation),
                    None => rejected(PluginErrorCode::Stale, "Unknown editor local action"),
                };
            }
        };
        if !gate.as_ref().is_some_and(|gate| gate.get()) {
            return rejected(
                PluginErrorCode::Busy,
                "Original entry observation is not active",
            );
        }
        if document.lifecycle() != HostLifecycle::Open {
            return rejected(PluginErrorCode::Closing, "Original document is not open");
        }
        let stamp = match document.stamp() {
            Ok(stamp) => stamp,
            Err(error) => return IntentAdmission::Rejected(PanelBridgeError::Access(error)),
        };
        match mailbox.try_borrow_mut() {
            Ok(mut mailbox) => mailbox.admit(invocation, key, stamp, task),
            Err(_) => rejected(PluginErrorCode::Busy, "Navigation mailbox is borrowed"),
        }
    })
}
impl ViewerHost {
    pub(super) fn advance_source_navigation(
        &mut self,
        turn: &HostTurn,
    ) -> Result<(), ViewerHostError> {
        if turn.remaining_callbacks != 0 {
            return Ok(());
        }
        let shared = self
            .cards
            .try_borrow()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .navigation
            .clone();
        let mut mailbox = shared
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let Some(pending) = mailbox.pending.as_ref() else {
            return Ok(());
        };
        if !same_projection(&pending.stamp, &turn.stamp) {
            // The accepted intent is now stale metadata, not a lease to a replacement export.
            mailbox.pending = None;
            let error = mailbox.failed(ViewerHostError::ForeignCheckpoint);
            self.last_error.get_or_insert(error);
            return Ok(());
        }
        if mailbox.blocked {
            return Ok(());
        }
        let active = match self.cards.try_borrow() {
            Ok(active) => active,
            Err(_) => return Err(mailbox.failed(ViewerHostError::UnsupportedCallback)),
        };
        if active.active_task != Some(pending.task)
            || active.active.is_none()
            || !active
                .live_opening
                .as_ref()
                .is_some_and(|entry| entry.phase == LiveEntryPhase::Active)
        {
            drop(active);
            let error = mailbox.failed(ViewerHostError::ForeignCheckpoint);
            self.last_error.get_or_insert(error);
            return Ok(());
        }
        drop(active);
        if let Err(error) = mailbox.request(|| self.request_viewing_return(turn)) {
            // The intent remains blocked with its original error; still let the original
            // editor drain/pump this turn. Returning early could strand its SDK terminal.
            self.last_error.get_or_insert(error);
            return Ok(());
        }
        drop(mailbox);
        #[cfg(test)]
        if let Some(probe) = super::super::viewer_native_probe::current() {
            probe.live_source_return_requested();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
