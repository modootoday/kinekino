use super::fixture::{Fixture, request};
use kinekino_jobs::{
    application::{ExecutionControl, JobExecutor, SourceBlobStore},
    domain::{PreparedJobInput, VerifiedJobOutput},
};
use kinekino_player::application::api::JobClient;
use kinekino_player::application::api::{ApiError, ErrorCode};
use kinekino_player::{
    application::{
        api::PlayerSdk,
        jobs::{GetJob, JobId, JobState},
    },
    infrastructure::{
        jobs,
        local_host::{EndpointClient, LocalHost, TransportOptions},
    },
};
use serde_json::{Value, json};
use std::{
    fs,
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};
pub(super) struct Gate {
    entered: Mutex<bool>,
    ready: Condvar,
    released: Mutex<bool>,
    resume: Condvar,
}
impl Gate {
    pub(super) fn wait_render(&self) {
        let (lock, _) = self
            .ready
            .wait_timeout_while(
                self.entered.lock().unwrap(),
                Duration::from_secs(110),
                |entered| !*entered,
            )
            .unwrap();
        assert!(*lock, "Original render did not reach publication gate");
    }
    pub(super) fn new() -> Self {
        Self {
            entered: Mutex::new(false),
            ready: Condvar::new(),
            released: Mutex::new(false),
            resume: Condvar::new(),
        }
    }
    fn pause(&self) -> Result<(), ApiError> {
        *self.entered.lock().unwrap() = true;
        self.ready.notify_all();
        let (lock, result) = self
            .resume
            .wait_timeout_while(
                self.released.lock().unwrap(),
                Duration::from_secs(10),
                |released| !*released,
            )
            .unwrap();
        if result.timed_out() && !*lock {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Controlled SDK test gate timed out",
            ));
        }
        Ok(())
    }
    fn wait(&self) {
        let (lock, result) = self
            .ready
            .wait_timeout_while(
                self.entered.lock().unwrap(),
                Duration::from_secs(8),
                |entered| !*entered,
            )
            .unwrap();
        assert!(*lock && !result.timed_out());
    }
    pub(super) fn release(&self) {
        *self.released.lock().unwrap() = true;
        self.resume.notify_all();
    }
}
pub(super) struct Release(pub Arc<Gate>);
impl Drop for Release {
    fn drop(&mut self) {
        self.0.release();
    }
}
pub(super) struct Paused {
    pub inner: Arc<dyn JobExecutor>,
    pub gate: Arc<Gate>,
    pub after: bool,
}
struct Control<'a> {
    inner: &'a dyn ExecutionControl,
    gate: &'a Gate,
    after: bool,
}
impl ExecutionControl for Control<'_> {
    fn execution_identity(&self) -> Option<&kinekino_jobs::application::JobAttemptIdentityV1> {
        self.inner.execution_identity()
    }
    fn cancellation_requested(&self) -> bool {
        self.inner.cancellation_requested()
    }
    fn deadline(&self) -> Instant {
        self.inner.deadline()
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        if self.after {
            self.inner.reserve_publication()?;
        }
        self.gate.pause()?;
        if !self.after {
            self.inner.reserve_publication()?;
        }
        Ok(())
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        self.inner.begin_verification()
    }
}
impl JobExecutor for Paused {
    fn owned_executor(
        &self,
        operation: &str,
    ) -> Option<&kinekino_jobs::application::owned_attempt::OwnedExecutorPort> {
        if self.after {
            return None;
        }
        self.inner
            .owned_executor(operation)
            .map(|_| self as &kinekino_jobs::application::owned_attempt::OwnedExecutorPort)
    }
    fn operations(&self) -> Vec<String> {
        self.inner.operations()
    }
    fn supports(&self, operation: &str) -> bool {
        self.inner.supports(operation)
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.inner.execution_timeout_limit_ms(operation)
    }
    fn normalize_parameters(&self, operation: &str, parameters: Value) -> Result<Value, ApiError> {
        self.inner.normalize_parameters(operation, parameters)
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        self.inner.prepare(operation, parameters, expected, blobs)
    }
    fn execute(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        self.inner.execute(
            input,
            blobs,
            &Control {
                inner: control,
                gate: &self.gate,
                after: self.after,
            },
        )
    }
}
impl kinekino_jobs::application::owned_attempt::OwnedAttemptExecutor for Paused {
    type Candidate = Box<dyn kinekino_jobs::application::owned_attempt::OwnedOutputCandidate>;
    fn prepare_owned(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &kinekino_jobs::application::owned_attempt::AttemptScope<'_>,
    ) -> Result<
        Box<
            dyn kinekino_jobs::application::owned_attempt::OwnedAttemptSession<
                    Candidate = Self::Candidate,
                >,
        >,
        ApiError,
    > {
        let inner = self
            .inner
            .owned_executor(&input.operation)
            .expect("Original Core clip executor")
            .prepare_owned(input, blobs, scope)?;
        Ok(Box::new(PausedSession {
            inner,
            gate: self.gate.clone(),
            candidate: None,
        }))
    }
}
struct PausedSession {
    inner: Box<
        dyn kinekino_jobs::application::owned_attempt::OwnedAttemptSession<
                Candidate = Box<
                    dyn kinekino_jobs::application::owned_attempt::OwnedOutputCandidate,
                >,
            >,
    >,
    gate: Arc<Gate>,
    candidate: Option<Box<dyn kinekino_jobs::application::owned_attempt::OwnedOutputCandidate>>,
}
impl kinekino_jobs::application::owned_attempt::OwnedAttemptSession for PausedSession {
    type Candidate = Box<dyn kinekino_jobs::application::owned_attempt::OwnedOutputCandidate>;
    fn run(
        &mut self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &kinekino_jobs::application::owned_attempt::AttemptScope<'_>,
    ) -> kinekino_jobs::application::owned_attempt::SessionRun<Self::Candidate> {
        use kinekino_jobs::application::owned_attempt::SessionRun;
        match self.inner.run(input, blobs, scope) {
            SessionRun::Candidate(candidate) => {
                self.candidate = Some(candidate);
                match self.gate.pause() {
                    Ok(()) => SessionRun::Candidate(self.candidate.take().unwrap()),
                    Err(error) => SessionRun::Failed(error),
                }
            }
            other => other,
        }
    }
    fn poll_close(
        &mut self,
        deadline: Instant,
    ) -> Result<kinekino_jobs::application::owned_attempt::OwnerObservation, ApiError> {
        if let Some(candidate) = &mut self.candidate
            && !candidate.poll_release(deadline)?
        {
            return Ok(kinekino_jobs::application::owned_attempt::OwnerObservation::Pending);
        }
        self.inner.poll_close(deadline)
    }
}
#[test]
fn actual_cli_running_cancel_ack_and_publication_winner_keep_real_file_receipts() {
    kinekino_platform::install();
    for (range, after) in [(false, false), (false, true), (true, false), (true, true)] {
        let fixture = Fixture::named(match (range, after) {
            (false, false) => "cancel-before",
            (false, true) => "cancel-after",
            (true, false) => "range-cancel-before",
            (true, true) => "range-cancel-after",
        });
        let gate = Arc::new(Gate::new());
        let executor = Arc::new(Paused {
            inner: jobs::subtitle_executor(),
            gate: gate.clone(),
            after,
        });
        let host = jobs::start_host_with_executor(&fixture.root.join("store"), executor).unwrap();
        let sdk = Arc::new(PlayerSdk::default().with_jobs(host.client()));
        let local = LocalHost::start(sdk, TransportOptions::default()).unwrap();
        let endpoint = local.endpoint().to_owned();
        let client = EndpointClient::new(&endpoint, TransportOptions::default()).unwrap();
        let release = Release(gate.clone());
        let source = fixture.root.join("source.srt");
        let output = fixture.root.join("output.vtt");
        let original = "1\n00:00:01,001 --> 00:00:03,456\n원문 日本語\n\n";
        fs::write(&source, original).unwrap();
        let mut submit = request(
            "jobs.submit",
            json!({"operation":"subtitles.convert","parameters":{"input_path":source,"output_path":output,"input_format":"srt","output_format":"vtt"}}),
        );
        if range {
            submit.payload["operation"] = json!("subtitles.export_range");
            submit.payload["parameters"]["range"] = json!({"in_us":"0","out_us":"4000000"});
        }
        submit.context.idempotency_key = Some("controlled-file".into());
        let accepted = fixture.call("submit", &submit, Some(&endpoint), None, 0);
        let id: JobId = serde_json::from_value(accepted["data"]["job"]["job_id"].clone()).unwrap();
        gate.wait();
        let cancellation = fixture.call(
            "running-cancel",
            &request("jobs.cancel", json!({"job_id":id,"expected_attempt":"1"})),
            Some(&endpoint),
            None,
            0,
        );
        assert_eq!(
            cancellation["data"]["disposition"],
            if after {
                "publication_in_progress"
            } else {
                "requested"
            }
        );
        assert_eq!(
            cancellation["data"]["job"]["state"],
            if after { "running" } else { "cancelling" }
        );
        assert!(!output.exists());
        gate.release();
        drop(release);
        let deadline = Instant::now() + Duration::from_secs(8);
        let result = loop {
            let job = client
                .get_job(Default::default(), GetJob { job_id: id.clone() })
                .unwrap();
            if job.state.terminal() {
                break job;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(
            result.state,
            if after {
                JobState::Completed
            } else {
                JobState::Cancelled
            }
        );
        let observed = fixture.call(
            "terminal",
            &request("jobs.get", json!({"job_id":id})),
            Some(&endpoint),
            None,
            0,
        );
        assert_eq!(
            observed["data"]["state"],
            if after { "completed" } else { "cancelled" }
        );
        assert_eq!(output.exists(), after);
        assert_eq!(fs::read_to_string(&source).unwrap(), original);
        if after {
            let bytes = fs::read(&output).unwrap();
            let sha = kinekino_player::infrastructure::content_digest::sha256(
                &mut std::io::Cursor::new(&bytes),
                1024 * 1024,
            )
            .unwrap()
            .1;
            assert_eq!(observed["data"]["output"]["sha256"], sha);
        } else {
            assert_eq!(observed["data"]["output"], Value::Null);
        }
        drop(local);
        host.shutdown().unwrap();
    }
}
