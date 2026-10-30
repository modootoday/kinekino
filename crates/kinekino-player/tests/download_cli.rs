#![cfg(windows)]
//! Real CLI transport with an explicitly injected LOCAL test source; no provider support claim.
pub use kinekino_player::{application, domain};
#[path = "../src/infrastructure/immutable_storage_paths.rs"]
#[allow(
    dead_code,
    reason = "Shared storage adapter contains unrelated operations"
)]
pub mod immutable_storage_paths;
mod infrastructure {
    pub(crate) use crate::immutable_storage_paths;
    pub use kinekino_player::infrastructure::content_digest;
    pub(crate) use kinekino_player::infrastructure::os_policy;
    pub(crate) mod download {
        pub(crate) use crate::media_probe;
    }
}
#[path = "support/download_cli_host.rs"]
mod fixture;
#[path = "support/retaining_job_host.rs"]
mod host_closeout;
#[path = "../src/infrastructure/download/media_probe.rs"]
pub(crate) mod media_probe;
#[path = "support/owned_process_evidence.rs"]
pub mod owned_process_evidence;
use kinekino_player::application::{
    api::ErrorCode,
    download::{DOWNLOAD_OPERATION, DownloadRequest},
};
use kinekino_player::infrastructure::download::download_registry;
use serde_json::json;

// The mounted helper's unchanged inline tests consume this hash adapter.
pub fn hash(bytes: &[u8]) -> String {
    kinekino_player::infrastructure::content_digest::sha256(
        &mut std::io::Cursor::new(bytes),
        bytes.len() as u64,
    )
    .unwrap()
    .1
}
#[test]
fn absent_factory_does_not_advertise_download() {
    kinekino_platform::install();
    let executor = download_registry(vec![], None).unwrap();
    assert!(!executor.supports(DOWNLOAD_OPERATION));
    assert!(executor.operations().is_empty());
    assert!(executor.owned_executor(DOWNLOAD_OPERATION).is_none());
    assert_eq!(
        executor
            .normalize_parameters(DOWNLOAD_OPERATION, json!({}))
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
}
#[test]
fn request_cannot_supply_url_tool_or_path_authority() {
    kinekino_platform::install();
    let valid = json!({"source":{"selection_id":"fixture-local","descriptor_sha256":"a".repeat(64)},"destination_id":"fixture-output","maximum_bytes":"100"});
    assert!(DownloadRequest::decode(valid.clone()).is_ok());
    for (field, value) in [
        ("url", json!("https://example.invalid/media")),
        ("tool", json!("cmd.exe")),
        ("output_path", json!("C:/outside")),
    ] {
        let mut rejected = valid.clone();
        rejected[field] = value;
        assert_eq!(
            DownloadRequest::decode(rejected).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    let mut rejected = valid;
    rejected["destination_id"] = json!("../outside");
    assert_eq!(
        DownloadRequest::decode(rejected).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
}
#[test]
#[ignore = "Requires allocated pinned real player-cli/ffprobe/local media and fresh evidence"]
fn cli_local_source_uses_shared_executor_and_single_publication() {
    kinekino_platform::install();
    fixture::exercise(fixture::Case::Normal);
}
#[test]
#[ignore = "Requires allocated pinned real player-cli/ffprobe/local media and fresh evidence"]
fn cli_local_truncated_source_is_not_published() {
    kinekino_platform::install();
    fixture::exercise(fixture::Case::Truncated);
}
#[test]
#[ignore = "Requires allocated pinned real player-cli/local media and fresh evidence"]
fn cli_local_source_cancel_retains_owner_until_closed() {
    kinekino_platform::install();
    fixture::exercise(fixture::Case::Cancel);
}
#[test]
#[ignore = "Requires allocated pinned real player-cli/local media and fresh evidence"]
fn cli_existing_destination_keeps_original_bytes() {
    kinekino_platform::install();
    fixture::exercise(fixture::Case::Existing);
}
#[test]
#[ignore = "Requires allocated pinned real player-cli/local fixture paths and fresh evidence"]
fn cli_unadmitted_download_remains_unsupported() {
    kinekino_platform::install();
    fixture::exercise(fixture::Case::Unadmitted);
}

#[test]
fn shared_executor_enforces_request_bound_before_output_or_read() {
    kinekino_platform::install();
    for (host_bound, request_bound) in [(1024, 16), (16, 1024)] {
        exercise_request_bound(host_bound, request_bound);
    }
}

fn exercise_request_bound(host_bound: u64, request_bound: u64) {
    use application::{download::*, jobs::*};
    use kinekino_jobs::domain::PreparedJobInput;
    use std::{
        path::Path,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        time::{Duration, Instant},
    };
    struct State {
        closed: AtomicBool,
        reads: AtomicUsize,
        outputs: AtomicUsize,
        cutoff: Mutex<Option<Instant>>,
    }
    struct Source {
        evidence: TransferEvidence,
        state: Arc<State>,
    }
    impl DownloadSession for Source {
        fn evidence(&self) -> &TransferEvidence {
            &self.evidence
        }
        fn check_current(
            &self,
            _: &dyn ExecutionControl,
        ) -> Result<(), application::api::ApiError> {
            Ok(())
        }
        fn read(
            &mut self,
            _: &mut [u8],
            _: &dyn ExecutionControl,
        ) -> Result<usize, application::api::ApiError> {
            self.state.reads.fetch_add(1, Ordering::AcqRel);
            Err(application::api::ApiError::new(
                ErrorCode::EngineFailed,
                "Read must not precede admitted byte bound",
            ))
        }
        fn close_owned(&mut self) -> Result<(), application::api::ApiError> {
            self.state.closed.store(true, Ordering::Release);
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
        ) -> Result<Box<dyn DownloadSession + Send>, application::api::ApiError> {
            Ok(Box::new(self.0.take().unwrap()))
        }
        fn poll_close(&mut self, _: Instant) -> Result<bool, application::api::ApiError> {
            Ok(true)
        }
    }
    struct Output(Arc<State>);
    impl DownloadOutput for Output {
        fn create(&self) -> Result<Box<dyn PartialOutput>, application::api::ApiError> {
            panic!("Owned executor cannot use synchronous fallback")
        }
        fn create_owned(
            &self,
        ) -> Result<Box<dyn PartialOutput + Send>, application::api::ApiError> {
            self.0.outputs.fetch_add(1, Ordering::AcqRel);
            Err(application::api::ApiError::new(
                ErrorCode::EngineFailed,
                "Output must not precede admitted byte bound",
            ))
        }
    }
    struct Verifier;
    impl DownloadVerifier for Verifier {
        fn verify(
            &self,
            _: &Path,
            _: &MediaIdentity,
            _: &dyn ExecutionControl,
        ) -> Result<(), application::api::ApiError> {
            panic!("Rejected bytes cannot reach verification")
        }
    }
    struct Preparation(Arc<State>, u64);
    impl DownloadPreparation for Preparation {
        fn prepare(
            &self,
            _: &PreparedJobInput,
            _: &dyn SourceBlobStore,
            scope: &owned_attempt::AttemptScope<'_>,
        ) -> Result<PreparedDownload, application::api::ApiError> {
            *self.0.cutoff.lock().unwrap() = Some(scope.deadline());
            Ok(PreparedDownload {
                source: Box::new(Opener(Some(Source {
                    evidence: TransferEvidence {
                        identity: MediaIdentity {
                            media: "bounded".into(),
                            tracks: "bounded".into(),
                        },
                        response_length: Some(32),
                        trusted_sha256: None,
                    },
                    state: self.0.clone(),
                }))),
                output: Arc::new(Output(self.0.clone())),
                verifier: Arc::new(Verifier),
                maximum_bytes: self.1,
            })
        }
    }
    let state = Arc::new(State {
        closed: AtomicBool::new(false),
        reads: AtomicUsize::new(0),
        outputs: AtomicUsize::new(0),
        cutoff: Mutex::new(None),
    });
    let root = std::env::temp_dir().join(format!(
        "download-request-bound-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let executor = download_registry(
        vec![],
        Some(Arc::new(Preparation(state.clone(), host_bound))),
    )
    .unwrap();
    let host =
        kinekino_player::infrastructure::jobs::start_retaining_host_with_executor(&root, executor)
            .unwrap();
    let action = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let client = host.client();
        let submitted = client
            .submit(
                SubmitJob {
                    operation: DOWNLOAD_OPERATION.into(),
                    parameters: json!({"source":{"selection_id":"bounded","descriptor_sha256":"a".repeat(64)},"destination_id":"bounded-output","maximum_bytes":request_bound.to_string()}),
                    expected_source_sha256: None,
                    execution_timeout_ms: kinekino_media_contracts::api_values::DecimalU64(15_000),
                },
                "request-bound",
            )
            .unwrap();
        let admission = Instant::now() + Duration::from_secs(8);
        let record = loop {
            let record = client
                .get(GetJob {
                    job_id: submitted.job.job_id.clone(),
                })
                .unwrap();
            let cutoff = state.cutoff.lock().unwrap().unwrap_or(admission);
            assert!(Instant::now() < cutoff);
            if record.state.terminal() {
                break record;
            }
            std::thread::sleep(Duration::from_millis(2));
        };
        assert_eq!(record.state, JobState::Failed);
        assert_eq!(record.failure.unwrap().code, ErrorCode::ResourceExhausted);
        assert!(!record.publication_started);
        assert!(record.output.is_none());
        assert!(state.closed.load(Ordering::Acquire));
        assert_eq!(state.reads.load(Ordering::Acquire), 0);
        assert_eq!(state.outputs.load(Ordering::Acquire), 0);
    }));
    let close = host_closeout::close(host).err();
    host_closeout::conclude(action, close);
    std::fs::remove_dir_all(root).unwrap();
}
