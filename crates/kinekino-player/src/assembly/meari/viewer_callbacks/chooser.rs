//! Opt-in host-held metadata and preparation results; never a live Open authority.
use super::codec::{ReadProfile, manifest_for};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        launch::native_proposal::{NativeProposalSelection, NativeProposalTarget},
        proposals::{
            InspectProposal, StoredProposal,
            list::{ListProposals, ProposalListPage, ProposalListRow},
        },
    },
    assembly::meari::viewer_read_actions::{ChooserActions, ChooserIntent},
    domain::proposals::ProposalPayload,
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::TerminalAcceptance;
use meari::application::{
    host_callbacks::{HostCallbacks, LocalActionInvocation, PendingCallback},
    plugins::*,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChooserPhase {
    Idle,
    Listing,
    PageReady,
    Inspecting,
    DocumentReady,
    Prepared,
    Cancelling,
    Failed,
    Closing,
}

#[derive(Clone, Debug)]
pub struct DocumentChoice {
    pub key: String,
    pub metadata: ProposalListRow,
}
#[derive(Clone, Debug)]
pub struct ItemChoice {
    pub key: String,
    pub target: NativeProposalTarget,
}
/// Adapter-local carrier. Consumers must still perform R1 target-bound current inspection.
#[derive(Clone, Debug)]
pub struct PreparedSelection {
    pub scope: PluginScope,
    pub generation: u64,
    pub selection: Arc<NativeProposalSelection>,
}
struct Queued {
    callback: PendingCallback,
    intent: ChooserIntent,
}
enum ReadRequest {
    List(ListProposals),
    Inspect(ProposalListRow),
}
struct Active {
    ticket: PluginTicket,
    generation: u64,
    intent: ChooserIntent,
    request: ReadRequest,
    invalidated: bool,
}
enum ReadResult {
    Page(ProposalListPage),
    Document(Box<StoredProposal>),
}
pub(super) struct Chooser {
    scope: Option<PluginScope>,
    query: ListProposals,
    generation: u64,
    counter: u64,
    phase: ChooserPhase,
    actions: Option<ChooserActions>,
    queued: Option<Queued>,
    active: Option<Active>,
    page: Option<ProposalListPage>,
    documents: Vec<DocumentChoice>,
    document: Option<StoredProposal>,
    items: Vec<ItemChoice>,
    prepared: Option<PreparedSelection>,
    retained: Option<TerminalAcceptance>,
    staged: Option<Result<ReadResult, PluginError>>,
    failure: Option<PluginError>,
}
fn stale(message: &str) -> PluginError {
    PluginError::new(PluginErrorCode::Stale, message)
}
fn busy() -> PluginError {
    PluginError::new(
        PluginErrorCode::Busy,
        "Chooser retains another original intent",
    )
}
fn invalid(message: &str) -> PluginError {
    PluginError::new(PluginErrorCode::InvalidPayload, message)
}
fn sdk_error(code: ErrorCode, message: &str) -> PluginError {
    let mut error = PluginError::new(PluginErrorCode::Adapter, message);
    let manifest = manifest_for(ReadProfile::ProposalChooser);
    if let Some(schema) = &manifest.capabilities["proposals.list"].error_schema {
        error.details = PluginPayload::encode(schema.clone(), &ApiError::new(code, message)).ok();
    }
    error
}
impl Chooser {
    pub(super) fn new(mut query: ListProposals) -> Result<Self, PluginError> {
        if query.cursor.is_some() {
            return Err(invalid("Initial chooser query cannot carry a cursor"));
        }
        query
            .validate()
            .map_err(|e| sdk_error(e.code, &e.message))?;
        query.cursor = None;
        Ok(Self {
            scope: None,
            query,
            generation: 0,
            counter: 0,
            phase: ChooserPhase::Idle,
            actions: None,
            queued: None,
            active: None,
            page: None,
            documents: Vec::new(),
            document: None,
            items: Vec::new(),
            prepared: None,
            retained: None,
            staged: None,
            failure: None,
        })
    }
    pub(super) fn bind(
        &mut self,
        callbacks: &mut HostCallbacks,
        scope: &PluginScope,
    ) -> Result<(), PluginError> {
        self.advance()?;
        self.scope = Some(scope.clone());
        self.retire_choices();
        self.phase = ChooserPhase::Idle;
        self.actions = Some(ChooserActions::bind(callbacks)?);
        Ok(())
    }
    fn advance(&mut self) -> Result<(), PluginError> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| stale("Chooser generation exhausted"))?;
        Ok(())
    }
    fn key(&mut self, kind: &str) -> Result<String, PluginError> {
        self.counter = self
            .counter
            .checked_add(1)
            .ok_or_else(|| stale("Chooser key counter exhausted"))?;
        Ok(format!(
            "chooser-{kind}-{}-{}",
            self.generation, self.counter
        ))
    }
    fn retire_choices(&mut self) {
        self.documents.clear();
        self.document = None;
        self.items.clear();
        self.prepared = None;
    }
    pub(super) fn phase(&self) -> ChooserPhase {
        self.phase
    }
    pub(super) fn documents(&self) -> &[DocumentChoice] {
        &self.documents
    }
    pub(super) fn items(&self) -> &[ItemChoice] {
        &self.items
    }
    pub(super) fn prepared(&self) -> Option<&PreparedSelection> {
        self.prepared.as_ref().filter(|_| {
            self.queued.is_none() && self.active.is_none() && self.phase == ChooserPhase::Prepared
        })
    }
    pub(super) fn matches_prepared(&self, candidate: &PreparedSelection) -> bool {
        self.prepared().is_some_and(|current| {
            current.scope == candidate.scope
                && current.generation == candidate.generation
                && Arc::ptr_eq(&current.selection, &candidate.selection)
        })
    }
    pub(super) fn failure(&self) -> Option<&PluginError> {
        self.failure.as_ref()
    }
    pub(super) fn retained(&self) -> Option<&TerminalAcceptance> {
        self.retained.as_ref()
    }
    pub(super) fn page(&self) -> Option<&ProposalListPage> {
        self.page.as_ref()
    }
    pub(super) fn document(&self) -> Option<&StoredProposal> {
        self.document.as_ref()
    }
    pub(super) fn owns(&self, ticket: &PluginTicket) -> bool {
        self.active
            .as_ref()
            .is_some_and(|active| active.ticket == *ticket)
    }
    pub(super) fn active_ticket(&self) -> Option<&PluginTicket> {
        self.active.as_ref().map(|active| &active.ticket)
    }
    pub(super) fn cancel_requested(&mut self) {
        self.queued = None;
        self.retire_choices();
        if let Some(active) = &mut self.active {
            active.invalidated = true;
            if self.phase != ChooserPhase::Closing {
                self.phase = ChooserPhase::Cancelling;
            }
        } else if self.phase != ChooserPhase::Closing {
            self.phase = ChooserPhase::Idle;
        }
    }
    pub(super) fn set_query(&mut self, query: ListProposals) -> Result<(), PluginError> {
        if self.active.is_some() || self.queued.is_some() {
            return Err(busy());
        }
        if self.phase == ChooserPhase::Closing {
            return Err(stale("Chooser is closing"));
        }
        if query.cursor.is_some() {
            return Err(invalid("Chooser query cannot import a cursor"));
        }
        query
            .validate()
            .map_err(|error| sdk_error(error.code, &error.message))?;
        self.advance()?;
        self.retire_choices();
        self.page = None;
        self.query = query;
        self.phase = ChooserPhase::Idle;
        self.failure = None;
        Ok(())
    }
    fn intent(&self, invocation: &LocalActionInvocation) -> Result<ChooserIntent, PluginError> {
        if self.scope.as_ref() != Some(invocation.scope()) {
            return Err(stale("Chooser callback belongs to another scope"));
        }
        self.actions
            .as_ref()
            .and_then(|actions| actions.intent(invocation))
            .ok_or_else(|| stale("Foreign chooser callback key"))
    }
    pub(super) fn enqueue(
        &mut self,
        invocation: &LocalActionInvocation,
        callback: &PendingCallback,
    ) -> Result<(), PluginError> {
        if self.phase == ChooserPhase::Closing {
            return Err(stale("Chooser is closing"));
        }
        let intent = self.intent(invocation)?;
        if let Some(queued) = &self.queued {
            return if queued.intent == intent {
                Ok(())
            } else {
                Err(busy())
            };
        }
        if let Some(active) = &self.active {
            return if active.intent == intent {
                Ok(())
            } else {
                Err(busy())
            };
        }
        self.validate_intent(&intent)?;
        self.queued = Some(Queued {
            callback: PendingCallback {
                handle: callback.handle.clone(),
                invocation: callback.invocation.clone(),
            },
            intent,
        });
        Ok(())
    }
    fn validate_intent(&self, intent: &ChooserIntent) -> Result<(), PluginError> {
        match intent {
            ChooserIntent::List => Ok(()),
            ChooserIntent::Next
                if self
                    .page
                    .as_ref()
                    .is_some_and(|page| page.next_cursor.is_some())
                    && self.phase == ChooserPhase::PageReady =>
            {
                Ok(())
            }
            ChooserIntent::Document(key) if self.documents.iter().any(|row| row.key == *key) => {
                Ok(())
            }
            ChooserIntent::Item(key) if self.items.iter().any(|row| row.key == *key) => Ok(()),
            _ => Err(stale("Chooser row or page is stale")),
        }
    }
    pub(super) fn drain(
        &mut self,
        callbacks: &HostCallbacks,
        scope: &PluginScope,
        registry: &mut PluginRegistry,
        registration: &RegistrationHandle,
    ) -> Result<(), PluginError> {
        let Some(queued) = self.queued.as_ref() else {
            return Ok(());
        };
        let validated = callbacks
            .dispatch_local(
                &queued.callback.handle,
                queued.callback.invocation.arguments.clone(),
                scope,
            )
            .map_err(|_| stale("Queued chooser callback expired"))
            .and_then(|invocation| self.intent(&invocation));
        if !validated
            .as_ref()
            .is_ok_and(|intent| *intent == queued.intent)
        {
            let error = validated
                .err()
                .unwrap_or_else(|| stale("Queued chooser key changed"));
            self.queued = None;
            self.fault(&error);
            return Err(error);
        }
        let queued = self.queued.take().expect("checked queued chooser intent");
        self.validate_intent(&queued.intent)?;
        if let ChooserIntent::Item(key) = &queued.intent {
            return self.select_item(key);
        }
        let request = self.request(&queued.intent)?;
        self.advance()?;
        match &request {
            ReadRequest::List(_) => {
                self.retire_choices();
                self.phase = ChooserPhase::Listing;
            }
            ReadRequest::Inspect(_) => {
                self.document = None;
                self.items.clear();
                self.prepared = None;
                self.phase = ChooserPhase::Inspecting;
            }
        }
        self.failure = None;
        self.retained = None;
        self.staged = None;
        let (command, payload) = encode_request(&request)?;
        match registry.dispatch(registration, command, payload) {
            Ok(receipt) => {
                self.active = Some(Active {
                    ticket: receipt.ticket,
                    generation: self.generation,
                    intent: queued.intent,
                    request,
                    invalidated: false,
                });
                Ok(())
            }
            Err(error) => {
                self.fault(&error);
                Err(error)
            }
        }
    }
    fn request(&self, intent: &ChooserIntent) -> Result<ReadRequest, PluginError> {
        match intent {
            ChooserIntent::List => Ok(ReadRequest::List(self.query.clone())),
            ChooserIntent::Next => {
                let mut query = self.query.clone();
                query.cursor = self.page.as_ref().and_then(|page| page.next_cursor.clone());
                if query.cursor.is_none() {
                    return Err(stale("No chooser continuation"));
                }
                Ok(ReadRequest::List(query))
            }
            ChooserIntent::Document(key) => self
                .documents
                .iter()
                .find(|row| row.key == *key)
                .map(|row| ReadRequest::Inspect(row.metadata.clone()))
                .ok_or_else(|| stale("Unknown document key")),
            ChooserIntent::Item(_) => Err(invalid("Item choice is local only")),
        }
    }
    fn select_item(&mut self, key: &str) -> Result<(), PluginError> {
        let target = self
            .items
            .iter()
            .find(|item| item.key == key)
            .ok_or_else(|| stale("Unknown item key"))?
            .target
            .clone();
        if self
            .prepared
            .as_ref()
            .is_some_and(|prepared| prepared.selection.target == target)
        {
            self.phase = ChooserPhase::Prepared;
            return Ok(());
        }
        self.advance()?;
        let stored = self
            .document
            .as_ref()
            .ok_or_else(|| stale("No prepared full document"))?;
        let document = &stored.document;
        self.prepared = Some(PreparedSelection {
            scope: self
                .scope
                .clone()
                .ok_or_else(|| stale("Chooser scope missing"))?,
            generation: self.generation,
            selection: Arc::new(NativeProposalSelection {
                proposal_id: document.proposal_id.clone(),
                document_revision: document.draft_revision,
                source_id: document.source.source_id().into(),
                source_revision: document.source.source_revision(),
                source_sha256: document.source.source_sha256().into(),
                target,
            }),
        });
        self.phase = ChooserPhase::Prepared;
        Ok(())
    }
    pub(super) fn retain(&mut self, acceptance: TerminalAcceptance) {
        let Some(active) = &self.active else {
            return;
        };
        if active.ticket != acceptance.event.ticket {
            return;
        }
        let acknowledged = acceptance.owner_ack.is_ok();
        self.retained = Some(acceptance);
        if self.staged.is_none() {
            self.staged = Some(decode_result(
                &active.request,
                &self
                    .retained
                    .as_ref()
                    .expect("retained original acceptance")
                    .event,
            ));
        }
        if !acknowledged {
            return;
        }
        let active = self.active.take().expect("matched active ticket");
        if active.generation != self.generation
            || active.invalidated
            || self.phase == ChooserPhase::Closing
        {
            self.staged = None;
            if self.phase != ChooserPhase::Closing {
                self.phase = ChooserPhase::Idle;
            }
            return;
        }
        let result = self
            .staged
            .take()
            .expect("decoded retained terminal")
            .and_then(|result| self.install(result));
        if let Err(error) = result {
            self.fault(&error);
        }
    }
    fn install(&mut self, result: ReadResult) -> Result<(), PluginError> {
        self.failure = None;
        match result {
            ReadResult::Page(page) => {
                let mut rows = Vec::with_capacity(page.rows.len());
                for metadata in &page.rows {
                    rows.push(DocumentChoice {
                        key: self.key("document")?,
                        metadata: metadata.clone(),
                    });
                }
                self.page = Some(page);
                self.documents = rows;
                self.phase = ChooserPhase::PageReady;
            }
            ReadResult::Document(stored) => {
                let mut items = Vec::new();
                for target in targets(&stored)? {
                    items.push(ItemChoice {
                        key: self.key("item")?,
                        target,
                    });
                }
                self.items = items;
                self.document = Some(*stored);
                self.phase = ChooserPhase::DocumentReady;
            }
        }
        Ok(())
    }
    pub(super) fn fault(&mut self, error: &PluginError) {
        self.failure = Some(error.clone());
        self.retire_choices();
        if self.phase != ChooserPhase::Closing {
            self.phase = ChooserPhase::Failed;
        }
    }
    pub(super) fn close(&mut self) {
        self.queued = None;
        self.retire_choices();
        self.phase = ChooserPhase::Closing;
    }
}
fn encode_request(request: &ReadRequest) -> Result<(&'static str, PluginPayload), PluginError> {
    let manifest = manifest_for(ReadProfile::ProposalChooser);
    match request {
        ReadRequest::List(query) => Ok((
            "proposals.list",
            PluginPayload::encode(
                manifest.capabilities["proposals.list"]
                    .request_schema
                    .clone(),
                query,
            )?,
        )),
        ReadRequest::Inspect(row) => Ok((
            "proposals.inspect",
            PluginPayload::encode(
                manifest.capabilities["proposals.inspect"]
                    .request_schema
                    .clone(),
                &InspectProposal {
                    proposal_id: row.proposal_id.clone(),
                },
            )?,
        )),
    }
}
fn decode_result(request: &ReadRequest, event: &PluginEvent) -> Result<ReadResult, PluginError> {
    let payload = match &event.event {
        PluginEventKind::Completed(payload) => payload,
        PluginEventKind::Failed(error) => return Err(error.clone()),
        _ => return Err(invalid("Expected terminal chooser result")),
    };
    let manifest = manifest_for(ReadProfile::ProposalChooser);
    let schema = &manifest.capabilities[event.ticket.capability()].reply_schema;
    match request {
        ReadRequest::List(query) => {
            let page: ProposalListPage = payload.decode(schema)?;
            validate_page(&page, query)?;
            Ok(ReadResult::Page(page))
        }
        ReadRequest::Inspect(row) => {
            let stored: StoredProposal = payload.decode(schema)?;
            validate_document(&stored, row)?;
            Ok(ReadResult::Document(Box::new(stored)))
        }
    }
}
fn item_count(stored: &StoredProposal) -> usize {
    match &stored.document.draft {
        Some(ProposalPayload::Cards { cards }) => cards.len(),
        Some(ProposalPayload::Highlights { highlights }) => highlights.len(),
        Some(ProposalPayload::Correction { edits }) => edits.len(),
        Some(ProposalPayload::DocumentCorrection { edits }) => edits.len(),
        None => 0,
    }
}
fn validate_document(stored: &StoredProposal, row: &ProposalListRow) -> Result<(), PluginError> {
    stored.validate().map_err(invalid)?;
    let document = &stored.document;
    if document.proposal_id != row.proposal_id
        || document.generated.task != row.task
        || document.draft_revision != row.draft_revision
        || document.state != row.draft_state
        || document.source.source_id() != row.source_id
        || document.source.source_revision() != row.source_revision
        || document.source.source_sha256() != row.source_sha256
        || item_count(stored) != row.typed_item_count as usize
    {
        return Err(stale(
            "Preparation inspect differs from selected metadata; explicit refresh required",
        ));
    }
    Ok(())
}
fn validate_page(page: &ProposalListPage, query: &ListProposals) -> Result<(), PluginError> {
    use crate::domain::identifiers::{valid_id, valid_sha256};
    if page.schema_version != "proposal-list/1"
        || page.rows.len() > query.page_limit()
        || !valid_sha256(&page.dataset_witness)
        || !valid_sha256(&page.snapshot_witness)
        || page.next_cursor.as_ref().is_some_and(|cursor| {
            cursor.is_empty()
                || cursor.len() > crate::application::proposals::list::MAX_CURSOR_BYTES
        })
        || page
            .rows
            .windows(2)
            .any(|pair| pair[0].proposal_id >= pair[1].proposal_id)
        || page.rows.iter().any(|row| {
            !valid_id(&row.proposal_id)
                || !valid_id(&row.source_id)
                || !valid_sha256(&row.source_sha256)
                || row.draft_revision.0 == 0
                || query.task.is_some_and(|task| task != row.task)
        })
    {
        return Err(invalid(
            "Chooser page differs from bounded typed list contract",
        ));
    }
    Ok(())
}
fn targets(stored: &StoredProposal) -> Result<Vec<NativeProposalTarget>, PluginError> {
    if item_count(stored) > 4096 {
        return Err(sdk_error(
            ErrorCode::ResourceExhausted,
            "Chooser item key budget exceeded",
        ));
    }
    let Some(payload) = &stored.document.draft else {
        return Ok(vec![NativeProposalTarget::ManualRepair {
            task: stored.document.generated.task,
        }]);
    };
    Ok(match payload {
        ProposalPayload::Cards { cards } => (0..cards.len())
            .map(|index| NativeProposalTarget::Card {
                index: index as u32,
            })
            .collect(),
        ProposalPayload::Highlights { highlights } => (0..highlights.len())
            .map(|index| NativeProposalTarget::Highlight {
                index: index as u32,
            })
            .collect(),
        ProposalPayload::Correction { edits } => edits
            .iter()
            .enumerate()
            .map(|(index, edit)| NativeProposalTarget::CueCorrection {
                index: index as u32,
                cue_id: edit.cue_id.clone(),
            })
            .collect(),
        ProposalPayload::DocumentCorrection { edits } => edits
            .iter()
            .enumerate()
            .map(|(index, edit)| NativeProposalTarget::DocumentCorrection {
                index: index as u32,
                source_ref: edit.source_ref.clone(),
            })
            .collect(),
    })
}
#[cfg(test)]
mod tests;
