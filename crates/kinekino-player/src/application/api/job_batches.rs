use super::{ApiError, ErrorCode, Request};
use crate::application::{
    job_batches::{BatchAction, service::*},
    jobs::JobsCapability,
};
use serde_json::{Value, json};
pub(super) const COMMANDS: [&str; 5] = [
    "job_batches.create",
    "job_batches.inspect",
    "job_batches.submit",
    "job_batches.refresh",
    "job_batches.retry_failed",
];
pub(super) fn is_command(command: &str) -> bool {
    COMMANDS.contains(&command)
}
pub(super) fn execute(request: &Request, port: Option<&dyn BatchApi>) -> Result<Value, ApiError> {
    let port = port.ok_or_else(|| {
        ApiError::new(
            ErrorCode::Unsupported,
            "Batch backend requires an explicit same-root Jobs host",
        )
    })?;
    let invalid = || {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Batch command context is invalid",
        )
    };
    if request.context.timeout_ms.is_some() || request.context.idempotency_key.is_some() {
        return Err(invalid());
    }
    let expected = request
        .context
        .expected_revision
        .as_deref()
        .map(|value| {
            value
                .parse::<u64>()
                .ok()
                .filter(|n| *n > 0)
                .ok_or_else(invalid)
        })
        .transpose()?;
    if request.command == "job_batches.create" && expected.is_some() {
        return Err(invalid());
    }
    if !matches!(
        request.command.as_str(),
        "job_batches.create" | "job_batches.inspect"
    ) && expected.is_none()
    {
        return Err(invalid());
    }
    fn input<T: serde::de::DeserializeOwned>(value: &Value) -> Result<T, ApiError> {
        serde_json::from_value(value.clone())
            .map_err(|error| ApiError::new(ErrorCode::InvalidArgument, error.to_string()))
    }
    let result = match request.command.as_str() {
        "job_batches.create" => serde_json::to_value(port.create(input(&request.payload)?)?),
        "job_batches.inspect" => {
            serde_json::to_value(port.inspect(input(&request.payload)?, expected)?)
        }
        command => {
            let action = match command {
                "job_batches.submit" => BatchAction::Submit,
                "job_batches.refresh" => BatchAction::Refresh,
                "job_batches.retry_failed" => BatchAction::RetryFailed,
                _ => return Err(invalid()),
            };
            serde_json::to_value(port.execute(
                input(&request.payload)?,
                expected.ok_or_else(invalid)?,
                action,
            )?)
        }
    };
    result.map_err(|_| ApiError::new(ErrorCode::Internal, "Batch response serialization failed"))
}
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
pub(super) fn describe(
    input: &Value,
    available: bool,
    jobs: JobsCapability,
) -> Result<Value, ApiError> {
    let command = input["command"]
        .as_str()
        .filter(|name| is_command(name))
        .filter(|_| input.as_object().is_some_and(|fields| fields.len() == 1))
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "schema requires one batch command",
            )
        })?;
    let identity = json!({"type":"string","minLength":1,"maxLength":80,"pattern":"^[A-Za-z0-9_.-]+$","not":{"enum":[".",".."]}});
    let payload = match command {
        "job_batches.create" => object(
            json!({"batch_id":identity,"items":{"type":"array","minItems":1,"maxItems":128,"items":object(json!({"item_id":identity,"request":super::job_schema::payload("jobs.submit", &jobs)}), &["item_id","request"])}}),
            &["batch_id", "items"],
        ),
        "job_batches.inspect" => object(
            json!({"batch_id":identity,"offset":{"type":"integer","minimum":0,"default":0},"limit":{"type":"integer","minimum":1,"maximum":8,"default":8}}),
            &["batch_id"],
        ),
        _ => object(json!({"batch_id":identity}), &["batch_id"]),
    };
    let mut context = super::schema::context_schema();
    context["properties"]["idempotency_key"] = json!({"type":"null"});
    context["properties"]["timeout_ms"] = json!({"type":"null"});
    if command == "job_batches.create" {
        context["properties"]["expected_revision"] = json!({"type":"null"})
    }
    if !matches!(command, "job_batches.create" | "job_batches.inspect") {
        context["properties"]["expected_revision"] = json!({"type":"string","pattern":"^[0-9]+$","description":"Positive decimal u64, range checked"});
        context["required"] = json!(["request_id", "api_version", "expected_revision"]);
    }
    let receipt = object(
        json!({"schema_version":{"const":"1"},"batch_id":identity,"revision":{"type":"string","pattern":"^[1-9][0-9]*$"},"total":{"type":"integer","minimum":1,"maximum":128},"states":{"type":"object","additionalProperties":{"type":"integer","minimum":0}}}),
        &["schema_version", "batch_id", "revision", "total", "states"],
    );
    let data = if command == "job_batches.inspect" {
        object(
            json!({"receipt":receipt,"entries":{"type":"array","maxItems":8,"items":entry_schema(&identity)},"next_offset":{"type":["integer","null"],"minimum":0}}),
            &["receipt", "entries", "next_offset"],
        )
    } else {
        receipt
    };
    Ok(
        json!({"schema_version":"1","command":command,"status":if available {"supported"} else {"unsupported"},"request":object(json!({"context":context,"command":{"const":command},"payload":payload}), &["context","command","payload"]),"response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[data,{"type":"null"}]},"error":{"anyOf":[error_schema(),{"type":"null"}]}}), &["api_version","request_id","ok","data","error"]),"error_codes":ErrorCode::ALL,"limits":{"capture_input_bytes":crate::application::job_batches::MAX_BATCH_INPUT_BYTES,"page_bytes":MAX_PAGE_BYTES,"automatic_execution_on_open":false,"create_duplicate":"revision_conflict; inspect original batch_id","mutations_require_expected_revision":true,"capture_operations_require_explicit_adapter_opt_in":true}}),
    )
}
fn error_schema() -> Value {
    object(
        json!({"code":{"enum":ErrorCode::ALL},"message":{"type":"string"},"retryable":{"type":"boolean"},"details":{},"remediation":{"type":"string"},"request_id":{"type":"string"}}),
        &[
            "code",
            "message",
            "retryable",
            "details",
            "remediation",
            "request_id",
        ],
    )
}
fn entry_schema(identity: &Value) -> Value {
    let captured_request = object(
        json!({"operation":{"type":"string"},"parameters":{"type":"object"},"expected_source_sha256":{"type":"string","pattern":"^[a-f0-9]{64}$"},"execution_timeout_ms":{"type":"string","pattern":"^[0-9]+$"}}),
        &[
            "operation",
            "parameters",
            "expected_source_sha256",
            "execution_timeout_ms",
        ],
    );
    let nullable_error = json!({"anyOf":[error_schema(),{"type":"null"}]});
    let progress = json!({"oneOf":[
        object(json!({"state":{"const":"waiting"}}), &["state"]),
        object(json!({"state":{"const":"submit_pending"}}), &["state"]),
        object(json!({"state":{"const":"submit_error"},"error":error_schema()}), &["state","error"]),
        object(json!({"state":{"const":"observed"},"job":super::job_schema::record(),"last_error":nullable_error}), &["state","job","last_error"]),
        object(json!({"state":{"const":"retry_pending"},"before":super::job_schema::record(),"last_error":nullable_error}), &["state","before","last_error"])
    ]});
    object(
        json!({"input":object(json!({"item_id":identity,"request":captured_request}), &["item_id","request"]),"progress":progress}),
        &["input", "progress"],
    )
}
pub(super) fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands
            .iter_mut()
            .filter(|entry| entry["command"].as_str().is_some_and(is_command))
        {
            command["status"] = json!(if available {
                "supported"
            } else {
                "unsupported"
            });
        }
    }
    value["job_batches"] = json!({"backend_available":available,"commands":COMMANDS,"automatic_execution_on_open":false,"same_jobs_root_required":true,"scope_request_selectable":false,"partial_failure_retry":"explicit_failed_only_original_jobs_cas","page_max_bytes":MAX_PAGE_BYTES,"large_single_entry":"resource_exhausted; original journal retained"});
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::{api::PlayerSdk, subtitles::storage::UnavailableStorage};
    #[test]
    fn unconfigured_sdk_discovers_batches_as_unavailable_with_explicit_cas_schema() {
        let sdk = PlayerSdk::new(UnavailableStorage);
        let cap = sdk.execute(Request::new("capabilities"));
        let data = cap.data.unwrap();
        assert_eq!(data["commands"].as_array().unwrap().len(), 119);
        assert_eq!(data["job_batches"]["backend_available"], false);
        for command in COMMANDS {
            assert_eq!(
                sdk.execute(Request::new(command)).error.unwrap().code,
                ErrorCode::Unsupported
            );
            let mut request = Request::new("schema");
            request.payload = json!({"command":command});
            let schema = sdk.execute(request).data.unwrap();
            assert_eq!(schema["status"], "unsupported");
            if command == "job_batches.inspect" {
                let operation = &schema["response"]["properties"]["data"]["anyOf"][0]["properties"]
                    ["entries"]["items"]["properties"]["input"]["properties"]["request"]["properties"]
                    ["operation"];
                assert_eq!(operation["type"], "string");
                assert!(operation.get("enum").is_none());
                assert!(operation.get("const").is_none());
            }
            assert_eq!(
                schema["response"]["properties"]["data"]["anyOf"]
                    .as_array()
                    .unwrap()
                    .len(),
                2
            );
            if !matches!(command, "job_batches.create" | "job_batches.inspect") {
                assert!(
                    schema["request"]["properties"]["context"]["required"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|key| key == "expected_revision")
                );
            }
        }
    }
}
