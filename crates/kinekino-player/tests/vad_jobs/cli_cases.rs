//! Actual CLI consumption of an injected concrete adapter, without default registration.
use crate::{host_closeout, vad_job_executor::VadJobExecutor, vad_release_fixture};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_player::{
    application::{
        api::{PlayerSdk, Request},
        audio_vad::VadSourceRequest,
        jobs::{GetJob, JobRecord, JobState},
    },
    infrastructure::{
        jobs,
        local_host::{LocalHost, PendingLocalHost, TransportOptions, WorkerJoin},
    },
};
use kinekino_vad::{application::ProfileAdmission, domain::VadProfile};
use serde_json::{Value, json};
use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

struct ChangeAfterSubmit {
    original: Arc<dyn ProfileAdmission + Send + Sync>,
    calls: AtomicUsize,
    source: PathBuf,
}
impl ProfileAdmission for ChangeAfterSubmit {
    fn authorize(&self, profile: &VadProfile) -> Result<(), ApiError> {
        self.original.authorize(profile)?;
        if self.calls.fetch_add(1, Ordering::SeqCst) == 1 {
            fs::write(&self.source, b"changed original after CLI submit")
                .map_err(kinekino_core::file_publication::io_error)?;
        }
        Ok(())
    }
}
fn call(root: &Path, label: &str, endpoint: &str, request: &Request) -> Value {
    let directory = root.join(label);
    fs::create_dir(&directory).unwrap();
    fs::write(
        directory.join("request.json"),
        serde_json::to_vec(request).unwrap(),
    )
    .unwrap();
    let mut runner = crate::cli_fixture::CliRunner::start(endpoint, request);
    let receipt = runner.finish_preserving(&directory);
    assert_eq!(receipt.code, 0, "{receipt:?}");
    receipt.response["data"].clone()
}
fn request(command: &str, payload: Value) -> Request {
    let mut request = Request::new(command);
    request.context.request_id = format!("vad-cli-{}", command.replace('.', "-"));
    request.payload = payload;
    request
}
fn terminal(
    client: &dyn kinekino_player::application::jobs::JobsApi,
    job: &JobRecord,
) -> JobRecord {
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        let current = client
            .get(GetJob {
                job_id: job.job_id.clone(),
            })
            .unwrap();
        if current.state.terminal() {
            return current;
        }
        assert!(Instant::now() < until, "Original job observation expired");
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn close_transport(pending: &mut Option<PendingLocalHost>, root: &Path, label: &str) -> bool {
    let until = Instant::now() + Duration::from_secs(5);
    let Some(owner) = pending.as_mut() else {
        return true;
    };
    loop {
        let status = owner.poll();
        let terminal = status.pipe != WorkerJoin::Pending && status.sdk != WorkerJoin::Pending;
        let expired = Instant::now() >= until;
        if terminal || expired {
            // Evidence IO must not unwind the original cleanup carrier.
            if let Err(error) = fs::write(
                root.join(format!("{label}.txt")),
                format!("status={status:?}\nobservation_expired={expired}\n"),
            ) {
                use std::io::Write;
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "transport evidence failed: {error}"
                );
                return false;
            }
        }
        if terminal {
            let complete = status.is_quiescent();
            pending.take();
            return complete;
        }
        if expired {
            return false;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

pub(super) fn exercise() {
    std::env::var("JOB_CLI_SHA256")
        .expect("Explicit actual CLI gate requires its fresh binary pin");
    let fixture = vad_release_fixture::release_file_fixture();
    let root = fixture.root;
    eprintln!("vad_cli_fixture_root={}", root.display());
    let logs = root.join("cli-evidence");
    fs::create_dir(&logs).unwrap();
    let original_sha = fixture.request.audio_source.as_str().to_owned();
    let hook = Arc::new(ChangeAfterSubmit {
        original: fixture.admission,
        calls: AtomicUsize::new(0),
        source: root.join("source.bin"),
    });
    let evidence = fixture.execution.evidence_root.clone();
    assert!(evidence.is_dir());
    assert_eq!(fs::read_dir(&evidence).unwrap().count(), 0);
    assert!(!fixture.execution.worker.exists());
    let executor = Arc::new(VadJobExecutor::new(
        fixture.inputs,
        hook.clone(),
        fixture.execution,
    ));
    let mut host = Some(
        jobs::start_retaining_host_with_executor(&root.join("jobs"), executor.clone()).unwrap(),
    );
    let mut local = None;
    let mut pending = None;
    let mut close_failure = None;
    let mut transport_closed = true;
    let action = catch_unwind(AssertUnwindSafe(|| {
        let client = host.as_ref().unwrap().client();
        local = Some(
            LocalHost::start(
                Arc::new(PlayerSdk::default().with_jobs(client.clone())),
                TransportOptions::default(),
            )
            .unwrap(),
        );
        let endpoint = local.as_ref().unwrap().endpoint().to_owned();
        let mut submit = request(
            "jobs.submit",
            json!({"operation":"audio.vad", "parameters":{
            "profile_id":fixture.public_profile_id,"expected_profile_manifest_sha256":fixture.request.profile_manifest.as_str(),
            "audio_artifact_id":fixture.public_audio_id,"start_sample":fixture.request.start,"end_sample":fixture.request.end,"destination_id":fixture.public_destination_id},
            "expected_source_sha256":original_sha,"execution_timeout_ms":"30000"}),
        );
        submit.context.idempotency_key = Some("vad-cli-original".into());
        let accepted = call(&logs, "submit", &endpoint, &submit);
        assert_eq!(accepted["created"], true);
        let initial: JobRecord = serde_json::from_value(accepted["job"].clone()).unwrap();
        let finished = terminal(client.as_ref(), &initial);
        assert_eq!(finished.state, JobState::NeedsInput);
        assert_eq!(
            finished.failure.as_ref().unwrap().code,
            ErrorCode::SourceExpired
        );
        let get = request("jobs.get", json!({"job_id":finished.job_id}));
        let remote = call(&logs, "get", &endpoint, &get);
        assert_eq!(remote, serde_json::to_value(&finished).unwrap());
        let duplicate = call(&logs, "dedup", &endpoint, &submit);
        assert_eq!(duplicate["created"], false);
        assert_eq!(duplicate["job"]["input_sha256"], finished.input_sha256);
        assert_eq!(hook.calls.load(Ordering::SeqCst), 2);
        pending = Some(local.take().unwrap().begin_shutdown());
        assert!(close_transport(&mut pending, &logs, "first-transport"));
        if let Err(error) = host_closeout::close(host.take().unwrap()) {
            close_failure = Some(error);
            return;
        }
        drop(client);
        let readonly = jobs::read_only(&root.join("jobs")).unwrap();
        let reopened = readonly
            .get(GetJob {
                job_id: finished.job_id.clone(),
            })
            .unwrap();
        assert_eq!(reopened.input_sha256, finished.input_sha256);
        drop(readonly);
        host = Some(
            jobs::start_retaining_host_with_executor(&root.join("jobs"), executor.clone()).unwrap(),
        );
        let client = host.as_ref().unwrap().client();
        local = Some(
            LocalHost::start(
                Arc::new(PlayerSdk::default().with_jobs(client.clone())),
                TransportOptions::default(),
            )
            .unwrap(),
        );
        let endpoint = local.as_ref().unwrap().endpoint().to_owned();
        let mut retry = request("jobs.retry", json!({"job_id":reopened.job_id}));
        retry.context.expected_revision = Some(reopened.revision.0.to_string());
        let retried: JobRecord =
            serde_json::from_value(call(&logs, "retry", &endpoint, &retry)).unwrap();
        let rejected = terminal(client.as_ref(), &retried);
        assert_eq!(rejected.attempt.0, 2);
        assert_eq!(rejected.state, JobState::NeedsInput);
        assert_eq!(rejected.input_sha256, finished.input_sha256);
        assert_eq!(
            rejected.failure.as_ref().unwrap().code,
            ErrorCode::SourceExpired
        );
        assert_eq!(
            VadSourceRequest::from_prepared(&rejected.input)
                .unwrap()
                .original_audio_sha256(),
            original_sha
        );
        assert!(rejected.output.is_none());
        assert!(!rejected.publication_started);
        assert_eq!(
            call(&logs, "retry-get", &endpoint, &get),
            serde_json::to_value(&rejected).unwrap()
        );
        assert_eq!(hook.calls.load(Ordering::SeqCst), 3);
        assert_eq!(fs::read_dir(&evidence).unwrap().count(), 0);
        assert!(!root.join("result.json").exists());
    }));
    if let Some(original) = local.take() {
        pending = Some(original.begin_shutdown());
    }
    if !close_transport(&mut pending, &logs, "final-transport") {
        transport_closed = false;
        if let Some(owner) = pending.take() {
            Box::leak(Box::new(owner));
        }
        if let Some(owner) = host.take() {
            Box::leak(Box::new(owner));
        }
    } else if let Some(original) = host.take()
        && let Err(error) = host_closeout::close(original)
    {
        close_failure = Some(error);
    }
    host_closeout::conclude(action, close_failure);
    assert!(
        transport_closed,
        "Original transport incomplete; original dependency retained, no cleanup claim"
    );
}
