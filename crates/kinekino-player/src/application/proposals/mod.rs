#[path = "ports.rs"]
mod ports;
pub use ports::ProposalStore;
mod capture;
pub mod card_export;
pub mod cue_edit;
pub mod guarded;
pub mod list;
pub mod output_review;
pub mod schema;
pub mod source_context;
pub mod source_import;
pub mod subtitle_export;
use crate::{
    application::api::{ApiError, ErrorCode, Request},
    domain::{api_values::DecimalU64, model_review::valid_id, proposals::*},
};
pub use capture::CapturedProposal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use crate::domain::proposals::StoredProposal;
pub struct UnavailableProposalStore;
impl ProposalStore for UnavailableProposalStore {
    fn available(&self) -> bool {
        false
    }
    fn load(&self, _: &str) -> Result<StoredProposal, ApiError> {
        Err(unavailable())
    }
    fn publish(&self, _: Option<u64>, _: &StoredProposal) -> Result<(), ApiError> {
        Err(unavailable())
    }
}
fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Proposal storage adapter unavailable",
    )
}
fn invalid(m: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, m)
}
fn identity(id: &str) -> Result<(), ApiError> {
    if valid_id(id) {
        Ok(())
    } else {
        Err(invalid("Invalid proposal identity"))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProposal {
    pub proposal_id: String,
    pub source: SourceSnapshot,
    pub generated: GeneratedResult,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectProposal {
    pub proposal_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditProposal {
    pub proposal_id: String,
    pub draft: ProposalPayload,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptProposal {
    pub proposal_id: String,
    pub target_source_revision: DecimalU64,
}
pub fn is_command(s: &str) -> bool {
    matches!(
        s,
        "proposals.create"
            | "proposals.list"
            | "proposals.import_historical"
            | "proposals.inspect"
            | source_context::COMMAND
            | "proposals.edit"
            | "proposals.accept"
    )
}
pub fn create(
    store: &dyn ProposalStore,
    input: CreateProposal,
) -> Result<ProposalDocument, ApiError> {
    create_bound(store, input, None)
}
pub(crate) fn create_bound(
    store: &dyn ProposalStore,
    input: CreateProposal,
    historical: Option<HistoricalBinding>,
) -> Result<ProposalDocument, ApiError> {
    identity(&input.proposal_id)?;
    input.source.validate().map_err(invalid)?;
    if !input.source.supports_task(input.generated.task) {
        return Err(invalid("Source kind does not support this proposal task"));
    }
    if input.generated.raw_text.len() > 262_144 {
        return Err(invalid("Generated text exceeds import limit"));
    }
    let parsed = serde_json::from_str::<ProposalPayload>(&input.generated.raw_text)
        .map_err(|_| "Generated output does not match the typed proposal schema")
        .and_then(|p| {
            p.validate(&input.source, input.generated.task)?;
            Ok(p)
        });
    let (draft, import_error) = match parsed {
        Ok(p) => (Some(p), None),
        Err(e) => (None, Some(e.into())),
    };
    let document = ProposalDocument {
        schema_version: proposal_document_version(&input.source, historical.as_ref()).into(),
        proposal_id: input.proposal_id,
        source: input.source,
        generated: input.generated,
        draft_revision: DecimalU64(1),
        state: if draft.is_some() {
            DraftState::Editable
        } else {
            DraftState::NeedsRepair
        },
        draft,
        import_error,
    };
    let state = StoredProposal {
        document: document.clone(),
        acceptance: None,
        historical,
    };
    crate::application::ai_provenance::verify_bound(store.historical_reader(), &state)?;
    store.publish(None, &state)?;
    Ok(document)
}
pub fn inspect(
    store: &dyn ProposalStore,
    input: InspectProposal,
) -> Result<StoredProposal, ApiError> {
    identity(&input.proposal_id)?;
    verified_load(store, &input.proposal_id)
}
fn verified_load(store: &dyn ProposalStore, id: &str) -> Result<StoredProposal, ApiError> {
    let state = store.load(id)?;
    crate::application::ai_provenance::verify_bound(store.historical_reader(), &state)?;
    Ok(state)
}
fn mutation_error(error: ProposalMutationError) -> ApiError {
    match error {
        ProposalMutationError::InvalidArgument(message) => invalid(message),
        ProposalMutationError::RevisionConflict(message) => {
            ApiError::new(ErrorCode::RevisionConflict, message)
        }
        ProposalMutationError::NeedsRepair => {
            let mut error = ApiError::new(
                ErrorCode::NeedsInput,
                "Imported output needs manual repair before acceptance",
            );
            error.remediation = "edit_proposal_draft".into();
            error
        }
    }
}
pub fn edit(
    store: &dyn ProposalStore,
    input: EditProposal,
    expected: u64,
) -> Result<ProposalDocument, ApiError> {
    identity(&input.proposal_id)?;
    let state = edit_state(
        verified_load(store, &input.proposal_id)?,
        input.draft,
        expected,
    )
    .map_err(mutation_error)?;
    store.publish(Some(expected), &state)?;
    Ok(state.document)
}
pub fn accept(
    store: &dyn ProposalStore,
    input: AcceptProposal,
    expected: u64,
) -> Result<AcceptanceReceipt, ApiError> {
    identity(&input.proposal_id)?;
    let accepted = accept_state(
        verified_load(store, &input.proposal_id)?,
        input.target_source_revision,
        expected,
    )
    .map_err(mutation_error)?;
    if accepted.publication_required {
        store.publish(Some(expected), &accepted.state)?;
    }
    Ok(accepted.receipt)
}
pub fn execute(request: &Request, store: &dyn ProposalStore) -> Result<Value, ApiError> {
    if !store.available() {
        return Err(unavailable());
    }
    if request.context.idempotency_key.is_some() || request.context.timeout_ms.is_some() {
        return Err(invalid(
            "Proposal commands do not accept idempotency_key or timeout_ms",
        ));
    }
    if matches!(
        request.command.as_str(),
        "proposals.create"
            | "proposals.import_historical"
            | "proposals.inspect"
            | "proposals.list"
            | source_context::COMMAND
    ) && request.context.expected_revision.is_some()
    {
        return Err(invalid(
            "This proposal command does not accept expected_revision",
        ));
    }
    fn input<T: serde::de::DeserializeOwned>(v: &Value) -> Result<T, ApiError> {
        serde_json::from_value(v.clone()).map_err(|e| invalid(e.to_string()))
    }
    let expected = || {
        request
            .context
            .expected_revision
            .as_ref()
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| invalid("expected_revision is required for editing and acceptance"))
    };
    let result = match request.command.as_str() {
        "proposals.list" => serde_json::to_value(list::execute(
            store,
            list::ListProposals::decode(request.payload.clone())?,
            &request.context.request_id,
        )?),
        "proposals.create" => serde_json::to_value(create(store, input(&request.payload)?)?),
        "proposals.import_historical" => serde_json::to_value(
            crate::application::ai_provenance::import(store, input(&request.payload)?)?,
        ),
        "proposals.inspect" => serde_json::to_value(inspect(store, input(&request.payload)?)?),
        source_context::COMMAND => serde_json::to_value(source_context::inspect(
            store,
            input(&request.payload)?,
            &request.context.request_id,
        )?),
        "proposals.edit" => {
            serde_json::to_value(edit(store, input(&request.payload)?, expected()?)?)
        }
        "proposals.accept" => {
            serde_json::to_value(accept(store, input(&request.payload)?, expected()?)?)
        }
        _ => return Err(unavailable()),
    };
    result.map_err(|_| ApiError::new(ErrorCode::Internal, "Proposal serialization failed"))
}
