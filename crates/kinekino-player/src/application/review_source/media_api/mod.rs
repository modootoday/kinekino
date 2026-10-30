//! Shared SDK preparation and read contracts. Trusted ports own paths and selections.
#[path = "ports.rs"]
mod ports;
use super::{MutationControl, media_preparation::MediaSelectionRelation};
use crate::{
    application::api::{ApiError, ErrorCode, Request},
    domain::review_source::ReviewCase,
};
pub use ports::MediaPreparationPort;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
pub const PREPARE: &str = "reviews.media.prepare";
pub const INSPECT: &str = "reviews.media.inspect";
pub const COMMANDS: &[&str] = &[PREPARE, INSPECT];
pub fn is_command(command: &str) -> bool {
    COMMANDS.contains(&command)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaSelectionRequest {
    pub base_case: ReviewCase,
    pub selection_id: String,
}
impl MediaSelectionRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        self.base_case.validate().map_err(|_| invalid())?;
        if !crate::domain::review_source::valid_review_object_id(&self.selection_id) {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaPreparationReceipt {
    pub selection_id: String,
    pub case: ReviewCase,
    pub relation: MediaSelectionRelation,
    pub replayed: bool,
}
impl MediaPreparationReceipt {
    pub fn validate(&self, input: &MediaSelectionRequest) -> Result<(), ApiError> {
        input.validate()?;
        self.case.validate().map_err(|_| invalid())?;
        let r = &self.relation;
        if self.selection_id != input.selection_id
            || r.schema_version != "review-media-selection/1"
            || r.base_case != input.base_case
            || self.case.package_sha256 == input.base_case.package_sha256
            || self.case.item_id != input.base_case.item_id
            || self.case.source_request_sha256 != input.base_case.source_request_sha256
            || r.selection_authority != "explicit_local_selection"
            || r.historical_media_verified
            || r.timeline_verified
            || r.quality_approved
            || !crate::domain::model_review::valid_sha256(&r.original_candidate_sha256)
            || !crate::domain::review_source::valid_review_object_id(&r.media.asset_id)
            || !crate::domain::model_review::valid_sha256(&r.media.sha256)
            || r.media.bytes.0 == 0
            || r.media.bytes.0 > 1_073_741_824
        {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Prepared media response identity differs",
            ));
        }
        Ok(())
    }
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid media selection request or context",
    )
}
struct Control<'a> {
    deadline: Instant,
    caller: Option<&'a dyn MutationControl>,
}
impl MutationControl for Control<'_> {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if let Some(caller) = self.caller {
            caller.checkpoint()?;
        }
        if Instant::now() >= self.deadline {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Media preparation deadline elapsed",
            ));
        }
        Ok(())
    }
}
pub fn execute(
    port: Option<&dyn MediaPreparationPort>,
    request: &Request,
    caller: Option<&dyn MutationControl>,
) -> Result<Value, ApiError> {
    request.validate()?;
    if !is_command(&request.command)
        || request.context.expected_revision.is_some()
        || request.context.idempotency_key.is_some()
        || (request.command == INSPECT && request.context.timeout_ms.is_some())
    {
        return Err(invalid());
    }
    let input: MediaSelectionRequest =
        serde_json::from_value(request.payload.clone()).map_err(|_| invalid())?;
    input.validate()?;
    let timeout = request.context.timeout_ms.unwrap_or(30_000);
    if timeout > 120_000 {
        return Err(invalid());
    }
    let port = port.ok_or_else(|| {
        ApiError::new(
            ErrorCode::Unsupported,
            "Media preparation adapter unavailable",
        )
    })?;
    let receipt = if request.command == PREPARE {
        if !port.writable() {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Media preparation writes unavailable",
            ));
        }
        let control = Control {
            deadline: Instant::now()
                .checked_add(Duration::from_millis(timeout))
                .ok_or_else(invalid)?,
            caller,
        };
        control.checkpoint()?;
        port.prepare(&input, &control)?
    } else {
        port.inspect(&input)?
    };
    receipt.validate(&input)?;
    serde_json::to_value(receipt).map_err(|_| {
        ApiError::new(
            ErrorCode::EngineFailed,
            "Media preparation response encoding failed",
        )
    })
}
pub fn schema(input: &Value, readable: bool, writable: bool) -> Result<Value, ApiError> {
    let command = input["command"].as_str().ok_or_else(invalid)?;
    if input.as_object().is_none_or(|object| object.len() != 1) || !is_command(command) {
        return Err(invalid());
    }
    let sha = json!({"type":"string","pattern":"^[a-f0-9]{64}$"});
    let id = json!({"type":"string","minLength":1,"maxLength":128,"pattern":"^[A-Za-z0-9_-]+$"});
    let case = json!({"type":"object","additionalProperties":false,"required":["package_sha256","item_id","source_request_sha256"],"properties":{"package_sha256":sha,"item_id":id,"source_request_sha256":sha}});
    let mut context = crate::application::api::schema::context_schema();
    for key in ["expected_revision", "idempotency_key", "timeout_ms"] {
        context["properties"][key] = json!({"type":"null"});
    }
    if command == PREPARE {
        context["properties"]["timeout_ms"] =
            json!({"type":["integer","null"],"minimum":1,"maximum":120000});
    }
    let media = json!({"type":"object","additionalProperties":false,"required":["asset_id","sha256","bytes","media_type"],"properties":{"asset_id":id,"sha256":sha,"bytes":{"type":"string","pattern":"^[1-9][0-9]*$"},"media_type":{"type":"string"}}});
    let relation = json!({"type":"object","additionalProperties":false,"required":["schema_version","base_case","original_candidate_sha256","media","selection_authority","historical_media_verified","timeline_verified","quality_approved"],"properties":{"schema_version":{"const":"review-media-selection/1"},"base_case":case,"original_candidate_sha256":sha,"media":media,"selection_authority":{"const":"explicit_local_selection"},"historical_media_verified":{"const":false},"timeline_verified":{"const":false},"quality_approved":{"const":false}}});
    let data = json!({"type":"object","additionalProperties":false,"required":["selection_id","case","relation","replayed"],"properties":{"selection_id":id,"case":case,"relation":relation,"replayed":{"type":"boolean"}}});
    Ok(
        json!({"schema_version":"1","command":command,"status":if readable && (command == INSPECT || writable) {"supported"}else{"unsupported"},
        "request":{"type":"object","additionalProperties":false,"required":["context","command","payload"],"properties":{"context":context,"command":{"const":command},"payload":{"type":"object","additionalProperties":false,"required":["base_case","selection_id"],"properties":{"base_case":case,"selection_id":id}}}},
        "response":{"type":"object","additionalProperties":false,"required":["api_version","request_id","ok","data","error"],"properties":{"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[data,{"type":"null"}]},"error":{"type":["object","null"]}}},
        "error_codes":ErrorCode::ALL,"strong_cancel_available":false,"publication":"deterministic immutable package; no automatic retry after uncertain delivery"}),
    )
}
pub fn capabilities(value: &mut Value, readable: bool, writable: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"].as_str().is_some_and(is_command) {
                command["status"] =
                    json!(if readable && (command["command"] == INSPECT || writable) {
                        "supported"
                    } else {
                        "unsupported"
                    });
            }
        }
    }
    value["review_media_preparation"] = json!({"read_available":readable,"write_available":readable&&writable,"historical_media_verified":false,"timeline_verified":false,"quality_approved":false});
}
