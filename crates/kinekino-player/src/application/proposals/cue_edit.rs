//! Cue text editing reuses the proposal's immutable source and revision publication.
use super::{Correction, EditProposal, ProposalPayload, ProposalStore};
use crate::application::api::{ApiError, ErrorCode, Request};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const COMMAND: &str = "subtitles.edit_cues";

/// Replaces the complete correction draft; omitted cues retain original source text.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EditCues {
    pub proposal_id: String,
    pub edits: Vec<Correction>,
}

pub fn execute(request: &Request, store: &dyn ProposalStore) -> Result<Value, ApiError> {
    request.validate()?;
    if request.command != COMMAND {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Expected cue edit command",
        ));
    }
    if !store.available() {
        return Err(super::unavailable());
    }
    let input: EditCues = serde_json::from_value(request.payload.clone())
        .map_err(|e| ApiError::new(ErrorCode::InvalidArgument, e.to_string()))?;
    let mut translated = request.clone();
    translated.command = "proposals.edit".into();
    translated.payload = serde_json::to_value(EditProposal {
        proposal_id: input.proposal_id,
        draft: ProposalPayload::Correction { edits: input.edits },
    })
    .map_err(|_| ApiError::new(ErrorCode::Internal, "Cue edit serialization failed"))?;
    super::execute(&translated, store)
}

pub fn describe(input: &Value, available: bool) -> Result<Value, ApiError> {
    if input.as_object().is_none_or(|v| v.len() != 1) || input["command"] != COMMAND {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Expected only cue edit command",
        ));
    }
    let mut schema = super::schema::describe(&json!({"command":"proposals.edit"}), available)?;
    schema["command"] = json!(COMMAND);
    schema["request"]["properties"]["command"] = json!({"const":COMMAND});
    let payload = &mut schema["request"]["properties"]["payload"];
    payload["properties"]
        .as_object_mut()
        .expect("proposal payload schema")
        .remove("draft");
    payload["properties"]["edits"] = super::schema::cue_edits_type();
    payload["required"] = json!(["proposal_id", "edits"]);
    schema["mutation_scope"] = json!(
        "replace_complete_correction_draft; omitted cues use source text; explicit acceptance required"
    );
    schema["output_scope"] =
        json!("new_proposal_draft_revision; no rendering or project overwrite");
    schema
        .as_object_mut()
        .expect("proposal schema")
        .remove("accept_retry");
    schema["cue_timing_edit"] = json!(false);
    schema["in_flight_cancel"] = json!(false);
    schema["quality_approved"] = json!(false);
    Ok(schema)
}

pub fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for item in commands {
            if item["command"] == COMMAND {
                item["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["cue_edit"] = json!({"available":available,"storage":"proposal_store","source_kind":"cue_snapshot_v1","mutation":"replace_complete_correction_draft","omitted_cues":"original_source_text","timing_edit":false,"document_correction":false,"explicit_accept_required":true,"automatic_application_enabled":false,"quality_approved":false,"in_flight_cancel":false,"timeout_supported":false});
}
