//! Actual original host capture/submission; synthetic adapter refusal is separate.
use super::*;
use crate::{
    application::{
        job_batches::{BatchDocument, CapturedBatchInput, validate_initial},
        jobs::{GetJob, ListJobs, SubmitJob},
    },
    domain::api_values::DecimalU64,
};
use kinekino_jobs::{
    application::{ExecutionControl, ExecutorRegistry, SourceBlobStore},
    domain::{JobState, PreparedJobInput, VerifiedJobOutput},
    infrastructure::WorkerShutdownOutcome,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Host {
    original: Option<RetainingWorkerHost>,
    evidence: PathBuf,
}
impl Host {
    fn new(case: &str, executor: Arc<dyn JobExecutor>) -> Self {
        let parent = PathBuf::from(
            std::env::var_os("BATCH_CAPTURE_EVIDENCE_DIR").expect("Explicit new evidence root"),
        );
        assert!(parent.is_absolute());
        let evidence = parent.join(format!("capture-{case}-{}", std::process::id()));
        fs::create_dir(&evidence).unwrap();
        let root = evidence.join("jobs");
        fs::create_dir(&root).unwrap();
        Self {
            original: Some(start_retaining_host_with_executor(&root, executor).unwrap()),
            evidence,
        }
    }
    fn client(&self) -> Arc<dyn JobsApi> {
        self.original.as_ref().unwrap().client()
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        let Some(original) = self.original.take() else {
            return;
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut pending = original.begin_shutdown(deadline);
        loop {
            match pending.poll(deadline) {
                WorkerShutdownOutcome::Complete(done) => {
                    fs::write(self.evidence.join("shutdown.json"),serde_json::to_vec_pretty(&json!({"receipt":done.receipt,"error":done.errors.first(),"secondary_count":done.errors.secondary().len()})).unwrap()).unwrap();
                    if done.errors.first().is_some() {
                        std::process::exit(101);
                    }
                    break;
                }
                WorkerShutdownOutcome::Pending(owned) => {
                    pending = owned;
                    if Instant::now() >= deadline {
                        eprintln!("Original capture host remains pending");
                        std::process::exit(101);
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }
}
fn input(root: &Path) -> SubmitJob {
    SubmitJob {
        operation: "subtitles.convert".into(),
        parameters: json!({"input_path":root.join("source.srt"),"output_path":root.join("result.vtt"),"input_format":"srt","output_format":"vtt"}),
        expected_source_sha256: None,
        execution_timeout_ms: DecimalU64(30000),
    }
}
fn count(client: &dyn JobsApi) -> usize {
    client
        .list(ListJobs {
            cursor: None,
            limit: Some(50),
        })
        .unwrap()
        .jobs
        .len()
}

#[test]
#[ignore = "Central original-host allocation and retained generated subtitle evidence"]
fn actual_subtitle_capture_pins_source_without_admission_then_submits_and_replays() {
    let registry = ExecutorRegistry::new(vec![subtitle_executor()]).unwrap();
    let host = Host::new("subtitle", Arc::new(registry));
    let client = host.client();
    let source = b"1\n00:00:01,000 --> 00:00:02,000\nCaptured subtitle\n\n";
    fs::write(host.evidence.join("source.srt"), source).unwrap();
    fs::write(host.evidence.join("source-original.srt"), source).unwrap();
    let request = input(&host.evidence);
    let mut secret = request.clone();
    secret.parameters["credential"] = json!("fixture-must-not-persist");
    assert_eq!(
        client.capture(secret).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut remote = request.clone();
    remote.parameters["input_path"] =
        json!("https://invalid.test/source?token=fixture-must-not-persist");
    assert!(client.capture(remote).is_err());
    assert_eq!(count(client.as_ref()), 0);
    assert_eq!(
        fs::read_dir(host.evidence.join("jobs/job-sources"))
            .unwrap()
            .count(),
        0
    );
    let captured = client.capture(request.clone()).unwrap();
    assert_eq!(count(client.as_ref()), 0);
    assert!(!host.evidence.join("result.vtt").exists());
    let source_sha = format!("{:x}", Sha256::digest(source));
    assert_eq!(
        captured.request().expected_source_sha256.as_deref(),
        Some(source_sha.as_str())
    );
    assert_eq!(captured.request().parameters["allow_loss"], false);
    let again = client.capture(request).unwrap();
    assert_eq!(
        serde_json::to_value(captured.request()).unwrap(),
        serde_json::to_value(again.request()).unwrap()
    );
    assert_eq!(
        fs::read(
            host.evidence
                .join("jobs/job-sources")
                .join(format!("{source_sha}.utf8"))
        )
        .unwrap(),
        source
    );
    let document = BatchDocument::from_captured(
        "a".repeat(64),
        "capture-fixture".into(),
        vec![CapturedBatchInput::new("first".into(), captured.clone()).unwrap()],
    )
    .unwrap();
    validate_initial(&document, &"a".repeat(64)).unwrap();
    fs::write(
        host.evidence.join("typed-captured-document.json"),
        serde_json::to_vec_pretty(&document).unwrap(),
    )
    .unwrap();
    let normalized = captured.into_request();
    let changed = b"1\n00:00:01,000 --> 00:00:02,000\nChanged fixture\n\n";
    fs::write(host.evidence.join("source-changed.srt"), changed).unwrap();
    fs::write(host.evidence.join("source.srt"), changed).unwrap();
    assert_eq!(
        client
            .submit(normalized.clone(), "captured-first")
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
    assert_eq!(count(client.as_ref()), 0);
    assert!(!host.evidence.join("result.vtt").exists());
    fs::write(host.evidence.join("source.srt"), source).unwrap();
    let submitted = client.submit(normalized.clone(), "captured-first").unwrap();
    assert!(submitted.created);
    let deadline = Instant::now() + Duration::from_secs(20);
    let done = loop {
        let job = client
            .get(GetJob {
                job_id: submitted.job.job_id.clone(),
            })
            .unwrap();
        if job.state.terminal() {
            break job;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    };
    fs::write(
        host.evidence.join("completed.json"),
        serde_json::to_vec_pretty(&done).unwrap(),
    )
    .unwrap();
    assert_eq!(done.state, JobState::Completed, "{:?}", done.failure);
    let bytes = fs::read(host.evidence.join("result.vtt")).unwrap();
    assert!(bytes.starts_with(b"WEBVTT"));
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        done.output.as_ref().unwrap().sha256
    );
    fs::write(host.evidence.join("source.srt"), changed).unwrap();
    let replay = client.submit(normalized.clone(), "captured-first").unwrap();
    assert!(!replay.created);
    assert_eq!(replay.job.job_id, done.job_id);
    assert_eq!(replay.job.attempt, DecimalU64(1));
    assert_eq!(count(client.as_ref()), 1);
    assert_eq!(
        client.capture(normalized.clone()).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    let root = host.evidence.join("jobs");
    let evidence = host.evidence.clone();
    drop(client);
    drop(host);
    assert_eq!(
        read_only(&root)
            .unwrap()
            .capture(normalized)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    let stopped: Value =
        serde_json::from_slice(&fs::read(evidence.join("shutdown.json")).unwrap()).unwrap();
    assert_eq!(stopped["receipt"]["completed_total"], 1);
    assert!(stopped["error"].is_null());
}

struct CaptureAdapter {
    admitted: bool,
    normalizations: AtomicUsize,
}
impl JobExecutor for CaptureAdapter {
    fn operations(&self) -> Vec<String> {
        vec!["fixture.capture".into()]
    }
    fn supports(&self, operation: &str) -> bool {
        operation == "fixture.capture"
    }
    fn supports_batch_capture(&self, _: &str) -> bool {
        self.admitted
    }
    fn normalize_parameters(&self, _: &str, mut parameters: Value) -> Result<Value, ApiError> {
        parameters["generation"] = json!(self.normalizations.fetch_add(1, Ordering::SeqCst));
        Ok(parameters)
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        _: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        assert!(self.admitted);
        Ok(PreparedJobInput {
            operation: operation.into(),
            parameters,
            source: blobs.put_utf8("synthetic capture source")?,
            execution_timeout_ms: DecimalU64(30000),
        })
    }
    fn execute(
        &self,
        _: &PreparedJobInput,
        _: &dyn SourceBlobStore,
        _: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        panic!("Capture never executes this synthetic adapter")
    }
}
#[test]
#[ignore = "Central original-host allocation for unsupported and non-idempotent capture refusal"]
fn original_host_refuses_unadmitted_and_non_idempotent_capture_without_queuing() {
    for admitted in [false, true] {
        let host = Host::new(
            if admitted { "changing" } else { "unadmitted" },
            Arc::new(CaptureAdapter {
                admitted,
                normalizations: AtomicUsize::new(0),
            }),
        );
        let client = host.client();
        let request = SubmitJob {
            operation: "fixture.capture".into(),
            parameters: json!({}),
            expected_source_sha256: None,
            execution_timeout_ms: DecimalU64(30000),
        };
        assert_eq!(
            client.capture(request).unwrap_err().code,
            ErrorCode::Unsupported
        );
        assert_eq!(count(client.as_ref()), 0);
        let evidence = host.evidence.clone();
        drop(client);
        drop(host);
        let stopped: Value =
            serde_json::from_slice(&fs::read(evidence.join("shutdown.json")).unwrap()).unwrap();
        assert_eq!(stopped["receipt"]["completed_total"], 0);
        assert!(stopped["error"].is_null());
    }
}
