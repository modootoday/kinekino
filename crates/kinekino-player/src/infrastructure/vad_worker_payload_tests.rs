use super::super::vad_fixed_inputs::{VadReleaseTestFixture, release_file_fixture};
use super::*;
use kinekino_media_contracts::errors::ErrorCode;
use kinekino_vad_executor::application::{RunControl, VadAdmissionPorts, prepare_vad_admission};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

struct Original {
    cancel: AtomicBool,
    end: Instant,
}
impl Original {
    fn fresh() -> Self {
        Self {
            cancel: AtomicBool::new(false),
            end: Instant::now() + Duration::from_secs(10),
        }
    }
}
impl RunControl for Original {
    fn deadline(&self) -> Instant {
        self.end
    }
    fn cancellation_requested(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
}
fn fixture() -> (VadReleaseTestFixture, Arc<PreparedVadAdmission>, Vec<u8>) {
    let fixture = release_file_fixture();
    let admission = Arc::new(
        prepare_vad_admission(
            &fixture.request,
            VadAdmissionPorts {
                profiles: fixture.inputs.as_ref(),
                audio: fixture.inputs.as_ref(),
                destinations: fixture.inputs.as_ref(),
                admission: fixture.admission.as_ref(),
                blobs: fixture.blobs.as_ref(),
            },
        )
        .unwrap(),
    );
    // Wire labels only. This fixture creates no SDK attempt identity or scope.
    let context: serde_json::Value = serde_json::from_str(admission.descriptor().raw()).unwrap();
    let request = serde_json::to_vec(&serde_json::json!({"schema_version":"1","job_id":"synthetic-stage-only","attempt":"1",
        "input_sha256":"a".repeat(64),"source_blob_sha256":"b".repeat(64),"submission_request_sha256":null,
        "descriptor_sha256":admission.descriptor().decoded().raw().sha256(),"context":context})).unwrap();
    (fixture, admission, request)
}
fn stage_root(fixture: &VadReleaseTestFixture, request: &[u8]) -> PathBuf {
    let root = fixture.root.join("worker-stage");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("stdin.raw"), request).unwrap();
    root
}
fn producer(
    fixture: &VadReleaseTestFixture,
    admission: Arc<PreparedVadAdmission>,
) -> ReleasedWorkerPayload {
    let worker = admission.descriptor().decoded().profile().runtime().clone();
    ReleasedWorkerPayload::new(fixture.inputs.clone(), admission, worker).unwrap()
}
#[test]
fn staged_payload_keeps_actual_release_request_and_sidecar_guards_after_producer_drop() {
    let (fixture, admission, request) = fixture();
    let root = fixture.root.clone();
    let stage = stage_root(&fixture, &request);
    let original_manifest = fs::read(root.join("release.json")).unwrap();
    let original_descriptor = admission.descriptor().raw().as_bytes().to_vec();
    let producer = producer(&fixture, admission.clone());
    let staged = producer
        .stage(&request, &stage, &Original::fresh())
        .unwrap();
    drop(producer);
    drop(admission);
    drop(fixture);
    assert_eq!(
        fs::read(stage.join("release.manifest.json")).unwrap(),
        original_manifest
    );
    assert_eq!(
        fs::read(stage.join("descriptor.json")).unwrap(),
        original_descriptor
    );
    assert_eq!(fs::read(stage.join("model.onnx")).unwrap(), b"synthetic");
    crate::infrastructure::os_policy::guarded_mutation(fs::write(
        stage.join("stdin.raw"),
        b"replace request",
    ));
    crate::infrastructure::os_policy::guarded_mutation(fs::write(
        stage.join("model.onnx"),
        b"replace staged model",
    ));
    crate::infrastructure::os_policy::guarded_mutation(fs::write(
        root.join("release.json"),
        b"replace source release",
    ));
    drop(staged);
    fs::write(root.join("release.json"), b"released after owner drop").unwrap();
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn changed_original_request_is_rejected_before_any_sidecar_write() {
    let (fixture, admission, request) = fixture();
    let root = fixture.root.clone();
    let stage = stage_root(&fixture, b"changed request");
    let producer = producer(&fixture, admission);
    let error = producer
        .stage(&request, &stage, &Original::fresh())
        .err()
        .unwrap();
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert!(!stage.join("descriptor.json").exists());
    drop(producer);
    drop(fixture);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn sidecar_collision_preserves_original_evidence_and_admission_owner() {
    let (fixture, admission, request) = fixture();
    let root = fixture.root.clone();
    let stage = stage_root(&fixture, &request);
    fs::write(stage.join("descriptor.json"), b"original evidence").unwrap();
    let producer = producer(&fixture, admission.clone());
    assert!(
        producer
            .stage(&request, &stage, &Original::fresh())
            .is_err()
    );
    assert_eq!(
        fs::read(stage.join("descriptor.json")).unwrap(),
        b"original evidence"
    );
    assert!(!stage.join("release.manifest.json").exists());
    crate::infrastructure::os_policy::guarded_mutation(fs::write(
        root.join("model.bin"),
        b"replace source",
    ));
    drop(producer);
    drop(admission);
    drop(fixture);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn worker_runtime_pin_mismatch_is_rejected_before_payload_creation() {
    let (fixture, admission, _) = fixture();
    let root = fixture.root.clone();
    let worker = ArtifactPin::new(
        "other-worker".into(),
        "e".repeat(64),
        kinekino_media_contracts::api_values::DecimalU64(17),
    )
    .unwrap();
    let error = ReleasedWorkerPayload::new(fixture.inputs.clone(), admission.clone(), worker)
        .err()
        .unwrap();
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    assert!(!root.join("worker-stage").exists());
    drop(admission);
    drop(fixture);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn cancelled_payload_preparation_leaves_no_sidecars_or_ack() {
    let (fixture, admission, request) = fixture();
    let root = fixture.root.clone();
    let stage = stage_root(&fixture, &request);
    let producer = producer(&fixture, admission);
    let original = Original::fresh();
    original.cancel.store(true, Ordering::SeqCst);
    let error = producer.stage(&request, &stage, &original).err().unwrap();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert!(!stage.join("descriptor.json").exists());
    assert!(original.cancel.load(Ordering::SeqCst));
    drop(producer);
    drop(fixture);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn expired_payload_preparation_leaves_no_sidecars_or_new_budget() {
    let (fixture, admission, request) = fixture();
    let root = fixture.root.clone();
    let stage = stage_root(&fixture, &request);
    let producer = producer(&fixture, admission);
    let original = Original {
        cancel: AtomicBool::new(false),
        end: Instant::now(),
    };
    let error = producer.stage(&request, &stage, &original).err().unwrap();
    assert_eq!(error.code, ErrorCode::Timeout);
    assert!(!stage.join("descriptor.json").exists());
    drop(producer);
    drop(fixture);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn static_worker_constructor_retains_sources_without_starting_or_reading_worker() {
    use super::super::vad_job_attempts::assemble_static_worker_attempt_executor;
    let (fixture, admission, _) = fixture();
    let root = fixture.root.clone();
    assert!(!fixture.execution.worker.exists());
    let input = kinekino_jobs::domain::PreparedJobInput {
        operation: "audio.vad.candidate".into(),
        parameters: serde_json::json!({"synthetic_only":true}),
        source: admission.descriptor().source().clone(),
        execution_timeout_ms: kinekino_media_contracts::api_values::DecimalU64(1000),
    };
    let mut configuration = fixture.execution;
    configuration.worker_pin = admission.descriptor().decoded().profile().runtime().clone();
    let executor = assemble_static_worker_attempt_executor(
        configuration,
        &input,
        admission.clone(),
        fixture.inputs.clone(),
    )
    .unwrap();
    drop(admission);
    drop(fixture.inputs);
    drop(fixture.admission);
    drop(fixture.blobs);
    crate::infrastructure::os_policy::guarded_mutation(fs::write(
        root.join("model.bin"),
        b"source still owned",
    ));
    drop(executor);
    fs::write(root.join("model.bin"), b"source released").unwrap();
    fs::remove_dir_all(root).unwrap();
}
