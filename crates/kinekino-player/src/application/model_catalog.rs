//! Shared read-only catalog port for GUI and CLI. Declarations do not enable execution.
#[path = "model_catalog/ports.rs"]
mod ports;
use crate::{
    application::api::{ApiError, ErrorCode, Request},
    domain::model_catalog::*,
};
pub use ports::ModelCatalogPort;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
mod verification;
pub use verification::{
    ModelFileCheck, ModelFileCheckState, ModelVerification, VerifyModel, verify_model,
};
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalFileState {
    Missing,
    PresentUnverified,
    SizeMismatch,
}
#[derive(Serialize)]
pub struct ModelFileObservation {
    pub path: String,
    pub state: LocalFileState,
    pub observed_bytes: Option<crate::domain::api_values::DecimalU64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct List {
    task: Option<ModelTask>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inspect {
    model_id: String,
    revision: String,
}
pub fn is_command(command: &str) -> bool {
    matches!(command, "models.list" | "models.inspect" | "models.verify")
}
pub fn execute(port: Option<&dyn ModelCatalogPort>, request: &Request) -> Result<Value, ApiError> {
    execute_controlled(port, request, None)
}
pub fn execute_controlled(
    port: Option<&dyn ModelCatalogPort>,
    request: &Request,
    control: Option<&dyn crate::application::review_source::MutationControl>,
) -> Result<Value, ApiError> {
    let port = port
        .ok_or_else(|| ApiError::new(ErrorCode::Unsupported, "Model catalog is not configured"))?;
    if request.context.expected_revision.is_some()
        || request.context.idempotency_key.is_some()
        || request.context.timeout_ms.is_some() && request.command != "models.verify"
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Read-only catalog does not accept mutation/timeout context",
        ));
    }
    if request.command == "models.verify" {
        return verification::execute(port, request, control);
    }
    let (catalog, digest) = port.snapshot()?;
    catalog.validate().map_err(invalid)?;
    if !crate::domain::identifiers::valid_sha256(&digest) {
        return Err(invalid("Invalid catalog digest"));
    }
    let data = match request.command.as_str() {
        "models.list" => {
            let input: List = serde_json::from_value(request.payload.clone())
                .map_err(|_| invalid("Expected optional task filter"))?;
            let summaries:Vec<_>=catalog.models.iter().filter(|m|input.task.as_ref().is_none_or(|task|task==&m.task)).map(|m|json!({"model_id":m.model_id,"task":m.task,"repo_id":m.repo_id,"revision":m.revision,"format":m.format,"quantization":m.quantization,"license":m.license,"runtime":m.runtime,"min_runtime_version":m.min_runtime_version,"file_count":m.files.len(),"local_files":"not_checked"})).collect();
            json!({"models":summaries})
        }
        "models.inspect" => {
            let input: Inspect = serde_json::from_value(request.payload.clone())
                .map_err(|_| invalid("Exact model_id and immutable revision required"))?;
            if !crate::domain::identifiers::valid_id(&input.model_id)
                || !fixed_revision(&input.revision)
            {
                return Err(invalid("Exact model identity required"));
            }
            let model = catalog
                .models
                .iter()
                .find(|m| m.model_id == input.model_id && m.revision == input.revision)
                .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Model revision not declared"))?;
            json!({"manifest":model,"local_files":port.inspect_files(model)?,"integrity":"not_verified","declared_measurements":"not_verified"})
        }
        _ => return Err(invalid("Unknown model catalog command")),
    };
    Ok(
        json!({"catalog_sha256":digest,"data":data,"runtime_admitted":false,"quality_approved":false,"license_permission":"not_verified"}),
    )
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
pub fn capabilities(value: &mut Value, available: bool, verification: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for entry in commands {
            if entry["command"].as_str().is_some_and(is_command) {
                entry["status"] = json!(if available
                    && (entry["command"] != "models.verify" || verification)
                {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["model_catalog"] = json!({"available":available,"read_only":true,"manifest_is_runtime_admission":false,"integrity_verification":verification,"installation":false,"removal":false,"in_process_sdk_execution_cooperative_cancel":verification,"wire_cancel_ack":false,"hard_preemption":false});
}
pub fn schema(payload: &Value, available: bool, verification: bool) -> Result<Value, ApiError> {
    let command = payload["command"]
        .as_str()
        .filter(|s| is_command(s))
        .ok_or_else(|| invalid("Unknown model catalog schema"))?;
    if payload.as_object().is_none_or(|p| p.len() != 1) {
        return Err(invalid("Unexpected schema fields"));
    }
    let task = json!({"enum":[null,"asr","text","ocr","subject","depth","speaker_segmentation","speaker_embedding","vad","alignment"]});
    let input = if command == "models.verify" {
        verification::input_schema()
    } else if command == "models.list" {
        json!({"type":"object","additionalProperties":false,"properties":{"task":task}})
    } else {
        json!({"type":"object","additionalProperties":false,"required":["model_id","revision"],"properties":{"model_id":{"type":"string","minLength":1,"maxLength":80},"revision":{"type":"string","pattern":"^[a-f0-9]{40}$"}}})
    };
    let mut context = crate::application::api::schema::context_schema();
    for field in ["expected_revision", "idempotency_key", "timeout_ms"] {
        context["properties"][field] = json!({"type":"null"});
    }
    if command == "models.verify" {
        context["properties"]["timeout_ms"] =
            json!({"type":["integer","null"],"minimum":1,"maximum":120000,"default":30000});
    }
    let request = json!({"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","additionalProperties":false,"required":["context","command","payload"],"properties":{"context":context,"command":{"const":command},"payload":input}});
    let data = json!({"type":"object","additionalProperties":false,"required":["catalog_sha256","data","runtime_admitted","quality_approved","license_permission"],"properties":{"catalog_sha256":{"type":"string","pattern":"^[a-f0-9]{64}$"},"data":{"type":"object"},"runtime_admitted":{"const":false},"quality_approved":{"const":false},"license_permission":{"const":"not_verified"}}});
    let mut example = Request::new(command);
    example.payload = if command == "models.verify" {
        json!({"model_id":"model","revision":"a".repeat(40),"catalog_sha256":"b".repeat(64),"maximum_bytes":"1073741824"})
    } else if command == "models.list" {
        json!({})
    } else {
        json!({"model_id":"model","revision":"a".repeat(40)})
    };
    Ok(
        json!({"schema_version":"1","command":command,"status":if available && (command!="models.verify" || verification){"supported"}else{"unsupported"},"request":request,"response":{"type":"object","required":["api_version","request_id","ok","data","error"],"properties":{"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[data,{"type":"null"}]},"error":{"type":["object","null"]}}},"error_codes":ErrorCode::ALL,"example":example,"limitations":"Verify observes retained local file hashes at one invocation. Inspect only observes sizes. No runtime admission, installation, license permission or quality approval."}),
    )
}
