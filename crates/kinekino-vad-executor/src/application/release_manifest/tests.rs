use super::*;
use crate::application::source_tests::admitted_fixture;
use serde_json::{Value, json};

fn pin(value: &ArtifactPin) -> Value {
    json!({"id":value.id(),"sha256":value.sha256(),"bytes":value.bytes().to_string()})
}
fn fixture() -> Value {
    let (admission, _) = admitted_fixture();
    let p = admission.descriptor().decoded().profile();
    json!({"schema_version":"vad.release/1","task":"audio.vad","profile_id":p.id(),
        "repo_id":p.repo(),"immutable_revision":p.revision(),"format":"onnx","quantization":p.precision(),
        "license_id":p.license_id(),"files":{"weights":pin(p.weights()),"license_document":pin(p.license_document()),
        "runtime":pin(p.runtime()),"runtime_dependencies":[],"runtime_notices":[{"id":"native-notice","sha256":"b".repeat(64),"bytes":"1"}]},
        "runtime":{"minimum_version":"1.31.0","provider":p.provider(),"intra_threads":p.intra_threads(),"inter_threads":p.inter_threads()},
        "provenance":{"source_model":"declared-upstream","source_revision":"1".repeat(40),"conversion_recipe":"declared-conversion","conversion_version":"1"},
        "evaluation_id":null,"measured_memory_bytes":null})
}
fn decode(value: &Value) -> Result<VadReleaseManifest, ApiError> {
    let raw = serde_json::to_vec(value).unwrap();
    let expected = ArtifactPin::new(
        "release".into(),
        format!("{:x}", Sha256::digest(&raw)),
        DecimalU64(raw.len() as u64),
    )
    .unwrap();
    decode_release_manifest(&raw, &expected)
}
fn with_notice_sizes(sizes: &[u64]) -> Value {
    let mut value = fixture();
    value["files"]["runtime_notices"] = Value::Array(sizes.iter().enumerate().map(|(i, size)| {
        json!({"id":format!("native-notice-{i}"),"sha256":"b".repeat(64),"bytes":size.to_string()})
    }).collect());
    value
}
#[test]
fn acquired_and_exact_limit_notice_pins_decode_without_changing_model_limits() {
    for sizes in [
        vec![338_088, 1073],
        vec![MAX_RUNTIME_NOTICE_BYTES],
        vec![65_536; 16],
    ] {
        decode(&with_notice_sizes(&sizes)).unwrap();
    }
    let mut value = fixture();
    value["files"]["license_document"]["bytes"] = json!("65537");
    assert_eq!(
        decode(&value).err().unwrap().code,
        ErrorCode::InvalidArgument
    );
}
#[test]
fn notice_file_total_and_count_bounds_are_independent() {
    let half = kinekino_vad::domain::MAX_RUNTIME_NOTICES_TOTAL_BYTES / 2;
    decode(&with_notice_sizes(&[half, half])).unwrap();
    for sizes in [
        vec![MAX_RUNTIME_NOTICE_BYTES + 1],
        vec![half, half + 1],
        vec![1; 17],
    ] {
        assert_eq!(
            decode(&with_notice_sizes(&sizes)).err().unwrap().code,
            ErrorCode::InvalidArgument
        );
    }
}
#[test]
fn release_identity_is_distinct_from_private_snapshot_and_metadata_remains_declared() {
    let release = decode(&fixture()).unwrap();
    assert_eq!(release.artifacts().len(), 4);
    assert_eq!(release.profile().precision(), "fp32");
    assert_eq!(release.declared_evaluation_id(), None);
    assert_eq!(release.declared_memory_bytes(), None);
    assert_eq!(release.declared_provenance()[0], "declared-upstream");
}
#[test]
fn original_release_bytes_and_size_are_checked() {
    let raw = serde_json::to_vec(&fixture()).unwrap();
    let pin = ArtifactPin::new(
        "release".into(),
        "0".repeat(64),
        DecimalU64(raw.len() as u64),
    )
    .unwrap();
    assert_eq!(
        decode_release_manifest(&raw, &pin).err().unwrap().code,
        ErrorCode::SourceExpired
    );
    let huge = vec![0; MAX_RELEASE_MANIFEST_BYTES as usize + 1];
    assert_eq!(
        decode_release_manifest(&huge, &pin).err().unwrap().code,
        ErrorCode::ResourceExhausted
    );
}
#[test]
fn private_descriptor_unknown_fields_and_wrong_task_are_rejected() {
    for pointer in [
        "/schema_version",
        "/task",
        "/format",
        "/quantization",
        "/immutable_revision",
        "/provenance/source_revision",
    ] {
        let mut wire = fixture();
        *wire.pointer_mut(pointer).unwrap() = json!("unsupported");
        assert!(decode(&wire).is_err(), "{pointer}");
    }
    let mut wire = fixture();
    wire["worker"] = json!("C:/untrusted.exe");
    assert!(decode(&wire).is_err());
}
#[test]
fn duplicate_artifact_ids_and_missing_notices_are_rejected() {
    let mut wire = fixture();
    wire["files"]["runtime_notices"] = json!([]);
    assert!(decode(&wire).is_err());
    let mut wire = fixture();
    wire["files"]["runtime_notices"][0] = wire["files"]["runtime"].clone();
    assert!(decode(&wire).is_err());
}
#[test]
fn declared_memory_remains_lossless_without_becoming_a_measurement() {
    let mut wire = fixture();
    wire["measured_memory_bytes"] = json!(u64::MAX.to_string());
    assert_eq!(
        decode(&wire).unwrap().declared_memory_bytes(),
        Some(DecimalU64(u64::MAX))
    );
    wire["measured_memory_bytes"] = json!(42);
    assert!(decode(&wire).is_err());
}
