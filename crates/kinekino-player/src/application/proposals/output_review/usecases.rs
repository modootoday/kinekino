use super::*;
use crate::{
    application::api::{Request, RequestContext},
    domain::proposals::output_review::{OUTPUT_HASH_FORMAT, OutputReviewTarget},
};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub fn is_command(command: &str) -> bool {
    matches!(
        command,
        "proposals.inspect_output_review_target"
            | "proposals.record_output_decision"
            | "proposals.output_decisions"
    )
}
fn supported(store: &dyn ProposalStore) -> Result<(), ApiError> {
    if !store.available() || !store.output_review_available() {
        return Err(unavailable());
    }
    Ok(())
}
fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn identity(id: &str) -> Result<(), ApiError> {
    if !valid_id(id) {
        return Err(invalid("Invalid proposal identity"));
    }
    Ok(())
}
pub fn inspect_target(
    store: &dyn ProposalStore,
    input: InspectOutputReviewTarget,
) -> Result<OutputReviewInspection, ApiError> {
    supported(store)?;
    let session = open_session(store, &input.proposal_id)?;
    let target = OutputReviewTarget::from_stored(session.current())
        .map_err(|m| ApiError::new(ErrorCode::NeedsInput, m))?;
    let criteria_document = criteria_for(target.task)?;
    let criteria_identity = criteria_document
        .identity()
        .map_err(|m| ApiError::new(ErrorCode::Internal, m))?;
    session.verify_current_evidence()?;
    let evidence_observation = if session.current().historical.is_some() {
        EvidenceObservation::HistoricalBindingReverifiedInSession
    } else {
        EvidenceObservation::NotHistoricallyBound
    };
    Ok(OutputReviewInspection {
        schema_version: "1".into(),
        state: session.current().clone(),
        target,
        criteria_document,
        criteria_identity,
        evidence_observation,
    })
}
pub fn record_decision(
    store: &dyn ProposalStore,
    input: RecordOutputDecision,
    expected_revision: u64,
) -> Result<StoredOutputDecision, ApiError> {
    supported(store)?;
    identity(&input.proposal_id)?;
    if expected_revision == 0 {
        return Err(invalid("Expected output revision must be positive"));
    }
    if input.proposal_id != input.decision.target.proposal_id {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Output decision route differs from selected proposal",
        ));
    }
    let mut session = open_session(store, &input.proposal_id)?;
    if expected_revision != session.current().document.draft_revision.0
        || expected_revision != input.decision.target.output_revision.0
    {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Accepted output revision changed; inspect before recording",
        ));
    }
    if session.current().acceptance.is_none() {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Current proposal has no accepted output",
        ));
    }
    if input.decision.target.schema_version != "1"
        || input.decision.target.hash_format != OUTPUT_HASH_FORMAT
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Unsupported accepted output target or hash version",
        ));
    }
    input
        .decision
        .target
        .validate_against(session.current())
        .map_err(|m| ApiError::new(ErrorCode::RevisionConflict, m))?;
    let criteria = require_current_criteria(&input.decision.criteria)?;
    let prepared = PreparedOutputDecision::new(input.decision, session.current(), criteria)
        .map_err(invalid)?;
    session.append(&prepared)
}
pub fn output_decisions(
    store: &dyn ProposalStore,
    input: ListOutputDecisions,
) -> Result<OutputDecisionHistory, ApiError> {
    supported(store)?;
    identity(&input.proposal_id)?;
    store.output_decisions(&input.proposal_id)
}
fn input<T: DeserializeOwned>(value: &Value) -> Result<T, ApiError> {
    serde_json::from_value(value.clone()).map_err(|e| invalid(e.to_string()))
}
fn context(context: &RequestContext, record: bool) -> Result<Option<u64>, ApiError> {
    if context.idempotency_key.is_some() || context.timeout_ms.is_some() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Output review transport idempotency and timeout are unavailable",
        ));
    }
    if record {
        return context
            .expected_revision
            .as_ref()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|n| *n > 0)
            .map(Some)
            .ok_or_else(|| invalid("expected_revision must name the current output revision"));
    }
    if context.expected_revision.is_some() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Output review reads do not accept expected_revision",
        ));
    }
    Ok(None)
}
pub fn execute(request: &Request, store: &dyn ProposalStore) -> Result<Value, ApiError> {
    let expected = context(
        &request.context,
        request.command == "proposals.record_output_decision",
    )?;
    let result = match request.command.as_str() {
        "proposals.inspect_output_review_target" => {
            serde_json::to_value(inspect_target(store, input(&request.payload)?)?)
        }
        "proposals.record_output_decision" => serde_json::to_value(record_decision(
            store,
            input(&request.payload)?,
            expected.ok_or_else(|| invalid("Expected output revision required"))?,
        )?),
        "proposals.output_decisions" => {
            serde_json::to_value(output_decisions(store, input(&request.payload)?)?)
        }
        _ => return Err(invalid("Unknown output review command")),
    };
    result.map_err(|_| ApiError::new(ErrorCode::Internal, "Cannot encode output review response"))
}
