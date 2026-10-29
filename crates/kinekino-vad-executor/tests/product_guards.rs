//! Allocated snapshot must supply the exact existing product guard file at the
//! workspace root. These are direct source consumers, not copied guard policies.
#[path = "../../kinekino-player/tests/vad_parent_worker/child_configuration.rs"]
mod child_configuration;
#[path = "../../kinekino-player/src/infrastructure/immutable_storage_paths.rs"]
mod immutable_storage_paths;
#[path = "support/infrastructure.rs"]
mod infrastructure;
#[path = "../../kinekino-player/src/infrastructure/vad_fixed_inputs.rs"]
mod vad_fixed_inputs;
#[path = "../../kinekino-player/src/infrastructure/vad_job_asset.rs"]
mod vad_job_asset;
#[path = "../../kinekino-player/src/infrastructure/vad_job_attempts.rs"]
mod vad_job_attempts;
#[path = "../../kinekino-player/src/infrastructure/vad_job_files.rs"]
mod vad_job_files;
#[path = "../../kinekino-player/tests/support/vad_release_fixture.rs"]
mod vad_release_fixture;
#[path = "../../kinekino-player/src/infrastructure/vad_release_inputs.rs"]
mod vad_release_inputs;
#[path = "../../kinekino-player/src/infrastructure/vad_worker_payload.rs"]
mod vad_worker_payload;
#[path = "../../kinekino-player/src/infrastructure/vad_worker_payload_files.rs"]
mod vad_worker_payload_files;

use kinekino_vad_executor::ArtifactPin;
use std::{
    collections::BTreeMap,
    fs,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

#[test]
fn shared_release_fixture_is_consumed_without_enabling_its_worker() {
    use kinekino_jobs::application::SourceBlobStore;
    use kinekino_vad_executor::application::{VadAdmissionPorts, prepare_vad_admission};
    let fixture: vad_release_fixture::VadReleaseTestFixture =
        vad_release_fixture::release_file_fixture();
    assert!(!fixture.execution.worker.exists());
    assert_eq!(
        fixture
            .inputs
            .resolve_profile(fixture.public_profile_id, &fixture.request.profile_manifest)
            .unwrap(),
        fixture.request.profile
    );
    assert_eq!(
        fixture
            .inputs
            .resolve_audio(fixture.public_audio_id, &fixture.request.audio_source)
            .unwrap(),
        fixture.request.audio
    );
    assert_eq!(
        fixture
            .inputs
            .resolve_destination(fixture.public_destination_id)
            .unwrap(),
        fixture.request.destination
    );
    let admitted = prepare_vad_admission(
        &fixture.request,
        VadAdmissionPorts {
            profiles: fixture.inputs.as_ref(),
            audio: fixture.inputs.as_ref(),
            destinations: fixture.inputs.as_ref(),
            admission: fixture.admission.as_ref(),
            blobs: fixture.blobs.as_ref(),
        },
    )
    .unwrap();
    assert_eq!(
        fixture
            .blobs
            .read_utf8(admitted.descriptor().source())
            .unwrap(),
        admitted.descriptor().raw()
    );
    let root = fixture.root.clone();
    drop(admitted);
    drop(fixture);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reusable_product_entrypoint_retains_parent_guards_without_starting_worker() {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "vad-product-entry-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir(&root).unwrap();
    let worker = root.join("worker.bin");
    fs::write(&worker, b"not a worker").unwrap();
    let executor =
        vad_job_attempts::assemble_attempt_executor(vad_job_attempts::VadAttemptConfiguration {
            worker: worker.clone(),
            worker_pin: ArtifactPin::new(
                "worker".into(),
                "a".repeat(64),
                kinekino_media_contracts::api_values::DecimalU64(12),
            )
            .unwrap(),
            evidence_root: root.clone(),
            cleanup_reserve: Duration::from_secs(1),
            environment: BTreeMap::new(),
        })
        .unwrap();
    // This is inert composition, not binary-pin admission or engine execution.
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    assert!(vad_job_files::VadJobEvidenceRoot::from_application_root(&root).is_err());
    drop(executor);
    let released = vad_job_files::VadJobEvidenceRoot::from_application_root(&root).unwrap();
    drop(released);
    fs::remove_file(root.join(".vad-worker.lock")).unwrap();
    fs::remove_file(worker).unwrap();
    fs::remove_dir(root).unwrap();
}
