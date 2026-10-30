#![cfg(windows)]
//! Synthetic ownership checkpoint; not provider/decoder/product CLI acceptance.
#[path = "support/download_owned_module.rs"]
// Match the existing media_download target's public transfer-contract mount.
// Legacy synchronous consumers remain in that separate foundation target.
pub mod download;
#[path = "support/retaining_job_host.rs"]
mod host_closeout;
#[path = "../src/infrastructure/immutable_storage_paths.rs"]
#[allow(
    dead_code,
    reason = "Unchanged shared storage guard has unrelated helpers"
)]
pub(crate) mod immutable_storage_paths;
#[path = "../src/infrastructure/download/media_probe.rs"]
pub(crate) mod media_probe;
#[path = "support/download_real_probe.rs"]
mod real_probe;
#[path = "../src/infrastructure/download/staged_output.rs"]
pub(crate) mod staged_output;
#[path = "../src/infrastructure/storage_directory_identity.rs"]
pub(crate) mod storage_directory_identity;
mod application {
    pub(crate) use crate::download;
    pub use kinekino_player::application::{api, jobs};
}
mod infrastructure {
    pub(crate) use crate::{immutable_storage_paths, storage_directory_identity};
    pub(crate) use kinekino_player::infrastructure::content_digest;
    pub(crate) use kinekino_player::infrastructure::os_policy;
    pub(crate) mod download {
        pub(crate) use crate::media_probe;
    }
}
mod domain {
    pub use kinekino_player::domain::identifiers;
}
use application::{
    api::{ApiError, ErrorCode},
    jobs::{CancelJob, GetJob, JobsApi, SubmitJob, WatchJobs},
};
use download::{
    DownloadPreparation, DownloadSession, DownloadVerifier, MediaIdentity, OwnedVerification,
    PendingMediaVerification, PreparedDownload, SourceOpener, TransferEvidence, VerificationLease,
    VerificationPoll, owned_session::DownloadOwnedExecutor,
};
use kinekino_jobs::{
    application::{
        ExecutionControl, JobExecutor, SourceBlobStore,
        owned_attempt::{AttemptScope, OwnedExecutorPort, OwnerObservation},
    },
    domain::{JobId, JobRecord, JobState, PreparedJobInput, VerifiedJobOutput},
};
use kinekino_media_contracts::api_values::DecimalU64;
use media_probe::{ExpectedTrack, MediaProbe, ProbeContainer, ProbePlan, ProbeToolPin, TrackKind};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    os::windows::fs::OpenOptionsExt,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

fn identity() -> MediaIdentity {
    MediaIdentity {
        media: "fixture-media".into(),
        tracks: "video-0-audio-1".into(),
    }
}
fn unsupported() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Synthetic owned test has no legacy execution",
    )
}
struct Source {
    evidence: TransferEvidence,
    offset: usize,
    closed: Arc<AtomicBool>,
}
impl DownloadSession for Source {
    fn evidence(&self) -> &TransferEvidence {
        &self.evidence
    }
    fn check_current(&self, _: &dyn ExecutionControl) -> Result<(), ApiError> {
        Ok(())
    }
    fn read(&mut self, bytes: &mut [u8], _: &dyn ExecutionControl) -> Result<usize, ApiError> {
        let count = bytes.len().min(1024 - self.offset);
        bytes[..count].fill(7);
        self.offset += count;
        Ok(count)
    }
    fn close_owned(&mut self) -> Result<(), ApiError> {
        self.closed.store(true, Ordering::Release);
        Ok(())
    }
}
struct Opener {
    closed: Arc<AtomicBool>,
}
impl SourceOpener for Opener {
    fn owner_observation(&self) -> kinekino_jobs::application::owned_attempt::OwnerObservation {
        kinekino_jobs::application::owned_attempt::OwnerObservation::NotStarted
    }
    fn open(
        &mut self,
        _: &dyn ExecutionControl,
    ) -> Result<Box<dyn DownloadSession + Send>, ApiError> {
        Ok(Box::new(Source {
            evidence: TransferEvidence {
                identity: identity(),
                response_length: Some(1024),
                trusted_sha256: None,
            },
            offset: 0,
            closed: self.closed.clone(),
        }))
    }
    fn poll_close(&mut self, _: Instant) -> Result<bool, ApiError> {
        Ok(true)
    }
}
struct Gate {
    release: AtomicBool,
    unwound: AtomicBool,
    inject: bool,
}
struct GatedProbe {
    inner: MediaProbe,
    gate: Arc<Gate>,
}
struct PendingProbe {
    inner: Box<dyn PendingMediaVerification>,
    gate: Arc<Gate>,
}
impl DownloadVerifier for GatedProbe {
    fn verify(
        &self,
        _: &Path,
        _: &MediaIdentity,
        _: &dyn ExecutionControl,
    ) -> Result<(), ApiError> {
        Err(unsupported())
    }
    fn begin_owned(
        &self,
        lease: Arc<dyn VerificationLease>,
        selected: MediaIdentity,
        control: &dyn ExecutionControl,
    ) -> Result<OwnedVerification, ApiError> {
        match self.inner.begin_owned(lease, selected, control)? {
            OwnedVerification::Pending(inner) => {
                Ok(OwnedVerification::Pending(Box::new(PendingProbe {
                    inner,
                    gate: self.gate.clone(),
                })))
            }
            value => Ok(value),
        }
    }
}
impl PendingMediaVerification for PendingProbe {
    fn owner_observation(&self) -> OwnerObservation {
        self.inner.owner_observation()
    }
    fn request_cancel(&mut self) -> Result<(), ApiError> {
        self.inner.request_cancel()
    }
    fn first_error(&self) -> Option<&ApiError> {
        self.inner.first_error()
    }
    fn poll(&mut self, control: &dyn ExecutionControl) -> VerificationPoll {
        if self.gate.unwound.load(Ordering::Acquire) && !self.gate.release.load(Ordering::Acquire) {
            return VerificationPoll::Pending;
        }
        let result = self.inner.poll(control);
        if self.gate.inject && !self.gate.unwound.swap(true, Ordering::AcqRel) {
            assert!(matches!(result, VerificationPoll::Pending));
            assert!(
                self.inner.first_error().is_none(),
                "Launch failed before injected unwind"
            );
            panic!("injected run unwind after original probe launch");
        }
        result
    }
}
struct Preparation {
    root: PathBuf,
    tool: Arc<ProbeToolPin>,
    gate: Arc<Gate>,
    closed: Arc<AtomicBool>,
}
impl DownloadPreparation for Preparation {
    fn prepare(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        _: &AttemptScope<'_>,
    ) -> Result<PreparedDownload, ApiError> {
        if blobs.read_utf8(&input.source)? != "synthetic-original-download-v1" {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original fixture input changed",
            ));
        }
        Ok(PreparedDownload {
            source: Box::new(Opener {
                closed: self.closed.clone(),
            }),
            output: Arc::new(staged_output::StagedDownloadOutput::new(
                &self.root.join("output"),
                "result.mp4",
            )?),
            verifier: Arc::new(GatedProbe {
                gate: self.gate.clone(),
                inner: MediaProbe {
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
                        minimum_duration_us: 900_000,
                        maximum_duration_us: 1_100_000,
                    }),
                    evidence_root: self.root.join("probe"),
                    cleanup_reserve: Duration::from_secs(2),
                    sequence: AtomicU64::new(0),
                },
            }),
            maximum_bytes: 1024,
        })
    }
}
struct Executor {
    owned: DownloadOwnedExecutor,
}
impl JobExecutor for Executor {
    fn owned_executor(&self, operation: &str) -> Option<&OwnedExecutorPort> {
        self.supports(operation)
            .then_some(&self.owned as &OwnedExecutorPort)
    }
    fn operations(&self) -> Vec<String> {
        vec!["production.download_media".into()]
    }
    fn supports(&self, operation: &str) -> bool {
        operation == "production.download_media"
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        if !self.supports(operation) || expected.is_some() {
            return Err(unsupported());
        }
        Ok(PreparedJobInput {
            operation: operation.into(),
            parameters,
            source: blobs.put_utf8("synthetic-original-download-v1")?,
            execution_timeout_ms: DecimalU64(15_000),
        })
    }
    fn execute(
        &self,
        _: &PreparedJobInput,
        _: &dyn SourceBlobStore,
        _: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        Err(unsupported())
    }
}
fn wait_until(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(8);
    while !predicate() {
        assert!(
            Instant::now() < deadline,
            "Fixture observation expired; preserve original evidence"
        );
        thread::sleep(Duration::from_millis(2));
    }
}
fn terminal(client: &dyn JobsApi, id: &JobId) -> JobRecord {
    wait_until(|| {
        client
            .get(GetJob { job_id: id.clone() })
            .unwrap()
            .state
            .terminal()
    });
    client.get(GetJob { job_id: id.clone() }).unwrap()
}
fn descendants(root: &Path, name: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            found.extend(descendants(&entry.path(), name));
        } else if entry.file_name() == name {
            found.push(entry.path());
        }
    }
    found
}
struct ProbeFixture {
    bin: PathBuf,
    held_bin: PathBuf,
    sha256: String,
    bytes: u64,
    evidence_root: PathBuf,
}

/// Explicit allocation (`DOWNLOAD_PROBE_*`, `DOWNLOAD_OWNED_EVIDENCE_ROOT`) wins; otherwise
/// the std-only child is compiled once per run into a fresh directory with the same rustc.
fn fixture() -> &'static ProbeFixture {
    static FIXTURE: std::sync::OnceLock<ProbeFixture> = std::sync::OnceLock::new();
    FIXTURE.get_or_init(|| {
        if let Some(bin) = std::env::var_os("DOWNLOAD_PROBE_BIN") {
            return ProbeFixture {
                bin: PathBuf::from(bin),
                held_bin: PathBuf::from(
                    std::env::var_os("DOWNLOAD_PROBE_HELD_BIN")
                        .expect("Fresh allocated fixture BIN required"),
                ),
                sha256: std::env::var("DOWNLOAD_PROBE_SHA256").expect("Fresh fixture SHA required"),
                bytes: std::env::var("DOWNLOAD_PROBE_BYTES")
                    .expect("Fresh fixture size required")
                    .parse()
                    .unwrap(),
                evidence_root: PathBuf::from(
                    std::env::var_os("DOWNLOAD_OWNED_EVIDENCE_ROOT")
                        .expect("Allocated evidence root required"),
                ),
            };
        }
        let root = std::env::temp_dir().join(format!(
            "download-owned-probe-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let bin = root.join("download-probe.exe");
        let source =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/download_probe_child.rs");
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let status = std::process::Command::new(rustc)
            .args(["--edition", "2024", "-o"])
            .arg(&bin)
            .arg(&source)
            .status()
            .expect("rustc must be runnable to allocate the probe child");
        assert!(status.success(), "probe child failed to compile");
        let held_bin = root.join("download-probe-held.exe");
        fs::copy(&bin, &held_bin).unwrap();
        let bytes = fs::read(&bin).unwrap();
        let evidence_root = root.join("evidence");
        fs::create_dir(&evidence_root).unwrap();
        ProbeFixture {
            bin,
            held_bin,
            sha256: format!("{:x}", Sha256::digest(&bytes)),
            bytes: bytes.len() as u64,
            evidence_root,
        }
    })
}

enum Case {
    Normal,
    ExistingTarget,
    DroppedClient,
    RunUnwind,
}
fn exercise(case: Case) {
    // No fallback to an old/cached binary: either the caller allocates all three pins, or
    // this run compiles a fresh child from source and pins exactly what it produced.
    let held = matches!(case, Case::DroppedClient | Case::RunUnwind);
    let fixture = fixture();
    let executable = if held {
        &fixture.held_bin
    } else {
        &fixture.bin
    };
    let actual = fs::read(executable).unwrap();
    assert_eq!(actual.len() as u64, fixture.bytes);
    assert_eq!(format!("{:x}", Sha256::digest(&actual)), fixture.sha256);
    let executable = executable.clone();
    let base = fixture.evidence_root.clone();
    let label = match case {
        Case::Normal => "normal",
        Case::ExistingTarget => "existing-target",
        Case::DroppedClient => "client-drop",
        Case::RunUnwind => "run-unwind",
    };
    let root = base.join(label);
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("output")).unwrap();
    fs::create_dir(root.join("probe")).unwrap();
    fs::create_dir(root.join("jobs")).unwrap();
    if matches!(case, Case::ExistingTarget) {
        fs::write(
            root.join("output/result.mp4"),
            b"original destination must survive",
        )
        .unwrap();
    }
    let gate = Arc::new(Gate {
        release: AtomicBool::new(false),
        unwound: AtomicBool::new(false),
        inject: matches!(case, Case::RunUnwind),
    });
    let closed = Arc::new(AtomicBool::new(false));
    let executor = Arc::new(Executor {
        owned: DownloadOwnedExecutor {
            preparation: Arc::new(Preparation {
                root: root.clone(),
                tool: Arc::new(ProbeToolPin {
                    executable,
                    sha256: fixture.sha256.clone(),
                    bytes: fixture.bytes,
                }),
                gate: gate.clone(),
                closed: closed.clone(),
            }),
        },
    });
    let host = kinekino_player::infrastructure::jobs::start_retaining_host_with_executor(
        &root.join("jobs"),
        executor,
    )
    .unwrap();
    let action = catch_unwind(AssertUnwindSafe(|| {
        let client = host.client();
        let admitted = client
            .submit(
                SubmitJob {
                    operation: "production.download_media".into(),
                    parameters: json!({}),
                    expected_source_sha256: None,
                    execution_timeout_ms: DecimalU64(15_000),
                },
                label,
            )
            .unwrap();
        let id = admitted.job.job_id.clone();
        if held {
            wait_until(|| descendants(&root.join("probe"), "probe.ready").len() == 1);
            assert!(!closed.load(Ordering::Acquire));
            let parts: Vec<_> = fs::read_dir(root.join("output"))
                .unwrap()
                .map(|e| e.unwrap().path())
                .collect();
            assert_eq!(parts.len(), 1);
            let denied = OpenOptions::new()
                .write(true)
                .share_mode(7)
                .open(&parts[0])
                .unwrap_err();
            assert_eq!(
                denied.raw_os_error(),
                Some(32),
                "Original sealed writer lease must deny mutation"
            );
            assert!(!root.join("output/result.mp4").exists());
            if matches!(case, Case::RunUnwind) {
                assert!(gate.unwound.load(Ordering::Acquire));
            }
            drop(client);
            let replacement = host.client();
            let pending = replacement.get(GetJob { job_id: id.clone() }).unwrap();
            assert!(!pending.state.terminal());
            assert!(!pending.publication_started);
            if matches!(case, Case::DroppedClient) {
                replacement
                    .cancel(CancelJob {
                        job_id: id.clone(),
                        expected_attempt: pending.attempt,
                    })
                    .unwrap();
            }
            gate.release.store(true, Ordering::Release);
            let failed = terminal(replacement.as_ref(), &id);
            assert_ne!(failed.state, JobState::Completed);
            assert!(failed.output.is_none());
            assert!(!failed.publication_started);
            assert!(closed.load(Ordering::Acquire));
            assert_eq!(fs::read_dir(root.join("output")).unwrap().count(), 0);
        } else if matches!(case, Case::ExistingTarget) {
            let failed = terminal(client.as_ref(), &id);
            assert_ne!(failed.state, JobState::Completed);
            assert!(failed.output.is_none());
            assert_eq!(
                fs::read(root.join("output/result.mp4")).unwrap(),
                b"original destination must survive"
            );
            assert_eq!(fs::read_dir(root.join("output")).unwrap().count(), 1);
        } else {
            let finished = terminal(client.as_ref(), &id);
            assert_eq!(finished.state, JobState::Completed);
            assert!(finished.publication_started);
            assert_eq!(finished.attempt, admitted.job.attempt);
            let output = finished.output.as_ref().unwrap();
            let data = fs::read(&output.path).unwrap();
            assert_eq!(data, vec![7; 1024]);
            assert_eq!(output.sha256, format!("{:x}", Sha256::digest(&data)));
            assert_eq!(output.bytes, DecimalU64(1024));
            assert!(closed.load(Ordering::Acquire));
            let page = client
                .watch(WatchJobs {
                    after_sequence: DecimalU64(0),
                    limit: Some(128),
                })
                .unwrap();
            let records: Vec<_> = page
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
            let transitions = records
                .windows(2)
                .filter(|pair| !pair[0].job.publication_started && pair[1].job.publication_started)
                .count();
            assert_eq!(transitions, 1, "Exactly one durable publication transition");
            let mut marker = false;
            for path in descendants(&root.join("probe"), "stdout.raw") {
                let raw = fs::read(path).unwrap();
                if let Ok(value) = serde_json::from_slice::<Value>(&raw) {
                    marker |= value["fixture_original_stdin_seek"] == true
                        && value["fixture_original_stdin_eof"] == true;
                }
            }
            assert!(
                marker,
                "Original inherited stdin seek/EOF raw evidence required"
            );
        }
    }));
    gate.release.store(true, Ordering::Release);
    let close_failure = host_closeout::close(host).err();
    host_closeout::conclude(action, close_failure);
}
#[test]
fn original_stdin_seek_eof_and_single_core_publication() {
    kinekino_platform::install();
    exercise(Case::Normal);
}
#[test]
fn rejected_candidate_preserves_existing_destination() {
    kinekino_platform::install();
    exercise(Case::ExistingTarget);
}
#[test]
fn dropped_client_retains_original_child_lease_until_cancel_close() {
    kinekino_platform::install();
    exercise(Case::DroppedClient);
}
#[test]
fn run_unwind_retains_original_child_lease_until_same_owner_close() {
    kinekino_platform::install();
    exercise(Case::RunUnwind);
}
