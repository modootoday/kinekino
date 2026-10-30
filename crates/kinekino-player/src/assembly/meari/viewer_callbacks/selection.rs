//! Opt-in product choices share the original Viewing callback export and source scope.
use super::chooser::PreparedSelection;
use crate::assembly::meari::viewer_host::{
    LIVE_ENTRY_BINDING, LiveEntryRequest, TrustedLiveChoices,
};
use crate::{
    domain::projects::ProjectRevision,
    presentation::web_ui_adapter::proposals::{
        card_selection_queue::{CAPTURED_EDIT_BINDING, CARD_SELECTION_BINDING, CardSelectionQueue},
        project_card_choices::ProjectCardChoices,
    },
};
use meari::{
    application::host_callbacks::PendingCallback, domain::host_callbacks::CallbackArguments,
};
use meari::{
    application::{
        host_callbacks::{HostCallbacks, LocalActionInvocation, LocalActionKey},
        plugins::*,
    },
    domain::host_callbacks::CallbackSignature,
};
use meari_native::infrastructure::native_host::IntentAdmission;
use std::{
    cell::RefCell,
    num::{NonZeroU64, NonZeroUsize},
    rc::Rc,
};
use std::{collections::BTreeMap, sync::Arc};
type ResolvedLiveSelection = (
    Arc<crate::application::launch::native_proposal::NativeProposalSelection>,
    Option<PreparedSelection>,
);

struct PreparedOpenRow {
    key: String,
    prepared: PreparedSelection,
}
fn same_prepared(left: &PreparedSelection, right: &PreparedSelection) -> bool {
    left.scope == right.scope
        && left.generation == right.generation
        && Arc::ptr_eq(&left.selection, &right.selection)
}
fn stale_open() -> PluginError {
    PluginError::new(
        PluginErrorCode::Stale,
        "Prepared Open key, carrier or callback changed",
    )
}

/// One authenticated pending local choice. These rows never come from project captures.
pub(super) struct LiveSelection {
    rows:
        BTreeMap<String, Arc<crate::application::launch::native_proposal::NativeProposalSelection>>,
    key: Option<LocalActionKey>,
    queued_key: Option<String>,
    pending: Option<LiveEntryRequest>,
    prepared_row: Option<PreparedOpenRow>,
    pending_prepared: Option<PreparedSelection>,
    next_prepared_key: u64,
}
impl LiveSelection {
    pub(super) fn new(choices: TrustedLiveChoices) -> Self {
        Self {
            rows: choices.into_rows().into_iter().collect(),
            key: None,
            queued_key: None,
            pending: None,
            prepared_row: None,
            pending_prepared: None,
            next_prepared_key: 0,
        }
    }
    pub(super) fn for_chooser() -> Self {
        Self {
            rows: BTreeMap::new(),
            key: None,
            queued_key: None,
            pending: None,
            prepared_row: None,
            pending_prepared: None,
            next_prepared_key: 0,
        }
    }
    pub(super) fn register_prepared(
        &mut self,
        prepared: PreparedSelection,
    ) -> Result<String, PluginError> {
        if self.pending.is_some() {
            return Err(PluginError::new(
                PluginErrorCode::Busy,
                "Original Open intent is retained",
            ));
        }
        if let Some(row) = &self.prepared_row
            && same_prepared(&row.prepared, &prepared)
        {
            return Ok(row.key.clone());
        }
        let next = self
            .next_prepared_key
            .checked_add(1)
            .ok_or_else(stale_open)?;
        let key = format!(
            "prepared-open-{}-{}-{next}",
            prepared.scope.source_generation, prepared.generation
        );
        self.prepared_row = Some(PreparedOpenRow {
            key: key.clone(),
            prepared,
        });
        self.next_prepared_key = next;
        Ok(key)
    }
    pub(super) fn retire_prepared(&mut self) {
        self.prepared_row = None;
        if self.pending_prepared.take().is_some() {
            self.pending = None;
            self.queued_key = None;
        }
    }
    pub(super) fn bind(&mut self, callbacks: &mut HostCallbacks) -> Result<(), PluginError> {
        self.retire();
        self.key = Some(callbacks.bind_local(LIVE_ENTRY_BINDING.into(), CallbackSignature::Text)?);
        Ok(())
    }
    pub(super) fn admit(
        &mut self,
        invocation: &LocalActionInvocation,
        pending: &PendingCallback,
        current: Option<&PreparedSelection>,
    ) -> IntentAdmission<PluginError> {
        if !self
            .key
            .as_ref()
            .is_some_and(|key| invocation.matches_key(key))
        {
            return IntentAdmission::Rejected(PluginError::new(
                PluginErrorCode::Stale,
                "Live entry key differs",
            ));
        }
        let CallbackArguments::Text(key) = invocation.arguments() else {
            return IntentAdmission::Rejected(PluginError::new(
                PluginErrorCode::InvalidPayload,
                "Explicit live row key required",
            ));
        };
        let (selected, prepared) = match self.resolve(key, current) {
            Ok(resolved) => resolved,
            Err(error) => return IntentAdmission::Rejected(error),
        };
        if let Some(queued) = &self.queued_key {
            return if queued == key && self.validate_pending(invocation, current).is_ok() {
                IntentAdmission::Accepted
            } else {
                IntentAdmission::Rejected(PluginError::new(
                    if self.pending_prepared.is_some() || prepared.is_some() {
                        PluginErrorCode::Busy
                    } else {
                        PluginErrorCode::Closing
                    },
                    "A different live entry is retained",
                ))
            };
        }
        self.pending = Some(LiveEntryRequest {
            selected,
            pending: PendingCallback {
                handle: pending.handle.clone(),
                invocation: pending.invocation.clone(),
            },
        });
        self.queued_key = Some(key.clone());
        self.pending_prepared = prepared;
        IntentAdmission::Accepted
    }
    fn resolve(
        &self,
        key: &str,
        current: Option<&PreparedSelection>,
    ) -> Result<ResolvedLiveSelection, PluginError> {
        if let Some(selected) = self.rows.get(key) {
            return Ok((selected.clone(), None));
        }
        let row = self
            .prepared_row
            .as_ref()
            .filter(|row| row.key == key)
            .ok_or_else(stale_open)?;
        if !current.is_some_and(|current| same_prepared(&row.prepared, current)) {
            return Err(stale_open());
        }
        Ok((row.prepared.selection.clone(), Some(row.prepared.clone())))
    }
    pub(super) fn validate_pending(
        &self,
        invocation: &LocalActionInvocation,
        current: Option<&PreparedSelection>,
    ) -> Result<Option<PreparedSelection>, PluginError> {
        if !self.matches(invocation) {
            return Err(stale_open());
        }
        let CallbackArguments::Text(key) = invocation.arguments() else {
            return Err(stale_open());
        };
        if self.queued_key.as_ref() != Some(key) {
            return Err(stale_open());
        }
        let request = self.pending.as_ref().ok_or_else(stale_open)?;
        let (selected, prepared) = self.resolve(key, current)?;
        if !Arc::ptr_eq(&request.selected, &selected) {
            return Err(stale_open());
        }
        match (&self.pending_prepared, &prepared) {
            (None, None) => Ok(None),
            (Some(pending), Some(prepared)) if same_prepared(pending, prepared) => {
                Ok(Some(prepared.clone()))
            }
            _ => Err(stale_open()),
        }
    }
    pub(super) fn request(&self) -> Option<&LiveEntryRequest> {
        self.pending.as_ref()
    }
    pub(super) fn matches(&self, invocation: &LocalActionInvocation) -> bool {
        self.key
            .as_ref()
            .is_some_and(|key| invocation.matches_key(key))
    }
    pub(super) fn retire(&mut self) -> bool {
        self.prepared_row = None;
        self.pending_prepared = None;
        self.queued_key = None;
        self.pending.take().is_some()
    }
}

pub(super) struct ViewerSelection {
    project: ProjectRevision,
    generation: NonZeroU64,
    capacity: NonZeroUsize,
    queue: Option<Rc<RefCell<CardSelectionQueue>>>,
    key: Option<LocalActionKey>,
    edit_key: Option<LocalActionKey>,
    stored: Option<crate::domain::projects::StoredProjectRevision>,
}
impl ViewerSelection {
    pub(super) fn props(&self) -> std::collections::BTreeMap<String, meari::domain::Value> {
        if self.stored.is_none() {
            return std::collections::BTreeMap::new();
        }
        use crate::presentation::web_ui_adapter::proposals::capture_choices::{
            self, CaptureChoiceRow,
        };
        use meari::domain::{Value, host_callbacks::CallbackReference};
        let choices = ProjectCardChoices::new(&self.project, self.generation)
            .expect("immutable choices validated before export");
        let rows = choices
            .rows()
            .iter()
            .map(|row| CaptureChoiceRow {
                key: row.key().into(),
                number: (row.item_index() + 1).to_string(),
                front: row.card().front.clone(),
                back: row.card().back.clone(),
                label: format!("카드 {}: {}", row.item_index() + 1, row.card().front),
                editable: self.stored.is_some() && row.capture().proposal_origin.is_some(),
            })
            .collect();
        let mut props = capture_choices::props(rows, self.stored.is_some());
        if self.stored.is_some() {
            props.insert(
                CAPTURED_EDIT_BINDING.into(),
                Value::CallbackReference(CallbackReference {
                    binding: CAPTURED_EDIT_BINDING.into(),
                    signature: CallbackSignature::Text,
                }),
            );
        }
        props
    }
    pub(super) fn new(
        project: ProjectRevision,
        generation: NonZeroU64,
        capacity: NonZeroUsize,
    ) -> Result<Self, PluginError> {
        ProjectCardChoices::new(&project, generation).map_err(|_| {
            PluginError::new(
                PluginErrorCode::InvalidPayload,
                "Selected project Card choices are invalid",
            )
        })?;
        Ok(Self {
            project,
            generation,
            capacity,
            queue: None,
            key: None,
            edit_key: None,
            stored: None,
        })
    }
    pub(super) fn from_stored(
        stored: crate::domain::projects::StoredProjectRevision,
        generation: NonZeroU64,
        capacity: NonZeroUsize,
    ) -> Result<Self, PluginError> {
        let mut selected = Self::new(stored.project.clone(), generation, capacity)?;
        selected.stored = Some(stored);
        Ok(selected)
    }
    pub(super) fn bind(
        &mut self,
        callbacks: &mut HostCallbacks,
        scope: &PluginScope,
    ) -> Result<(), PluginError> {
        if self.queue.is_none() {
            let choices =
                ProjectCardChoices::new(&self.project, self.generation).map_err(|_| {
                    PluginError::new(
                        PluginErrorCode::InvalidPayload,
                        "Selected project Card choices are invalid",
                    )
                })?;
            let queue = if let Some(stored) = &self.stored {
                CardSelectionQueue::from_stored(
                    scope.clone(),
                    stored.clone(),
                    self.generation,
                    self.capacity,
                )
                .map_err(|_| {
                    PluginError::new(
                        PluginErrorCode::InvalidPayload,
                        "Invalid stored capture choices",
                    )
                })?
            } else {
                CardSelectionQueue::new(scope.clone(), &choices, self.capacity)
            };
            self.queue = Some(Rc::new(RefCell::new(queue)));
        }
        self.key =
            Some(callbacks.bind_local(CARD_SELECTION_BINDING.into(), CallbackSignature::Text)?);
        if self.stored.is_some() {
            self.edit_key =
                Some(callbacks.bind_local(CAPTURED_EDIT_BINDING.into(), CallbackSignature::Text)?);
        }
        Ok(())
    }
    pub(super) fn queue(&self) -> Option<Rc<RefCell<CardSelectionQueue>>> {
        self.queue.clone()
    }
    pub(super) fn admit(
        &mut self,
        invocation: LocalActionInvocation,
    ) -> Option<IntentAdmission<PluginError>> {
        if let Some(edit_key) = &self.edit_key
            && invocation.matches_key(edit_key)
        {
            let Some(queue) = &self.queue else {
                return Some(IntentAdmission::Capacity);
            };
            let mut queue = match queue.try_borrow_mut() {
                Ok(queue) => queue,
                Err(_) => return Some(IntentAdmission::Capacity),
            };
            return Some(queue.admit_captured_edit(invocation, edit_key));
        }
        let key = self.key.as_ref()?;
        if !invocation.matches_key(key) {
            return None;
        }
        // This export names immutable project captures, not an authorized current-live
        // editor. Do not enqueue an edit intent or fall through to the SDK plugin.
        Some(IntentAdmission::Rejected(PluginError::new(
            PluginErrorCode::PermissionDenied,
            "Project captures are read-only; inspection does not grant live edit permission",
        )))
    }
}
