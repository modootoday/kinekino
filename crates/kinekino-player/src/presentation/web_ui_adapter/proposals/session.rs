use kinekino_media_contracts::errors::ApiError;
use kinekino_player::{
    application::proposals::{AcceptProposal, EditProposal},
    domain::{api_values::DecimalU64, proposals::*},
};
use meari::application::plugins::PluginScope;
use serde::Serialize;
use std::sync::Arc;

#[path = "manual_repair.rs"]
mod manual_repair;
pub use manual_repair::{ManualRepairEntry, ManualRepairSource};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewScope {
    pub view: PluginScope,
    pub proposal_id: String,
    pub source_id: String,
    pub source_sha256: String,
    pub source_revision: DecimalU64,
}
impl ReviewScope {
    fn matches(&self, document: &ProposalDocument) -> bool {
        self.proposal_id == document.proposal_id
            && self.source_id == document.source.source_id()
            && self.source_sha256 == document.source.source_sha256()
            && self.source_revision == document.source.source_revision()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingKind {
    Save,
    Accept,
}
#[derive(Clone, Debug)]
pub struct PendingToken {
    owner: Arc<()>,
    scope: ReviewScope,
    sequence: u64,
    kind: PendingKind,
    revision: DecimalU64,
}
impl PartialEq for PendingToken {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner, &other.owner)
            && self.scope == other.scope
            && self.sequence == other.sequence
            && self.kind == other.kind
            && self.revision == other.revision
    }
}
impl Eq for PendingToken {}
#[derive(Serialize)]
pub struct ReviewDisplay<'a> {
    pub snapshot: super::ProposalView,
    pub working_draft: Option<&'a ProposalPayload>,
    pub dirty: bool,
    pub pending_kind: Option<PendingKind>,
    pub sdk_error: Option<&'a ApiError>,
    pub refresh_required: bool,
}
#[derive(Clone)]
pub enum MutationReceipt {
    Saved(ProposalDocument),
    Accepted(AcceptanceReceipt),
}
pub struct ResolvedMutation {
    pub mutation: MutationReceipt,
    pub inspected: StoredProposal,
}
pub struct EditIntent {
    pub token: PendingToken,
    pub expected_revision: DecimalU64,
    pub request: EditProposal,
}
pub struct AcceptIntent {
    pub token: PendingToken,
    pub expected_revision: DecimalU64,
    pub request: AcceptProposal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewError {
    Stale,
    Busy,
    SaveRequired,
    NoDraft,
    InvalidSnapshot,
    Exhausted,
    RefreshRequired,
    InvalidSelection,
    ExistingDraft,
}

/// Per-view interaction state. No IO, SDK mutation, CAS evaluation, retries or automatic acceptance.
pub struct ReviewSession {
    owner: Arc<()>,
    scope: ReviewScope,
    stored: StoredProposal,
    draft: Option<ProposalPayload>,
    dirty: bool,
    sequence: u64,
    pending: Option<PendingToken>,
    draft_generation: u64,
    error: Option<ApiError>,
    confirmed: Option<MutationReceipt>,
    document_choice: String,
    document_selection: Option<DocumentTextRef>,
}
impl ReviewSession {
    pub fn new(view: PluginScope, stored: StoredProposal) -> Result<Self, ViewError> {
        if view.source_generation == 0 {
            return Err(ViewError::InvalidSnapshot);
        }
        stored.validate().map_err(|_| ViewError::InvalidSnapshot)?;
        let scope = ReviewScope {
            view,
            proposal_id: stored.document.proposal_id.clone(),
            source_id: stored.document.source.source_id().into(),
            source_sha256: stored.document.source.source_sha256().into(),
            source_revision: stored.document.source.source_revision(),
        };
        Ok(Self {
            owner: Arc::new(()),
            scope,
            draft: stored.document.draft.clone(),
            stored,
            dirty: false,
            sequence: 0,
            draft_generation: 0,
            pending: None,
            error: None,
            confirmed: None,
            document_choice: String::new(),
            document_selection: None,
        })
    }
    pub fn scope(&self) -> &ReviewScope {
        &self.scope
    }
    pub(super) fn input_owner(&self) -> &Arc<()> {
        &self.owner
    }
    pub fn stored(&self) -> &StoredProposal {
        &self.stored
    }
    pub fn draft(&self) -> Option<&ProposalPayload> {
        self.draft.as_ref()
    }
    pub fn document_choice(&self) -> &str {
        &self.document_choice
    }
    pub fn document_selection(&self) -> Option<&DocumentTextRef> {
        self.document_selection.as_ref()
    }
    pub fn selected_document_text(&self) -> Option<&str> {
        let reference = self.document_selection.as_ref()?;
        let SourceSnapshot::Document(source) = &self.stored.document.source else {
            return None;
        };
        source
            .texts
            .iter()
            .find(|text| text.inner_pointer == reference.inner_pointer)
            .map(|text| text.text.as_str())
    }
    /// One-based UI selection resolves through the immutable canonical snapshot, never player state.
    pub fn choose_document_text(
        &mut self,
        scope: &ReviewScope,
        choice: String,
    ) -> Result<(), ViewError> {
        self.ready(scope)?;
        if self.draft.is_some() {
            return Err(ViewError::ExistingDraft);
        }
        self.document_choice = choice;
        self.document_selection = None;
        let SourceSnapshot::Document(source) = &self.stored.document.source else {
            return Err(ViewError::InvalidSelection);
        };
        let position = self
            .document_choice
            .trim()
            .parse::<usize>()
            .ok()
            .and_then(|value| value.checked_sub(1))
            .ok_or(ViewError::InvalidSelection)?;
        let text = source
            .texts
            .get(position)
            .ok_or(ViewError::InvalidSelection)?;
        self.document_selection = Some(
            source
                .text_reference(&text.inner_pointer)
                .ok_or(ViewError::InvalidSelection)?,
        );
        Ok(())
    }
    /// Explicit user action only. Empty working input is not SDK validation or raw JSON repair.
    pub fn begin_document_repair(&mut self, scope: &ReviewScope) -> Result<(), ViewError> {
        self.ready(scope)?;
        if self.draft.is_some() {
            return Err(ViewError::ExistingDraft);
        }
        let reference = self
            .document_selection
            .clone()
            .ok_or(ViewError::InvalidSelection)?;
        self.set_draft(
            scope,
            ProposalPayload::DocumentCorrection {
                edits: vec![DocumentCorrection {
                    source_ref: reference,
                    text: String::new(),
                }],
            },
        )
    }
    pub fn dirty(&self) -> bool {
        self.dirty
    }
    pub fn pending(&self) -> Option<&PendingToken> {
        self.pending.as_ref()
    }
    pub fn error(&self) -> Option<&ApiError> {
        self.error.as_ref()
    }
    pub fn confirmed_mutation(&self) -> Option<&MutationReceipt> {
        self.confirmed.as_ref()
    }
    pub fn display(&self) -> Result<ReviewDisplay<'_>, &'static str> {
        Ok(ReviewDisplay {
            snapshot: super::project(&self.stored)?,
            working_draft: self.draft.as_ref(),
            dirty: self.dirty,
            pending_kind: self.pending.as_ref().map(|token| token.kind),
            sdk_error: self.error.as_ref(),
            refresh_required: self.confirmed.is_some(),
        })
    }

    pub fn set_draft(
        &mut self,
        scope: &ReviewScope,
        draft: ProposalPayload,
    ) -> Result<(), ViewError> {
        self.ready(scope)?;
        let generation = self
            .draft_generation
            .checked_add(1)
            .ok_or(ViewError::Exhausted)?;
        self.draft = Some(draft);
        self.draft_generation = generation;
        self.dirty = true;
        self.error = None;
        Ok(())
    }
    pub fn text_fields(&self) -> Vec<super::TextField> {
        self.draft.as_ref().map_or_else(Vec::new, |draft| {
            super::fields::project(&self.owner, self.draft_generation, draft)
        })
    }
    pub fn review_items(&self) -> Vec<super::ReviewItem> {
        self.draft.as_ref().map_or_else(Vec::new, |draft| {
            super::fields::items(&self.owner, self.draft_generation, draft)
        })
    }
    pub fn change_text(
        &mut self,
        scope: &ReviewScope,
        input: &super::TextInput,
        text: String,
    ) -> Result<(), ViewError> {
        self.ready(scope)?;
        if !Arc::ptr_eq(&input.owner, &self.owner) || input.generation != self.draft_generation {
            return Err(ViewError::Stale);
        }
        let mut draft = self.draft.clone().ok_or(ViewError::NoDraft)?;
        if !super::fields::replace(&mut draft, input, text) {
            return Err(ViewError::Stale);
        }
        self.set_draft(scope, draft)
    }
    fn ready(&self, scope: &ReviewScope) -> Result<(), ViewError> {
        if scope != &self.scope {
            return Err(ViewError::Stale);
        }
        if self.confirmed.is_some() {
            return Err(ViewError::RefreshRequired);
        }
        if self.pending.is_some() {
            return Err(ViewError::Busy);
        }
        Ok(())
    }
    fn begin(&mut self, scope: &ReviewScope, kind: PendingKind) -> Result<PendingToken, ViewError> {
        self.ready(scope)?;
        self.sequence = self.sequence.checked_add(1).ok_or(ViewError::Exhausted)?;
        let token = PendingToken {
            owner: self.owner.clone(),
            scope: self.scope.clone(),
            sequence: self.sequence,
            kind,
            revision: self.stored.document.draft_revision,
        };
        self.pending = Some(token.clone());
        self.error = None;
        Ok(token)
    }
    pub fn begin_save(&mut self, scope: &ReviewScope) -> Result<EditIntent, ViewError> {
        self.ready(scope)?;
        let draft = self.draft.clone().ok_or(ViewError::NoDraft)?;
        let token = self.begin(scope, PendingKind::Save)?;
        Ok(EditIntent {
            token,
            expected_revision: self.stored.document.draft_revision,
            request: EditProposal {
                proposal_id: self.scope.proposal_id.clone(),
                draft,
            },
        })
    }
    pub fn begin_accept(&mut self, scope: &ReviewScope) -> Result<AcceptIntent, ViewError> {
        self.ready(scope)?;
        if self.dirty {
            return Err(ViewError::SaveRequired);
        }
        if self.draft.is_none() {
            return Err(ViewError::NoDraft);
        }
        let token = self.begin(scope, PendingKind::Accept)?;
        Ok(AcceptIntent {
            token,
            expected_revision: self.stored.document.draft_revision,
            request: AcceptProposal {
                proposal_id: self.scope.proposal_id.clone(),
                target_source_revision: self.scope.source_revision,
            },
        })
    }
    /// Host supplies a fresh inspect after a completed operation, not an invented merged receipt.
    pub fn complete(
        &mut self,
        token: &PendingToken,
        result: Result<ResolvedMutation, ApiError>,
    ) -> Result<(), ViewError> {
        if self.pending.as_ref() != Some(token) || token.scope != self.scope {
            return Err(ViewError::Stale);
        }
        if self.confirmed.is_some() {
            return Err(ViewError::RefreshRequired);
        }
        match result {
            Ok(resolved) => {
                let stored = resolved.inspected;
                let observed = match resolved.mutation {
                    MutationReceipt::Saved(document)
                        if token.kind == PendingKind::Save && self.scope.matches(&document) =>
                    {
                        Some(document.draft_revision)
                    }
                    MutationReceipt::Accepted(receipt)
                        if token.kind == PendingKind::Accept
                            && receipt.proposal_id == self.scope.proposal_id
                            && receipt.source_id == self.scope.source_id
                            && receipt.source_sha256 == self.scope.source_sha256
                            && receipt.source_revision == self.scope.source_revision
                            && stored.acceptance.as_ref().is_some_and(|saved| {
                                saved.draft_revision == receipt.draft_revision
                                    && saved.output_revision == receipt.output_revision
                            }) =>
                    {
                        Some(receipt.output_revision)
                    }
                    _ => None,
                };
                if !self.scope.matches(&stored.document)
                    || stored.validate().is_err()
                    || observed != Some(stored.document.draft_revision)
                    || stored.document.draft_revision.0 <= token.revision.0
                {
                    self.pending = None;
                    return Err(ViewError::InvalidSnapshot);
                }
                let Some(generation) = self.draft_generation.checked_add(1) else {
                    self.pending = None;
                    return Err(ViewError::Exhausted);
                };
                self.draft_generation = generation;
                self.draft = stored.document.draft.clone();
                self.stored = stored;
                self.dirty = false;
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error);
            }
        }
        self.pending = None;
        Ok(())
    }

    /// Mutation succeeded, but its follow-up inspect failed. Never present this as a failed save.
    pub fn require_refresh(
        &mut self,
        token: &PendingToken,
        mutation: MutationReceipt,
        inspect_error: ApiError,
    ) -> Result<(), ViewError> {
        self.confirm_mutation(token, mutation)?;
        self.error = Some(inspect_error);
        Ok(())
    }

    pub(super) fn confirm_mutation(
        &mut self,
        token: &PendingToken,
        mutation: MutationReceipt,
    ) -> Result<(), ViewError> {
        if self.pending.as_ref() != Some(token) || token.scope != self.scope {
            return Err(ViewError::Stale);
        }
        if self.confirmed.is_some() {
            return Err(ViewError::RefreshRequired);
        }
        let revision = match &mutation {
            MutationReceipt::Saved(document)
                if token.kind == PendingKind::Save && self.scope.matches(document) =>
            {
                document.draft_revision
            }
            MutationReceipt::Accepted(receipt)
                if token.kind == PendingKind::Accept
                    && receipt.proposal_id == self.scope.proposal_id
                    && receipt.source_id == self.scope.source_id
                    && receipt.source_sha256 == self.scope.source_sha256
                    && receipt.source_revision == self.scope.source_revision =>
            {
                receipt.output_revision
            }
            _ => return Err(ViewError::InvalidSnapshot),
        };
        if revision.0 <= token.revision.0 {
            return Err(ViewError::InvalidSnapshot);
        }
        self.confirmed = Some(mutation);
        Ok(())
    }

    /// Explicit inspect only; no mutation is reissued while a confirmed result needs refresh.
    pub fn complete_refresh(
        &mut self,
        token: &PendingToken,
        inspected: Result<StoredProposal, ApiError>,
    ) -> Result<(), ViewError> {
        if self.pending.as_ref() != Some(token) || token.scope != self.scope {
            return Err(ViewError::Stale);
        }
        let mutation = self.confirmed.clone().ok_or(ViewError::InvalidSnapshot)?;
        let stored = match inspected {
            Ok(stored) => stored,
            Err(error) => {
                self.error = Some(error);
                return Ok(());
            }
        };
        self.confirmed = None;
        let result = self.complete(
            token,
            Ok(ResolvedMutation {
                mutation: mutation.clone(),
                inspected: stored,
            }),
        );
        if result.is_err() {
            self.confirmed = Some(mutation);
            self.pending = Some(token.clone());
        }
        result
    }
}
