//! Fixture adapters only. Production jobs/core/output/probe/CLI policies are reused.
use super::*;
use application::{
    api::{ApiError, PlayerSdk, Request},
    download::{
        DownloadPreparation, DownloadSession, MediaIdentity, OwnedTransferControl,
        OwnedTransferError, PreparedDownload, SourceOpener, TransferEvidence,
    },
    jobs::{ExecutionControl, GetJob, JobState, SourceBlobStore, owned_attempt::AttemptScope},
};
use kinekino_jobs::domain::PreparedJobInput;
use kinekino_player::infrastructure::{
    jobs,
    local_host::{LocalHost, TransportOptions},
    subtitle_files::SubtitleFiles,
};
use media_probe::{ExpectedTrack, MediaProbe, ProbeContainer, ProbePlan, ProbeToolPin, TrackKind};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

const MANIFEST_SHA: &str = "c90b2c168ea25c78889531008d89dfed79c7cd60cb3ef6ac733b23b8b1275c25";
const TOOL_SHA: &str = "efb014c8f39091c90f3616154fe20ab8470f5f943f6a48ec4fd30c479c7dd697";
#[derive(Clone, Copy)]
pub(super) enum Case {
    Normal,
    Truncated,
    Cancel,
    Existing,
    Unadmitted,
}
struct Signals {
    reading: AtomicBool,
    closed: AtomicBool,
    cutoff: Mutex<Option<Instant>>,
}
struct Source {
    file: fs::File,
    _guards: Vec<fs::File>,
    evidence: TransferEvidence,
    remaining: u64,
    case: Case,
    signals: Arc<Signals>,
}
impl DownloadSession for Source {
    fn evidence(&self) -> &TransferEvidence {
        &self.evidence
    }
    fn check_current(&self, _: &dyn ExecutionControl) -> Result<(), ApiError> {
        Ok(())
    }
    fn read(
        &mut self,
        bytes: &mut [u8],
        control: &dyn ExecutionControl,
    ) -> Result<usize, ApiError> {
        if control.cancellation_requested() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Local legacy read cancelled",
            ));
        }
        let limit = bytes.len().min(self.remaining as usize);
        let count = self.file.read(&mut bytes[..limit]).map_err(io_error)?;
        self.remaining -= count as u64;
        Ok(count)
    }
    fn read_owned(
        &mut self,
        bytes: &mut [u8],
        control: &dyn OwnedTransferControl,
    ) -> Result<usize, OwnedTransferError> {
        self.signals.reading.store(true, Ordering::Release);
        if matches!(self.case, Case::Cancel) {
            loop {
                control.checkpoint()?;
                std::thread::sleep(Duration::from_millis(2));
            }
        }
        control.checkpoint()?;
        self.read(bytes, control)
            .map_err(OwnedTransferError::Failure)
    }
    fn close_owned(&mut self) -> Result<(), ApiError> {
        self.signals.closed.store(true, Ordering::Release);
        Ok(())
    }
}
struct Opener(Option<Source>);
impl SourceOpener for Opener {
    fn owner_observation(&self) -> kinekino_jobs::application::owned_attempt::OwnerObservation {
        kinekino_jobs::application::owned_attempt::OwnerObservation::NotStarted
    }
    fn open(
        &mut self,
        _: &dyn ExecutionControl,
    ) -> Result<Box<dyn DownloadSession + Send>, ApiError> {
        self.0
            .take()
            .map(|source| Box::new(source) as Box<dyn DownloadSession + Send>)
            .ok_or_else(|| ApiError::new(ErrorCode::SourceExpired, "Original local source spent"))
    }
    fn poll_close(&mut self, _: Instant) -> Result<bool, ApiError> {
        self.0.take();
        Ok(true)
    }
}
struct Preparation {
    media: PathBuf,
    media_sha: String,
    bytes: u64,
    tool: Arc<ProbeToolPin>,
    root: PathBuf,
    case: Case,
    signals: Arc<Signals>,
}
fn identity() -> MediaIdentity {
    MediaIdentity {
        media: "fixture-local".into(),
        tracks: "h264+aac".into(),
    }
}
fn io_error(error: std::io::Error) -> ApiError {
    let mut e = ApiError::new(ErrorCode::EngineFailed, "Local CLI fixture IO failed");
    e.details = json!({"io_kind":format!("{:?}",error.kind()),"raw_os_error":error.raw_os_error()});
    e
}
impl DownloadPreparation for Preparation {
    fn prepare(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> Result<PreparedDownload, ApiError> {
        let request = DownloadRequest::from_prepared(input, blobs)?;
        if request.source.selection_id != "fixture-local"
            || request.source.descriptor_sha256 != MANIFEST_SHA
            || request.destination_id != "fixture-output"
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "No original fixture selection matches request",
            ));
        }
        *self.signals.cutoff.lock().unwrap() = Some(scope.deadline());
        let (_, guards) = immutable_storage_paths::root(self.media.parent().unwrap())?;
        let mut file = immutable_storage_paths::file(&self.media)?;
        let pin = kinekino_player::infrastructure::content_digest::sha256(&mut file, self.bytes)?;
        if pin != (self.bytes, self.media_sha.clone()) {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original local media changed",
            ));
        }
        file.seek(SeekFrom::Start(0)).map_err(io_error)?;
        let length = if matches!(self.case, Case::Truncated) {
            16
        } else {
            self.bytes
        };
        Ok(PreparedDownload {
            source: Box::new(Opener(Some(Source {
                file,
                _guards: guards,
                evidence: TransferEvidence {
                    identity: identity(),
                    response_length: Some(length),
                    trusted_sha256: None,
                },
                remaining: length,
                case: self.case,
                signals: self.signals.clone(),
            }))),
            output: Arc::new(
                kinekino_player::infrastructure::download::StagedDownloadOutput::new(
                    &self.root.join("output"),
                    "result.media",
                )?,
            ),
            verifier: Arc::new(MediaProbe {
                tool: self.tool.clone(),
                plan: Arc::new(ProbePlan {
                    identity: identity(),
                    container: ProbeContainer::Mp4,
                    tracks: vec![
                        ExpectedTrack {
                            index: 0,
                            kind: TrackKind::Video,
                            codec: Some("h264".into()),
                        },
                        ExpectedTrack {
                            index: 1,
                            kind: TrackKind::Audio,
                            codec: Some("aac".into()),
                        },
                    ],
                    minimum_duration_us: 12_000_000,
                    maximum_duration_us: 12_000_000,
                }),
                evidence_root: self.root.join("probe"),
                cleanup_reserve: Duration::from_secs(2),
                sequence: AtomicU64::new(0),
            }),
            maximum_bytes: application::download::MAX_DOWNLOAD_BYTES,
        })
    }
}
fn pin(path: &Path, sha: &str, bytes: u64) -> Vec<fs::File> {
    let (_, mut guards) = immutable_storage_paths::root(path.parent().unwrap()).unwrap();
    let mut file = immutable_storage_paths::file(path).unwrap();
    assert_eq!(
        kinekino_player::infrastructure::content_digest::sha256(&mut file, bytes).unwrap(),
        (bytes, sha.to_owned())
    );
    guards.push(file);
    guards
}
fn rpc(
    cli: &Path,
    endpoint: &str,
    root: &Path,
    number: &mut u64,
    command: &str,
    payload: Value,
    key: Option<&str>,
) -> Value {
    *number += 1;
    let evidence = root.join(format!("rpc-{number}"));
    fs::create_dir(&evidence).unwrap();
    let mut request = Request::new(command);
    request.payload = payload;
    request.context.request_id = format!("download-cli-{number}");
    request.context.idempotency_key = key.map(str::to_owned);
    let input = serde_json::to_vec(&request).unwrap();
    let result = owned_process_evidence::run_process_with_arguments(
        cli.into(),
        &evidence,
        &input,
        vec![
            "--input".into(),
            "-".into(),
            "--endpoint".into(),
            endpoint.into(),
        ],
        Duration::from_secs(15),
        true,
    );
    let output = match result {
        Ok(output) => output,
        Err(failure) => {
            let failure = failure.closeout();
            let observation = failure.observation();
            Box::leak(failure);
            panic!("Original CLI failure retained: {observation}");
        }
    };
    assert_eq!(
        output.receipt.family,
        kinekino_process::domain::observation::FamilyObservation::Empty
    );
    assert!(!output.receipt.forced);
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["request_id"], request.context.request_id);
    let expected_exit = if response["ok"] == true {
        0
    } else {
        serde_json::from_value::<ApiError>(response["error"].clone())
            .unwrap()
            .exit_code()
    };
    assert_eq!(output.receipt.child_exit, Some(u32::from(expected_exit)));
    response
}
pub(super) fn exercise(case: Case) {
    let env = |name| {
        let path = PathBuf::from(
            std::env::var_os(name).expect("Allocated absolute fixture path required"),
        );
        assert!(path.is_absolute());
        path
    };
    let label = match case {
        Case::Normal => "normal",
        Case::Truncated => "truncated",
        Case::Cancel => "cancel",
        Case::Existing => "existing",
        Case::Unadmitted => "unadmitted",
    };
    let root = env("DOWNLOAD_CLI_EVIDENCE_ROOT").join(label);
    fs::create_dir(&root).unwrap();
    for dir in ["jobs", "output", "probe"] {
        fs::create_dir(root.join(dir)).unwrap();
    }
    let cli = env("DOWNLOAD_CLI_BIN");
    let cli_sha = std::env::var("DOWNLOAD_CLI_SHA256").unwrap();
    let cli_bytes = std::env::var("DOWNLOAD_CLI_BYTES")
        .unwrap()
        .parse()
        .unwrap();
    let _cli_guards = pin(&cli, &cli_sha, cli_bytes);
    let manifest = env("DOWNLOAD_REAL_MEDIA_MANIFEST");
    let raw = fs::read(&manifest).unwrap();
    assert!(raw.len() <= 4096);
    assert_eq!(format!("{:x}", Sha256::digest(&raw)), MANIFEST_SHA);
    let fixture: Value = serde_json::from_slice(&raw).unwrap();
    let media = PathBuf::from(fixture["path"].as_str().unwrap());
    let bytes = fixture["bytes"].as_u64().unwrap();
    let sha = fixture["sha256"].as_str().unwrap().to_owned();
    let tool = Arc::new(ProbeToolPin {
        executable: env("DOWNLOAD_REAL_FFPROBE_BIN"),
        sha256: TOOL_SHA.into(),
        bytes: 82_620_928,
    });
    let _tool_guards = pin(&tool.executable, TOOL_SHA, tool.bytes);
    let signals = Arc::new(Signals {
        reading: AtomicBool::new(false),
        closed: AtomicBool::new(false),
        cutoff: Mutex::new(None),
    });
    if matches!(case, Case::Existing) {
        fs::write(root.join("output/result.media"), b"original destination").unwrap();
    }
    let preparation = Arc::new(Preparation {
        media,
        media_sha: sha.clone(),
        bytes,
        tool,
        root: root.clone(),
        case,
        signals: signals.clone(),
    });
    let admitted = if matches!(case, Case::Unadmitted) {
        None
    } else {
        Some(preparation as Arc<dyn DownloadPreparation>)
    };
    let executor = download_registry(vec![jobs::subtitle_executor()], admitted).unwrap();
    let host = jobs::start_retaining_host_with_executor(&root.join("jobs"), executor).unwrap();
    let local = match LocalHost::start(
        Arc::new(PlayerSdk::new(SubtitleFiles).with_jobs(host.client())),
        TransportOptions::default(),
    ) {
        Ok(local) => local,
        Err(error) => {
            let close = host_closeout::close(host).err();
            host_closeout::conclude(Err(Box::new(format!("Local host start: {error:?}"))), close);
            unreachable!()
        }
    };
    let action = catch_unwind(AssertUnwindSafe(|| {
        let mut number = 0;
        let endpoint = local.endpoint();
        let capabilities = rpc(
            &cli,
            endpoint,
            &root,
            &mut number,
            "capabilities",
            json!({}),
            None,
        );
        assert_eq!(capabilities["ok"], true);
        let registered = capabilities["data"]["jobs"]["operations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|op| op == DOWNLOAD_OPERATION);
        assert_eq!(registered, !matches!(case, Case::Unadmitted));
        let schema = rpc(
            &cli,
            endpoint,
            &root,
            &mut number,
            "schema",
            json!({"command":"jobs.submit"}),
            None,
        );
        assert_eq!(schema["ok"], true);
        let payload_schema = &schema["data"]["request"]["properties"]["payload"];
        let variants = payload_schema["oneOf"]
            .as_array()
            .cloned()
            .unwrap_or_else(|| vec![payload_schema.clone()]);
        let download_schema = variants
            .iter()
            .find(|variant| variant["properties"]["operation"]["const"] == DOWNLOAD_OPERATION);
        assert_eq!(download_schema.is_some(), registered);
        if let Some(schema) = download_schema {
            assert_eq!(
                schema["properties"]["expected_source_sha256"]["type"],
                "null"
            );
            assert_eq!(
                schema["properties"]["parameters"],
                DownloadRequest::schema()
            );
            assert_eq!(
                schema["properties"]["execution_timeout_ms"]["default"],
                "15000"
            );
        }
        let submitted = rpc(
            &cli,
            endpoint,
            &root,
            &mut number,
            "jobs.submit",
            json!({"operation":DOWNLOAD_OPERATION,"parameters":{"source":{"selection_id":"fixture-local","descriptor_sha256":MANIFEST_SHA},"destination_id":"fixture-output","maximum_bytes":bytes.to_string()},"expected_source_sha256":null,"execution_timeout_ms":"15000"}),
            Some(label),
        );
        if matches!(case, Case::Unadmitted) {
            assert_eq!(submitted["ok"], false);
            let error: ApiError = serde_json::from_value(submitted["error"].clone()).unwrap();
            assert_eq!(error.code, ErrorCode::Unsupported);
            assert!(!signals.reading.load(Ordering::Acquire));
            assert_eq!(fs::read_dir(root.join("output")).unwrap().count(), 0);
            return;
        }
        assert_eq!(submitted["ok"], true);
        let id: application::jobs::JobId = submitted["data"]["job"]["job_id"]
            .as_str()
            .unwrap()
            .to_owned()
            .try_into()
            .unwrap();
        let admission_cutoff = Instant::now() + Duration::from_secs(8);
        loop {
            let current = host.client().get(GetJob { job_id: id.clone() }).unwrap();
            let cutoff = signals.cutoff.lock().unwrap().unwrap_or(admission_cutoff);
            if matches!(case, Case::Cancel) && signals.reading.load(Ordering::Acquire) {
                let cancel = rpc(
                    &cli,
                    endpoint,
                    &root,
                    &mut number,
                    "jobs.cancel",
                    json!({"job_id":id,"expected_attempt":current.attempt}),
                    None,
                );
                assert_eq!(cancel["ok"], true);
                break;
            }
            if current.state.terminal() {
                break;
            }
            assert!(Instant::now() < cutoff, "Original attempt cutoff expired");
            std::thread::sleep(Duration::from_millis(2));
        }
        loop {
            let current = host.client().get(GetJob { job_id: id.clone() }).unwrap();
            if current.state.terminal() {
                break;
            }
            let cutoff = signals.cutoff.lock().unwrap().unwrap_or(admission_cutoff);
            assert!(Instant::now() < cutoff, "Original cutoff expired");
            std::thread::sleep(Duration::from_millis(2));
        }
        let result = rpc(
            &cli,
            endpoint,
            &root,
            &mut number,
            "jobs.get",
            json!({"job_id":id}),
            None,
        );
        assert_eq!(result["ok"], true);
        let record: application::jobs::JobRecord =
            serde_json::from_value(result["data"].clone()).unwrap();
        assert!(signals.closed.load(Ordering::Acquire));
        match case {
            Case::Unadmitted => unreachable!("Rejected before job admission"),
            Case::Normal => {
                assert_eq!(record.state, JobState::Completed);
                let events = rpc(
                    &cli,
                    endpoint,
                    &root,
                    &mut number,
                    "jobs.watch",
                    json!({"after_sequence":"0","limit":128}),
                    None,
                );
                assert_eq!(events["ok"], true);
                let events: application::jobs::EventPage =
                    serde_json::from_value(events["data"].clone()).unwrap();
                let records: Vec<_> = events
                    .events
                    .iter()
                    .filter(|event| event.job.job_id == id)
                    .collect();
                assert_eq!(
                    records
                        .iter()
                        .filter(|event| event.job.state == JobState::Completed)
                        .count(),
                    1
                );
                assert_eq!(
                    records
                        .windows(2)
                        .filter(|pair| !pair[0].job.publication_started
                            && pair[1].job.publication_started)
                        .count(),
                    1
                );
                let output = record.output.unwrap();
                assert_eq!(output.sha256, sha);
                assert_eq!(output.bytes.0, bytes);
                assert_eq!(
                    format!("{:x}", Sha256::digest(fs::read(&output.path).unwrap())),
                    sha
                );
            }
            Case::Cancel => {
                assert_eq!(record.state, JobState::Cancelled);
                assert!(!record.publication_started);
                assert!(record.output.is_none());
                assert_eq!(
                    fs::read_dir(root.join("probe")).unwrap().count(),
                    0,
                    "Source-stage cancellation is not ffprobe child cancellation"
                );
            }
            Case::Truncated => {
                assert_eq!(record.state, JobState::Failed);
                assert_eq!(
                    record.failure.as_ref().unwrap().code,
                    ErrorCode::EngineFailed
                );
                assert!(record.output.is_none());
                assert!(!record.publication_started);
            }
            Case::Existing => {
                assert_eq!(record.state, JobState::NeedsInput);
                assert_eq!(
                    record.failure.as_ref().unwrap().code,
                    ErrorCode::RevisionConflict
                );
                assert!(record.output.is_none());
                assert!(
                    record.publication_started,
                    "Core reserves publication before non-overwrite rename rejects the existing target"
                );
            }
        }
        if matches!(case, Case::Existing) {
            assert_eq!(
                fs::read(root.join("output/result.media")).unwrap(),
                b"original destination"
            );
        } else if !matches!(case, Case::Normal) {
            assert_eq!(fs::read_dir(root.join("output")).unwrap().count(), 0);
        }
    }));
    let mut local_owner = local.begin_shutdown();
    let cutoff = Instant::now() + Duration::from_secs(5);
    while !local_owner.poll().is_quiescent() {
        if Instant::now() >= cutoff {
            Box::leak(Box::new(local_owner));
            Box::leak(Box::new(host));
            if let Err(panic) = action {
                resume_unwind(panic);
            }
            panic!("Original local/SDK owners retained; incomplete cleanup");
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let close = host_closeout::close(host).err();
    host_closeout::conclude(action, close);
}
