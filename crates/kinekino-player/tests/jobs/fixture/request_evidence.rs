use kinekino_player::{application::api::Request, infrastructure::content_digest};
use serde_json::Value;

pub(super) fn request_link(
    label: &str,
    request: &Request,
    endpoint: &str,
    bytes: &[u8],
    expected_exit: i32,
) -> std::io::Result<Value> {
    let (length, sha) =
        content_digest::sha256(&mut std::io::Cursor::new(bytes), bytes.len() as u64)
            .map_err(|error| std::io::Error::other(format!("{error:?}")))?;
    Ok(
        serde_json::json!({"schema_version":1,"label":label,"command":request.command,"request_id":request.context.request_id,
        "endpoint":endpoint,"operation":request.payload.get("operation").and_then(Value::as_str),
        "request_bytes":length,"request_sha256":sha,"expected_exit":expected_exit,
        "request_evidence":format!("logs/{label}.request.json"),"stdin_evidence":format!("logs/owned-requests/{label}/owned/stdin.raw"),
        "process_evidence":format!("logs/owned-requests/{label}/process.json"),"failure_evidence":format!("logs/owned-requests/{label}/failure.json"),
        "meaning":"persisted invocation intent before supervised launch; not proof of spawn, admission or successful response"}),
    )
}
pub(super) fn request_settlement(
    link: &Value,
    actual_exit: Option<u64>,
    returned_expected_response: bool,
    failure: Option<Value>,
) -> Value {
    serde_json::json!({"schema_version":1,"request":link,"actual_exit":actual_exit,
        "returned_expected_response":returned_expected_response,"failure":failure,
        "meaning":"returned expected exit and parse/deadline checks; may be an expected API error, not media success"})
}

#[cfg(test)]
mod request_evidence_tests {
    use super::super::request;
    use super::*;
    use serde_json::json;
    use std::path::Path;
    #[test]
    fn link_hashes_exact_stdin_bytes_and_keeps_original_request_identity() {
        kinekino_platform::install();
        let request = request("version", json!({}));
        let bytes = serde_json::to_vec(&request).unwrap();
        let link = request_link("one", &request, "original-endpoint", &bytes, 0).unwrap();
        let mut spaced = bytes.clone();
        spaced.push(b'\n');
        let other = request_link("one", &request, "original-endpoint", &spaced, 0).unwrap();
        assert_eq!(link["request_bytes"], bytes.len());
        assert_eq!(other["request_bytes"], bytes.len() + 1);
        assert_ne!(link["request_sha256"], other["request_sha256"]);
        assert_eq!(link["request_id"], "jobs-fixture");
        assert_eq!(link["command"], "version");
        assert_eq!(link["endpoint"], "original-endpoint");
    }
    #[test]
    fn expected_api_error_completion_is_distinct_from_successful_api_response() {
        kinekino_platform::install();
        let link = request_link(
            "closed",
            &request("playback.engine", json!({})),
            "original-endpoint",
            b"original bytes",
            4,
        )
        .unwrap();
        let settled = request_settlement(&link, Some(4), true, None);
        assert_eq!(settled["returned_expected_response"], true);
        assert_eq!(settled["actual_exit"], 4);
        assert_eq!(settled["request"]["expected_exit"], 4);
        assert!(settled["failure"].is_null());
    }
    #[test]
    fn failure_observation_does_not_replace_original_error_or_invent_spawn_or_completion() {
        kinekino_platform::install();
        use super::super::super::kinekino_process::CliFailure;
        let original: Box<CliFailure> = CliFailure::not_spawned(
            "original-stage",
            std::io::Error::other("original first error"),
            Path::new("original-evidence"),
        );
        let settled = request_settlement(
            &json!({"label":"failed"}),
            None,
            false,
            Some(original.observation()),
        );
        assert_eq!(settled["returned_expected_response"], false);
        assert!(settled["actual_exit"].is_null());
        assert_eq!(settled["failure"]["ownership"], "not_spawned");
        assert_eq!(original.stage, "original-stage");
        assert_eq!(original.error.to_string(), "original first error");
    }
}
