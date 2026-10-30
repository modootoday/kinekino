use super::{ApiError, ErrorCode, Request};
use crate::application::{
    clip_render::{self, ClipSourceInspector, InspectClipSource},
    review_source::MutationControl,
};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
pub(super) fn is_command(command: &str) -> bool {
    command == "clip_sources.inspect"
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
struct Deadline(Instant);
impl MutationControl for Deadline {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if Instant::now() >= self.0 {
            Err(ApiError::new(
                ErrorCode::Timeout,
                "Clip source inspection deadline elapsed",
            ))
        } else {
            Ok(())
        }
    }
}
pub(super) fn execute(
    request: &Request,
    inspector: Option<&dyn ClipSourceInspector>,
) -> Result<Value, ApiError> {
    if request.context.idempotency_key.is_some() || request.context.expected_revision.is_some() {
        return Err(invalid(
            "Source inspection accepts neither idempotency nor expected revision",
        ));
    }
    let timeout = request.context.timeout_ms.unwrap_or(30_000);
    if !(1..=120_000).contains(&timeout) {
        return Err(invalid("Source inspection timeout must be 1..120000ms"));
    }
    let input: InspectClipSource = serde_json::from_value(request.payload.clone())
        .map_err(|_| invalid("Expected strict media_id payload; paths are host configured"))?;
    let inspector = inspector.ok_or_else(|| {
        ApiError::new(
            ErrorCode::Unsupported,
            "Explicit clip source inspector unavailable",
        )
    })?;
    serde_json::to_value(clip_render::inspect_clip_source(
        inspector,
        input,
        &Deadline(Instant::now() + Duration::from_millis(timeout)),
    )?)
    .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid source observation response"))
}
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
pub(super) fn describe(payload: &Value, available: bool) -> Result<Value, ApiError> {
    if payload["command"] != "clip_sources.inspect"
        || payload.as_object().is_none_or(|p| p.len() != 1)
    {
        return Err(invalid("Schema accepts only clip_sources.inspect command"));
    }
    let mut context = super::schema::context_schema();
    for field in ["idempotency_key", "expected_revision"] {
        context["properties"][field] = json!({"type":"null"});
    }
    context["properties"]["timeout_ms"] =
        json!({"type":["integer","null"],"minimum":1,"maximum":120000,"default":30000});
    let sha = json!({"type":"string","pattern":"^[0-9a-f]{64}$"});
    let observation = object(
        json!({"source":super::clip_projects::source_schema(),"reference_stream_index":{"type":"integer","minimum":0,"maximum":4294967295u64},"duration_basis":{"enum":["reference_stream_ticks","container_reported_span"]},"raw_probe_sha256":sha,"probe_sha256":sha,"color_transfer":{"type":["string","null"]},"rotation_degrees":{"type":["integer","null"]}}),
        &[
            "source",
            "reference_stream_index",
            "duration_basis",
            "raw_probe_sha256",
            "probe_sha256",
            "color_transfer",
            "rotation_degrees",
        ],
    );
    Ok(
        json!({"schema_version":"1","command":"clip_sources.inspect","status":if available {"supported"} else {"unsupported"},"request":object(json!({"context":context,"command":{"const":"clip_sources.inspect"},"payload":object(json!({"media_id":{"type":"string","pattern":"^[A-Za-z0-9_.-]{1,80}$"}}), &["media_id"])}), &["context","command","payload"]),"response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[observation,{"type":"null"}]},"error":{"type":["object","null"]}}), &["api_version","request_id","ok","data","error"]),"constraints":constraints(available),"error_codes":ErrorCode::ALL}),
    )
}
fn constraints(available: bool) -> Value {
    json!({"backend_available":available,"fixed_catalog_only":true,"paths_in_payload":false,"timeout_max_ms":120000,"in_flight_cancel":false,"observation_is_render_authority":false,"duration_is_last_frame_proof":false,"color_verified":false,"quality_approved":false})
}
pub(super) fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for row in commands {
            if row["command"] == "clip_sources.inspect" {
                row["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["clip_sources"] = constraints(available);
}
