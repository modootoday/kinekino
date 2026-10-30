//! Original worker + actual subtitle conversion with one explicit pre-publication fault.
use super::*;
use crate::application::{
    job_batches::{BatchAction, run},
    jobs::{JobState, JobsApi},
};
use kinekino_jobs::{
    application::{ExecutionControl, JobExecutor, SourceBlobStore},
    domain::{PreparedJobInput, VerifiedJobOutput},
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

struct FailOnce {
    original: Arc<dyn JobExecutor>,
    output: String,
    failed: AtomicBool,
    calls: Mutex<Vec<serde_json::Value>>,
}
impl JobExecutor for FailOnce {
    fn supports_batch_capture(&self, operation: &str) -> bool {
        self.original.supports_batch_capture(operation)
    }
    fn operations(&self) -> Vec<String> {
        self.original.operations()
    }
    fn supports(&self, operation: &str) -> bool {
        self.original.supports(operation)
    }
    fn normalize_parameters(
        &self,
        operation: &str,
        parameters: serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        self.original.normalize_parameters(operation, parameters)
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.original.execution_timeout_limit_ms(operation)
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: serde_json::Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        self.original
            .prepare(operation, parameters, expected, blobs)
    }
    fn execute(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        self.calls
            .lock()
            .unwrap()
            .push(json!({"parameters":input.parameters,"source_sha256":input.source.sha256}));
        if input.parameters["output_path"].as_str() == Some(self.output.as_str())
            && !self.failed.swap(true, Ordering::SeqCst)
        {
            let mut error = ApiError::new(
                ErrorCode::EngineFailed,
                "Explicit fixture failure before original subtitle publication",
            );
            error.details = json!({"fixture":"one_prepublication_fault","automatic_retry":false});
            return Err(error);
        }
        self.original.execute(input, blobs, control)
    }
}
fn settle(store: &BatchFiles, jobs: &dyn JobsApi, mut document: BatchDocument) -> BatchDocument {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        document = run(
            store,
            jobs,
            "partial-1",
            document.revision.0,
            BatchAction::Refresh,
        )
        .unwrap();
        if document
            .entries
            .iter()
            .all(|entry| entry.progress.job().is_some_and(|job| job.state.terminal()))
        {
            return document;
        }
        assert!(
            Instant::now() < deadline,
            "Original batch jobs did not settle"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
#[ignore = "Central original worker allocation with retained partial-failure/native journal evidence"]
fn original_worker_partial_failure_preserves_success_and_retries_only_failed_after_reopen() {
    let parent = PathBuf::from(
        std::env::var_os("BATCH_FILES_EVIDENCE_DIR").expect("Explicit evidence root"),
    );
    assert!(parent.is_absolute());
    let evidence = parent.join(format!("partial-worker-{}", std::process::id()));
    fs::create_dir(&evidence).unwrap();
    let jobs_root = evidence.join("jobs");
    fs::create_dir(&jobs_root).unwrap();
    let failed_output = evidence.join("retry.vtt");
    let executor = Arc::new(FailOnce {
        original: crate::infrastructure::jobs::subtitle_executor(),
        output: failed_output.to_str().unwrap().into(),
        failed: AtomicBool::new(false),
        calls: Mutex::new(vec![]),
    });
    let host = Host {
        original: Some(
            crate::infrastructure::jobs::start_retaining_host_with_executor(
                &jobs_root,
                executor.clone(),
            )
            .unwrap(),
        ),
        evidence: evidence.clone(),
    };
    let jobs = host.original.as_ref().unwrap().client();
    let store = BatchFiles::open(&jobs_root).unwrap();
    let mut inputs = Vec::new();
    for (item, output) in [
        ("success", evidence.join("success.vtt")),
        ("retry", failed_output.clone()),
    ] {
        let source = evidence.join(format!("{item}.srt"));
        fs::write(
            &source,
            format!("1\n00:00:01,000 --> 00:00:02,000\nActual {item} fixture\n\n"),
        )
        .unwrap();
        let captured = jobs.capture(SubmitJob { operation: "subtitles.convert".into(),
            parameters: json!({"input_path":source,"output_path":output,"input_format":"srt","output_format":"vtt"}),
            expected_source_sha256: None, execution_timeout_ms: DecimalU64(30000) }).unwrap();
        inputs.push(CapturedBatchInput::new(item.into(), captured).unwrap());
    }
    let initial =
        BatchDocument::from_captured(store.scope_sha256().into(), "partial-1".into(), inputs)
            .unwrap();
    store.create(&initial).unwrap();
    let submitted = run(&store, jobs.as_ref(), "partial-1", 1, BatchAction::Submit).unwrap();
    let first = settle(&store, jobs.as_ref(), submitted);
    fs::write(
        evidence.join("first-terminal.json"),
        serde_json::to_vec_pretty(&first).unwrap(),
    )
    .unwrap();
    let success = first.entries[0].progress.job().unwrap();
    let failed = first.entries[1].progress.job().unwrap();
    assert_eq!(success.state, JobState::Completed);
    assert_eq!(failed.state, JobState::Failed);
    assert!(!failed.publication_started);
    assert_eq!(
        failed.failure.as_ref().unwrap().details["fixture"],
        "one_prepublication_fault"
    );
    let success_record = serde_json::to_vec(success).unwrap();
    let success_id = success.job_id.clone();
    let failed_id = failed.job_id.clone();
    let success_bytes = fs::read(evidence.join("success.vtt")).unwrap();
    assert!(success_bytes.starts_with(b"WEBVTT"));
    assert!(!failed_output.exists());
    let first_bytes = serde_json::to_vec(&first).unwrap();
    drop(store);
    let store = BatchFiles::open(&jobs_root).unwrap();
    let reopened = store.acquire("partial-1").unwrap().load().unwrap();
    assert_eq!(serde_json::to_vec(&reopened).unwrap(), first_bytes);
    let retry = run(
        &store,
        jobs.as_ref(),
        "partial-1",
        reopened.revision.0,
        BatchAction::RetryFailed,
    )
    .unwrap();
    let complete = settle(&store, jobs.as_ref(), retry);
    fs::write(
        evidence.join("completed.json"),
        serde_json::to_vec_pretty(&complete).unwrap(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_vec(complete.entries[0].progress.job().unwrap()).unwrap(),
        success_record
    );
    assert_eq!(
        complete.entries[0].progress.job().unwrap().job_id,
        success_id
    );
    let retried = complete.entries[1].progress.job().unwrap();
    assert_eq!(retried.job_id, failed_id);
    assert_eq!(retried.state, JobState::Completed);
    assert_eq!(retried.attempt.0, 2);
    assert_eq!(
        fs::read(evidence.join("success.vtt")).unwrap(),
        success_bytes
    );
    let retried_bytes = fs::read(&failed_output).unwrap();
    assert!(retried_bytes.starts_with(b"WEBVTT"));
    assert_eq!(
        hash(&retried_bytes),
        retried.output.as_ref().unwrap().sha256
    );
    let calls = executor.calls.lock().unwrap();
    assert_eq!(calls.len(), 3);
    fs::write(
        evidence.join("execution-calls.json"),
        serde_json::to_vec_pretty(&*calls).unwrap(),
    )
    .unwrap();
    drop(calls);
    let before = serde_json::to_vec(&complete).unwrap();
    let same = run(
        &store,
        jobs.as_ref(),
        "partial-1",
        complete.revision.0,
        BatchAction::Submit,
    )
    .unwrap();
    assert_eq!(serde_json::to_vec(&same).unwrap(), before);
    assert_eq!(executor.calls.lock().unwrap().len(), 3);
    assert_eq!(
        jobs.list(ListJobs {
            cursor: None,
            limit: Some(10)
        })
        .unwrap()
        .jobs
        .len(),
        2
    );
    // Reopen after terminal success must preserve both immutable observations.
    drop(store);
    let store = BatchFiles::open(&jobs_root).unwrap();
    assert_eq!(
        serde_json::to_vec(&store.acquire("partial-1").unwrap().load().unwrap()).unwrap(),
        before
    );
}
