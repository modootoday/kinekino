use super::*;
fn object(fields: Value) -> Value {
    let required: Vec<_> = fields.as_object().unwrap().keys().cloned().collect();
    json!({"type":"object","additionalProperties":false,"properties":fields,"required":required})
}
fn id() -> Value {
    json!({"type":"string","pattern":"^[A-Za-z0-9_.-]{1,80}$"})
}
fn sha() -> Value {
    json!({"type":"string","pattern":"^[a-f0-9]{64}$"})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^(0|[1-9][0-9]*)$"})
}
fn file() -> Value {
    object(
        json!({"name":{"type":"string","maxLength":80},"sha256":sha(),"bytes":decimal(),"media_type":{"enum":["image/png","application/json"]}}),
    )
}
fn response() -> Value {
    let pin =
        crate::application::asset_bundle::AssetBundleRequest::schema()["properties"]["project"]
            .clone();
    let input = crate::application::api::asset_projects::schema::frame_input();
    let binding = object(
        json!({"frame_id":id(),"image":input["properties"]["image"],"mask":input["properties"]["mask"]}),
    );
    let manifest = object(
        json!({"schema_version":{"const":"asset-sources/1"},"project":pin,"project_file":file(),"bindings":{"type":"array","maxItems":4096,"items":binding},"originals":{"type":"array","maxItems":8192,"items":file()},"missing_frames":{"type":"array","maxItems":4096,"items":id()},"quality_approved":{"const":false},"review_admitted":{"const":false}}),
    );
    let publication =
        crate::application::api::asset_projects::schema::response()["properties"]["publication"]
            .clone();
    let data = object(
        json!({"archive_id":id(),"manifest_sha256":sha(),"manifest_bytes":decimal(),"manifest":manifest,"project":publication}),
    );
    object(
        json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[data,{"type":"null"}]},"error":{"type":["object","null"]}}),
    )
}
pub(in crate::application::api) fn describe(
    payload: &Value,
    available: bool,
) -> Result<Value, ApiError> {
    let command = payload["command"]
        .as_str()
        .filter(|c| is_command(c))
        .ok_or_else(|| {
            ApiError::new(ErrorCode::InvalidArgument, "Unknown source schema command")
        })?;
    if payload.as_object().is_none_or(|p| p.len() != 1) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Schema accepts only command",
        ));
    }
    let input = if command == "asset_sources.export" {
        object(
            json!({"archive_id":id(),"project":crate::application::asset_bundle::AssetBundleRequest::schema()["properties"]["project"],"missing_frames":{"enum":["reject","preserve_gap"]}}),
        )
    } else {
        object(json!({"archive_id":id(),"manifest_sha256":sha()}))
    };
    let mut context = crate::application::api::schema::context_schema();
    for field in ["timeout_ms", "idempotency_key", "expected_revision"] {
        context["properties"][field] = json!({"type":"null"});
    }
    Ok(
        json!({"schema_version":"1","command":command,"status":if available{"experimental"}else{"unsupported"},"request":object(json!({"context":context,"command":{"const":command},"payload":input})),"response":response(),"error_codes":ErrorCode::ALL,"constraints":{"backend_available":available,"create_only":true,"missing_policy_explicit":true,"render_units_required":false,"point_in_time_response":true,"jobs_publication":false,"quality_approved":false,"review_admitted":false,"in_flight_cancel":false}}),
    )
}
