use super::{ApiError, ErrorCode, Request};
use crate::application::catalog::CatalogApi;
use serde_json::{Value, json};
pub fn is_command(command: &str) -> bool {
    matches!(
        command,
        "catalog.search" | "catalog.read_raw" | "episodes.list" | "episodes.inspect_resolution"
    )
}
pub fn execute(request: &Request, catalog: &dyn CatalogApi) -> Result<Value, ApiError> {
    if !catalog.available() {
        return Err(crate::application::catalog::unavailable());
    }
    if request.context.idempotency_key.is_some()
        || request.context.expected_revision.is_some()
        || request.context.timeout_ms.is_some()
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Catalog reads use a fixed 15-second HTTP budget and do not accept context mutation or timeout fields",
        ));
    }
    let invalid = |e: serde_json::Error| ApiError::new(ErrorCode::InvalidArgument, e.to_string());
    if request.command == "episodes.inspect_resolution" {
        if !catalog.episode_resolution_read_available() {
            return Err(crate::application::catalog::unavailable());
        }
        return serde_json::to_value(catalog.inspect_episode_resolution(
            serde_json::from_value(request.payload.clone()).map_err(invalid)?,
        )?)
        .map_err(|_| {
            ApiError::new(
                ErrorCode::Internal,
                "Episode resolution inspection serialization failed",
            )
        });
    }
    if request.command == "catalog.read_raw" {
        if !catalog.raw_read_available() {
            return Err(crate::application::catalog::unavailable());
        }
        return serde_json::to_value(
            catalog.read_raw(serde_json::from_value(request.payload.clone()).map_err(invalid)?)?,
        )
        .map_err(|_| {
            ApiError::new(
                ErrorCode::Internal,
                "Catalog raw chunk serialization failed",
            )
        });
    }
    if request.payload.get("result_artifact_sha256").is_some() {
        if !catalog.result_read_available() {
            return Err(crate::application::catalog::unavailable());
        }
        let input = serde_json::from_value(request.payload.clone()).map_err(invalid)?;
        return serde_json::to_value(catalog.read_result(&request.command, input)?).map_err(|_| {
            ApiError::new(ErrorCode::Internal, "Catalog result serialization failed")
        });
    }
    if !catalog.live_available() {
        return Err(crate::application::catalog::unavailable());
    }
    let result = match request.command.as_str() {
        "catalog.search" => {
            catalog.search(serde_json::from_value(request.payload.clone()).map_err(invalid)?)?
        }
        "episodes.list" => {
            catalog.episodes(serde_json::from_value(request.payload.clone()).map_err(invalid)?)?
        }
        _ => return Err(crate::application::catalog::unavailable()),
    };
    serde_json::to_value(result)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Catalog response serialization failed"))
}
pub fn capabilities(value: &mut Value, catalog: &dyn CatalogApi) {
    let available = catalog.available();
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if let Some(name) = command["command"].as_str().filter(|name| is_command(name)) {
                let command_available = if name == "episodes.inspect_resolution" {
                    catalog.episode_resolution_read_available()
                } else if name == "catalog.read_raw" {
                    catalog.raw_read_available()
                } else {
                    available
                };
                command["status"] = json!(if command_available {
                    "experimental"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["catalog"] = json!({"backend_available":available,"live_read_available":catalog.live_available(),"result_read_available":catalog.result_read_available(),"raw_read_available":catalog.raw_read_available(),"raw_chunk_max_bytes":65536,"site_verified":false,"status":if available {"experimental"} else {"unsupported"},"maturity":"experimental","category_listing_only":true,"categories":["airing","completed","movies"],"text_search":false,"genres":false,"sort":false,"media_resolve":false,"raw_inline":false,"body_max_bytes":2097152,"http_timeout_ms":15000,"automatic_retry":false});
    value["catalog"]["episode_resolution_read_available"] =
        json!(catalog.episode_resolution_read_available());
}
pub fn schema(payload: &Value, catalog: &dyn CatalogApi) -> Result<Value, ApiError> {
    let available = catalog.available();
    let command = payload["command"]
        .as_str()
        .filter(|v| is_command(v))
        .ok_or_else(|| {
            ApiError::new(ErrorCode::InvalidArgument, "Unknown catalog schema command")
        })?;
    if payload.as_object().is_none_or(|o| o.len() != 1) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Unknown schema field",
        ));
    }
    if command == "catalog.read_raw" {
        return super::catalog_raw_schema::describe(catalog.raw_read_available());
    }
    if command == "episodes.inspect_resolution" {
        return super::episode_resolution_schema::describe(
            catalog.episode_resolution_read_available(),
        );
    }
    let fields = live_payload_schema(command);
    let example = if command == "catalog.search" {
        json!({"category":"airing","page":1})
    } else {
        json!({"series_url":"https://catalog.example.test/c/example"})
    };
    let mut context = super::schema::context_schema();
    for field in ["timeout_ms", "idempotency_key", "expected_revision"] {
        context["properties"][field] = json!({"type":"null"});
    }
    let mut request = Request::new(command);
    request.payload = example;
    if !catalog.live_available() && catalog.result_read_available() {
        request.payload = json!({"result_artifact_sha256":"0000000000000000000000000000000000000000000000000000000000000000"});
    }
    let stored = json!({"type":"object","additionalProperties":false,"required":["result_artifact_sha256"],"properties":{"result_artifact_sha256":{"type":"string","pattern":"^[0-9a-f]{64}$"}}});
    let fields = if !available || catalog.live_available() && catalog.result_read_available() {
        json!({"oneOf":[fields,stored]})
    } else if catalog.result_read_available() {
        stored
    } else {
        fields
    };
    Ok(
        json!({"schema_version":"1","command":command,"status":if available {"experimental"} else {"unsupported"},"maturity":"experimental","live_read_available":catalog.live_available(),"result_read_available":catalog.result_read_available(),"site_verified":false,"scope":"category listing or series episodes only; live HTTP or verified stored browser-job result; actual site selectors unverified","request":{"type":"object","additionalProperties":false,"required":["context","command","payload"],"properties":{"context":context,"command":{"const":command},"payload":fields}},"response":super::catalog_response_schema::response(),"example":request,"raw_artifact":{"body":"objects/<body_sha256>.body","metadata":"captures/<artifact_id>.json","identity":"sha256","paths":"relative to explicit catalog root","observed_at":"UTC RFC3339"},"result_artifact":{"path":"results/<sha256>.json","max_bytes":2097152,"source_identity":"request_descriptor_sha256; not a frozen web page","readback":"same command with result_artifact_sha256 only"},"error_codes":ErrorCode::ALL}),
    )
}
pub(super) fn live_payload_schema(command: &str) -> Value {
    if command == "catalog.search" {
        json!({"type":"object","additionalProperties":false,"required":["category"],"properties":{"category":{"enum":["airing","completed","movies"]},"page":{"type":"integer","minimum":1,"maximum":10000,"default":1}}})
    } else {
        json!({"type":"object","additionalProperties":false,"required":["series_url"],"properties":{"series_url":{"type":"string","description":"Allowed https://catalog.example.test/c/<id> URL; no query/fragment/credentials"}}})
    }
}
