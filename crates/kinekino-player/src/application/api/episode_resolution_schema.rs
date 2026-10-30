use super::{ApiError, Request};
use serde_json::{Value, json};

pub(super) fn describe(available: bool) -> Result<Value, ApiError> {
    let command = "episodes.inspect_resolution";
    let sha = json!({"type":"string","pattern":"^[0-9a-f]{64}$"});
    let mut context = super::schema::context_schema();
    for field in ["timeout_ms", "idempotency_key", "expected_revision"] {
        context["properties"][field] = json!({"type":"null"});
    }
    let payload = json!({"type":"object","additionalProperties":false,"required":["resolution_artifact_id"],"properties":{"resolution_artifact_id":sha}});
    let data = json!({"type":"object","additionalProperties":false,"required":["resolution_artifact_id","source_descriptor_sha256","manifest_artifact_id","validation_profile","manifest","referer_kind","manifest_verified","referenced_resources_verified","media_decoded","native_playback","live_session_capability","url_expiration_verified","source_uris_redacted"],"properties":{"resolution_artifact_id":sha,"source_descriptor_sha256":sha,"manifest_artifact_id":sha,"validation_profile":{"const":"hls-bounded-v1"},"manifest":{"type":"object","description":"Bounded HLS kind, reference counts, encryption metadata and end-list flag"},"referer_kind":{"enum":["absent","empty","url"]},"manifest_verified":{"const":true},"referenced_resources_verified":{"const":false},"media_decoded":{"const":false},"native_playback":{"const":false},"live_session_capability":{"const":false},"url_expiration_verified":{"const":false},"source_uris_redacted":{"const":true}}});
    let mut example = Request::new(command);
    example.payload = json!({"resolution_artifact_id":"0000000000000000000000000000000000000000000000000000000000000000"});
    Ok(
        json!({"schema_version":"1","command":command,"status":if available {"experimental"} else {"unsupported"},"request":{"type":"object","additionalProperties":false,"required":["context","command","payload"],"properties":{"context":context,"command":{"const":command},"payload":payload}},"response":{"type":"object","required":["api_version","request_id","ok","data","error"],"properties":{"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[data,{"type":"null"}]},"error":{"type":["object","null"]}}},"example":example,"network":false,"semantics":"Reverify stored resolution, referenced original bytes, header provenance and HLS; historical evidence only, no decoding/playback/live URL claim"}),
    )
}
