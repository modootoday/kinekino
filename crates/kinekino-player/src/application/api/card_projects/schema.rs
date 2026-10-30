use super::*;
use serde_json::json;
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","description":"Decimal u64; numeric range checked by application"})
}
fn id() -> Value {
    json!({"type":"string","minLength":1,"maxLength":80,"pattern":"^[A-Za-z0-9_.-]+$"})
}
fn sha() -> Value {
    json!({"type":"string","pattern":"^[a-f0-9]{64}$"})
}
fn text(bytes: usize) -> Value {
    json!({"type":"string","maxLength":bytes,"description":"UTF8 byte bound and control characters checked by application; draft may be empty"})
}
fn nullable(value: Value) -> Value {
    json!({"anyOf":[value,{"type":"null"}]})
}
fn document() -> Value {
    let content = object(
        json!({"front":text(8192),"back":text(16384),"source_refs":{"type":"array","maxItems":4096,"uniqueItems":true,"items":id()}}),
        &["front", "back", "source_refs"],
    );
    let frame = object(
        json!({"artifact_id":id(),"png_sha256":sha(),"source_sha256":sha(),"declared_source_pts_us":nullable(decimal())}),
        &["artifact_id", "png_sha256", "source_sha256"],
    );
    let card = object(
        json!({"card_id":id(),"role":{"enum":["cover","body","closing"]},"text_kind":{"enum":["quotation","summary","manual"]},"content":content,"frame":nullable(frame),"alt_text_draft":text(8192)}),
        &["card_id", "role", "text_kind", "content", "alt_text_draft"],
    );
    let canvas = object(
        json!({"width":{"type":"integer","minimum":1,"maximum":4096},"height":{"type":"integer","minimum":1,"maximum":4096},"template_id":id(),"template_version":{"type":"string","minLength":1,"maxLength":128},"template_sha256":sha()}),
        &[
            "width",
            "height",
            "template_id",
            "template_version",
            "template_sha256",
        ],
    );
    let layout = object(
        json!({"canvas":canvas,"cards":{"type":"array","maxItems":128,"items":card},"caption_draft":text(16384)}),
        &["canvas", "cards", "caption_draft"],
    );
    object(
        json!({"schema_version":{"const":"1"},"document_id":id(),"revision":decimal(),"source":crate::application::proposals::schema::source_type()["oneOf"][0].clone(),"layout":layout}),
        &[
            "schema_version",
            "document_id",
            "revision",
            "source",
            "layout",
        ],
    )
}
fn response() -> Value {
    let publication = object(
        json!({"schema_version":{"const":"1"},"publication_revision":decimal(),"parent_publication_sha256":nullable(sha()),"document":document()}),
        &[
            "schema_version",
            "publication_revision",
            "parent_publication_sha256",
            "document",
        ],
    );
    object(
        json!({"publication_sha256":sha(),"latest_publication_revision":decimal(),"publication":publication}),
        &[
            "publication_sha256",
            "latest_publication_revision",
            "publication",
        ],
    )
}
pub(in crate::application::api) fn describe(
    payload: &Value,
    available: bool,
) -> Result<Value, ApiError> {
    let command = payload["command"]
        .as_str()
        .filter(|s| super::is_command(s))
        .ok_or_else(|| invalid("Unknown card project schema command"))?;
    if payload.as_object().is_none_or(|o| o.len() != 1) {
        return Err(invalid("Schema accepts only command"));
    }
    let payload = if command == "card_projects.inspect" {
        object(
            json!({"document_id":id(),"publication_revision":nullable(decimal())}),
            &["document_id"],
        )
    } else {
        object(json!({"document":document()}), &["document"])
    };
    let mut context = crate::application::api::schema::context_schema();
    for field in ["timeout_ms", "idempotency_key"] {
        context["properties"][field] = json!({"type":"null"});
    }
    if command == "card_projects.save_revision" {
        context["properties"]["expected_revision"] = decimal();
        context["required"]
            .as_array_mut()
            .expect("context required")
            .push(json!("expected_revision"));
    } else {
        context["properties"]["expected_revision"] = json!({"type":"null"});
    }
    Ok(
        json!({"schema_version":"1","command":command,"status":if available {"supported"} else {"unsupported"},
        "request":object(json!({"context":context,"command":{"const":command},"payload":payload}), &["context","command","payload"]),
        "response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":nullable(response()),"error":{"type":["object","null"]}}), &["api_version","request_id","ok","data","error"]),
        "error_codes":ErrorCode::ALL,"constraints":{"document_max_bytes":crate::domain::card_document::MAX_DOCUMENT_BYTES,"publication_revision_distinct_from_edit_revision":true,"source_snapshot_immutable":true,"full_history_validated":true,"idempotency_supported":false,"timeout_supported":false,"in_flight_cancel":false,"quality_approved":false,"media_provenance_verified":false,"export_cards_supported":false}}),
    )
}
pub(in crate::application::api) fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for row in commands {
            if row["command"].as_str().is_some_and(super::is_command) {
                row["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["card_projects"] = json!({"backend_available":available,"record_schema_version":"1","immutable_publications":true,"publication_revision_distinct_from_edit_revision":true,"source_snapshot_immutable":true,"full_history_validated":true,"idempotency_supported":false,"timeout_supported":false,"in_flight_cancel":false,"quality_approved":false,"media_provenance_verified":false,"export_cards_supported":false});
}
