//! Snapshot wire projection; decoding and publication remain behind the application port.
use super::{ApiError, ErrorCode, Request};
use crate::application::snapshot_export::{ExportSnapshot, SnapshotExportPort};
use serde_json::{Value, json};
pub(super) const COMMAND: &str = "production.export_snapshot";
pub(super) type SnapshotPort = dyn SnapshotExportPort + Send + Sync;

pub(super) fn execute(request: &Request, port: Option<&SnapshotPort>) -> Result<Value, ApiError> {
    if request.context.expected_revision.is_some()
        || request.context.idempotency_key.is_some()
        || request.context.timeout_ms.is_some()
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Snapshot export does not support revision, idempotency or timeout options",
        ));
    }
    let input: ExportSnapshot = serde_json::from_value(request.payload.clone()).map_err(|_| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Expected fixed SHA256, absolute PNG paths and an explicit integer pixel crop",
        )
    })?;
    let port = port.ok_or_else(|| {
        ApiError::new(
            ErrorCode::Unsupported,
            "No snapshot export adapter configured",
        )
    })?;
    serde_json::to_value(crate::application::snapshot_export::export(port, &input)?)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid snapshot export receipt"))
}
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn sha() -> Value {
    json!({"type":"string","pattern":"^[a-f0-9]{64}$"})
}
fn crop_schema() -> Value {
    let coordinate = json!({"type":"integer","minimum":0,"maximum":u32::MAX});
    let dimension = json!({"type":"integer","minimum":1,"maximum":u32::MAX});
    object(
        json!({"x":coordinate,"y":coordinate,"width":dimension,"height":dimension}),
        &["x", "y", "width", "height"],
    )
}
pub(super) fn schema(request: &Request, available: bool) -> Result<Value, ApiError> {
    if request
        .payload
        .as_object()
        .is_none_or(|payload| payload.len() != 1)
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Snapshot schema accepts command only",
        ));
    }
    let mut context = super::schema::context_schema();
    for field in ["expected_revision", "idempotency_key", "timeout_ms"] {
        context["properties"][field] =
            json!({"type":"null","description":"Unavailable for synchronous snapshot export"});
    }
    let path = json!({"type":"string","minLength":1,"description":"Absolute local .png path; runtime path checks apply"});
    let payload = object(
        json!({"source_path":path,"expected_source_sha256":sha(),"output_path":path,"crop":crop_schema()}),
        &[
            "source_path",
            "expected_source_sha256",
            "output_path",
            "crop",
        ],
    );
    let response_data = object(
        json!({"output_path":path,"source_sha256":sha(),"output_sha256":sha(),"source_width":{"type":"integer","minimum":1,"maximum":u32::MAX},"source_height":{"type":"integer","minimum":1,"maximum":u32::MAX},"crop":crop_schema(),"output_bytes":{"type":"integer","minimum":1,"maximum":crate::domain::frame_capture::MAX_FRAME_PNG_BYTES},"metadata_preserved":{"const":false}}),
        &[
            "output_path",
            "source_sha256",
            "output_sha256",
            "source_width",
            "source_height",
            "crop",
            "output_bytes",
            "metadata_preserved",
        ],
    );
    Ok(json!({
        "schema_version":"1","command":COMMAND,"status":if available{"supported"}else{"unsupported"},
        "request":object(json!({"context":context,"command":{"const":COMMAND},"payload":payload}), &["context","command","payload"]),
        "response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[response_data,{"type":"null"}]},"error":{"type":["object","null"]}}), &["api_version","request_id","ok","data","error"]),
        "error_codes":ErrorCode::ALL,"source_pts_available":false,"automatic_retry":false,
        "example":{"context":super::RequestContext::default(),"command":COMMAND,"payload":{"source_path":"D:/frames/source.png","expected_source_sha256":"a".repeat(64),"output_path":"D:/exports/crop.png","crop":{"x":0,"y":0,"width":1,"height":1}}}
    }))
}
pub(super) fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"] == COMMAND {
                command["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["snapshot_export"] = json!({"available":available,"formats":["png"],"animated_png":false,"coordinate_space":"source_pixels_top_left_exclusive_edges","source_sha256_required":true,"max_input_output_bytes":crate::domain::frame_capture::MAX_FRAME_PNG_BYTES,"max_pixels":crate::domain::frame_capture::MAX_FRAME_PIXELS,"max_decoded_bytes":crate::domain::frame_capture::MAX_FRAME_PIXELS*4,"overwrite":false,"metadata_preserved":false,"color_managed_visual_equivalence":false,"source_pts_available":false,"ffmpeg_required":false,"automatic_retry":false,"timeout_supported":false});
}
