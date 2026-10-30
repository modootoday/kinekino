//! External Rust fault watcher; no product test hooks or alternate CLI binary.
use super::*;
use std::{collections::BTreeSet, io::Write, path::Path};
const SENTINEL: &[u8] = b"original normal CLI receipt collision evidence";
fn roots(pid: u32) -> BTreeSet<PathBuf> {
    let prefix = format!("frame-owned-{pid}-");
    fs::read_dir(std::env::temp_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&prefix)
        })
        .collect()
}
pub(super) struct Fault {
    root: PathBuf,
    before: BTreeSet<PathBuf>,
    pid: u32,
    collisions: usize,
}
pub(super) fn prepare(pid: u32, collisions: usize) -> thread::JoinHandle<Fault> {
    assert!((1..=2).contains(&collisions) || collisions == 17);
    let before = roots(pid);
    thread::spawn(move || {
        let cutoff = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(root) = roots(pid).difference(&before).next() {
                for index in 0..collisions {
                    let mut file = fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(root.join(format!("receipt-{index:04}.json")))
                        .expect("immutable collision must precede first receipt; do not overwrite");
                    file.write_all(SENTINEL).unwrap();
                    file.sync_all().unwrap();
                }
                return Fault {
                    root: root.clone(),
                    before,
                    pid,
                    collisions,
                };
            }
            assert!(
                Instant::now() < cutoff,
                "original CLI frame root not observed"
            );
            thread::sleep(Duration::from_millis(1));
        }
    })
}
fn media() -> PathBuf {
    PathBuf::from(std::env::var_os("FRAME_HOST_MEDIA").expect("explicit pinned existing portrait"))
}
pub(super) fn request(root: &Path) -> Request {
    let guard = custody::Guard::acquire(&media());
    assert_eq!(
        guard.sha256,
        "0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483"
    );
    let mut request = Request::new("production.capture_frame");
    request.payload = json!({"source_path":media(),"source_pts_us":"500000","output_path":root.join("frame.png")});
    request
}
pub(super) fn recover(
    root: &Path,
    watcher: thread::JoinHandle<Fault>,
    operation: &Value,
    endpoint: &str,
    call: &impl Fn(&str, &str, Value) -> Value,
) {
    let fault = watcher.join().unwrap();
    let evidence = &fault.root;
    fs::write(root.join("FAULT.json"),serde_json::to_vec_pretty(&json!({"original_evidence_root":evidence,"mechanism":"external create_new immutable receipt collision","collision_count":fault.collisions,"host_pid":fault.pid,"roots_before":fault.before})).unwrap()).unwrap();
    for generation in 1..=fault.collisions.min(16) {
        let cutoff = Instant::now() + Duration::from_secs(10);
        for poll in 0.. {
            let state = call(
                &format!("recovery-pending-{generation}-{poll}"),
                "execution.peek",
                json!({"operation":operation}),
            );
            assert_ne!(
                state["state"]["phase"], "terminal",
                "fault failed to retain original operation: {state}"
            );
            if state["state"]["receipt_recovery"]["generation"] == generation {
                assert_eq!(state["state"]["receipt_recovery"]["retry_requested"], false);
                break;
            }
            assert!(Instant::now() < cutoff);
        }
        for index in 0..fault.collisions {
            assert_eq!(
                fs::read(evidence.join(format!("receipt-{index:04}.json"))).unwrap(),
                SENTINEL
            );
        }
        assert!(
            !evidence
                .join(format!("receipt-{:04}.json", fault.collisions))
                .exists(),
            "no automatic receipt retry"
        );
        use std::os::windows::fs::OpenOptionsExt;
        assert!(
            fs::OpenOptions::new()
                .read(true)
                .share_mode(0)
                .open(media())
                .is_err(),
            "original media guard retained while pending"
        );
        if generation > 1 {
            let directory = root.join(format!("stale-retry-{generation}"));
            fs::create_dir(&directory).unwrap();
            let mut request = Request::new("execution.retry_receipt");
            request.payload = json!({"operation":operation,"expected_generation":generation-1});
            fs::write(
                directory.join("request.json"),
                serde_json::to_vec_pretty(&request).unwrap(),
            )
            .unwrap();
            let mut runner = cli::CliRunner::start(endpoint, &request);
            let rejected = runner.finish_preserving(&directory);
            assert_ne!(rejected.code, 0);
            assert_eq!(rejected.response["error"]["code"], "source_expired");
            let still = call(
                &format!("after-stale-{generation}"),
                "execution.peek",
                json!({"operation":operation}),
            );
            assert_eq!(still["state"]["receipt_recovery"]["generation"], generation);
            assert_eq!(still["state"]["receipt_recovery"]["retry_requested"], false);
        }
        let retried = call(
            &format!("receipt-retry-{generation}"),
            "execution.retry_receipt",
            json!({"operation":operation,"expected_generation":generation}),
        );
        assert_eq!(retried["disposition"], "requested");
        assert_eq!(retried["completion"], "unobserved");
    }
    if fault.collisions == 17 {
        let cutoff = Instant::now() + Duration::from_secs(10);
        let mut poll = 0usize;
        let exhausted = loop {
            let state = call(
                &format!("exhausted-{poll}"),
                "execution.peek",
                json!({"operation":operation}),
            );
            assert_eq!(state["state"]["phase"], "running");
            if state["state"]["receipt_recovery"]["attempt_budget_exhausted"] == true {
                break state;
            }
            assert!(Instant::now() < cutoff);
            poll += 1;
        };
        assert_eq!(exhausted["state"]["receipt_recovery"]["generation"], 16);
        let directory = root.join("exhausted-retry");
        fs::create_dir(&directory).unwrap();
        let mut request = Request::new("execution.retry_receipt");
        request.payload = json!({"operation":operation,"expected_generation":16});
        fs::write(
            directory.join("request.json"),
            serde_json::to_vec_pretty(&request).unwrap(),
        )
        .unwrap();
        let mut runner = cli::CliRunner::start(endpoint, &request);
        let rejected = runner.finish_preserving(&directory);
        assert_eq!(rejected.response["error"]["code"], "resource_exhausted");
        assert_ne!(rejected.code, 0);
        for index in 0..17 {
            assert_eq!(
                fs::read(evidence.join(format!("receipt-{index:04}.json"))).unwrap(),
                SENTINEL
            );
        }
        assert!(!evidence.join("receipt-0017.json").exists());
        assert!(!root.join("frame.png").exists());
        use std::os::windows::fs::OpenOptionsExt;
        assert!(
            fs::OpenOptions::new()
                .read(true)
                .share_mode(0)
                .open(media())
                .is_err(),
            "source guard still retained after exhaustion"
        );
        let after = roots(fault.pid);
        let added: Vec<_> = after.difference(&fault.before).cloned().collect();
        assert_eq!(added, vec![evidence.clone()]);
        fs::write(root.join("EXHAUSTED.json"),serde_json::to_vec_pretty(&json!({"observation":exhausted,"roots_after":after,"new_roots":added,"final_receipt_recorded":false,"cleanup_complete":false})).unwrap()).unwrap();
    }
}
pub(super) fn verify_terminal(root: &Path, response: &Value) {
    assert_eq!(
        custody::Guard::acquire(&media()).sha256,
        "0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483"
    );
    assert_eq!(response["error"]["code"], "engine_failed");
    let owned = &response["error"]["details"]["kinekino_process"];
    assert_eq!(owned["physical_closed"], true);
    assert_eq!(owned["receipt_recorded"], true);
    let fault: Value = serde_json::from_slice(&fs::read(root.join("FAULT.json")).unwrap()).unwrap();
    let evidence = PathBuf::from(fault["original_evidence_root"].as_str().unwrap());
    assert_eq!(
        Path::new(owned["evidence_root"].as_str().unwrap()),
        evidence
    );
    let collisions = fault["collision_count"].as_u64().unwrap();
    for index in 0..collisions {
        assert_eq!(
            fs::read(evidence.join(format!("receipt-{index:04}.json"))).unwrap(),
            SENTINEL
        );
    }
    let receipt: Value = serde_json::from_slice(
        &fs::read(evidence.join(format!("receipt-{collisions:04}.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt["family"], "Empty");
    assert_eq!(receipt["forced"], false);
    assert_eq!(receipt["child_exit"], 0);
    let before: BTreeSet<PathBuf> = serde_json::from_value(fault["roots_before"].clone()).unwrap();
    let after = roots(fault["host_pid"].as_u64().unwrap().try_into().unwrap());
    let added: Vec<_> = after.difference(&before).cloned().collect();
    fs::write(
        root.join("ROOTS-AFTER.json"),
        serde_json::to_vec_pretty(&json!({"roots":after,"new_roots":added})).unwrap(),
    )
    .unwrap();
    assert_eq!(
        added,
        vec![evidence.clone()],
        "no additional owned frame process created during explicit receipt retry"
    );
    assert!(
        !root.join("frame.png").exists(),
        "receipt recovery must not rerun capture or publish a PNG"
    );
    fs::write(
        root.join("RECOVERED-RECEIPT.json"),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
}
