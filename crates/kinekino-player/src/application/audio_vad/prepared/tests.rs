use super::*;
use kinekino_jobs::domain::SourceBlobRef;
use kinekino_media_contracts::api_values::DecimalU64;
use serde_json::json;

#[cfg(windows)]
#[path = "../../../../tests/support/retaining_job_host.rs"]
mod host_closeout;
#[cfg(windows)]
mod persistence;

fn parameters() -> Value {
    json!({"profile_id":"vad.release-1","expected_profile_manifest_sha256":"a".repeat(64),
        "audio_artifact_id":"audio.original-1","start_sample":"0005","end_sample":"16000",
        "destination_id":"analysis.result-1"})
}
fn input(original: &str) -> PreparedJobInput {
    let request = VadSourceRequest::decode(parameters(), Some(original)).unwrap();
    PreparedJobInput {
        operation: VAD_JOB_OPERATION.into(),
        parameters: request.prepared_parameters().unwrap(),
        source: SourceBlobRef {
            sha256: "c".repeat(64),
            bytes: DecimalU64(25),
        },
        execution_timeout_ms: DecimalU64(7000),
    }
}

#[test]
fn serialized_prepared_input_preserves_original_audio_without_aliasing_descriptor_or_manifest() {
    let original = "b".repeat(64);
    let initial = input(&original);
    let stored = serde_json::to_vec(&initial).unwrap();
    let reopened: PreparedJobInput = serde_json::from_slice(&stored).unwrap();
    assert_eq!(initial.sha256().unwrap(), reopened.sha256().unwrap());
    let restored = VadSourceRequest::from_prepared(&reopened).unwrap();
    assert_eq!(restored.original_audio_sha256(), original);
    assert_eq!(
        restored.parameters().expected_profile_manifest_sha256,
        "a".repeat(64)
    );
    assert_eq!(reopened.source.sha256, "c".repeat(64));
    assert_eq!(reopened.execution_timeout_ms, DecimalU64(7000));
    assert_eq!(restored.parameters().start_sample, DecimalU64(5));
    assert!(super::super::normalize_parameters(reopened.parameters).is_err());
}

#[test]
fn original_pin_changes_prepared_fingerprint_without_becoming_public_parameter() {
    let left = input(&"b".repeat(64));
    let right = input(&"d".repeat(64));
    assert_eq!(left.parameters["request"], right.parameters["request"]);
    assert_eq!(left.source.sha256, right.source.sha256);
    assert_ne!(left.sha256().unwrap(), right.sha256().unwrap());
}

#[test]
fn stored_request_rejects_missing_pin_unknown_version_fields_and_other_operation() {
    let original = input(&"b".repeat(64));
    let mut missing = original.clone();
    missing
        .parameters
        .as_object_mut()
        .unwrap()
        .remove("original_source_sha256");
    assert!(VadSourceRequest::from_prepared(&missing).is_err());
    for replacement in [json!(null), json!("not-a-sha")] {
        let mut changed = original.clone();
        changed.parameters["original_source_sha256"] = replacement;
        assert!(VadSourceRequest::from_prepared(&changed).is_err());
    }
    let mut unknown = original.clone();
    unknown.parameters["schema_version"] = json!("2");
    assert_eq!(
        VadSourceRequest::from_prepared(&unknown).unwrap_err().code,
        ErrorCode::Unsupported
    );
    let mut extra = original.clone();
    extra.parameters["worker"] = json!("worker.exe");
    assert!(VadSourceRequest::from_prepared(&extra).is_err());
    let mut other = original;
    other.operation = "subtitles.convert".into();
    assert_eq!(
        VadSourceRequest::from_prepared(&other).unwrap_err().code,
        ErrorCode::Unsupported
    );
}
