//! Read-only declared metadata, never media admission or quality approval.
use super::{
    InspectProposal, ProposalStore, StoredProposal, source_import::ImportedRequestContext,
};
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::proposals::SourceSnapshot,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
pub const COMMAND: &str = "proposals.inspect_source_context";
pub const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceContextInspection {
    pub schema_version: String,
    pub state: StoredProposal,
    pub context: ImportedRequestContext,
    pub timing_authority: String,
    pub media_binding: String,
    pub quality_approved: bool,
}
pub fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Historical source context reader is unavailable",
    )
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        "Source context does not match its retained proposal",
    )
}
impl SourceContextInspection {
    pub(crate) fn new(
        state: StoredProposal,
        context: ImportedRequestContext,
    ) -> Result<Self, ApiError> {
        let value = Self {
            schema_version: "proposal-source-context/1".into(),
            state,
            context,
            timing_authority: "declared_request_only".into(),
            media_binding: "not_verified".into(),
            quality_approved: false,
        };
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.schema_version != "proposal-source-context/1"
            || self.timing_authority != "declared_request_only"
            || self.media_binding != "not_verified"
            || self.quality_approved
            || self.state.historical.is_none()
        {
            return Err(invalid());
        }
        self.state.validate().map_err(|_| invalid())?;
        self.context.source.validate().map_err(|_| invalid())?;
        if serde_json::to_value(&self.state.document.source).map_err(|_| invalid())?
            != serde_json::to_value(&self.context.source).map_err(|_| invalid())?
        {
            return Err(invalid());
        }
        match &self.context.source {
            SourceSnapshot::Document(_) if self.context.declared_cue_times.is_empty() => {}
            SourceSnapshot::Cue(source)
                if source.cues.len() == self.context.declared_cue_times.len() =>
            {
                for (cue, time) in source.cues.iter().zip(&self.context.declared_cue_times) {
                    if cue.cue_id != time.cue_id
                        || cue.start_us.is_some()
                        || cue.end_us.is_some()
                        || time.declared_in_us.0 >= time.declared_out_us.0
                    {
                        return Err(invalid());
                    }
                }
            }
            _ => return Err(invalid()),
        }
        Ok(())
    }
}
pub fn inspect(
    store: &dyn ProposalStore,
    input: InspectProposal,
    request_id: &str,
) -> Result<SourceContextInspection, ApiError> {
    super::identity(&input.proposal_id)?;
    if !store.source_context_available() {
        return Err(unavailable());
    }
    let value = store.inspect_source_context(&input.proposal_id)?;
    value.validate()?;
    if value.state.document.proposal_id != input.proposal_id {
        return Err(invalid());
    }
    // Count before allocating the transport Value, then enforce the full envelope.
    if !matches!(
        crate::application::sdk_execution::encoded_size(
            &value,
            MAX_RESPONSE_BYTES.saturating_sub(4096)
        ),
        crate::application::sdk_execution::EncodingMeasurement::Exact(_)
    ) || request_id.len() > 1024
    {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Source context response exceeds its byte budget",
        ));
    }
    let envelope = crate::application::api::Response {
        api_version: crate::application::api::API_VERSION,
        request_id: request_id.into(),
        ok: true,
        data: Some(serde_json::to_value(&value).map_err(|_| invalid())?),
        error: None,
    };
    if !matches!(
        crate::application::sdk_execution::encoded_size(&envelope, MAX_RESPONSE_BYTES),
        crate::application::sdk_execution::EncodingMeasurement::Exact(_)
    ) {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Source context response envelope exceeds its byte budget",
        ));
    }
    Ok(value)
}
pub fn response_schema() -> Value {
    let decimal = json!({"type":"string","pattern":"^[0-9]+$"});
    let mut source = super::schema::source_type();
    for key in ["start_us", "end_us"] {
        source["oneOf"][0]["properties"]["cues"]["items"]["properties"][key] =
            json!({"type":"null"});
    }
    json!({"type":"object","additionalProperties":false,"required":["schema_version","state","context","timing_authority","media_binding","quality_approved"],"properties":{
        "schema_version":{"const":"proposal-source-context/1"},"state":super::schema::response_type("proposals.import_historical"),
        "context":{"type":"object","additionalProperties":false,"required":["source","declared_cue_times"],"properties":{"source":source,"declared_cue_times":{"type":"array","maxItems":4096,"items":{"type":"object","additionalProperties":false,"required":["cue_id","declared_in_us","declared_out_us"],"properties":{"cue_id":{"type":"string","maxLength":80},"declared_in_us":decimal,"declared_out_us":decimal}}}}},
        "timing_authority":{"const":"declared_request_only"},"media_binding":{"const":"not_verified"},"quality_approved":{"const":false}}})
}
pub fn describe(input: &Value, available: bool) -> Result<Value, ApiError> {
    if input.as_object().is_none_or(|v| v.len() != 1) || input["command"] != COMMAND {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "schema requires only this command",
        ));
    }
    let mut context = crate::application::api::schema::context_schema();
    for key in ["expected_revision", "idempotency_key", "timeout_ms"] {
        context["properties"][key] = json!({"type":"null"});
    }
    Ok(
        json!({"schema_version":"1","command":COMMAND,"status":if available {"supported"} else {"unsupported"},"request":{"type":"object","additionalProperties":false,"required":["context","command","payload"],"properties":{"context":context,"command":{"const":COMMAND},"payload":{"type":"object","additionalProperties":false,"required":["proposal_id"],"properties":{"proposal_id":{"type":"string","minLength":1,"maxLength":80}}}}},"response":{"type":"object","required":["api_version","request_id","ok","data","error"],"properties":{"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[response_schema(),{"type":"null"}]},"error":{"type":["object","null"]}}},"error_codes":ErrorCode::ALL,"response_max_bytes":MAX_RESPONSE_BYTES,"timing_authority":"declared_request_only","media_binding":"not_verified","in_flight_cancel":false,"sdk_execution_queued_cancel":true}),
    )
}
pub fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"] == COMMAND {
                command["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["proposals"]["source_context"] = json!({"available":available,"timing_authority":"declared_request_only","media_binding":"not_verified","quality_approved":false,"response_max_bytes":MAX_RESPONSE_BYTES});
}
