//! Actual native journal IO; generated subtitle capture, no model or GUI execution.
mod partial;
use super::*;
use crate::{
    application::{
        job_batches::{BatchProgress, CapturedBatchInput},
        jobs::{ListJobs, SubmitJob},
    },
    domain::api_values::DecimalU64,
};
use kinekino_jobs::infrastructure::{RetainingWorkerHost, WorkerShutdownOutcome};
use serde_json::json;
use std::time::{Duration, Instant};
struct Host {
    original: Option<RetainingWorkerHost>,
    evidence: PathBuf,
}
impl Drop for Host {
    fn drop(&mut self) {
        let Some(host) = self.original.take() else {
            return;
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut pending = host.begin_shutdown(deadline);
        loop {
            match pending.poll(deadline) {
                WorkerShutdownOutcome::Complete(done) => {
                    fs::write(self.evidence.join("shutdown.json"), serde_json::to_vec_pretty(&json!({"receipt":done.receipt,"error":done.errors.first(),"secondary_count":done.errors.secondary().len()})).unwrap()).unwrap();
                    if done.errors.first().is_some() {
                        std::process::exit(101)
                    }
                    break;
                }
                WorkerShutdownOutcome::Pending(owned) => {
                    pending = owned;
                    if Instant::now() >= deadline {
                        std::process::exit(101)
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }
}
#[test]
#[ignore = "Central native journal allocation; retains original source and all event evidence"]
fn native_journal_reopens_cas_locks_binding_and_partial_records_are_verified() {
    let parent = PathBuf::from(
        std::env::var_os("BATCH_FILES_EVIDENCE_DIR").expect("Explicit evidence root"),
    );
    assert!(parent.is_absolute());
    let evidence = parent.join(format!("native-batch-{}", std::process::id()));
    fs::create_dir(&evidence).unwrap();
    let jobs_root = evidence.join("jobs");
    fs::create_dir(&jobs_root).unwrap();
    let host = Host {
        original: Some(crate::infrastructure::jobs::start_retaining_host(&jobs_root).unwrap()),
        evidence: evidence.clone(),
    };
    let jobs = host.original.as_ref().unwrap().client();
    let source = evidence.join("source.srt");
    fs::write(
        &source,
        b"1\n00:00:01,000 --> 00:00:02,000\nNative journal fixture\n\n",
    )
    .unwrap();
    let captured = jobs.capture(SubmitJob {
        operation: "subtitles.convert".into(), parameters: json!({"input_path":source,"output_path":evidence.join("output.vtt"),"input_format":"srt","output_format":"vtt"}),
        expected_source_sha256: None, execution_timeout_ms: DecimalU64(30000),
    }).unwrap();
    let store = BatchFiles::open(&jobs_root).unwrap();
    fs::write(evidence.join("scope.txt"), store.scope_sha256()).unwrap();
    let initial = BatchDocument::from_captured(
        store.scope_sha256().into(),
        "actual-1".into(),
        vec![CapturedBatchInput::new("first".into(), captured).unwrap()],
    )
    .unwrap();
    store.create(&initial).unwrap();
    assert_eq!(
        store.create(&initial).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let mut lease = store.acquire("actual-1").unwrap();
    assert!(store.acquire("actual-1").is_err());
    let mut next = lease.load().unwrap();
    next.revision.0 += 1;
    next.entries[0].progress = BatchProgress::SubmitPending;
    lease.publish(1, &next).unwrap();
    assert_eq!(
        lease.publish(1, &next).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let expected = serde_json::to_vec(&next).unwrap();
    drop(lease);
    let scope = store.scope_sha256().to_owned();
    drop(store);
    let reopened = BatchFiles::open(&jobs_root).unwrap();
    assert_eq!(reopened.scope_sha256(), scope);
    let directory = reopened.directory("actual-1").unwrap();
    fs::write(
        directory.join(".pending-interrupted.json"),
        b"partial unpublished bytes",
    )
    .unwrap();
    assert_eq!(
        serde_json::to_vec(&reopened.acquire("actual-1").unwrap().load().unwrap()).unwrap(),
        expected
    );
    let final_events: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.file_name().unwrap().to_str().unwrap().starts_with("r-"))
        .collect();
    assert_eq!(final_events.len(), 2);
    let delta = final_events
        .iter()
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("r-00000000000000000002-")
        })
        .unwrap();
    let bytes = fs::read(delta).unwrap();
    let delta_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(delta_json["payload"]["kind"], "item");
    assert!(delta_json["payload"].get("document").is_none());
    let foreign_root = evidence.join("foreign-jobs");
    fs::create_dir(&foreign_root).unwrap();
    let foreign = BatchFiles::open(&foreign_root).unwrap();
    assert_ne!(foreign.scope_sha256(), scope);
    let foreign_directory = foreign.directory("actual-1").unwrap();
    fs::create_dir(&foreign_directory).unwrap();
    for path in &final_events {
        fs::copy(path, foreign_directory.join(path.file_name().unwrap())).unwrap();
    }
    assert_eq!(
        foreign
            .acquire("actual-1")
            .unwrap()
            .load()
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    fs::write(
        directory.join(format!("r-00000000000000000003-{}.json", "a".repeat(64))),
        b"partial final record",
    )
    .unwrap();
    assert_eq!(
        reopened
            .acquire("actual-1")
            .unwrap()
            .load()
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    assert_eq!(fs::read(delta).unwrap(), bytes);
    assert_eq!(
        jobs.list(ListJobs {
            cursor: None,
            limit: Some(10)
        })
        .unwrap()
        .jobs
        .len(),
        0
    );
    assert!(!evidence.join("output.vtt").exists());
    // Host Drop records the original owner shutdown even when an assertion unwinds.
}
