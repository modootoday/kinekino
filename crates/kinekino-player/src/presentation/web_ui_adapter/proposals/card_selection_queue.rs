//! Bounded UI admission over host-held project choices; not an SDK or navigation-policy queue.
use super::project_card_choices::ProjectCardChoices;
use kinekino_player::{
    application::launch::native_proposal::NativeProposalSelection,
    domain::projects::{ProjectProposalSnapshot, ProjectRevision, StoredProjectRevision},
};
use meari::{
    application::{
        host_callbacks::{LocalActionInvocation, LocalActionKey},
        plugins::*,
    },
    domain::host_callbacks::CallbackArguments,
};
use meari_native::infrastructure::native_host::IntentAdmission;
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    num::{NonZeroU64, NonZeroUsize},
    rc::Rc,
    sync::Arc,
};

pub const CARD_SELECTION_BINDING: &str = "onCardSelection";
pub const CAPTURED_EDIT_BINDING: &str = "onCapturedCardEdit";

/// Preserves saved working edits and captured identity while a current inspect is pending.
pub struct CardSelectionTarget {
    pub selection: NativeProposalSelection,
    pub project: Arc<ProjectRevision>,
    stored_project: Option<Arc<StoredProjectRevision>>,
}
impl CardSelectionTarget {
    /// Host-held SDK-returned data, not renderer-supplied authority.
    pub fn stored_capture(&self) -> Option<(&StoredProjectRevision, &ProjectProposalSnapshot)> {
        let project = self.stored_project.as_deref()?;
        let capture =
            project.project.proposals.iter().find(|capture| {
                capture.captured.document.proposal_id == self.selection.proposal_id
            })?;
        Some((project, capture))
    }
}
#[must_use = "Retain the original route ticket and selected project through inspect/mount failure"]
pub struct CardSelectionRoute {
    pub ticket: super::super::player_shell::RouteTicket,
    pub target: Arc<CardSelectionTarget>,
}
pub struct CardSelectionQueue {
    scope: PluginScope,
    choices: BTreeMap<String, Arc<CardSelectionTarget>>,
    pending: VecDeque<Arc<CardSelectionTarget>>,
    capacity: NonZeroUsize,
}
impl CardSelectionQueue {
    /// Uses the existing bridge's current committed local key, including after reprojection.
    pub fn bridge_port(queue: Rc<RefCell<Self>>) -> super::window_bridge::PanelLocalIntentPort {
        Box::new(move |session, invocation| {
            let key = match session.local_action_key(CARD_SELECTION_BINDING) {
                Ok(key) => key,
                Err(error) => {
                    return IntentAdmission::Rejected(
                        super::window_bridge::PanelBridgeError::Plugin(error),
                    );
                }
            };
            let mut queue = match queue.try_borrow_mut() {
                Ok(queue) => queue,
                Err(_) => return IntentAdmission::Capacity,
            };
            match queue.admit(invocation, key) {
                IntentAdmission::Accepted => IntentAdmission::Accepted,
                IntentAdmission::Capacity => IntentAdmission::Capacity,
                IntentAdmission::Rejected(error) => {
                    IntentAdmission::Rejected(super::window_bridge::PanelBridgeError::Plugin(error))
                }
            }
        })
    }
    pub fn new(
        scope: PluginScope,
        choices: &ProjectCardChoices<'_>,
        capacity: NonZeroUsize,
    ) -> Self {
        let project = Arc::new(choices.project().clone());
        Self {
            scope,
            choices: choices
                .rows()
                .iter()
                .map(|row| {
                    (
                        row.key().into(),
                        Arc::new(CardSelectionTarget {
                            selection: row.live_inspection_selection(),
                            project: project.clone(),
                            stored_project: None,
                        }),
                    )
                })
                .collect(),
            pending: VecDeque::new(),
            capacity,
        }
    }
    pub fn from_stored(
        scope: PluginScope,
        stored: StoredProjectRevision,
        generation: NonZeroU64,
        capacity: NonZeroUsize,
    ) -> Result<Self, super::project_card_choices::CardChoiceError> {
        let choices = ProjectCardChoices::new(&stored.project, generation)?;
        let mut queue = Self::new(scope, &choices, capacity);
        let stored = Arc::new(stored);
        for target in queue.choices.values_mut() {
            Arc::get_mut(target)
                .expect("new queue has no exported targets")
                .stored_project = Some(stored.clone());
        }
        Ok(queue)
    }
    pub fn scope(&self) -> &PluginScope {
        &self.scope
    }
    /// Origin presence only enables an explicit entry request. The SDK still decides mutation.
    pub fn admit_captured_edit(
        &mut self,
        invocation: LocalActionInvocation,
        key: &LocalActionKey,
    ) -> IntentAdmission<PluginError> {
        if invocation.scope() != &self.scope || !invocation.matches_key(key) {
            return IntentAdmission::Rejected(PluginError::new(
                PluginErrorCode::Stale,
                "Captured entry scope differs",
            ));
        }
        let eligible = match invocation.arguments() {
            CallbackArguments::Text(row) => self
                .choices
                .get(row)
                .and_then(|target| target.stored_capture())
                .is_some_and(|(_, capture)| capture.proposal_origin.is_some()),
            _ => false,
        };
        if !eligible {
            return IntentAdmission::Rejected(PluginError::new(
                PluginErrorCode::PermissionDenied,
                "Legacy or unknown capture remains read-only; request an explicit live capture",
            ));
        }
        self.admit(invocation, key)
    }
    /// Invoke immediately after the real renderer/HostCallbacks guard, using the current key.
    /// Invalid local input never falls through to SDK dispatch, including when capacity is full.
    pub fn admit(
        &mut self,
        invocation: LocalActionInvocation,
        current_key: &LocalActionKey,
    ) -> IntentAdmission<PluginError> {
        if invocation.scope() != &self.scope || !invocation.matches_key(current_key) {
            return IntentAdmission::Rejected(PluginError::new(
                PluginErrorCode::Stale,
                "Card selection owner differs",
            ));
        }
        let CallbackArguments::Text(key) = invocation.arguments() else {
            return IntentAdmission::Rejected(PluginError::new(
                PluginErrorCode::InvalidPayload,
                "Card selection requires a displayed row key",
            ));
        };
        let Some(choice) = self.choices.get(key) else {
            return IntentAdmission::Rejected(PluginError::new(
                PluginErrorCode::Stale,
                "Card selection row is no longer displayed",
            ));
        };
        if self.pending.len() >= self.capacity.get() {
            return IntentAdmission::Capacity;
        }
        self.pending.push_back(choice.clone());
        IntentAdmission::Accepted
    }
    pub fn front(&self) -> Option<&Arc<CardSelectionTarget>> {
        self.pending.front()
    }
    /// Uses the existing ShellState guard and consumes a queued target only after acceptance.
    /// A route request is still awaiting the real renderer's commit observation.
    pub fn request_route(
        &mut self,
        state: &mut super::super::player_shell::ShellState,
        catalog: &super::super::player_shell::RouteCatalog,
        guards: super::super::player_shell::NavigationGuards,
    ) -> Result<Option<CardSelectionRoute>, super::super::player_shell::NavigationError> {
        let Some(target) = self.pending.front().cloned() else {
            return Ok(None);
        };
        let ticket = state.request_route(catalog, "studio.cards", guards)?;
        let _ = self.pending.pop_front();
        Ok(Some(CardSelectionRoute { ticket, target }))
    }
    /// The original ShellState must accept its existing dirty/pending/recovery guard first.
    /// This only transfers presentation data, never starts inspection or changes a route.
    pub fn pop(&mut self) -> Option<Arc<CardSelectionTarget>> {
        self.pending.pop_front()
    }
    pub fn len(&self) -> usize {
        self.pending.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
#[path = "card_selection_queue/tests.rs"]
mod tests;
