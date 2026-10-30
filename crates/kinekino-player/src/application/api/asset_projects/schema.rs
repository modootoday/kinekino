use super::*;
use serde_json::json;
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","description":"u64 decimal string; positive revisions validated by domain"})
}
fn id() -> Value {
    json!({"type":"string","minLength":1,"maxLength":80,"pattern":"^[A-Za-z0-9_.-]+$"})
}
fn sha() -> Value {
    json!({"type":"string","pattern":"^[a-f0-9]{64}$"})
}
fn nullable(value: Value) -> Value {
    json!({"anyOf":[value,{"type":"null"}]})
}
fn integer(min: u64, max: u64) -> Value {
    json!({"type":"integer","minimum":min,"maximum":max})
}
fn pin() -> Value {
    object(
        json!({"artifact_id":id(),"revision":decimal(),"sha256":sha()}),
        &["artifact_id", "revision", "sha256"],
    )
}
fn video() -> Value {
    let pts = json!({"type":"string","pattern":"^(0|-?[1-9][0-9]*)$","description":"signed i64 integer ticks, validated without f64"});
    let clock = object(
        json!({"origin_pts":pts,"time_base_numerator":integer(1,u32::MAX.into()),"time_base_denominator":integer(1,u32::MAX.into()),"observation_sha256":sha()}),
        &[
            "origin_pts",
            "time_base_numerator",
            "time_base_denominator",
            "observation_sha256",
        ],
    );
    object(
        json!({"source_sha256":sha(),"stream_index":integer(0,u32::MAX.into()),"pts":pts,"clock":clock,"rotation_degrees":{"enum":[0,90,180,270]}}),
        &[
            "source_sha256",
            "stream_index",
            "pts",
            "clock",
            "rotation_degrees",
        ],
    )
}
pub(in crate::application::api) fn image() -> Value {
    let mut observed = video();
    observed["properties"]["kind"] = json!({"const":"observed_video"});
    observed["required"]
        .as_array_mut()
        .unwrap()
        .push(json!("kind"));
    let origin =
        json!({"oneOf":[object(json!({"kind":{"const":"static_png"}}), &["kind"]), observed]});
    object(
        json!({"blob":pin(),"width":integer(1,u32::MAX.into()),"height":integer(1,u32::MAX.into()),"origin":origin,"declared_color":{"const":"srgb"}}),
        &["blob", "width", "height", "origin", "declared_color"],
    )
}
pub(in crate::application::api) fn frame_input() -> Value {
    let mask = object(
        json!({"blob":pin(),"source":image(),"encoding":{"enum":["gray8_coverage","rgba8_alpha"]},"origin":{"const":"imported_unverified"}}),
        &["blob", "source", "encoding", "origin"],
    );
    let review = json!({"oneOf":[object(json!({"kind":{"const":"reference"},"record":pin()}), &["kind","record"]),object(json!({"kind":{"const":"unreviewed_acknowledged"},"mask":pin()}), &["kind","mask"])]});
    object(
        json!({"image":image(),"mask":mask,"review":review}),
        &["image", "mask", "review"],
    )
}
pub(in crate::application::api) fn transform() -> Value {
    object(
        json!({"trim":{"type":"boolean"},"mask_threshold":integer(0,255),"feather_radius":integer(0,64),"scale":object(json!({"numerator":integer(1,16),"denominator":integer(1,16)}), &["numerator","denominator"]),"pivot":object(json!({"x":integer(0,1_000_000),"y":integer(0,1_000_000)}), &["x","y"]),"alpha_mode":{"enum":["straight","premultiplied"]},"color":{"const":"srgb"}}),
        &[
            "trim",
            "mask_threshold",
            "feather_radius",
            "scale",
            "pivot",
            "alpha_mode",
            "color",
        ],
    )
}
pub(in crate::application::api) fn position() -> Value {
    json!({"oneOf":[object(json!({"kind":{"const":"static_png"}}), &["kind"]),object(json!({"kind":{"const":"video"},"expected":video()}), &["kind","expected"])]})
}
fn document() -> Value {
    let position = position();
    let slot = object(
        json!({"position":position,"selected_image":nullable(image())}),
        &["position"],
    );
    let frame = json!({"oneOf":[object(json!({"state":{"const":"available"},"frame_id":id(),"input":frame_input()}), &["state","frame_id","input"]),object(json!({"state":{"const":"missing"},"frame_id":id(),"reason":{"type":"string","minLength":1,"maxLength":2048}}), &["state","frame_id","reason"])]});
    let layer = object(
        json!({"layer_id":id(),"kind":{"enum":["foreground","background"]},"visible":{"type":"boolean"},"locked":{"type":"boolean"},"transform":transform()}),
        &["layer_id", "kind", "visible", "locked", "transform"],
    );
    object(
        json!({"schema_version":{"const":"asset-project/1"},"document_id":id(),"revision":decimal(),"sources":{"type":"object","propertyNames":id(),"maxProperties":4096,"additionalProperties":slot},"order_basis":{"enum":["single_frame","declared_frame_order","source_video_clock"]},"frames":{"type":"array","maxItems":4096,"items":frame},"layers":{"type":"array","minItems":1,"maxItems":64,"items":layer}}),
        &[
            "schema_version",
            "document_id",
            "revision",
            "sources",
            "order_basis",
            "frames",
            "layers",
        ],
    )
}
pub(in crate::application::api) fn response() -> Value {
    let publication = object(
        json!({"schema_version":{"const":"asset-project-publication/1"},"publication_revision":decimal(),"parent_publication_sha256":nullable(sha()),"document":document()}),
        &[
            "schema_version",
            "publication_revision",
            "parent_publication_sha256",
            "document",
        ],
    );
    object(
        json!({"publication_sha256":sha(),"latest_publication_revision":decimal(),"pending_publication_count":decimal(),"history_bytes":decimal(),"publication":publication}),
        &[
            "publication_sha256",
            "latest_publication_revision",
            "pending_publication_count",
            "history_bytes",
            "publication",
        ],
    )
}
fn constraints(available: bool) -> Value {
    json!({"backend_available":available,"record_schema_version":"asset-project-publication/1","document_max_bytes":crate::domain::asset_generation::MAX_ASSET_DOCUMENT_BYTES,"immutable_publications":true,"publication_revision_distinct_from_edit_revision":true,"source_slots_retained":true,"full_history_validated":true,"idempotency_supported":false,"timeout_supported":false,"in_flight_cancel":false,"quality_approved":false,"source_admitted":false,"review_admitted":false,"build_assets_supported":false,"cross_field_bounds_validated_by_domain":true})
}
pub(in crate::application::api) fn describe(
    payload: &Value,
    available: bool,
) -> Result<Value, ApiError> {
    let command = payload["command"]
        .as_str()
        .filter(|command| super::is_command(command))
        .ok_or_else(|| invalid("Unknown asset project schema command"))?;
    if payload.as_object().is_none_or(|object| object.len() != 1) {
        return Err(invalid("Schema accepts only command"));
    }
    let input = if command == "asset_projects.inspect" {
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
    if command == "asset_projects.save_revision" {
        context["properties"]["expected_revision"] = decimal();
        context["required"]
            .as_array_mut()
            .unwrap()
            .push(json!("expected_revision"));
    } else {
        context["properties"]["expected_revision"] = json!({"type":"null"});
    }
    Ok(
        json!({"schema_version":"1","command":command,"status":if available {"supported"} else {"unsupported"},"request":object(json!({"context":context,"command":{"const":command},"payload":input}), &["context","command","payload"]),"response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":nullable(response()),"error":{"type":["object","null"]}}), &["api_version","request_id","ok","data","error"]),"error_codes":ErrorCode::ALL,"constraints":constraints(available)}),
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
    value["asset_projects"] = constraints(available);
}
