//! SDK wire contract, shared by local and transported clients.
use super::api::{COMMANDS, is_command, is_read};
use crate::application::api::{ApiError, ErrorCode};
use serde_json::{Value, json};
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn id() -> Value {
    json!({"type":"string","minLength":1,"maxLength":128,"pattern":"^[A-Za-z0-9_-]+$"})
}
fn sha() -> Value {
    json!({"type":"string","pattern":"^[a-f0-9]{64}$"})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","description":"Checked decimal u64; revisions must be positive"})
}
fn nullable(value: Value) -> Value {
    json!({"anyOf":[value,{"type":"null"}]})
}
fn note() -> Value {
    json!({"type":"string","maxLength":8192,"description":"Maximum 8192 UTF-8 bytes, checked at runtime"})
}
fn case() -> Value {
    object(
        json!({"package_sha256":sha(),"item_id":id(),"source_request_sha256":sha()}),
        &["package_sha256", "item_id", "source_request_sha256"],
    )
}
fn media() -> Value {
    object(
        json!({"asset_id":id(),"sha256":sha()}),
        &["asset_id", "sha256"],
    )
}
fn association() -> Value {
    object(
        json!({"association_id":id(),"case":case(),"media":media(),"revision":decimal(),"timeline_status":{"const":"unverified"},"evidence_refs":{"type":"array","maxItems":0},"note":note()}),
        &[
            "association_id",
            "case",
            "media",
            "revision",
            "timeline_status",
            "evidence_refs",
            "note",
        ],
    )
}
fn mapping_record() -> Value {
    let tick = json!({"type":"string","maxLength":20,"pattern":"^(0|-?[1-9][0-9]*)$"});
    let positive = json!({"type":"string","maxLength":39,"pattern":"^[1-9][0-9]*$"});
    let ratio = object(
        json!({"numerator":positive,"denominator":positive}),
        &["numerator", "denominator"],
    );
    object(
        json!({"case":case(),"media":media(),"evidence":media(),"mapping_revision":decimal(),"source_time_base":ratio,"origin_pts":tick,"origin_time_base":ratio,"origin_observation_sha256":sha(),"source_start_us":decimal(),"source_end_us":decimal(),"start_frame_pts":tick,"end_frame_pts":tick}),
        &[
            "case",
            "media",
            "evidence",
            "mapping_revision",
            "source_time_base",
            "origin_pts",
            "origin_time_base",
            "origin_observation_sha256",
            "source_start_us",
            "source_end_us",
            "start_frame_pts",
            "end_frame_pts",
        ],
    )
}
fn stored_association() -> Value {
    let mut verified = association();
    verified["properties"]["timeline_status"] =
        object(json!({"verified":mapping_record()}), &["verified"]);
    verified["properties"]["evidence_refs"] =
        json!({"type":"array","minItems":1,"maxItems":1,"items":media()});
    json!({"oneOf":[association(),verified]})
}
fn draft() -> Value {
    object(
        json!({"draft_id":id(),"association_id":id(),"association_revision":decimal(),"draft_revision":decimal(),"authored_by":{"const":"user"},"ranges":{"type":"array","maxItems":64,"items":object(json!({"start_us":decimal(),"end_us":decimal()}), &["start_us","end_us"])},"note":note()}),
        &[
            "draft_id",
            "association_id",
            "association_revision",
            "draft_revision",
            "authored_by",
            "ranges",
            "note",
        ],
    )
}
fn payload(command: &str) -> Value {
    match command {
        "reviews.association.inspect" => object(
            json!({"case":case(),"association_id":id(),"revision":nullable(decimal())}),
            &["case", "association_id"],
        ),
        "reviews.range_draft.read" => object(
            json!({"case":case(),"draft_id":id(),"revision":nullable(decimal())}),
            &["case", "draft_id"],
        ),
        "reviews.range_draft.save" => object(
            json!({"case":case(),"draft":draft(),"create_association":nullable(association())}),
            &["case", "draft"],
        ),
        "reviews.association.revise" => object(
            json!({"case":case(),"association_id":id(),"media":media(),"note":note(),"timeline_evidence":nullable(media())}),
            &["case", "association_id", "media", "note"],
        ),
        _ => object(
            json!({"case":case(),"association_id":id(),"media":media(),"note":note(),"timeline_evidence":{"type":"null"}}),
            &["case", "association_id", "media", "note"],
        ),
    }
}
pub fn describe(
    payload_query: &Value,
    readable: bool,
    writable: bool,
    timeline_available: bool,
) -> Result<Value, ApiError> {
    let command = payload_query["command"]
        .as_str()
        .filter(|command| is_command(command))
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Unknown review source schema command",
            )
        })?;
    if payload_query
        .as_object()
        .is_none_or(|object| object.len() != 1)
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid schema query fields",
        ));
    }
    let mut context = crate::application::api::schema::context_schema();
    if is_read(command) {
        for field in ["expected_revision", "idempotency_key", "timeout_ms"] {
            context["properties"][field] = json!({"type":"null"});
        }
    } else {
        context["properties"]["idempotency_key"] = id();
        context["required"]
            .as_array_mut()
            .unwrap()
            .push(json!("idempotency_key"));
        context["properties"]["timeout_ms"] =
            json!({"type":["integer","null"],"minimum":1,"maximum":120000,"default":30000});
        context["properties"]["expected_revision"] = match command {
            "reviews.association.create" => json!({"type":"null"}),
            "reviews.association.revise" => {
                context["required"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!("expected_revision"));
                decimal()
            }
            _ => nullable(decimal()),
        };
    }
    let supported = if is_read(command) {
        readable
    } else {
        readable && writable
    };
    // Receipt body is the exact domain transaction wire format, also returned on replay.
    let mutation = json!({"oneOf":[
        object(json!({"kind":{"const":"create_association"},"payload":association()}), &["kind","payload"]),
        object(json!({"kind":{"const":"revise_association"},"payload":association()}), &["kind","payload"]),
        object(json!({"kind":{"const":"verify_association"},"payload":object(json!({"association_id":id(),"media":media(),"evidence":media(),"note":note()}), &["association_id","media","evidence","note"])}), &["kind","payload"]),
        object(json!({"kind":{"const":"save_draft"},"payload":object(json!({"draft":draft(),"create_association":nullable(association())}), &["draft","create_association"])}), &["kind","payload"])
    ]});
    let mutation_request = object(
        json!({"case":case(),"idempotency_key":id(),"expected_revision":nullable(decimal()),"mutation":mutation}),
        &["case", "idempotency_key", "expected_revision", "mutation"],
    );
    let receipt = object(
        json!({"envelope_sha256":sha(),"body":object(json!({"schema_version":{"const":"1"},"request":mutation_request,"association":stored_association(),"draft":nullable(draft())}), &["schema_version","request","association","draft"])}),
        &["envelope_sha256", "body"],
    );
    let response = if is_read(command) {
        let stored = if command == "reviews.association.inspect" {
            object(
                json!({"association":stored_association()}),
                &["association"],
            )
        } else {
            object(
                json!({"draft":draft(),"association":stored_association()}),
                &["draft", "association"],
            )
        };
        object(
            json!({"stored":stored,"media_check":{"const":"not_checked"},"playback_ready":{"const":false},"automatic_application_enabled":{"const":false}}),
            &[
                "stored",
                "media_check",
                "playback_ready",
                "automatic_application_enabled",
            ],
        )
    } else {
        object(
            json!({"receipt":receipt,"playback_ready":{"const":false},"automatic_application_enabled":{"const":false}}),
            &["receipt", "playback_ready", "automatic_application_enabled"],
        )
    };
    Ok(
        json!({"schema_version":"1","command":command,"status":if supported {"supported"}else{"unsupported"},
        "request":object(json!({"context":context,"command":{"const":command},"payload":payload(command)}), &["context","command","payload"]),
        "response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":nullable(response),"error":{"type":["object","null"]}}), &["api_version","request_id","ok","data","error"]),
        "error_codes":ErrorCode::ALL,"timeline_verification_available":readable&&writable&&timeline_available,"strong_cancel_available":false,
        "publication":"single immutable envelope; query same key after uncertain delivery; no automatic retry"}),
    )
}
pub fn capabilities(
    value: &mut Value,
    readable: bool,
    writable: bool,
    limits: Option<super::StoreLimits>,
    timeline_available: bool,
) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if let Some(name) = command["command"]
                .as_str()
                .filter(|name| COMMANDS.contains(name))
            {
                command["status"] = json!(if readable && (is_read(name) || writable) {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["review_source"] = json!({"read_available":readable,"write_available":readable&&writable,"timeline_verification_available":readable&&writable&&timeline_available,"strong_cancel_available":false,"playback_ready":false,"max_ranges":64,"max_note_utf8_bytes":8192,"max_timeout_ms":120000});
    value["review_source"]["storage_limits"] = json!(limits);
    value["review_source"]["committed_key_expiry"] = json!("none");
    value["review_source"]["capacity_behavior"] =
        json!("reject_new_publication_preserve_committed_replay");
}
