use serde_json::{Value, json};

pub(super) fn response() -> Value {
    let decimal = json!({"type":"string","pattern":"^[0-9]+$"});
    let sha = json!({"type":"string","pattern":"^[0-9a-f]{64}$"});
    let entry = json!({"type":"object","additionalProperties":false,"required":["id","title","url"],"properties":{"id":{"type":"string"},"title":{"type":"string"},"url":{"type":"string","format":"uri"}}});
    let artifact = json!({"type":"object","additionalProperties":false,"required":["artifact_id","body_sha256","body_bytes","body_path","metadata_path"],"properties":{"artifact_id":sha,"body_sha256":sha,"body_bytes":decimal,"body_path":{"type":"string","pattern":"^objects/[0-9a-f]{64}\\.body$"},"metadata_path":{"type":"string","pattern":"^captures/[0-9a-f]{64}\\.json$"}}});
    let capture = json!({"type":"object","additionalProperties":false,"required":["requested_url","final_url","redirects","observed_at","parser_revision","status","content_type","content_encoding","body_sha256","body_bytes"],"properties":{"requested_url":{"type":"string","format":"uri"},"final_url":{"type":"string","format":"uri"},"redirects":{"type":"array","maxItems":5,"items":{"type":"string","format":"uri"}},"observed_at":{"type":"string","format":"date-time","description":"UTC RFC3339"},"parser_revision":{"type":"string"},"status":{"type":"integer","minimum":100,"maximum":599},"content_type":{"type":"string"},"content_encoding":{"type":["string","null"]},"body_sha256":sha,"body_bytes":decimal}});
    let mut capture = capture;
    capture["properties"]["status"]["type"] = json!(["integer", "null"]);
    capture["properties"]["capture_kind"] = json!({"enum":["http_entity","dom_snapshot","browser_response_body","manifest_prefix","browser_event_trace","frame_dom_snapshot","episode_resolution","native_engine_diagnostic"]});
    capture["properties"]["browser"] = json!({"type":["object","null"],"description":"Observed engine/ticket/document generations/navigation/filter facts; DOM has null HTTP status"});
    capture["required"]
        .as_array_mut()
        .unwrap()
        .extend([json!("capture_kind"), json!("browser")]);
    let result = json!({"type":"object","additionalProperties":false,"required":["sha256","bytes","kind","operation","source_descriptor_sha256"],"properties":{"sha256":sha,"bytes":decimal,"kind":{"const":"catalog_result"},"operation":{"enum":["catalog.search","episodes.list"]},"source_descriptor_sha256":sha}});
    let data = json!({"type":"object","additionalProperties":false,"required":["title","series","entries","next_page","raw_artifact","capture","site_verified","result_artifact"],"properties":{"title":{"type":"string"},"series":{"anyOf":[entry,{"type":"null"}]},"entries":{"type":"array","maxItems":2000,"items":entry},"next_page":{"type":["string","null"]},"raw_artifact":artifact,"capture":capture,"site_verified":{"const":false},"result_artifact":{"anyOf":[result,{"type":"null"}]}}});
    let error = json!({"type":"object","required":["code","message","retryable","details","remediation","request_id"],"properties":{"code":{"enum":super::ErrorCode::ALL},"message":{"type":"string"},"retryable":{"type":"boolean"},"details":{"type":"object","description":"Received complete responses retain raw_artifact/capture; transport before complete body uses null. Transport diagnostics contain sanitized classifications only."},"remediation":{"type":"string"},"request_id":{"type":"string"}}});
    json!({"type":"object","additionalProperties":false,"required":["api_version","request_id","ok","data","error"],"properties":{"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[data,{"type":"null"}]},"error":{"anyOf":[error,{"type":"null"}]}}})
}
