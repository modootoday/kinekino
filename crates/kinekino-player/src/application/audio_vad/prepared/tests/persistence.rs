use super::host_closeout::{close, conclude};
use super::*;
use crate::{
    application::jobs::{CancelJob, GetJob, JobExecutor, RetryJob, SubmitJob},
    infrastructure::jobs,
};
use kinekino_jobs::{
    application::{ExecutionControl, SourceBlobStore},
    domain::VerifiedJobOutput,
};
use std::{
    fs,
    os::windows::fs::MetadataExt,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

struct TestRoot {
    root: PathBuf,
    parent: PathBuf,
}
impl TestRoot {
    fn new() -> Self {
        let parent = std::env::temp_dir();
        assert!(parent.is_absolute());
        let root = parent.join(format!(
            "player-vad-envelope-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("store")).unwrap();
        Self { root, parent }
    }
    fn store(&self) -> PathBuf {
        self.root.join("store")
    }
}
impl Drop for TestRoot {
    fn drop(&mut self) {
        // Preserve failed-test evidence, and delete only our newly created,
        // ordinary directory immediately inside the intended temporary root.
        if !thread::panicking()
            && self.root.parent() == Some(self.parent.as_path())
            && self
                .root
                .file_name()
                .is_some_and(|s| s.to_string_lossy().starts_with("player-vad-envelope-"))
            && fs::symlink_metadata(&self.root)
                .is_ok_and(|m| m.is_dir() && m.file_attributes() & 0x400 == 0)
        {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

/// Test-only producer exercises actual JobService/JobFiles. No VAD engine,
/// external worker, owned session, or product registry is installed here.
struct PrepareOnly {
    calls: Arc<AtomicUsize>,
}
impl JobExecutor for PrepareOnly {
    fn supports(&self, operation: &str) -> bool {
        operation == VAD_JOB_OPERATION
    }
    fn operations(&self) -> Vec<String> {
        vec![VAD_JOB_OPERATION.into()]
    }
    fn normalize_parameters(&self, _: &str, parameters: Value) -> Result<Value, ApiError> {
        crate::application::audio_vad::normalize_parameters(parameters)
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        if !self.supports(operation) {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Test operation unavailable",
            ));
        }
        let request = VadSourceRequest::decode(parameters, expected)?;
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(PreparedJobInput {
            operation: operation.into(),
            parameters: request.prepared_parameters()?,
            source: blobs.put_utf8("synthetic prepared descriptor bytes")?,
            execution_timeout_ms: DecimalU64(30000),
        })
    }
    fn execute(
        &self,
        _: &PreparedJobInput,
        _: &dyn SourceBlobStore,
        _: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Test producer has no execution engine",
        ))
    }
}
#[test]
fn actual_job_store_reopen_idempotency_and_retry_keep_original_audio_pin_without_reprepare() {
    let fixture = TestRoot::new();
    eprintln!("vad_envelope_fixture_root={}", fixture.root.display());
    let calls = Arc::new(AtomicUsize::new(0));
    let adapter = Arc::new(PrepareOnly {
        calls: calls.clone(),
    });
    let host = jobs::start_retaining_host_with_executor(&fixture.store(), adapter.clone()).unwrap();
    let mut client = host.client();
    let mut host = Some(host);
    let mut close_failure = None;
    let action = catch_unwind(AssertUnwindSafe(|| {
        let submission = SubmitJob {
            operation: VAD_JOB_OPERATION.into(),
            parameters: parameters(),
            expected_source_sha256: Some("b".repeat(64)),
            execution_timeout_ms: DecimalU64(7000),
        };
        let first = client
            .submit(submission.clone(), "original-source-persist")
            .unwrap();
        assert!(first.created);
        assert_eq!(first.job.input.execution_timeout_ms, DecimalU64(7000));
        let id = first.job.job_id.clone();
        client
            .cancel(CancelJob {
                job_id: id.clone(),
                expected_attempt: DecimalU64(1),
            })
            .unwrap();
        if let Err(error) = close(host.take().unwrap()) {
            close_failure = Some(error);
            return;
        }
        let readonly = jobs::read_only(&fixture.store()).unwrap();
        let saved = readonly.get(GetJob { job_id: id.clone() }).unwrap();
        assert!(saved.state.terminal());
        assert!(!saved.publication_started);
        let restored = VadSourceRequest::from_prepared(&saved.input).unwrap();
        assert_eq!(restored.original_audio_sha256(), "b".repeat(64));
        assert_ne!(saved.input.source.sha256, restored.original_audio_sha256());
        assert_eq!(
            restored.parameters().expected_profile_manifest_sha256,
            "a".repeat(64)
        );
        drop(readonly);
        let restarted =
            jobs::start_retaining_host_with_executor(&fixture.store(), adapter.clone()).unwrap();
        client = restarted.client();
        host = Some(restarted);
        let duplicate = client
            .submit(submission.clone(), "original-source-persist")
            .unwrap();
        assert!(!duplicate.created);
        assert_eq!(duplicate.job.job_id, id);
        assert_eq!(duplicate.job.input_sha256, saved.input_sha256);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let mut changed = submission;
        changed.expected_source_sha256 = Some("d".repeat(64));
        assert_eq!(
            client
                .submit(changed, "original-source-persist")
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let retry = client
            .retry(RetryJob { job_id: id }, duplicate.job.revision.0)
            .unwrap();
        assert_eq!(retry.attempt, DecimalU64(2));
        assert_eq!(retry.input_sha256, saved.input_sha256);
        assert_eq!(
            VadSourceRequest::from_prepared(&retry.input)
                .unwrap()
                .original_audio_sha256(),
            "b".repeat(64)
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }));
    if let Some(original) = host.take()
        && let Err(error) = close(original)
    {
        close_failure = Some(error);
    }
    drop(client);
    drop(adapter);
    conclude(action, close_failure);
}
