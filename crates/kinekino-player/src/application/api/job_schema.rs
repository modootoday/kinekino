use super::{ApiError, ErrorCode, jobs};
use crate::application::jobs::JobsCapability;
use serde_json::{Value, json};
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","description":"Decimal u64 string; range checked"})
}
fn nullable(value: Value) -> Value {
    json!({"anyOf":[value,{"type":"null"}]})
}
fn sha() -> Value {
    json!({"type":"string","pattern":"^[a-f0-9]{64}$"})
}
fn id() -> Value {
    json!({"type":"string","pattern":"^job-[a-f0-9]{32}$"})
}
fn cursor() -> Value {
    object(
        json!({"through_sequence":decimal(),"after_created_sequence":decimal()}),
        &["through_sequence", "after_created_sequence"],
    )
}
pub(super) fn record() -> Value {
    let source = object(
        json!({"sha256":sha(),"bytes":decimal()}),
        &["sha256", "bytes"],
    );
    let input = object(
        json!({"operation":{"type":"string"},"parameters":{"type":"object"},"source":source,"execution_timeout_ms":decimal()}),
        &["operation", "parameters", "source", "execution_timeout_ms"],
    );
    let output = object(
        json!({"path":{"type":"string"},"sha256":sha(),"bytes":decimal(),"receipt":{"type":"object"}}),
        &["path", "sha256", "bytes", "receipt"],
    );
    let failure = object(
        json!({"code":{"enum":ErrorCode::ALL},"message":{"type":"string"},"remediation":{"type":"string"},"details":{},"retryable":{"type":"boolean"}}),
        &["code", "message", "remediation", "details", "retryable"],
    );
    object(
        json!({"schema_version":{"const":"1"},"job_id":id(),"revision":decimal(),"attempt":decimal(),"input":input,"input_sha256":sha(),"submission":nullable(object(json!({"key_sha256":sha(),"request_sha256":sha()}),&["key_sha256","request_sha256"])),"state":{"enum":["queued","running","cancelling","verifying","completed","needs_input","failed","cancelled","interrupted"]},"cancel_requested":{"type":"boolean"},"publication_started":{"type":"boolean"},"output":nullable(output),"failure":nullable(failure)}),
        &[
            "schema_version",
            "job_id",
            "revision",
            "attempt",
            "input",
            "input_sha256",
            "submission",
            "state",
            "cancel_requested",
            "publication_started",
            "output",
            "failure",
        ],
    )
}
pub(super) fn payload(command: &str, cap: &JobsCapability) -> Value {
    let limit = json!({"type":"integer","minimum":1,"maximum":200,"default":50});
    match command {
        "jobs.submit" => {
            let defaults = vec!["subtitles.convert".to_string()];
            let operations = if cap.operations.is_empty() {
                &defaults
            } else {
                &cap.operations
            };
            let mut variants = operations.iter().map(|operation| {
                let browser = matches!(operation.as_str(), "catalog.search" | "episodes.list" | "episodes.resolve");
                let download = operation == crate::application::download::DOWNLOAD_OPERATION;
                let parameters = if operation == crate::application::asset_bundle::BUILD_OPERATION { crate::application::asset_bundle::AssetBundleRequest::schema() } else if operation == crate::application::clip_render::RENDER_OPERATION { crate::application::clip_render::ClipRenderRequest::schema() } else if operation == crate::application::card_bundle::EXPORT_OPERATION { crate::application::card_bundle::CardBundleRequest::schema() } else if download { crate::application::download::DownloadRequest::schema() } else if operation == "episodes.resolve" { object(json!({"episode_url":{"type":"string","format":"uri"}}), &["episode_url"]) } else if browser { super::catalog::live_payload_schema(operation) } else if operation == "subtitles.export_range" { crate::application::subtitles::export_range::schema() } else if operation == "subtitles.from_transcript" { crate::application::subtitles::from_transcript::schema() } else { crate::application::subtitles::convert::schema() };
                object(json!({"operation":{"const":operation},"parameters":parameters,"expected_source_sha256":if browser || download {json!({"type":"null"})}else{nullable(sha())},"execution_timeout_ms":{"type":"string","pattern":"^[0-9]+$","default":if download {"15000"} else {"30000"},"description":"Execution budget1..120000ms, excluding queue and transport"}}), &["operation", "parameters"])
            }).collect::<Vec<_>>();
            if variants.len() == 1 {
                variants.remove(0)
            } else {
                json!({"oneOf":variants})
            }
        }
        "jobs.list" => object(
            json!({"cursor":nullable(cursor()),"limit":nullable(limit)}),
            &[],
        ),
        "jobs.watch" => object(
            json!({"after_sequence":decimal(),"limit":nullable(limit)}),
            &["after_sequence"],
        ),
        "jobs.cancel" => object(
            json!({"job_id":id(),"expected_attempt":decimal()}),
            &["job_id", "expected_attempt"],
        ),
        _ => object(json!({"job_id":id()}), &["job_id"]),
    }
}
fn data(command: &str) -> Value {
    match command {
        "jobs.submit" => object(
            json!({"job":record(),"created":{"type":"boolean"}}),
            &["job", "created"],
        ),
        "jobs.cancel" => object(
            json!({"job":record(),"disposition":{"enum":["requested","already_terminal","publication_in_progress"]}}),
            &["job", "disposition"],
        ),
        "jobs.list" => object(
            json!({"jobs":{"type":"array","maxItems":200,"items":record()},"next_cursor":nullable(cursor())}),
            &["jobs", "next_cursor"],
        ),
        "jobs.watch" => object(
            json!({"events":{"type":"array","maxItems":200,"items":object(json!({"sequence":decimal(),"job":record()}),&["sequence","job"])},"next_sequence":decimal(),"latest_sequence":decimal(),"earliest_sequence":decimal(),"gap":{"type":"boolean"}}),
            &[
                "events",
                "next_sequence",
                "latest_sequence",
                "earliest_sequence",
                "gap",
            ],
        ),
        _ => record(),
    }
}
pub(super) fn describe(input: &Value, cap: JobsCapability) -> Result<Value, ApiError> {
    let command = input["command"]
        .as_str()
        .filter(|command| jobs::is_command(command))
        .filter(|_| input.as_object().is_some_and(|value| value.len() == 1))
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "schema requires one job command",
            )
        })?;
    let available = if jobs::is_write(command) {
        cap.execution_available
    } else {
        cap.read_available
    };
    let mut context = super::schema::context_schema();
    context["properties"]["timeout_ms"] = json!({"type":"null"});
    context["properties"]["expected_revision"] = if command == "jobs.retry" {
        decimal()
    } else {
        json!({"type":"null"})
    };
    context["properties"]["idempotency_key"] = if command == "jobs.submit" {
        json!({"type":"string","minLength":1,"maxLength":128})
    } else {
        json!({"type":"null"})
    };
    let mut required = vec!["request_id", "api_version"];
    if command == "jobs.submit" {
        required.push("idempotency_key");
    }
    if command == "jobs.retry" {
        required.push("expected_revision");
    }
    context["required"] = json!(required);
    Ok(
        json!({"schema_version":"1","command":command,"status":if available{"supported"}else{"unsupported"},"request":object(json!({"context":context,"command":{"const":command},"payload":payload(command,&cap)}),&["context","command","payload"]),"response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":nullable(data(command)),"error":{"type":["object","null"]}}),&["api_version","request_id","ok","data","error"]),"error_codes":ErrorCode::ALL,"watch_mode":"bounded_durable_event_page","record_is_not_current_file_availability":true,"unsupported_executors":["production.capture_frame","models","inference"],"browser_operations_maturity":"experimental","episode_resolution_scope":crate::application::episode_resolve::RESOLUTION_SCOPE,"episode_resolution_can_complete":cap.execution_available && cap.operations.iter().any(|operation|operation=="episodes.resolve"),"episode_media_decode_available":false,"browser_source_kind":"request_descriptor; source SHA does not freeze live web content","catalog_result_readback":"catalog.search or episodes.list with result_artifact_sha256","episode_result_readback":"jobs.get output.receipt.resolution_artifact via catalog.read_raw","cancel_ack_required":true,"automatic_restart_execution":false}),
    )
}
pub(super) fn capabilities(value: &mut Value, cap: JobsCapability) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if let Some(name) = command["command"]
                .as_str()
                .filter(|name| jobs::is_command(name))
            {
                let available = if jobs::is_write(name) {
                    cap.execution_available
                } else {
                    cap.read_available
                };
                command["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    let experimental = cap
        .operations
        .iter()
        .filter(|operation| {
            matches!(
                operation.as_str(),
                "catalog.search" | "episodes.list" | "episodes.resolve"
            )
        })
        .collect::<Vec<_>>();
    value["jobs"] = json!({"read_backend_available":cap.read_available,"execution_host_available":cap.execution_available,"operations":cap.operations,"experimental_operations":experimental,"episode_resolution_scope":crate::application::episode_resolve::RESOLUTION_SCOPE,"episode_resolution_can_complete":cap.execution_available && cap.operations.iter().any(|operation|operation=="episodes.resolve"),"episode_media_decode_available":false,"browser_source_kind":"request_descriptor; source SHA does not freeze live web content","catalog_result_readback":"catalog.search or episodes.list with result_artifact_sha256","episode_result_readback":"jobs.get output.receipt.resolution_artifact via catalog.read_raw","watch_mode":"bounded_durable_event_page","page_max_bytes":1048576,"cancel_uses_expected_attempt":true,"retry_uses_expected_revision":true,"automatic_restart_execution":false,"source_max_bytes":kinekino_jobs::domain::MAX_SOURCE_BYTES,"idempotency":"durable_normalized_request_first_wins; revision_conflict details.conflict_kind=idempotency_conflict"});
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn range_jobs_use_original_range_schema_and_keep_convert_schema_unchanged() {
        let cap = JobsCapability {
            read_available: true,
            execution_available: true,
            operations: vec!["subtitles.convert".into(), "subtitles.export_range".into()],
        };
        let value = payload("jobs.submit", &cap);
        let variants = value["oneOf"].as_array().unwrap();
        let convert = variants
            .iter()
            .find(|variant| variant["properties"]["operation"]["const"] == "subtitles.convert")
            .unwrap();
        let range = variants
            .iter()
            .find(|variant| variant["properties"]["operation"]["const"] == "subtitles.export_range")
            .unwrap();
        assert_eq!(
            convert["properties"]["parameters"],
            crate::application::subtitles::convert::schema()
        );
        assert_eq!(
            range["properties"]["parameters"],
            crate::application::subtitles::export_range::schema()
        );
        assert!(
            range["properties"]["parameters"]["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field == "range")
        );
    }
    #[test]
    fn unconfigured_job_schema_preserves_legacy_conversion_default() {
        let cap = JobsCapability {
            read_available: false,
            execution_available: false,
            operations: vec![],
        };
        let value = payload("jobs.submit", &cap);
        assert_eq!(
            value["properties"]["operation"]["const"],
            "subtitles.convert"
        );
        assert_eq!(
            value["properties"]["parameters"],
            crate::application::subtitles::convert::schema()
        );
    }
}
