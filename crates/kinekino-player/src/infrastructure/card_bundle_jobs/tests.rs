use super::*;
use crate::{
    application::{
        card_bundle::{CardBundlePublication, SavedCardProjectPin, SelectedCardFrame},
        card_projects,
        jobs::{CancelJob, GetJob, JobState, SubmitJob},
    },
    domain::card_document::{CardDocument, CardEditor},
    infrastructure::{card_projects::CardProjectFiles, jobs},
};
use kinekino_jobs::infrastructure::{RetainingWorkerHost, WorkerShutdownOutcome};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    time::{Duration, Instant},
};
struct Evidence(PathBuf);
impl Evidence {
    fn new(case: &str) -> Self {
        let parent = PathBuf::from(
            std::env::var_os("CARD_BUNDLE_JOBS_EVIDENCE_DIR")
                .expect("Explicit existing evidence root"),
        );
        assert!(parent.is_absolute());
        let root = parent.join(format!("jobs-{case}-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
struct Host {
    host: Option<RetainingWorkerHost>,
    root: PathBuf,
}
impl Host {
    fn new(evidence: &Evidence, executor: Arc<dyn JobExecutor>) -> Self {
        let root = evidence.0.join("jobs");
        fs::create_dir(&root).unwrap();
        Self {
            host: Some(jobs::start_retaining_host_with_executor(&root, executor).unwrap()),
            root: evidence.0.clone(),
        }
    }
    fn client(&self) -> Arc<dyn kinekino_jobs::application::JobsApi> {
        self.host.as_ref().unwrap().client()
    }
    fn wait(&self, id: &crate::application::jobs::JobId) -> crate::application::jobs::JobRecord {
        let deadline = Instant::now() + Duration::from_secs(25);
        loop {
            let job = self.client().get(GetJob { job_id: id.clone() }).unwrap();
            fs::write(
                self.root.join("last-job.json"),
                serde_json::to_vec_pretty(&job).unwrap(),
            )
            .unwrap();
            if job.state.terminal() {
                return job;
            }
            assert!(Instant::now() < deadline, "Original job did not finish");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        let Some(host) = self.host.take() else {
            return;
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut pending = host.begin_shutdown(deadline);
        loop {
            match pending.poll(deadline) {
                WorkerShutdownOutcome::Complete(done) => {
                    fs::write(self.root.join("shutdown.json"), serde_json::to_vec_pretty(&json!({"receipt":done.receipt,"error":done.errors.first(),"secondary_count":done.errors.secondary().len()})).unwrap()).unwrap();
                    if done.errors.first().is_some() {
                        std::process::exit(101);
                    }
                    break;
                }
                WorkerShutdownOutcome::Pending(original) => {
                    pending = original;
                    if Instant::now() >= deadline {
                        eprintln!("Original Jobs owner remains pending; no cleanup success");
                        std::process::exit(101);
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }
}
fn fixture(evidence: &Evidence) -> (Arc<dyn CardProjectStore>, CardBundleRequest) {
    let mut png_bytes = Vec::new();
    {
        let mut e = png::Encoder::new(&mut png_bytes, 8, 8);
        e.set_color(png::ColorType::Rgba);
        let mut w = e.write_header().unwrap();
        w.write_image_data(&[100, 50, 20, 255].repeat(64)).unwrap();
        w.finish().unwrap();
    }
    let frame_sha = digest(&png_bytes);
    let path = evidence.0.join("frame.png");
    fs::write(&path, &png_bytes).unwrap();
    let root = evidence.0.join("projects");
    fs::create_dir(&root).unwrap();
    let store: Arc<dyn CardProjectStore> = Arc::new(CardProjectFiles::open(&root).unwrap());
    let document:CardDocument=serde_json::from_value(json!({"schema_version":"1","document_id":"bundle-1","revision":"1","source":{"source_id":"cue-source","source_revision":"1","source_sha256":"a".repeat(64),"cues":[{"cue_id":"cue-1","text":"원문","start_us":null,"end_us":null}]},"layout":{"canvas":CardBundleFiles::current_proposed_canvas(),"cards":[{"card_id":"first","role":"cover","text_kind":"summary","content":{"front":"제목","back":"요약 초안","source_refs":["cue-1"]},"frame":{"artifact_id":"frame-1","png_sha256":frame_sha,"source_sha256":"a".repeat(64),"declared_source_pts_us":null},"alt_text_draft":"설명 초안"}],"caption_draft":"게시문 초안"}})).unwrap();
    let saved = card_projects::create(store.as_ref(), &document).unwrap();
    let request = CardBundleRequest {
        project: SavedCardProjectPin {
            document_id: document.document_id,
            publication_revision: DecimalU64(1),
            publication_sha256: saved.publication_sha256,
        },
        frames: vec![SelectedCardFrame {
            artifact_id: "frame-1".into(),
            path: path.to_str().unwrap().into(),
            png_sha256: frame_sha,
        }],
        output_directory: evidence.0.join("output").to_str().unwrap().into(),
        review_artifact_only: true,
    };
    (store, request)
}
fn submit(request: &CardBundleRequest) -> SubmitJob {
    SubmitJob {
        operation: OPERATION.into(),
        parameters: serde_json::to_value(request).unwrap(),
        expected_source_sha256: None,
        execution_timeout_ms: DecimalU64(30000),
    }
}

#[test]
#[ignore = "Explicit CPU allocation and fresh card bundle Jobs evidence"]
fn real_jobs_publish_complete_bundle_replay_reopen_and_editable_project() {
    let evidence = Evidence::new("complete");
    let (projects, request) = fixture(&evidence);
    let executor = Arc::new(CardBundleJobExecutor::new(projects.clone()));
    let host = Host::new(&evidence, executor);
    let client = host.client();
    let captured = client.capture(submit(&request)).unwrap();
    assert!(captured.request().expected_source_sha256.is_some());
    assert!(
        client
            .list(crate::application::jobs::ListJobs {
                cursor: None,
                limit: Some(50)
            })
            .unwrap()
            .jobs
            .is_empty()
    );
    assert!(!std::path::Path::new(&request.output_directory).exists());
    fs::write(
        evidence.0.join("captured-request.json"),
        serde_json::to_vec_pretty(captured.request()).unwrap(),
    )
    .unwrap();
    let captured = captured.into_request();
    let submitted = client
        .submit(captured.clone(), "card-bundle-complete")
        .unwrap();
    let done = host.wait(&submitted.job.job_id);
    assert_eq!(done.state, JobState::Completed, "{:?}", done.failure);
    let output = done.output.as_ref().unwrap();
    let manifest_bytes = fs::read(&output.path).unwrap();
    assert_eq!(digest(&manifest_bytes), output.sha256);
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(manifest["quality_approved"], false);
    assert_eq!(manifest["state"], "review_artifact");
    let directory = PathBuf::from(&request.output_directory);
    for file in manifest["files"].as_array().unwrap() {
        let bytes = fs::read(directory.join(file["name"].as_str().unwrap())).unwrap();
        assert_eq!(digest(&bytes), file["sha256"]);
    }
    assert_eq!(
        fs::read_to_string(directory.join("caption.txt")).unwrap(),
        "게시문 초안"
    );
    let publication: card_projects::CardPublication =
        serde_json::from_slice(&fs::read(directory.join("project.json")).unwrap()).unwrap();
    CardEditor::open(publication.document).unwrap();
    let source = manifest["source_frames"][0]["path"].as_str().unwrap();
    assert_eq!(
        digest(&fs::read(directory.join(source)).unwrap()),
        request.frames[0].png_sha256
    );
    let replay = client.submit(captured, "card-bundle-complete").unwrap();
    assert!(!replay.created);
    assert_eq!(replay.job.job_id, submitted.job.job_id);
    assert_eq!(
        projects
            .load("bundle-1", None)
            .unwrap()
            .publication
            .publication_revision,
        DecimalU64(1)
    );
    drop(host);
    let reopened = jobs::read_only(&evidence.0.join("jobs"))
        .unwrap()
        .get(GetJob {
            job_id: submitted.job.job_id,
        })
        .unwrap();
    assert_eq!(reopened.output.unwrap().sha256, output.sha256);
}

struct VerificationGate {
    inner: CardBundleJobExecutor,
    entered: Sender<()>,
    release: Mutex<Receiver<()>>,
    after_publication: bool,
}
impl VerificationGate {
    fn pause(&self) {
        self.entered.send(()).unwrap();
        self.release
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(15))
            .unwrap();
    }
}
struct GateControl<'a> {
    original: &'a dyn ExecutionControl,
    gate: &'a VerificationGate,
}
impl ExecutionControl for GateControl<'_> {
    fn execution_identity(&self) -> Option<&kinekino_jobs::application::JobAttemptIdentityV1> {
        self.original.execution_identity()
    }
    fn cancellation_requested(&self) -> bool {
        self.original.cancellation_requested()
    }
    fn deadline(&self) -> Instant {
        self.original.deadline()
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        if !self.gate.after_publication {
            self.gate.pause();
        }
        self.original.reserve_publication()
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        if self.gate.after_publication {
            self.gate.pause();
        }
        self.original.begin_verification()
    }
}
impl JobExecutor for VerificationGate {
    fn operations(&self) -> Vec<String> {
        self.inner.operations()
    }
    fn supports(&self, op: &str) -> bool {
        self.inner.supports(op)
    }
    fn normalize_parameters(&self, op: &str, p: Value) -> Result<Value, ApiError> {
        self.inner.normalize_parameters(op, p)
    }
    fn prepare(
        &self,
        op: &str,
        p: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        self.inner.prepare(op, p, expected, blobs)
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
            &GateControl {
                original: control,
                gate: self,
            },
        )
    }
}
#[test]
#[ignore = "Explicit CPU allocation and fresh card bundle Jobs evidence"]
fn real_jobs_cancel_before_publication_never_publishes_and_preserves_attempt_evidence() {
    let evidence = Evidence::new("cancel");
    let (projects, request) = fixture(&evidence);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let gate = Arc::new(VerificationGate {
        inner: CardBundleJobExecutor::new(projects),
        entered: entered_tx,
        release: Mutex::new(release_rx),
        after_publication: false,
    });
    let host = Host::new(&evidence, gate);
    let submitted = host
        .client()
        .submit(submit(&request), "bundle-cancel")
        .unwrap();
    entered_rx.recv_timeout(Duration::from_secs(20)).unwrap();
    let cancellation = host
        .client()
        .cancel(CancelJob {
            job_id: submitted.job.job_id.clone(),
            expected_attempt: submitted.job.attempt,
        })
        .unwrap();
    assert!(cancellation.job.cancel_requested);
    release_tx.send(()).unwrap();
    let done = host.wait(&submitted.job.job_id);
    assert_eq!(done.state, JobState::Cancelled);
    assert!(done.output.is_none());
    assert!(!PathBuf::from(&request.output_directory).exists());
    let stages: Vec<_> = fs::read_dir(&evidence.0)
        .unwrap()
        .map(Result::unwrap)
        .filter(|e| e.file_name().to_string_lossy().starts_with(".card-bundle-"))
        .collect();
    assert_eq!(stages.len(), 1);
    let attempt: Value =
        serde_json::from_slice(&fs::read(stages[0].path().join("attempt.json")).unwrap()).unwrap();
    assert_eq!(attempt["job_id"], submitted.job.job_id.as_str());
}

#[test]
#[ignore = "Explicit CPU allocation and fresh card bundle Jobs evidence"]
fn real_jobs_cancel_after_publication_is_not_an_ack_and_original_result_completes() {
    let evidence = Evidence::new("late-cancel");
    let (projects, request) = fixture(&evidence);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let gate = Arc::new(VerificationGate {
        inner: CardBundleJobExecutor::new(projects),
        entered: entered_tx,
        release: Mutex::new(release_rx),
        after_publication: true,
    });
    let host = Host::new(&evidence, gate);
    let submitted = host
        .client()
        .submit(submit(&request), "bundle-late-cancel")
        .unwrap();
    entered_rx.recv_timeout(Duration::from_secs(20)).unwrap();
    assert!(
        PathBuf::from(&request.output_directory)
            .join("manifest.json")
            .is_file()
    );
    let cancellation = host
        .client()
        .cancel(CancelJob {
            job_id: submitted.job.job_id.clone(),
            expected_attempt: submitted.job.attempt,
        })
        .unwrap();
    assert_eq!(
        cancellation.disposition,
        crate::application::jobs::CancelDisposition::PublicationInProgress
    );
    assert!(!cancellation.job.cancel_requested);
    release_tx.send(()).unwrap();
    let done = host.wait(&submitted.job.job_id);
    assert_eq!(done.state, JobState::Completed, "{:?}", done.failure);
    assert!(!done.cancel_requested);
}

struct RaceControl {
    target: PathBuf,
    reserved: AtomicBool,
    deadline: Instant,
}
impl ExecutionControl for RaceControl {
    fn cancellation_requested(&self) -> bool {
        false
    }
    fn deadline(&self) -> Instant {
        self.deadline
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        Ok(())
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        self.reserved.store(true, Ordering::SeqCst);
        fs::create_dir(&self.target).unwrap();
        fs::write(self.target.join("sentinel"), b"original").unwrap();
        Ok(())
    }
}
#[test]
#[ignore = "Explicit native no-overwrite directory publication evidence"]
fn publication_race_preserves_existing_directory_and_original_staging_identity() {
    let evidence = Evidence::new("race");
    let target = evidence.0.join("output");
    let control = RaceControl {
        target: target.clone(),
        reserved: AtomicBool::new(false),
        deadline: Instant::now() + Duration::from_secs(30),
    };
    let mut staging = publication::NativeBundlePublication
        .begin(target.to_str().unwrap())
        .unwrap();
    let manifest = staging
        .write_new("manifest.json", b"{}", "application/json")
        .unwrap();
    staging.verify_all(&control).unwrap();
    let error = staging.publish_new(&manifest, &control).unwrap_err();
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    assert!(control.reserved.load(Ordering::SeqCst));
    assert_eq!(fs::read(target.join("sentinel")).unwrap(), b"original");
    let recovery = staging.recovery_details();
    assert_eq!(recovery["published"], false);
    fs::write(
        evidence.0.join("recovery.json"),
        serde_json::to_vec_pretty(&recovery).unwrap(),
    )
    .unwrap();
    assert!(
        PathBuf::from(recovery["staging_directory"].as_str().unwrap())
            .join("manifest.json")
            .is_file()
    );
}
