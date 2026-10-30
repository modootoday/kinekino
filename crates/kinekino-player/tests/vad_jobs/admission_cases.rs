//! Ordinary-file original-scope scenarios. A caller supplies Media's existing
//! released-file fixture; this module does not manufacture model/manifest data.
use crate::host_closeout::{close, conclude};
use crate::{vad_job_executor::VadJobExecutor, vad_release_fixture::VadReleaseTestFixture};
use kinekino_jobs::domain::{JobRecord, JobState};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_player::{
    application::{
        audio_vad::VadSourceRequest,
        jobs::{CancelJob, GetJob, JobsApi, ListJobs, RetryJob, SubmitJob},
    },
    infrastructure::jobs,
};
use kinekino_vad::{application::ProfileAdmission, domain::VadProfile};
use serde_json::json;
use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub(super) enum Failure {
    MidAdmissionCancel,
    ChangedOriginalSource,
    OriginalDeadline,
}
struct AdmissionHook {
    original: Arc<dyn ProfileAdmission + Send + Sync>,
    calls: AtomicUsize,
    client: Mutex<Option<Weak<dyn JobsApi>>>,
    source: PathBuf,
    failure: Failure,
}
impl ProfileAdmission for AdmissionHook {
    fn authorize(&self, profile: &VadProfile) -> Result<(), ApiError> {
        self.original.authorize(profile)?;
        if self.calls.fetch_add(1, Ordering::SeqCst) == 1 {
            match self.failure {
                Failure::MidAdmissionCancel => {
                    let client = self
                        .client
                        .lock()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .upgrade()
                        .unwrap();
                    let jobs = client.list(ListJobs {
                        cursor: None,
                        limit: Some(128),
                    })?;
                    let running: Vec<_> = jobs
                        .jobs
                        .into_iter()
                        .filter(|job| job.state == JobState::Running)
                        .collect();
                    assert_eq!(running.len(), 1);
                    client.cancel(CancelJob {
                        job_id: running[0].job_id.clone(),
                        expected_attempt: running[0].attempt,
                    })?;
                }
                Failure::ChangedOriginalSource => {
                    fs::write(
                        &self.source,
                        b"changed after submit, before original audio re-admission",
                    )
                    .map_err(kinekino_core::file_publication::io_error)?;
                }
                Failure::OriginalDeadline => thread::sleep(Duration::from_millis(20)),
            }
        }
        Ok(())
    }
}

fn terminal(client: &dyn JobsApi, id: &kinekino_jobs::domain::JobId) -> JobRecord {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let job = client.get(GetJob { job_id: id.clone() }).unwrap();
        if job.state.terminal() {
            return job;
        }
        assert!(
            Instant::now() < deadline,
            "Original test job observation expired"
        );
        thread::sleep(Duration::from_millis(2));
    }
}
pub(super) fn exercise(fixture: VadReleaseTestFixture, failure: Failure) {
    let root = fixture.root;
    eprintln!("vad_job_fixture_root={}", root.display());
    let request = fixture.request;
    let blobs = fixture.blobs;
    let expected = match failure {
        Failure::MidAdmissionCancel => ErrorCode::Cancelled,
        Failure::ChangedOriginalSource => ErrorCode::SourceExpired,
        Failure::OriginalDeadline => ErrorCode::Timeout,
    };
    let hook = Arc::new(AdmissionHook {
        original: fixture.admission,
        calls: AtomicUsize::new(0),
        client: Mutex::new(None),
        source: root.join("source.bin"),
        failure,
    });
    let evidence = fixture.execution.evidence_root.clone();
    assert!(evidence.is_dir());
    assert_eq!(fs::read_dir(&evidence).unwrap().count(), 0);
    let unavailable_worker = fixture.execution.worker.clone();
    assert!(!unavailable_worker.exists());
    let executor = Arc::new(VadJobExecutor::new(
        fixture.inputs,
        hook.clone(),
        fixture.execution,
    ));
    let host =
        jobs::start_retaining_host_with_executor(&root.join("jobs"), executor.clone()).unwrap();
    let mut client = host.client();
    let mut host = Some(host);
    let mut close_failure = None;
    let action = catch_unwind(AssertUnwindSafe(|| {
        *hook.client.lock().unwrap() = Some(Arc::downgrade(&client));
        let timeout = if expected == ErrorCode::Timeout {
            1
        } else {
            30000
        };
        let submission = SubmitJob {
            operation: "audio.vad".into(),
            parameters: json!({"profile_id":fixture.public_profile_id,"expected_profile_manifest_sha256":request.profile_manifest.as_str(),
            "audio_artifact_id":fixture.public_audio_id,"start_sample":request.start,"end_sample":request.end,"destination_id":fixture.public_destination_id}),
            expected_source_sha256: Some(request.audio_source.as_str().into()),
            execution_timeout_ms: DecimalU64(timeout),
        };
        let admitted = client
            .submit(submission.clone(), "original-scope-test")
            .unwrap();
        assert!(admitted.created);
        let finished = terminal(client.as_ref(), &admitted.job.job_id);
        eprintln!(
            "vad_job_scope_observation={}",
            json!({
                "job_id":finished.job_id.as_str(),"attempt":finished.attempt,
                "state":finished.state,"failure":finished.failure.as_ref().map(|value|value.code),
                "authorize_calls":hook.calls.load(Ordering::SeqCst),
                "execution_timeout_ms":finished.input.execution_timeout_ms,
                "phase_meaning":"counter observation; expiry may precede second admission"
            })
        );
        assert!(!finished.publication_started);
        assert!(finished.output.is_none());
        assert_ne!(
            finished.state,
            JobState::Cancelled,
            "A preparation error is not cooperative cleanup ACK"
        );
        assert_eq!(finished.failure.as_ref().unwrap().code, expected);
        let restored = VadSourceRequest::from_prepared(&finished.input).unwrap();
        assert_eq!(
            restored.original_audio_sha256(),
            request.audio_source.as_str()
        );
        assert_eq!(finished.input_sha256, admitted.job.input_sha256);
        assert_eq!(finished.attempt, DecimalU64(1));
        assert_eq!(
            fs::read_dir(&evidence).unwrap().count(),
            0,
            "No attempt resource factory may run"
        );
        assert!(!root.join("result.json").exists());
        assert!(!unavailable_worker.exists());
        if expected != ErrorCode::Timeout {
            assert_eq!(hook.calls.load(Ordering::SeqCst), 2);
        }
        let calls = hook.calls.load(Ordering::SeqCst);
        let duplicate = client.submit(submission, "original-scope-test").unwrap();
        assert!(!duplicate.created);
        assert_eq!(duplicate.job.job_id, finished.job_id);
        assert_eq!(
            hook.calls.load(Ordering::SeqCst),
            calls,
            "Dedup must not re-admit changed actual input"
        );
        if expected == ErrorCode::SourceExpired {
            if let Err(error) = close(host.take().unwrap()) {
                close_failure = Some(error);
                return;
            }
            let readonly = jobs::read_only(&root.join("jobs")).unwrap();
            let reopened = readonly
                .get(GetJob {
                    job_id: finished.job_id.clone(),
                })
                .unwrap();
            assert_eq!(reopened.input_sha256, finished.input_sha256);
            assert_eq!(
                VadSourceRequest::from_prepared(&reopened.input)
                    .unwrap()
                    .original_audio_sha256(),
                request.audio_source.as_str()
            );
            drop(readonly);
            let restarted =
                jobs::start_retaining_host_with_executor(&root.join("jobs"), executor.clone())
                    .unwrap();
            client = restarted.client();
            *hook.client.lock().unwrap() = Some(Arc::downgrade(&client));
            host = Some(restarted);
            let retry = client
                .retry(
                    RetryJob {
                        job_id: reopened.job_id,
                    },
                    reopened.revision.0,
                )
                .unwrap();
            assert_eq!(retry.attempt, DecimalU64(2));
            assert_eq!(retry.input_sha256, finished.input_sha256);
            let rejected = terminal(client.as_ref(), &retry.job_id);
            assert_eq!(
                rejected.failure.as_ref().unwrap().code,
                ErrorCode::SourceExpired
            );
            assert!(rejected.output.is_none());
            assert!(!rejected.publication_started);
            assert_eq!(
                hook.calls.load(Ordering::SeqCst),
                3,
                "Retry must re-admit under its newly issued original scope"
            );
            assert_eq!(fs::read_dir(&evidence).unwrap().count(), 0);
            assert!(!root.join("result.json").exists());
        }
    }));
    if let Some(original) = host.take()
        && let Err(error) = close(original)
    {
        close_failure = Some(error);
    }
    drop(client);
    drop(executor);
    drop(blobs);
    drop(hook);
    conclude(action, close_failure);
    // Fixture cleanup remains with the shared Media fixture/caller; on failure,
    // no local recursive cleanup erases original records or raw diagnostics.
}
