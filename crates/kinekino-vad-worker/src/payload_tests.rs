#[path = "../../kinekino-vad/tests/support/mod.rs"]
#[allow(dead_code)]
mod support;
use crate::payload::{PreparedWorkerPayload, WorkerPayloadPorts};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::protocol::application::{ValidatedWorkerRequest, decode_worker_request};
use kinekino_vad::{
    application::{AnalysisControl, ArtifactReader, ProfileAdmission},
    domain::{ArtifactPin, VadProfile},
};
use std::{
    cell::Cell,
    io::{Cursor, Read},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

#[path = "payload_tests/control.rs"]
mod control;
#[path = "payload_tests/entry.rs"]
mod entry;
#[path = "payload_tests/production.rs"]
mod production;

struct Control {
    cancel: Cell<bool>,
    deadline: Instant,
}
impl Control {
    fn new() -> Self {
        Self {
            cancel: Cell::new(false),
            deadline: Instant::now() + Duration::from_secs(10),
        }
    }
}
impl AnalysisControl for Control {
    fn cancellation_requested(&self) -> bool {
        self.cancel.get()
    }
    fn deadline(&self) -> Instant {
        self.deadline
    }
}
struct Exact(VadProfile);
impl ProfileAdmission for Exact {
    fn authorize(&self, profile: &VadProfile) -> Result<(), ApiError> {
        if profile == &self.0 {
            Ok(())
        } else {
            Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Synthetic approved release differs",
            ))
        }
    }
}
struct Lease(Arc<AtomicUsize>);
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
struct Source<'a> {
    files: Vec<(ArtifactPin, Vec<u8>)>,
    opens: Cell<usize>,
    eof_cancel: Option<&'a Cell<bool>>,
}
struct EofCancel<'a> {
    cursor: Cursor<&'a [u8]>,
    flag: &'a Cell<bool>,
}
impl Read for EofCancel<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = self.cursor.read(buffer)?;
        if count == 0 {
            self.flag.set(true);
        }
        Ok(count)
    }
}
impl ArtifactReader for Source<'_> {
    fn open(&self, pin: &ArtifactPin) -> Result<Box<dyn Read + '_>, ApiError> {
        self.opens.set(self.opens.get() + 1);
        let (_, bytes) = self
            .files
            .iter()
            .find(|(key, _)| key == pin)
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Synthetic artifact absent"))?;
        let cursor = Cursor::new(bytes.as_slice());
        match self.eof_cancel {
            Some(flag) => Ok(Box::new(EofCancel { cursor, flag })),
            None => Ok(Box::new(cursor)),
        }
    }
}
fn fixture<'a>() -> (ValidatedWorkerRequest, Vec<u8>, Source<'a>) {
    fixture_for(
        support::descriptor(),
        b"descriptor-only-not-a-real-WAV".to_vec(),
    )
}
fn fixture_for<'a>(
    context: serde_json::Value,
    wav_bytes: Vec<u8>,
) -> (ValidatedWorkerRequest, Vec<u8>, Source<'a>) {
    let original = support::bytes(&context);
    let request = serde_json::json!({"schema_version":"1","job_id":"parent-label","attempt":"1",
        "input_sha256":"a".repeat(64),"source_blob_sha256":"b".repeat(64),
        "submission_request_sha256":null,"descriptor_sha256":support::pin("descriptor",&original).sha256(),
        "context":context});
    let raw = support::bytes(&request);
    let request = decode_worker_request(raw.as_slice(), raw.len() as u64, &original).unwrap();
    let p = request.expected().profile();
    let source = Source {
        files: vec![
            (p.weights().clone(), b"synthetic-model".to_vec()),
            (p.license_document().clone(), b"synthetic-license".to_vec()),
            (request.expected().input().wav().clone(), wav_bytes),
        ],
        opens: Cell::new(0),
        eof_cancel: None,
    };
    (request, original, source)
}

fn inference_fixture<'a>() -> (ValidatedWorkerRequest, Vec<u8>, Source<'a>) {
    let mut context = support::descriptor();
    let mut wav = b"RIFF".to_vec();
    wav.extend_from_slice(&(36_u32 + 1026).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt \x10\0\0\0\x01\0\x01\0\x80\x3e\0\0\0\x7d\0\0\x02\0\x10\0data");
    wav.extend_from_slice(&1026_u32.to_le_bytes());
    wav.resize(44 + 1026, 0);
    let pin = support::pin("wav", &wav);
    context["input"]["wav"] =
        serde_json::json!({"id":pin.id(),"sha256":pin.sha256(),"bytes":pin.bytes().to_string()});
    let (request, original, mut source) = fixture_for(context, wav);
    source.files.push((
        request.expected().profile().runtime().clone(),
        b"synthetic-runtime".to_vec(),
    ));
    (request, original, source)
}

struct CooperativeEngine<'a> {
    original: &'a Control,
    calls: Cell<usize>,
}
impl kinekino_vad::application::VadEngine for CooperativeEngine<'_> {
    fn run(
        &self,
        _: &VadProfile,
        model: &[u8],
        runtime: &[u8],
        samples: &[i16],
        _: &dyn AnalysisControl,
        scores: &mut dyn kinekino_vad::application::ScoreSink,
    ) -> Result<kinekino_vad::application::EngineCompletion, ApiError> {
        self.calls.set(self.calls.get() + 1);
        assert_eq!(model, b"synthetic-model");
        assert_eq!(runtime, b"synthetic-runtime");
        assert_eq!(samples.len(), 513);
        scores.push(0.2)?;
        self.original.cancel.set(true);
        Ok(kinekino_vad::application::EngineCompletion::Partial {
            processed_samples: 512,
        })
    }
}
#[test]
fn worker_runner_encodes_real_partial_without_resetting_original_control() {
    let (request, _, source) = inference_fixture();
    let control = Control::new();
    let engine = CooperativeEngine {
        original: &control,
        calls: Cell::new(0),
    };
    let raw = crate::run::run_request(
        &request,
        &Exact(request.expected().profile().clone()),
        &source,
        &engine,
        &control,
    )
    .unwrap();
    let decoded = kinekino_vad::protocol::application::decode_worker_outcome(
        raw.as_slice(),
        raw.len() as u64,
        request.expected(),
    )
    .unwrap();
    match decoded.result() {
        kinekino_vad::protocol::domain::WorkerResult::Partial {
            processed_samples,
            scores,
        } => {
            assert_eq!(*processed_samples, 512);
            assert_eq!(scores, &[0.2]);
        }
        _ => panic!("Worker observation became completion/error"),
    }
    assert_eq!(engine.calls.get(), 1);
    assert!(control.cancel.get());
    assert!(decoded.actual_runtime().is_none());
}
#[test]
fn worker_runner_pre_engine_cancel_encodes_failure_not_partial_zero() {
    let (request, _, source) = inference_fixture();
    let control = Control::new();
    control.cancel.set(true);
    let engine = CooperativeEngine {
        original: &control,
        calls: Cell::new(0),
    };
    let raw = crate::run::run_request(
        &request,
        &Exact(request.expected().profile().clone()),
        &source,
        &engine,
        &control,
    )
    .unwrap();
    let decoded = kinekino_vad::protocol::application::decode_worker_outcome(
        raw.as_slice(),
        raw.len() as u64,
        request.expected(),
    )
    .unwrap();
    assert!(matches!(
        decoded.result(),
        kinekino_vad::protocol::domain::WorkerResult::Failure {
            code: ErrorCode::Cancelled,
            ..
        }
    ));
    assert_eq!(engine.calls.get(), 0);
    assert_eq!(source.opens.get(), 0);
    assert!(control.cancel.get());
}
fn prepare(
    request: &ValidatedWorkerRequest,
    original: &[u8],
    source: &dyn ArtifactReader,
    control: &dyn AnalysisControl,
    lease: &Arc<Lease>,
    worker: &ArtifactPin,
) -> Result<PreparedWorkerPayload<Lease>, ApiError> {
    let profile = request.expected().profile();
    PreparedWorkerPayload::prepare(
        request,
        original,
        profile,
        worker,
        WorkerPayloadPorts {
            admission: &Exact(profile.clone()),
            reader: source,
            control,
        },
        lease,
    )
}
#[test]
fn prepared_payload_reads_three_roles_and_retains_original_owner() {
    let (request, original, source) = fixture();
    let dropped = Arc::new(AtomicUsize::new(0));
    let lease = Arc::new(Lease(dropped.clone()));
    let payload = prepare(
        &request,
        &original,
        &source,
        &Control::new(),
        &lease,
        request.expected().profile().runtime(),
    )
    .unwrap();
    assert_eq!(source.opens.get(), 3); // executable stays at its native guard, not a copied payload
    drop(lease);
    assert_eq!(dropped.load(Ordering::SeqCst), 0);
    drop(payload);
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
}
#[test]
fn worker_mismatch_fails_before_source_read_and_keeps_owner() {
    let (request, original, source) = fixture();
    let lease = Arc::new(Lease(Arc::new(AtomicUsize::new(0))));
    let wrong = support::pin("other-worker", b"different-worker");
    let error = prepare(
        &request,
        &original,
        &source,
        &Control::new(),
        &lease,
        &wrong,
    )
    .err()
    .unwrap();
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    assert_eq!(source.opens.get(), 0);
    assert_eq!(Arc::strong_count(&lease), 1);
}
#[test]
fn payload_rejects_changed_source_bytes() {
    let (request, original, mut source) = fixture();
    source.files[0].1[0] ^= 1;
    let lease = Arc::new(Lease(Arc::new(AtomicUsize::new(0))));
    let error = prepare(
        &request,
        &original,
        &source,
        &Control::new(),
        &lease,
        request.expected().profile().runtime(),
    )
    .err()
    .unwrap();
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert_eq!(source.opens.get(), 1);
    assert_eq!(Arc::strong_count(&lease), 1);
}
#[test]
fn payload_cancel_at_original_eof_does_not_become_prepared_or_ack() {
    let control = Control::new();
    let (request, original, mut source) = fixture();
    source.eof_cancel = Some(&control.cancel);
    let lease = Arc::new(Lease(Arc::new(AtomicUsize::new(0))));
    let error = prepare(
        &request,
        &original,
        &source,
        &control,
        &lease,
        request.expected().profile().runtime(),
    )
    .err()
    .unwrap();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(Arc::strong_count(&lease), 1);
}
#[test]
fn already_expired_payload_rejects_before_read() {
    let (request, original, source) = fixture();
    let control = Control {
        cancel: Cell::new(false),
        deadline: Instant::now(),
    };
    let lease = Arc::new(Lease(Arc::new(AtomicUsize::new(0))));
    let error = prepare(
        &request,
        &original,
        &source,
        &control,
        &lease,
        request.expected().profile().runtime(),
    )
    .err()
    .unwrap();
    assert_eq!(error.code, ErrorCode::Timeout);
    assert_eq!(source.opens.get(), 0);
}

mod ordinary_files {
    use super::*;
    use crate::staged_files::WorkerArtifactFiles;
    use kinekino_vad::application::read_verified_artifact;
    use std::{fs, path::PathBuf};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    fn root() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "vad-worker-payload-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn staged_files_and_original_owner_outlive_prepared_payload() {
        let (request, original, source) = fixture();
        let dropped = Arc::new(AtomicUsize::new(0));
        let lease = Arc::new(Lease(dropped.clone()));
        let payload = prepare(
            &request,
            &original,
            &source,
            &Control::new(),
            &lease,
            request.expected().profile().runtime(),
        )
        .unwrap();
        let path = root();
        let staged = payload.stage(&path, &Control::new()).unwrap();
        drop(payload);
        drop(lease);
        assert_eq!(dropped.load(Ordering::SeqCst), 0);
        crate::infrastructure::os_policy::guarded_mutation(fs::write(
            path.join("model.onnx"),
            b"replace",
        ));
        assert_eq!(fs::read(path.join("descriptor.json")).unwrap(), original);
        drop(staged);
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
        fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn existing_sidecar_is_preserved_and_failure_keeps_payload_owner() {
        let (request, original, source) = fixture();
        let lease = Arc::new(Lease(Arc::new(AtomicUsize::new(0))));
        let payload = prepare(
            &request,
            &original,
            &source,
            &Control::new(),
            &lease,
            request.expected().profile().runtime(),
        )
        .unwrap();
        let path = root();
        fs::write(path.join("descriptor.json"), b"original evidence").unwrap();
        assert!(payload.stage(&path, &Control::new()).is_err());
        assert_eq!(
            fs::read(path.join("descriptor.json")).unwrap(),
            b"original evidence"
        );
        assert_eq!(Arc::strong_count(&lease), 2);
        drop(payload);
        drop(lease);
        fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn fixed_role_reader_has_independent_cursors_and_actual_executable_pin() {
        let (request, original, source) = fixture();
        let lease = Arc::new(Lease(Arc::new(AtomicUsize::new(0))));
        let control = Control::new();
        let payload = prepare(
            &request,
            &original,
            &source,
            &control,
            &lease,
            request.expected().profile().runtime(),
        )
        .unwrap();
        let path = root();
        let executable = path.join("synthetic-worker.bin");
        fs::write(&executable, b"synthetic-runtime").unwrap();
        let staged = payload.stage(&path, &control).unwrap();
        let reader = WorkerArtifactFiles::open(
            &path,
            &executable,
            request.expected().descriptor(),
            request.expected().profile(),
        )
        .unwrap();
        for _ in 0..2 {
            assert_eq!(
                read_verified_artifact(
                    &reader,
                    request.expected().profile().runtime(),
                    1024,
                    &control
                )
                .unwrap(),
                b"synthetic-runtime"
            );
        }
        crate::infrastructure::os_policy::guarded_mutation(fs::write(&executable, b"changed"));
        assert!(reader.open(&support::pin("unknown", b"unknown")).is_err());
        drop(reader);
        drop(staged);
        drop(payload);
        drop(lease);
        fs::remove_dir_all(path).unwrap();
    }
}
