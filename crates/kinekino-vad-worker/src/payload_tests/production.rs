use super::*;
use crate::production::{ReleaseInputs, run_released_request};
use std::{fs, path::PathBuf};
struct Stage {
    root: PathBuf,
    executable: PathBuf,
    manifest: ArtifactPin,
}
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn stage(request: &ValidatedWorkerRequest, descriptor: &[u8], source: &Source<'_>) -> Stage {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "admitted-release-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir(&root).unwrap();
    let profile = request.expected().profile();
    for (name, pin) in [
        ("model.onnx", profile.weights()),
        ("model-license.txt", profile.license_document()),
        ("input.wav", request.expected().input().wav()),
        ("worker.fixture", profile.runtime()),
    ] {
        let bytes = &source.files.iter().find(|(key, _)| key == pin).unwrap().1;
        fs::write(root.join(name), bytes).unwrap();
    }
    fs::write(root.join("descriptor.json"), descriptor).unwrap();
    let p = |pin: &ArtifactPin| serde_json::json!({"id":pin.id(),"sha256":pin.sha256(),"bytes":pin.bytes().to_string()});
    let raw=serde_json::to_vec(&serde_json::json!({"schema_version":"vad.release/1","task":"audio.vad","profile_id":profile.id(),"repo_id":profile.repo(),"immutable_revision":profile.revision(),"format":"onnx","quantization":profile.precision(),"license_id":profile.license_id(),
        "files":{"weights":p(profile.weights()),"runtime":p(profile.runtime()),"license_document":p(profile.license_document()),"runtime_dependencies":[],"runtime_notices":[p(&support::pin("notice",b"synthetic-notice"))]},
        "runtime":{"minimum_version":"fixture-only","provider":profile.provider(),"intra_threads":2,"inter_threads":1},"provenance":{"source_model":"synthetic","source_revision":"1".repeat(40),"conversion_recipe":"unit-test","conversion_version":"1"},"evaluation_id":null,"measured_memory_bytes":null})).unwrap();
    let manifest = support::pin("release", &raw);
    fs::write(root.join("release.manifest.json"), raw).unwrap();
    Stage {
        executable: root.join("worker.fixture"),
        root,
        manifest,
    }
}
#[test]
fn approved_original_release_uses_existing_checked_partial_pipeline() {
    let (request, descriptor, source) = inference_fixture();
    let stage = stage(&request, &descriptor, &source);
    let control = Control::new();
    let engine = CooperativeEngine {
        original: &control,
        calls: Cell::new(0),
    };
    let admission = Exact(request.expected().profile().clone());
    let bytes = run_released_request(
        &request,
        &stage.root,
        &stage.executable,
        &ReleaseInputs {
            profile: request.expected().profile(),
            manifest: &stage.manifest,
            admission: &admission,
        },
        &engine,
        &control,
    )
    .unwrap();
    let outcome = kinekino_vad::protocol::application::decode_worker_outcome(
        bytes.as_slice(),
        bytes.len() as u64,
        request.expected(),
    )
    .unwrap();
    assert!(matches!(
        outcome.result(),
        kinekino_vad::protocol::domain::WorkerResult::Partial {
            processed_samples: 512,
            ..
        }
    ));
    assert_eq!(engine.calls.get(), 1);
    assert!(control.cancel.get());
}
#[test]
fn rejected_existing_admission_precedes_even_manifest_open() {
    struct Deny;
    impl ProfileAdmission for Deny {
        fn authorize(&self, _: &VadProfile) -> Result<(), ApiError> {
            Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "original issuer denied",
            ))
        }
    }
    let (request, descriptor, source) = inference_fixture();
    let stage = stage(&request, &descriptor, &source);
    fs::remove_file(stage.root.join("release.manifest.json")).unwrap();
    let control = Control::new();
    let engine = CooperativeEngine {
        original: &control,
        calls: Cell::new(0),
    };
    let error = run_released_request(
        &request,
        &stage.root,
        &stage.executable,
        &ReleaseInputs {
            profile: request.expected().profile(),
            manifest: &stage.manifest,
            admission: &Deny,
        },
        &engine,
        &control,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::PermissionDenied);
    assert_eq!(error.message, "original issuer denied");
    assert_eq!(engine.calls.get(), 0);
}
#[test]
fn same_size_changed_manifest_is_rejected_before_engine() {
    let (request, descriptor, source) = inference_fixture();
    let stage = stage(&request, &descriptor, &source);
    let path = stage.root.join("release.manifest.json");
    let mut raw = fs::read(&path).unwrap();
    raw[0] = b'[';
    fs::write(path, raw).unwrap();
    let control = Control::new();
    let engine = CooperativeEngine {
        original: &control,
        calls: Cell::new(0),
    };
    let admission = Exact(request.expected().profile().clone());
    let error = run_released_request(
        &request,
        &stage.root,
        &stage.executable,
        &ReleaseInputs {
            profile: request.expected().profile(),
            manifest: &stage.manifest,
            admission: &admission,
        },
        &engine,
        &control,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert_eq!(engine.calls.get(), 0);
}
#[test]
fn validly_hashed_other_release_is_not_the_approved_profile() {
    let (request, descriptor, source) = inference_fixture();
    let mut stage = stage(&request, &descriptor, &source);
    let path = stage.root.join("release.manifest.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["profile_id"] = "other-release".into();
    let raw = serde_json::to_vec(&value).unwrap();
    stage.manifest = support::pin("release", &raw);
    fs::write(path, raw).unwrap();
    let control = Control::new();
    let engine = CooperativeEngine {
        original: &control,
        calls: Cell::new(0),
    };
    let admission = Exact(request.expected().profile().clone());
    let error = run_released_request(
        &request,
        &stage.root,
        &stage.executable,
        &ReleaseInputs {
            profile: request.expected().profile(),
            manifest: &stage.manifest,
            admission: &admission,
        },
        &engine,
        &control,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert_eq!(engine.calls.get(), 0);
}
#[test]
fn different_actual_executable_bytes_cannot_reach_engine() {
    let (request, descriptor, source) = inference_fixture();
    let stage = stage(&request, &descriptor, &source);
    let mut raw = fs::read(&stage.executable).unwrap();
    raw[0] = b'X';
    fs::write(&stage.executable, raw).unwrap();
    let control = Control::new();
    let engine = CooperativeEngine {
        original: &control,
        calls: Cell::new(0),
    };
    let admission = Exact(request.expected().profile().clone());
    let bytes = run_released_request(
        &request,
        &stage.root,
        &stage.executable,
        &ReleaseInputs {
            profile: request.expected().profile(),
            manifest: &stage.manifest,
            admission: &admission,
        },
        &engine,
        &control,
    )
    .unwrap();
    let outcome = kinekino_vad::protocol::application::decode_worker_outcome(
        bytes.as_slice(),
        bytes.len() as u64,
        request.expected(),
    )
    .unwrap();
    assert!(matches!(
        outcome.result(),
        kinekino_vad::protocol::domain::WorkerResult::Failure {
            code: ErrorCode::SourceExpired,
            ..
        }
    ));
    assert_eq!(engine.calls.get(), 0);
}
