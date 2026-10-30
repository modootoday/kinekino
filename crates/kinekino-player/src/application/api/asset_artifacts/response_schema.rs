use crate::application::api::asset_projects::schema::{frame_input, transform};
use serde_json::{Value, json};
fn object(fields: Value) -> Value {
    let required: Vec<_> = fields.as_object().unwrap().keys().cloned().collect();
    json!({"type":"object","additionalProperties":false,"required":required,"properties":fields})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^(0|[1-9][0-9]*)$","description":"u64 decimal string; cross-field constraints checked by application"})
}
fn id() -> Value {
    json!({"type":"string","pattern":"^[A-Za-z0-9_.-]{1,80}$"})
}
fn sha() -> Value {
    json!({"type":"string","pattern":"^[a-f0-9]{64}$"})
}
fn array(item: Value, max: usize) -> Value {
    json!({"type":"array","maxItems":max,"items":item})
}
fn file() -> Value {
    object(
        json!({"name":{"type":"string","maxLength":80},"sha256":sha(),"bytes":decimal(),"media_type":{"enum":["application/json","application/octet-stream","image/png"]}}),
    )
}
fn unit() -> Value {
    object(json!({"frame_position":decimal(),"frame_id":id(),"layer_id":id()}))
}
fn ratio() -> Value {
    object(
        json!({"numerator":{"type":"string","pattern":"^(0|-?[1-9][0-9]*)$"},"denominator":decimal()}),
    )
}
fn integer() -> Value {
    json!({"type":"integer","minimum":0,"maximum":u32::MAX})
}
fn geometry() -> Value {
    let crop = object(json!({"x":integer(),"y":integer(),"width":integer(),"height":integer()}));
    let uv = object(
        json!({"left":integer(),"top":integer(),"right":integer(),"bottom":integer(),"denominator_x":integer(),"denominator_y":integer()}),
    );
    object(
        json!({"source_width":integer(),"source_height":integer(),"source_bounds":crop,"output_width":integer(),"output_height":integer(),"source_uv":uv,"pivot_x":ratio(),"pivot_y":ratio()}),
    )
}
fn request() -> Value {
    let mut fields = frame_input()["properties"].clone();
    fields["schema_version"] = json!({"const":"asset-frame-build/1"});
    fields["layer"] = json!({"enum":["foreground","background"]});
    fields["transform"] = transform();
    object(fields)
}
fn plan() -> Value {
    // Reuse the existing declared position schema, including exact video ticks/clock.
    let position = crate::application::api::asset_projects::schema::position();
    let frame = object(
        json!({"position":decimal(),"frame_id":id(),"source":position,"missing_reason":{"type":["string","null"],"maxLength":2048}}),
    );
    object(
        json!({"schema_version":{"const":"asset-bundle-plan/2"},"project":object(json!({"document_id":id(),"publication_revision":decimal(),"publication_sha256":sha()})),"order_basis":{"enum":["single_frame","declared_frame_order","source_video_clock"]},"missing_frame_policy":{"enum":["reject","preserve_gap"]},"frames":array(frame,4096),"units":array(unit(),4096)}),
    )
}
fn manifest() -> Value {
    let layer = object(
        json!({"unit":unit(),"request":request(),"geometry":geometry(),"raw_alpha_mode":{"enum":["straight","premultiplied"]},"png_alpha_mode":{"const":"straight"},"png_conversion":{"enum":["none","unassociate_srgb8_with_quantization"]},"color":{"const":"srgb"},"unreviewed_mask_warning":{"type":"boolean"},"rgba8":file(),"rgba_png":file(),"mask_png":file()}),
    );
    object(
        json!({"schema_version":{"const":"asset-bundle/1"},"plan":plan(),"project_file":file(),"layers":array(layer,4096),"state":{"const":"review_artifact"},"quality_approved":{"const":false},"colorimetry_verified":{"const":false}}),
    )
}
pub(super) fn response() -> Value {
    let execution = object(
        json!({"job_id":id(),"attempt":decimal(),"input_sha256":sha(),"source_blob_sha256":sha(),"submission_request_sha256":sha()}),
    );
    let publication = object(
        json!({"schema_version":{"const":"asset-artifact/1"},"artifact_id":id(),"execution":execution,"bundle_manifest":file(),"files":array(file(),4096*3+2),"review_artifact_only":{"const":true},"quality_approved":{"const":false}}),
    );
    let data = object(
        json!({"publication":publication,"publication_sha256":sha(),"publication_bytes":decimal(),"manifest":manifest()}),
    );
    object(
        json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[data,{"type":"null"}]},"error":{"type":["object","null"]}}),
    )
}
