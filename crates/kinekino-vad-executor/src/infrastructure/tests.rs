use super::{
    candidate::{CompleteCandidate, PublicationState},
    driver::Driver,
};
use crate::application::{OutputPublication, RunControl, WorkerRead, WorkerTransport};
use kinekino_jobs::{
    application::owned_attempt::{OwnedOutputCandidate, OwnerObservation},
    domain::VerifiedJobOutput,
};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::domain::ArtifactPin;
use kinekino_vad::protocol::{
    application::decode_descriptor,
    domain::{ExpectedWorkerContext, WorkerResult},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    io::Cursor,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

fn pin(id: &str, bytes: &[u8]) -> ArtifactPin {
    ArtifactPin::new(
        id.into(),
        format!("{:x}", Sha256::digest(bytes)),
        DecimalU64(bytes.len() as u64),
    )
    .unwrap()
}
fn fixture() -> (ExpectedWorkerContext, Vec<u8>) {
    let p = |id: &str, b: &[u8]| {
        let p = pin(id, b);
        json!({"id":p.id(),"sha256":p.sha256(),"bytes":p.bytes().to_string()})
    };
    let descriptor = json!({"schema_version":"1","profile":{"id":"fixture-only","repo":"onnx-community/silero-vad","revision":"1".repeat(40),"precision":"fp32","provider":"CPUExecutionProvider","intra_threads":2,"inter_threads":1,"license_id":"MIT","weights":p("weights",b"fake"),"license_document":p("license",b"MIT"),"runtime":p("runtime",b"fake")},"input":{"wav":p("wav",b"fake"),"pcm":p("pcm",&[0;1026]),"sample_rate":16000,"channels":1,"bits_per_sample":16,"samples":"513","start_sample":"0","end_sample":"513"}});
    let bytes = serde_json::to_vec(&descriptor).unwrap();
    let context = ExpectedWorkerContext::new(
        pin("request", b"request"),
        decode_descriptor(bytes.as_slice(), &pin("descriptor", &bytes)).unwrap(),
    );
    let outcome = json!({"schema_version":"1","request_sha256":context.request().sha256(),"descriptor_sha256":context.descriptor().raw().sha256(),"context":descriptor,"kind":"complete","processed_samples":"513","scores":[0.1,0.9],"failure":null,"actual_runtime":null});
    (context, serde_json::to_vec(&outcome).unwrap())
}
struct Control {
    deadline: Instant,
    cancelled: bool,
}
impl Control {
    fn live() -> Self {
        Self {
            deadline: Instant::now() + Duration::from_secs(5),
            cancelled: false,
        }
    }
}
impl RunControl for Control {
    fn deadline(&self) -> Instant {
        self.deadline
    }
    fn cancellation_requested(&self) -> bool {
        self.cancelled
    }
}
struct FakeTransport {
    launches: Arc<AtomicUsize>,
    polls: Arc<AtomicUsize>,
    bytes: Vec<u8>,
    panic: bool,
    fail: bool,
}
impl WorkerTransport for FakeTransport {
    fn launch_read(
        &mut self,
        _request: &[u8],
        _control: &dyn RunControl,
    ) -> Result<WorkerRead, ApiError> {
        self.launches.fetch_add(1, Ordering::SeqCst);
        // Models an original partial owner kept in self before any failure.
        if self.panic {
            panic!("original partial launch panic")
        }
        if self.fail {
            return Err(ApiError::new(ErrorCode::EngineFailed, "partial launch"));
        }
        Ok(WorkerRead {
            declared_bytes: self.bytes.len() as u64,
            stdout: Box::new(Cursor::new(self.bytes.clone())),
        })
    }
    fn poll_close(&mut self, _deadline: Instant) -> Result<OwnerObservation, ApiError> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        Ok(OwnerObservation::Pending)
    }
}
fn driver(
    bytes: Vec<u8>,
    context: ExpectedWorkerContext,
    panic: bool,
    fail: bool,
) -> (Driver, Arc<AtomicUsize>, Arc<AtomicUsize>) {
    let launches = Arc::new(AtomicUsize::new(0));
    let polls = Arc::new(AtomicUsize::new(0));
    (
        Driver::new(
            Box::new(FakeTransport {
                launches: launches.clone(),
                polls: polls.clone(),
                bytes,
                panic,
                fail,
            }),
            b"request".to_vec(),
            context,
        ),
        launches,
        polls,
    )
}
#[test]
fn inert_driver_has_no_launch_until_run_and_keeps_raw_candidate() {
    let (context, bytes) = fixture();
    let (mut d, l, _) = driver(bytes, context, false, false);
    assert_eq!(l.load(Ordering::SeqCst), 0);
    d.run(&Control::live()).unwrap();
    assert_eq!(l.load(Ordering::SeqCst), 1);
    let wire = d.observation.as_ref().unwrap();
    assert!(matches!(wire.result(), WorkerResult::Complete { .. }));
    assert!(wire.raw().bytes() > 0);
    assert!(wire.actual_runtime().is_none());
    assert!(d.run(&Control::live()).is_err());
    assert_eq!(l.load(Ordering::SeqCst), 1);
}
#[test]
fn cancellation_and_expired_original_deadline_prevent_launch() {
    for c in [
        Control {
            cancelled: true,
            ..Control::live()
        },
        Control {
            deadline: Instant::now(),
            cancelled: false,
        },
    ] {
        let (context, bytes) = fixture();
        let (mut d, l, _) = driver(bytes, context, false, false);
        assert!(d.run(&c).is_err());
        assert_eq!(l.load(Ordering::SeqCst), 0);
    }
}
#[test]
fn partial_launch_failure_retains_original_transport_for_close() {
    let (context, bytes) = fixture();
    let (mut d, l, p) = driver(bytes, context, false, true);
    assert!(d.run(&Control::live()).is_err());
    assert_eq!(
        d.transport.poll_close(Instant::now()).unwrap(),
        OwnerObservation::Pending
    );
    assert_eq!(p.load(Ordering::SeqCst), 1);
    assert!(d.run(&Control::live()).is_err());
    assert_eq!(l.load(Ordering::SeqCst), 1);
}
#[test]
fn launch_unwind_leaves_same_parent_driver_and_owner() {
    let (context, bytes) = fixture();
    let (mut d, l, p) = driver(bytes, context, true, false);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| d.run(&Control::live()))).is_err()
    );
    assert_eq!(
        d.transport.poll_close(Instant::now()).unwrap(),
        OwnerObservation::Pending
    );
    assert_eq!(p.load(Ordering::SeqCst), 1);
    assert!(d.run(&Control::live()).is_err());
    assert_eq!(l.load(Ordering::SeqCst), 1);
}
#[test]
fn malformed_wire_does_not_release_original_transport() {
    let (context, _) = fixture();
    let (mut d, _, p) = driver(b"not-json".to_vec(), context, false, false);
    assert!(d.run(&Control::live()).is_err());
    assert!(d.observation.is_none());
    assert_eq!(
        d.transport.poll_close(Instant::now()).unwrap(),
        OwnerObservation::Pending
    );
    assert_eq!(p.load(Ordering::SeqCst), 1);
}
struct FailingPublication {
    calls: AtomicUsize,
}
impl OutputPublication for FailingPublication {
    fn publish(
        &self,
        _output: &kinekino_vad::protocol::domain::WorkerOutcomeCandidate,
        _deadline: Instant,
    ) -> Result<(), ApiError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(ApiError::new(
            ErrorCode::EngineFailed,
            "original publish failure",
        ))
    }
    fn verify(
        &self,
        _output: &kinekino_vad::protocol::domain::WorkerOutcomeCandidate,
        _deadline: Instant,
    ) -> Result<VerifiedJobOutput, ApiError> {
        panic!("verification must not follow failed publication")
    }
}
#[test]
fn failed_publication_cannot_retry_or_verify() {
    let (context, bytes) = fixture();
    let (mut d, _, _) = driver(bytes, context, false, false);
    d.run(&Control::live()).unwrap();
    let store = Arc::new(FailingPublication {
        calls: AtomicUsize::new(0),
    });
    let mut c = CompleteCandidate {
        wire: d.observation.take().unwrap(),
        publication: store.clone(),
        deadline: Control::live().deadline,
        state: PublicationState::Fresh,
    };
    assert!(c.verify().is_err());
    assert!(c.publish().is_err());
    assert!(c.verify().is_err());
    assert!(c.publish().is_err());
    assert_eq!(store.calls.load(Ordering::SeqCst), 1);
}
#[test]
fn partial_wire_keeps_raw_observation_and_never_returns_publishable_candidate() {
    let (context, bytes) = fixture();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["kind"] = json!("partial");
    value["processed_samples"] = json!("0");
    value["scores"] = json!([]);
    let (mut d, _, _) = driver(serde_json::to_vec(&value).unwrap(), context, false, false);
    d.run(&Control::live()).unwrap();
    let store = Arc::new(FailingPublication {
        calls: AtomicUsize::new(0),
    });
    let result = super::session::map_outcome(&mut d, store.clone(), Control::live().deadline);
    assert!(matches!(
        result,
        kinekino_jobs::application::owned_attempt::SessionRun::Failed(_)
    ));
    assert!(matches!(
        d.observation.as_ref().unwrap().result(),
        WorkerResult::Partial { .. }
    ));
    assert_eq!(store.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn worker_cancelled_label_is_failure_not_parent_cooperative_marker() {
    let (context, bytes) = fixture();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["kind"] = json!("failure");
    value["processed_samples"] = serde_json::Value::Null;
    value["scores"] = json!([]);
    value["failure"] = json!({"code":"cancelled","message":"worker label"});
    let (mut d, _, _) = driver(serde_json::to_vec(&value).unwrap(), context, false, false);
    d.run(&Control::live()).unwrap();
    let store = Arc::new(FailingPublication {
        calls: AtomicUsize::new(0),
    });
    assert!(matches!(
        super::session::map_outcome(&mut d, store.clone(), Control::live().deadline),
        kinekino_jobs::application::owned_attempt::SessionRun::Failed(_)
    ));
    assert!(d.observation.is_some());
    assert_eq!(store.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn concrete_output_keeps_raw_context_and_absent_runtime_without_admission() {
    let (context, bytes) = fixture();
    let (mut d, _, _) = driver(bytes, context, false, false);
    d.run(&Control::live()).unwrap();
    let encoded = super::output::encode(d.observation.as_ref().unwrap()).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(
        value["worker_outcome_sha256"],
        d.observation.as_ref().unwrap().raw().sha256()
    );
    assert!(value["actual_runtime"].is_null());
    assert_eq!(value["runtime_admission_verified"], false);
    assert_eq!(value["scores"].as_array().unwrap().len(), 2);
}

#[test]
fn concrete_output_encoder_rejects_partial_before_storage() {
    let (context, bytes) = fixture();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["kind"] = json!("partial");
    value["processed_samples"] = json!("0");
    value["scores"] = json!([]);
    let (mut d, _, _) = driver(serde_json::to_vec(&value).unwrap(), context, false, false);
    d.run(&Control::live()).unwrap();
    assert!(super::output::encode(d.observation.as_ref().unwrap()).is_err());
}

#[test]
fn cooperative_partial_requires_both_parent_cancel_and_original_signal() {
    let (context, bytes) = fixture();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["kind"] = json!("partial");
    value["processed_samples"] = json!("0");
    value["scores"] = json!([]);
    let (mut d, _, _) = driver(serde_json::to_vec(&value).unwrap(), context, false, false);
    d.run(&Control::live()).unwrap();
    assert!(!super::session::cooperative_partial(
        d.observation.as_ref(),
        false,
        true
    ));
    assert!(!super::session::cooperative_partial(
        d.observation.as_ref(),
        true,
        false
    ));
    assert!(super::session::cooperative_partial(
        d.observation.as_ref(),
        true,
        true
    ));
    assert!(!super::session::cooperative_partial(None, true, true));
}

struct TestOutputFile(String);
impl super::output::GuardedOutputFile for TestOutputFile {
    fn path(&self) -> &str {
        &self.0
    }
    fn open_actual(&self) -> Result<std::fs::File, ApiError> {
        std::fs::File::open(&self.0).map_err(kinekino_core::file_publication::io_error)
    }
}
fn output_path() -> std::path::PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "vad-output-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir(&dir).unwrap();
    dir.join("result.json")
}
#[test]
fn concrete_publication_verifies_actual_file_and_rejects_changed_bytes() {
    let (context, bytes) = fixture();
    let (mut d, _, _) = driver(bytes, context, false, false);
    d.run(&Control::live()).unwrap();
    let wire = d.observation.as_ref().unwrap();
    let path = output_path();
    // This simple port tests publication/verification only. Actual ordinary-file
    // and directory lease guards are exercised by product vad_job_files tests.
    let output = super::output::JsonOutputPublication::new(Arc::new(TestOutputFile(
        path.to_str().unwrap().into(),
    )));
    let deadline = Control::live().deadline;
    output.publish(wire, deadline).unwrap();
    let verified = output.verify(wire, deadline).unwrap();
    let actual = std::fs::read(&path).unwrap();
    assert_eq!(verified.sha256, format!("{:x}", Sha256::digest(&actual)));
    assert_eq!(verified.bytes.0, actual.len() as u64);
    assert!(output.publish(wire, deadline).is_err());
    std::fs::write(&path, b"changed").unwrap();
    assert!(output.verify(wire, deadline).is_err());
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(path.parent().unwrap()).unwrap();
}
#[test]
fn expired_publication_preserves_absence_of_final_output() {
    let (context, bytes) = fixture();
    let (mut d, _, _) = driver(bytes, context, false, false);
    d.run(&Control::live()).unwrap();
    let path = output_path();
    let output = super::output::JsonOutputPublication::new(Arc::new(TestOutputFile(
        path.to_str().unwrap().into(),
    )));
    assert!(
        output
            .publish(d.observation.as_ref().unwrap(), Instant::now())
            .is_err()
    );
    assert!(!path.exists());
    std::fs::remove_dir(path.parent().unwrap()).unwrap();
}
