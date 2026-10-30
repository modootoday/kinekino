use super::*;
use kinekino_media_contracts::errors::ErrorCode;
use serde_json::{Value, json};

fn parameters() -> Value {
    json!({
        "profile_id":"vad.release-1",
        "expected_profile_manifest_sha256":"a".repeat(64),
        "audio_artifact_id":"audio.original-1",
        "start_sample":"0000",
        "end_sample":"18446744073709551615",
        "destination_id":"analysis.result-1"
    })
}

#[test]
fn normalization_preserves_lossless_samples_and_canonical_submission_fields() {
    let normalized = normalize_parameters(parameters()).unwrap();
    assert_eq!(normalized["start_sample"], "0");
    assert_eq!(normalized["end_sample"], u64::MAX.to_string());
    assert_eq!(
        normalize_parameters(normalized.clone()).unwrap(),
        normalized
    );
    let request = VadSourceRequest::decode(normalized, Some(&"b".repeat(64))).unwrap();
    assert_eq!(request.original_audio_sha256(), "b".repeat(64));
    assert_eq!(
        request.parameters().expected_profile_manifest_sha256,
        "a".repeat(64)
    );
}

#[test]
fn original_audio_pin_is_required_and_cannot_be_replaced_by_parameter_fields() {
    for pin in [None, Some(""), Some("not-a-sha"), Some(&"A".repeat(64))] {
        let error = VadSourceRequest::decode(parameters(), pin).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
    }
    for name in ["expected_audio_sha256", "descriptor_sha256", "wav_sha256"] {
        let mut value = parameters();
        value[name] = json!("b".repeat(64));
        assert!(normalize_parameters(value).is_err());
    }
}

#[test]
fn request_rejects_paths_programs_unknown_fields_and_lossy_sample_numbers() {
    for field in ["profile_id", "audio_artifact_id", "destination_id"] {
        for value in [
            "../outside",
            "C:\\worker.exe",
            "https://example.test/audio",
            "",
        ] {
            let mut input = parameters();
            input[field] = json!(value);
            assert!(normalize_parameters(input).is_err());
        }
    }
    for (field, value) in [
        ("worker", json!("worker.exe")),
        ("start_sample", json!(0)),
        ("end_sample", json!("18446744073709551616")),
    ] {
        let mut input = parameters();
        input[field] = value;
        assert!(normalize_parameters(input).is_err());
    }
}

#[test]
fn private_profile_snapshot_pin_is_not_a_public_release_manifest_alias() {
    let mut input = parameters();
    input
        .as_object_mut()
        .unwrap()
        .remove("expected_profile_manifest_sha256");
    input["expected_profile_snapshot_sha256"] = json!("a".repeat(64));
    assert!(normalize_parameters(input).is_err());
}
