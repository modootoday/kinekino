use crate::{
    vad_release_fixture::{VadReleaseTestFixture, release_file_fixture},
    vad_worker_payload::ReleasedWorkerPayload,
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::protocol::{application::decode_worker_request, domain::WorkerResult};
use kinekino_vad::{
    application::{AnalysisControl, EngineCompletion, ScoreSink, VadEngine},
    domain::VadProfile,
};
use kinekino_vad_executor::{
    application::{RunControl, VadAdmissionPorts, prepare_vad_admission},
    infrastructure::WorkerPayloadPreparation,
};
use std::{
    fs,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

struct Original {
    cancelled: AtomicBool,
    deadline: Instant,
}
impl RunControl for Original {
    fn deadline(&self) -> Instant {
        self.deadline
    }
    fn cancellation_requested(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}
impl AnalysisControl for Original {
    fn deadline(&self) -> Instant {
        self.deadline
    }
    fn cancellation_requested(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}
struct Engine<'a> {
    original: &'a Original,
    partial: bool,
    calls: AtomicUsize,
}
impl VadEngine for Engine<'_> {
    fn run(
        &self,
        _: &VadProfile,
        model: &[u8],
        runtime: &[u8],
        samples: &[i16],
        _: &dyn AnalysisControl,
        scores: &mut dyn ScoreSink,
    ) -> Result<EngineCompletion, ApiError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(model, b"synthetic");
        assert_eq!(runtime, b"synthetic-runtime");
        assert_eq!(samples.len(), 513);
        scores.push(0.2)?;
        if self.partial {
            self.original.cancelled.store(true, Ordering::SeqCst);
            Ok(EngineCompletion::Partial {
                processed_samples: 512,
            })
        } else {
            scores.push(0.7)?;
            Ok(EngineCompletion::Complete)
        }
    }
}

fn exercise(partial: bool, cancel_before_run: bool) {
    let _unadmitted_constructor = crate::vad_job_attempts::assemble_attempt_executor;
    let fixture: VadReleaseTestFixture = release_file_fixture();
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
    let context: serde_json::Value = serde_json::from_str(admission.descriptor().raw()).unwrap();
    let raw = serde_json::to_vec(&serde_json::json!({"schema_version":"1","job_id":"roundtrip-fixture-only","attempt":"1",
        "input_sha256":"a".repeat(64),"source_blob_sha256":"b".repeat(64),"submission_request_sha256":null,
        "descriptor_sha256":admission.descriptor().decoded().raw().sha256(),"context":context})).unwrap();
    let original = Original {
        cancelled: AtomicBool::new(false),
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let original_deadline = original.deadline;
    let root = fixture.root.clone();
    let stage = root.join("roundtrip-stage");
    fs::create_dir(&stage).unwrap();
    fs::write(stage.join("stdin.raw"), &raw).unwrap();
    let executable = root.join("inert-runtime.fixture");
    fs::write(&executable, b"synthetic-runtime").unwrap();
    let producer = ReleasedWorkerPayload::new(
        fixture.inputs.clone(),
        admission.clone(),
        admission.descriptor().decoded().profile().runtime().clone(),
    )
    .unwrap();
    let staged = producer.stage(&raw, &stage, &original).unwrap();
    let decoded = decode_worker_request(
        raw.as_slice(),
        raw.len() as u64,
        admission.descriptor().raw().as_bytes(),
    )
    .unwrap();
    let reader = kinekino_vad_worker::staged_files::WorkerArtifactFiles::open(
        &stage,
        &executable,
        decoded.expected().descriptor(),
        decoded.expected().profile(),
    )
    .unwrap();
    drop(producer);
    drop(admission);
    // Original staging lease retains request/sidecars and source admission.
    assert!(fs::write(stage.join("stdin.raw"), b"changed").is_err());
    original
        .cancelled
        .store(cancel_before_run, Ordering::SeqCst);
    let engine = Engine {
        original: &original,
        partial,
        calls: AtomicUsize::new(0),
    };
    let response = kinekino_vad_worker::run::run_request(
        &decoded,
        fixture.admission.as_ref(),
        &reader,
        &engine,
        &original,
    )
    .unwrap();
    let outcome = kinekino_vad::protocol::application::decode_worker_outcome(
        response.as_slice(),
        response.len() as u64,
        decoded.expected(),
    )
    .unwrap();
    match (cancel_before_run, partial, outcome.result()) {
        (
            true,
            _,
            WorkerResult::Failure {
                code: ErrorCode::Cancelled,
                ..
            },
        ) => assert_eq!(engine.calls.load(Ordering::SeqCst), 0),
        (
            false,
            true,
            WorkerResult::Partial {
                processed_samples,
                scores,
            },
        ) => {
            assert_eq!(*processed_samples, 512);
            assert_eq!(scores, &[0.2]);
            assert!(original.cancelled.load(Ordering::SeqCst));
        }
        (false, false, WorkerResult::Complete { scores }) => assert_eq!(scores, &[0.2, 0.7]),
        _ => panic!("Parent/worker contract changed"),
    }
    assert_eq!(original.deadline, original_deadline);
    assert!(outcome.actual_runtime().is_none());
    drop(reader);
    drop(staged);
    drop(fixture);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn parent_sidecars_worker_reader_and_common_response_preserve_complete() {
    kinekino_platform::install();
    exercise(false, false);
}
#[test]
fn parent_sidecars_worker_partial_preserves_original_cancel_and_cutoff() {
    kinekino_platform::install();
    exercise(true, false);
}
#[test]
fn parent_sidecars_worker_pre_engine_cancel_remains_failure() {
    kinekino_platform::install();
    exercise(false, true);
}
