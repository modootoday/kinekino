//! Synthetic assembly/lease cases only; no process, native engine or permission.
use crate::{
    vad_job_attempts::{VadAttemptConfiguration, assemble_parent_configured_attempt_executor},
    vad_release_fixture::release_file_fixture,
    vad_worker_payload::ReleasedWorkerPayload,
};
use kinekino_jobs::domain::PreparedJobInput;
use kinekino_media_contracts::api_values::DecimalU64;
use kinekino_vad::domain::ArtifactPin;
use kinekino_vad::protocol::application::decode_worker_request;
use kinekino_vad_executor::{
    application::{
        CHILD_CONFIGURATION_FILE, ParentSelectedRelease, RunControl, VadAdmissionPorts,
        decode_parent_configuration, prepare_vad_admission,
    },
    infrastructure::WorkerPayloadPreparation,
};
use std::{
    fs,
    sync::Arc,
    time::{Duration, Instant},
};
struct Original(Instant);
impl RunControl for Original {
    fn deadline(&self) -> Instant {
        self.0
    }
    fn cancellation_requested(&self) -> bool {
        false
    }
}
#[derive(Clone, Copy)]
enum Case {
    Matching,
    Digest,
    Release,
    Identity,
}
fn exercise(case: Case) {
    let fixture = release_file_fixture();
    let root = fixture.root.clone();
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
    let profile = admission.descriptor().decoded().profile();
    let selected = Arc::new(
        ParentSelectedRelease::authorize(
            profile.clone(),
            admission.profile_manifest().clone(),
            fixture.admission.as_ref(),
        )
        .unwrap(),
    );
    let worker_path = root.join("configured-worker.fixture");
    fs::write(&worker_path, b"synthetic-runtime").unwrap();
    let mut worker = profile.runtime().clone();
    let selected = match case {
        Case::Matching => selected,
        Case::Digest => {
            worker = ArtifactPin::new(
                worker.id().into(),
                "e".repeat(64),
                DecimalU64(worker.bytes()),
            )
            .unwrap();
            selected
        }
        Case::Identity => {
            worker = ArtifactPin::new(
                "other-worker".into(),
                worker.sha256().into(),
                DecimalU64(worker.bytes()),
            )
            .unwrap();
            selected
        }
        Case::Release => Arc::new(
            ParentSelectedRelease::authorize(
                profile.clone(),
                ArtifactPin::new(
                    "other-release".into(),
                    "e".repeat(64),
                    DecimalU64(admission.profile_manifest().bytes()),
                )
                .unwrap(),
                fixture.admission.as_ref(),
            )
            .unwrap(),
        ),
    };
    let evidence = root.join("host-owned-evidence");
    fs::create_dir(&evidence).unwrap();
    let input = PreparedJobInput {
        operation: "audio.vad.candidate".into(),
        parameters: serde_json::json!({"synthetic_only":true}),
        source: admission.descriptor().source().clone(),
        execution_timeout_ms: DecimalU64(1000),
    };
    let result = assemble_parent_configured_attempt_executor(
        VadAttemptConfiguration {
            worker: worker_path,
            worker_pin: worker,
            evidence_root: evidence.clone(),
            cleanup_reserve: Duration::from_millis(100),
            environment: Default::default(),
        },
        &input,
        admission.clone(),
        fixture.inputs.clone(),
        selected.clone(),
    );
    if !matches!(case, Case::Matching) {
        assert!(result.is_err());
        assert_eq!(fs::read_dir(&evidence).unwrap().count(), 0);
    } else {
        // Assembly is inert, then the exact producer stages a guarded selection.
        let executor = result.unwrap();
        let entries: Vec<_> = fs::read_dir(&evidence)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(entries, vec![std::ffi::OsString::from(".vad-worker.lock")]);
        assert!(
            crate::vad_job_files::VadJobEvidenceRoot::from_application_root(&evidence).is_err()
        );
        let context: serde_json::Value =
            serde_json::from_str(admission.descriptor().raw()).unwrap();
        let raw=serde_json::to_vec(&serde_json::json!({"schema_version":"1","job_id":"configured-fixture-only","attempt":"1","input_sha256":"a".repeat(64),"source_blob_sha256":"b".repeat(64),"submission_request_sha256":null,"descriptor_sha256":admission.descriptor().decoded().raw().sha256(),"context":context})).unwrap();
        let decoded = decode_worker_request(
            raw.as_slice(),
            raw.len() as u64,
            admission.descriptor().raw().as_bytes(),
        )
        .unwrap();
        let stage = root.join("configuration-stage");
        fs::create_dir(&stage).unwrap();
        fs::write(stage.join("stdin.raw"), &raw).unwrap();
        let producer = ReleasedWorkerPayload::new(
            fixture.inputs.clone(),
            admission.clone(),
            profile.runtime().clone(),
        )
        .unwrap()
        .with_selected_release(selected.clone())
        .unwrap();
        let staged = producer
            .stage(
                &raw,
                &stage,
                &Original(Instant::now() + Duration::from_secs(10)),
            )
            .unwrap();
        let (_, pin) = selected
            .encode_for_attempt(
                decoded.expected().request(),
                profile,
                admission.profile_manifest(),
                profile.runtime(),
            )
            .unwrap();
        let actual = fs::read(stage.join(CHILD_CONFIGURATION_FILE)).unwrap();
        let delegated =
            decode_parent_configuration(&actual, &pin, decoded.expected().request()).unwrap();
        delegated
            .check_request(profile, admission.profile_manifest())
            .unwrap();
        drop(producer);
        drop(executor);
        drop(crate::vad_job_files::VadJobEvidenceRoot::from_application_root(&evidence).unwrap());
        crate::infrastructure::os_policy::guarded_mutation(fs::write(
            stage.join(CHILD_CONFIGURATION_FILE),
            b"changed",
        ));
        drop(staged);
        assert!(fs::write(stage.join(CHILD_CONFIGURATION_FILE), b"changed").is_ok());
    }
    drop(selected);
    drop(admission);
    drop(fixture);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn selected_release_assembles_and_owns_guarded_configuration() {
    kinekino_platform::install();
    exercise(Case::Matching)
}
#[test]
fn different_digest_is_rejected_by_actual_assembly_before_spawn() {
    kinekino_platform::install();
    exercise(Case::Digest)
}
#[test]
fn different_release_is_rejected_by_actual_assembly_before_spawn() {
    kinekino_platform::install();
    exercise(Case::Release)
}
#[test]
fn different_identity_is_rejected_by_actual_assembly_before_spawn() {
    kinekino_platform::install();
    exercise(Case::Identity)
}
