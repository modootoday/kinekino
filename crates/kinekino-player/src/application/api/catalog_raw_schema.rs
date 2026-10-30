use super::{ApiError, ErrorCode, Request};
use serde_json::{Value, json};
pub(super) fn describe(available: bool) -> Result<Value, ApiError> {
    let decimal = json!({"type":"string","pattern":"^[0-9]+$"});
    let sha = json!({"type":"string","pattern":"^[0-9a-f]{64}$"});
    let payload = json!({"type":"object","additionalProperties":false,"required":["artifact_id"],"properties":{"artifact_id":{"type":"string","pattern":"^[0-9a-f]{64}$","description":"Metadata artifact ID from raw_artifact.artifact_id; not body SHA"},"offset":decimal,"limit":{"type":"integer","minimum":1,"maximum":65536,"default":16384},"encoding":{"enum":["utf8","hex"],"default":"utf8"}}});
    let data = json!({"type":"object","additionalProperties":false,"required":["raw_artifact","capture","encoding","offset","next_offset","chunk_bytes","chunk_sha256","text"],"properties":{"raw_artifact":{"type":"object","description":"Verified whole-artifact SHA/bytes and relative paths"},"capture":{"type":"object","description":"Original immutable capture metadata; no reinterpretation"},"encoding":{"enum":["utf8","hex"]},"offset":decimal,"next_offset":{"anyOf":[decimal,{"type":"null"}]},"chunk_bytes":decimal,"chunk_sha256":sha,"text":{"type":"string"}}});
    let mut context = super::schema::context_schema();
    for field in ["timeout_ms", "idempotency_key", "expected_revision"] {
        context["properties"][field] = json!({"type":"null"});
    }
    let mut example = Request::new("catalog.read_raw");
    example.payload = json!({"artifact_id":"0000000000000000000000000000000000000000000000000000000000000000","offset":"0","limit":4096,"encoding":"utf8"});
    Ok(
        json!({"schema_version":"1","command":"catalog.read_raw","status":if available {"experimental"}else{"unsupported"},"request":{"type":"object","additionalProperties":false,"required":["context","command","payload"],"properties":{"context":context,"command":{"const":"catalog.read_raw"},"payload":payload}},"response":{"type":"object","required":["api_version","request_id","ok","data","error"],"properties":{"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[data,{"type":"null"}]},"error":{"type":["object","null"]}}},"example":example,"error_codes":ErrorCode::ALL,"semantics":{"network":false,"whole_artifact_sha_verified_each_read":true,"utf8_offset":"byte offset at codepoint boundary; next_offset is authoritative","hex":"two lowercase hex characters per original byte","eof":"next_offset=null; offset may equal total body bytes","max_chunk_bytes":65536}}),
    )
}
