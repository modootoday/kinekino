//! Partial bundle failure, cancellation acknowledgement and explicit retry through the
//! original Jobs worker (roadmap M4-5). Fixture rasterizer/publication only: no native
//! renderer, no filesystem bundle adapter, no network. The original worker supplies the
//! attempt identity; nothing here constructs it.
use super::*;
use crate::{
    application::jobs::{
        CancelJob, ExecutionControl, GetJob, JobExecutor, JobRecord, JobState, RetryJob,
        SourceBlobStore, SubmitJob, VerifiedJobOutput,
    },
    infrastructure::jobs,
};
use kinekino_jobs::domain::PreparedJobInput;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::AtomicBool,
        mpsc::{self, Receiver, Sender},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const FRAME_BYTES: &[u8] = b"fixture-frame-png-bytes";

fn sha(bytes: &[u8]) -> String {
    kinekino_media_contracts::hex_lower(Sha256::digest(bytes))
}

/// Bundle prepared by the real use case from the shared fixture; the frame pin is moved to
/// the fixture bytes so the staging check compares real digests.
fn prepared() -> PreparedCardBundle {
    let (store, request, assets) = fixture();
    let mut bundle = prepare(&store, &assets, request).unwrap();
    for card in &mut bundle.cards {
        if let Some(frame) = &mut card.frame {
            frame.png_sha256 = sha(FRAME_BYTES);
        }
    }
    bundle
}

#[derive(Default)]
struct Ledger {
    /// One entry per `begin`, in order: every file name written into that staging.
    stagings: Vec<Vec<String>>,
    attempt_files: Vec<Value>,
    published: usize,
}

struct Publication {
    ledger: Arc<Mutex<Ledger>>,
    fail_verify: Arc<AtomicBool>,
    target_exists: Arc<AtomicBool>,
}
struct Staging {
    ledger: Arc<Mutex<Ledger>>,
    index: usize,
    output: String,
    verified: bool,
    fail_verify: bool,
    target_exists: bool,
}
impl CardBundlePublication for Publication {
    fn begin(&self, output_directory: &str) -> Result<Box<dyn CardBundleStaging>, ApiError> {
        let mut ledger = self.ledger.lock().unwrap();
        ledger.stagings.push(Vec::new());
        Ok(Box::new(Staging {
            ledger: self.ledger.clone(),
            index: ledger.stagings.len() - 1,
            output: output_directory.into(),
            verified: false,
            fail_verify: self.fail_verify.load(Ordering::SeqCst),
            target_exists: self.target_exists.load(Ordering::SeqCst),
        }))
    }
}
impl CardBundleStaging for Staging {
    fn write_new(
        &mut self,
        name: &str,
        bytes: &[u8],
        media_type: &str,
    ) -> Result<BundleFile, ApiError> {
        let mut ledger = self.ledger.lock().unwrap();
        assert!(
            !ledger.stagings[self.index].iter().any(|n| n == name),
            "create-only staging"
        );
        ledger.stagings[self.index].push(name.into());
        if name == "attempt.json" {
            ledger
                .attempt_files
                .push(serde_json::from_slice(bytes).unwrap());
        }
        Ok(BundleFile {
            name: name.into(),
            sha256: sha(bytes),
            bytes: DecimalU64(bytes.len() as u64),
            media_type: media_type.into(),
        })
    }
    fn verify_all(&mut self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        check_control(control)?;
        if self.fail_verify {
            return Err(error(
                ErrorCode::SourceExpired,
                "Fixture staged bytes changed before publication",
            ));
        }
        self.verified = true;
        Ok(())
    }
    fn publish_new(
        &mut self,
        manifest: &BundleFile,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        assert!(self.verified, "publication requires verified staging");
        if self.target_exists {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Fixture output already exists; bundle will not be replaced",
            ));
        }
        control.reserve_publication()?;
        self.ledger.lock().unwrap().published += 1;
        control.begin_verification()?;
        Ok(VerifiedJobOutput {
            path: format!("{}/manifest.json", self.output),
            sha256: manifest.sha256.clone(),
            bytes: manifest.bytes,
            receipt: json!({"fixture":true}),
        })
    }
    fn recovery_details(&self) -> Value {
        let ledger = self.ledger.lock().unwrap();
        json!({"staging":self.index,"written":ledger.stagings[self.index],"partial_cleanup_supported":false})
    }
}

type RenderPause = (Mutex<Sender<()>>, Mutex<Receiver<()>>);

struct Raster {
    fail_at: Arc<Mutex<Option<u64>>>,
    rendered: Arc<Mutex<Vec<u64>>>,
    /// When set, rendering the first card reports `started` and waits for `release`.
    pause: Option<RenderPause>,
}
impl CardBundleRasterizer for Raster {
    fn read_frame_png(&self, _: &SelectedCardFrame) -> Result<Vec<u8>, ApiError> {
        Ok(FRAME_BYTES.to_vec())
    }
    fn render_card(
        &self,
        _: &PreparedCardBundle,
        position: DecimalU64,
    ) -> Result<RenderedBundleCard, ApiError> {
        self.rendered.lock().unwrap().push(position.0);
        if let (Some((started, release)), 0) = (&self.pause, position.0) {
            started.lock().unwrap().send(()).unwrap();
            release
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(10))
                .unwrap();
        }
        if *self.fail_at.lock().unwrap() == Some(position.0) {
            let mut failure = error(ErrorCode::EngineFailed, "Fixture card render failed");
            failure.details = json!({"card_position":position});
            return Err(failure);
        }
        Ok(RenderedBundleCard {
            png: format!("card-{}", position.0).into_bytes(),
            provenance: json!({"position":position}),
        })
    }
}

/// Test-only executor: admits the prepared snapshot and runs the real `execute_bundle`.
struct Executor {
    bundle: PreparedCardBundle,
    raster: Raster,
    publication: Publication,
}
impl JobExecutor for Executor {
    fn operations(&self) -> Vec<String> {
        vec![EXPORT_OPERATION.into()]
    }
    fn supports(&self, operation: &str) -> bool {
        operation == EXPORT_OPERATION
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        _: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        Ok(PreparedJobInput {
            operation: operation.into(),
            parameters,
            source: blobs.put_utf8(&serde_json::to_string(&self.bundle).unwrap())?,
            execution_timeout_ms: DecimalU64(30_000),
        })
    }
    fn execute(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        let bundle: PreparedCardBundle =
            serde_json::from_str(&blobs.read_utf8(&input.source)?).unwrap();
        execute_bundle(&bundle, &self.raster, &self.publication, control)
    }
}

struct Case {
    root: PathBuf,
    host: Option<kinekino_jobs::infrastructure::WorkerHost>,
    ledger: Arc<Mutex<Ledger>>,
    fail_at: Arc<Mutex<Option<u64>>>,
    rendered: Arc<Mutex<Vec<u64>>>,
    fail_verify: Arc<AtomicBool>,
    target_exists: Arc<AtomicBool>,
}
impl Case {
    fn start(tag: &str, pause: Option<(Sender<()>, Receiver<()>)>) -> Self {
        let root = std::env::temp_dir().join(format!(
            "kk-card-partial-{tag}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let ledger = Arc::new(Mutex::new(Ledger::default()));
        let fail_at = Arc::new(Mutex::new(None));
        let rendered = Arc::new(Mutex::new(Vec::new()));
        let fail_verify = Arc::new(AtomicBool::new(false));
        let target_exists = Arc::new(AtomicBool::new(false));
        let executor = Executor {
            bundle: prepared(),
            raster: Raster {
                fail_at: fail_at.clone(),
                rendered: rendered.clone(),
                pause: pause.map(|(s, r)| (Mutex::new(s), Mutex::new(r))),
            },
            publication: Publication {
                ledger: ledger.clone(),
                fail_verify: fail_verify.clone(),
                target_exists: target_exists.clone(),
            },
        };
        let host = jobs::start_host_with_executor(&root, Arc::new(executor)).unwrap();
        Self {
            root,
            host: Some(host),
            ledger,
            fail_at,
            rendered,
            fail_verify,
            target_exists,
        }
    }
    fn client(&self) -> Arc<dyn kinekino_jobs::application::JobsApi> {
        self.host.as_ref().unwrap().client()
    }
    fn submit(&self, key: &str) -> JobRecord {
        let submit = SubmitJob {
            operation: EXPORT_OPERATION.into(),
            parameters: json!({"fixture":key}),
            expected_source_sha256: None,
            execution_timeout_ms: DecimalU64(30_000),
        };
        self.client().submit(submit, key).unwrap().job
    }
    fn wait(&self, job: &JobRecord, attempt: u64) -> JobRecord {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let current = self
                .client()
                .get(GetJob {
                    job_id: job.job_id.clone(),
                })
                .unwrap();
            if current.attempt.0 == attempt && current.state.terminal() {
                return current;
            }
            assert!(Instant::now() < deadline, "job stuck: {:?}", current.state);
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    fn staging(&self, index: usize) -> Vec<String> {
        self.ledger.lock().unwrap().stagings[index].clone()
    }
    fn published(&self) -> usize {
        self.ledger.lock().unwrap().published
    }
}
impl Drop for Case {
    fn drop(&mut self) {
        if let Some(host) = self.host.take() {
            host.shutdown().unwrap();
        }
        if !std::thread::panicking() {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

fn frame_name() -> String {
    format!("frame-{}.png", sha(FRAME_BYTES))
}

#[test]
fn render_failure_mid_bundle_keeps_original_error_publishes_nothing_and_retry_completes() {
    let case = Case::start("render", None);
    *case.fail_at.lock().unwrap() = Some(1);
    let job = case.submit("render-partial");
    let failed = case.wait(&job, 1);
    assert_eq!(failed.state, JobState::Failed);
    let failure = failed.failure.clone().unwrap();
    assert_eq!(failure.code, ErrorCode::EngineFailed);
    assert_eq!(failure.message, "Fixture card render failed");
    assert_eq!(failure.details["completed"], json!(false));
    assert_eq!(
        failure.details["primary_details"],
        json!({"card_position":DecimalU64(1)})
    );
    // The first card stays staged as evidence; the failing card, caption, alt text,
    // project and manifest are never written, and nothing is published.
    let first_attempt = vec![
        "attempt.json".to_owned(),
        frame_name(),
        "001-cover.png".into(),
        "001-cover.provenance.json".into(),
    ];
    assert_eq!(case.staging(0), first_attempt);
    assert_eq!(
        failure.details["bundle_recovery"]["written"],
        json!(first_attempt)
    );
    assert_eq!(
        failure.details["bundle_recovery"]["partial_cleanup_supported"],
        json!(false)
    );
    assert_eq!(case.published(), 0);
    assert!(failed.output.is_none());
    assert_eq!(*case.rendered.lock().unwrap(), vec![0, 1]);

    // Explicit retry renders the whole bundle again in a fresh staging under attempt 2;
    // the failed staging is neither reused nor modified.
    *case.fail_at.lock().unwrap() = None;
    case.client()
        .retry(
            RetryJob {
                job_id: job.job_id.clone(),
            },
            failed.revision.0,
        )
        .unwrap();
    let done = case.wait(&job, 2);
    assert_eq!(done.state, JobState::Completed, "{:?}", done.failure);
    assert_eq!(case.staging(0), first_attempt);
    assert_eq!(
        case.staging(1),
        vec![
            "attempt.json".to_owned(),
            frame_name(),
            "001-cover.png".into(),
            "001-cover.provenance.json".into(),
            "002-closing.png".into(),
            "002-closing.provenance.json".into(),
            "caption.txt".into(),
            "alt-text.json".into(),
            "project.json".into(),
            "manifest.json".into(),
        ]
    );
    assert_eq!(case.published(), 1);
    assert!(done.output.unwrap().path.ends_with("/manifest.json"));
    let attempts = &case.ledger.lock().unwrap().attempt_files;
    assert_eq!(attempts[0]["attempt"], json!(DecimalU64(1)));
    assert_eq!(attempts[1]["attempt"], json!(DecimalU64(2)));
    assert_eq!(attempts[0]["job_id"], attempts[1]["job_id"]);
    assert_eq!(attempts[1]["quality_approved"], json!(false));
    assert_eq!(*case.rendered.lock().unwrap(), vec![0, 1, 0, 1]);
}

#[test]
fn cancel_during_card_render_is_acknowledged_without_publication() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let case = Case::start("cancel", Some((started_tx, release_rx)));
    let job = case.submit("cancel-partial");
    started_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    case.client()
        .cancel(CancelJob {
            job_id: job.job_id.clone(),
            expected_attempt: DecimalU64(1),
        })
        .unwrap();
    release_tx.send(()).unwrap();
    let cancelled = case.wait(&job, 1);
    assert_eq!(cancelled.state, JobState::Cancelled);
    assert!(cancelled.output.is_none());
    assert!(!cancelled.publication_started);
    // Cancellation is observed right after the in-flight card returns: its PNG is not written.
    assert_eq!(
        case.staging(0),
        vec!["attempt.json".to_owned(), frame_name()]
    );
    assert_eq!(case.published(), 0);
    assert_eq!(*case.rendered.lock().unwrap(), vec![0]);
}

#[test]
fn verification_and_existing_output_failures_stop_before_publication() {
    let case = Case::start("publish", None);
    case.fail_verify.store(true, Ordering::SeqCst);
    let job = case.submit("verify-failure");
    let failed = case.wait(&job, 1);
    assert_eq!(failed.state, JobState::NeedsInput);
    let failure = failed.failure.unwrap();
    assert_eq!(failure.code, ErrorCode::SourceExpired);
    assert_eq!(failure.details["completed"], json!(false));
    assert_eq!(case.staging(0).last().unwrap(), "manifest.json");
    assert_eq!(case.published(), 0);

    case.fail_verify.store(false, Ordering::SeqCst);
    case.target_exists.store(true, Ordering::SeqCst);
    let job = case.submit("existing-output");
    let conflict = case.wait(&job, 1);
    assert_eq!(conflict.state, JobState::NeedsInput);
    assert!(!conflict.publication_started);
    let failure = conflict.failure.unwrap();
    assert_eq!(failure.code, ErrorCode::RevisionConflict);
    assert_eq!(failure.details["completed"], json!(false));
    assert_eq!(failure.details["bundle_recovery"]["staging"], json!(1));
    assert_eq!(case.published(), 0);
}

#[test]
fn missing_original_attempt_identity_is_refused_before_any_staging() {
    struct Unbound;
    impl ExecutionControl for Unbound {
        fn cancellation_requested(&self) -> bool {
            false
        }
        fn deadline(&self) -> Instant {
            Instant::now() + Duration::from_secs(30)
        }
        fn reserve_publication(&self) -> Result<(), ApiError> {
            panic!("no publication without identity")
        }
        fn begin_verification(&self) -> Result<(), ApiError> {
            panic!("no verification without identity")
        }
    }
    let ledger = Arc::new(Mutex::new(Ledger::default()));
    let publication = Publication {
        ledger: ledger.clone(),
        fail_verify: Arc::new(AtomicBool::new(false)),
        target_exists: Arc::new(AtomicBool::new(false)),
    };
    let raster = Raster {
        fail_at: Arc::new(Mutex::new(None)),
        rendered: Arc::new(Mutex::new(Vec::new())),
        pause: None,
    };
    let refused = execute_bundle(&prepared(), &raster, &publication, &Unbound).unwrap_err();
    assert_eq!(refused.code, ErrorCode::Unsupported);
    assert!(ledger.lock().unwrap().stagings.is_empty());
    assert!(raster.rendered.lock().unwrap().is_empty());
}
