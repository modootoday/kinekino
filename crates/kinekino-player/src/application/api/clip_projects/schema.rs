use super::*;
use serde_json::json;
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","description":"u64 decimal string; bounds validated by domain"})
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
fn positive() -> Value {
    json!({"type":"integer","minimum":1,"maximum":4294967295u64})
}
fn pin() -> Value {
    object(
        json!({"artifact_id":id(),"revision":decimal(),"sha256":sha()}),
        &["artifact_id", "revision", "sha256"],
    )
}
fn source_bound() -> Value {
    object(
        json!({"artifact":pin(),"source_sha256":sha()}),
        &["artifact", "source_sha256"],
    )
}
pub(in crate::application::api) fn source_schema() -> Value {
    let clock = object(
        json!({"origin_pts":{"type":"string","pattern":"^(0|-?[1-9][0-9]*)$"},"time_base_numerator":positive(),"time_base_denominator":positive(),"observation_sha256":sha()}),
        &[
            "origin_pts",
            "time_base_numerator",
            "time_base_denominator",
            "observation_sha256",
        ],
    );
    let track = object(
        json!({"stream_index":{"type":"integer","minimum":0,"maximum":4294967295u64},"kind":{"enum":["video","audio","subtitle"]},"codec_id":id()}),
        &["stream_index", "kind", "codec_id"],
    );
    object(
        json!({"media_id":id(),"sha256":sha(),"duration_us":decimal(),"width":positive(),"height":positive(),"clock":clock,"tracks":{"type":"array","minItems":1,"maxItems":64,"items":track}}),
        &[
            "media_id",
            "sha256",
            "duration_us",
            "width",
            "height",
            "clock",
            "tracks",
        ],
    )
}
fn document() -> Value {
    let source = source_schema();
    let crop_value = json!({"type":"integer","minimum":0,"maximum":1_000_000});
    let crop = object(
        json!({"left":crop_value,"top":crop_value,"width":crop_value,"height":crop_value}),
        &["left", "top", "width", "height"],
    );
    let crop_key = object(
        json!({"source_us":decimal(),"crop":crop,"fit":{"enum":["fit","fill"]}}),
        &["source_us", "crop", "fit"],
    );
    let audio = json!({"oneOf":[object(json!({"kind":{"const":"disabled"}}), &["kind"]),object(json!({"kind":{"const":"selected"},"stream_index":{"type":"integer","minimum":0,"maximum":4294967295u64},"volume_percent":{"type":"integer","minimum":0,"maximum":100},"muted":{"type":"boolean"}}), &["kind","stream_index","volume_percent","muted"])]});
    let subtitles = object(
        json!({"document":source_bound(),"cue_ids":{"type":"array","minItems":1,"maxItems":4096,"uniqueItems":true,"items":id()},"style":pin()}),
        &["document", "cue_ids", "style"],
    );
    let segment = object(
        json!({"segment_id":id(),"media_id":id(),"range":object(json!({"start_us":decimal(),"end_us":decimal()}), &["start_us","end_us"]),"video_stream_index":{"type":"integer","minimum":0,"maximum":4294967295u64},"crop_keys":{"type":"array","minItems":1,"maxItems":128,"items":crop_key},"audio":audio,"subtitles":nullable(subtitles),"subject_ref":nullable(source_bound())}),
        &[
            "segment_id",
            "media_id",
            "range",
            "video_stream_index",
            "crop_keys",
            "audio",
        ],
    );
    let preset = object(
        json!({"catalog":pin(),"width":positive(),"height":positive(),"frame_rate":object(json!({"numerator":positive(),"denominator":positive()}), &["numerator","denominator"]),"container":{"enum":["mp4","webm"]},"video_codec":{"enum":["h264","vp9","av1"]},"audio_codec":nullable(json!({"enum":["aac","opus"]})),"color":{"const":"sdr_bt709"}}),
        &[
            "catalog",
            "width",
            "height",
            "frame_rate",
            "container",
            "video_codec",
            "color",
        ],
    );
    object(
        json!({"schema_version":{"const":"1"},"document_id":id(),"revision":decimal(),"sources":{"type":"array","minItems":1,"maxItems":128,"items":source},"segments":{"type":"array","minItems":1,"maxItems":128,"items":segment},"preset":preset,"cut_mode":{"enum":["exact_reencode","fast_keyframe"]}}),
        &[
            "schema_version",
            "document_id",
            "revision",
            "sources",
            "segments",
            "preset",
            "cut_mode",
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
fn constraints(available: bool) -> Value {
    json!({"backend_available":available,"record_schema_version":"1","document_max_bytes":crate::domain::clip_document::MAX_DOCUMENT_BYTES,"immutable_publications":true,"publication_revision_distinct_from_edit_revision":true,"source_snapshot_immutable":true,"full_history_validated":true,"idempotency_supported":false,"timeout_supported":false,"in_flight_cancel":false,"quality_approved":false,"media_provenance_verified":false,"render_short_supported":false,"cross_field_bounds_validated_by_domain":true})
}
pub(in crate::application::api) fn describe(
    payload: &Value,
    available: bool,
) -> Result<Value, ApiError> {
    let command = payload["command"]
        .as_str()
        .filter(|command| super::is_command(command))
        .ok_or_else(|| invalid("Unknown clip project schema command"))?;
    if payload.as_object().is_none_or(|object| object.len() != 1) {
        return Err(invalid("Schema accepts only command"));
    }
    let input = if command == "clip_projects.inspect" {
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
    if command == "clip_projects.save_revision" {
        context["properties"]["expected_revision"] = decimal();
        context["required"]
            .as_array_mut()
            .expect("context required")
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
    value["clip_projects"] = constraints(available);
}
