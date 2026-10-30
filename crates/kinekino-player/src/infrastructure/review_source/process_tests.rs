//! Actual owned subprocess interruption; no production fault switch or original media.
use super::{
    tests::{Permit, body, root},
    *,
};
use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
const WORKER: &str = "infrastructure::review_source::process_tests::owned_process_worker";
const ROOT_ENV: &str = "KINEKINO_REVIEW_TEST_PROCESS_ROOT";
const MODE_ENV: &str = "KINEKINO_REVIEW_TEST_PROCESS_MODE";
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
fn hold(root: &Path) -> ! {
    fs::write(root.join("ready"), b"original child reached selected phase").unwrap();
    loop {
        std::thread::park_timeout(Duration::from_secs(1));
    }
}
struct HoldBeforeLink(PathBuf);
impl PublicationGuard for HoldBeforeLink {
    fn before_publish(&self) -> Result<(), ApiError> {
        hold(&self.0)
    }
}
#[test]
#[ignore = "Internal child helper; requires parent-owned unique fixture root and mode"]
fn owned_process_worker() {
    let root = PathBuf::from(std::env::var_os(ROOT_ENV).expect("parent fixture root required"));
    let mode = std::env::var(MODE_ENV).expect("parent mode required");
    let store = UserReviewFiles::open(&root, true).unwrap();
    let body = body();
    let mut transaction = store.open(&body.request.case, StoreAccess::Write).unwrap();
    match mode.as_str() {
        "reservation" => hold(&root),
        "before-link" => {
            transaction.publish(&body, &HoldBeforeLink(root)).unwrap();
        }
        "after-publication" => {
            let receipt = transaction.publish(&body, &Permit).unwrap();
            fs::write(
                root.join("child-receipt.json"),
                serde_json::to_vec(&receipt).unwrap(),
            )
            .unwrap();
            hold(&root);
        }
        _ => panic!("unknown parent mode"),
    }
}
fn spawn(root: &Path, mode: &str) -> OwnedChild {
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            WORKER,
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(ROOT_ENV, root)
        .env(MODE_ENV, mode)
        .stdin(Stdio::null())
        .stdout(File::create(root.join("child.stdout")).unwrap())
        .stderr(File::create(root.join("child.stderr")).unwrap())
        .spawn()
        .unwrap();
    let mut child = OwnedChild(child);
    let deadline = Instant::now() + Duration::from_secs(15);
    while !root.join("ready").exists() {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "worker exited before readiness: {}",
            root.display()
        );
        assert!(
            Instant::now() < deadline,
            "worker readiness timeout: {}",
            root.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    child
}
fn terminate(mut child: OwnedChild, root: &Path) {
    let pid = child.0.id();
    child.0.kill().unwrap();
    let status = child.0.wait().unwrap();
    assert!(!status.success());
    fs::write(
        root.join("parent-termination.json"),
        serde_json::to_vec(
            &serde_json::json!({"pid":pid,"exit_code":status.code(),"forced_by_owner":true}),
        )
        .unwrap(),
    )
    .unwrap();
}
#[test]
fn process_reservation_conflicts_then_recovers_after_owner_termination() {
    let root = root();
    let child = spawn(&root, "reservation");
    let store = UserReviewFiles::open(&root, true).unwrap();
    let body = body();
    for access in [StoreAccess::Read, StoreAccess::Write] {
        assert!(
            matches!(store.open(&body.request.case,access),Err(e) if e.code==ErrorCode::RevisionConflict)
        );
    }
    terminate(child, &root);
    let mut transaction = store.open(&body.request.case, StoreAccess::Write).unwrap();
    assert!(transaction.replay(&body.request).unwrap().is_none());
    transaction.publish(&body, &Permit).unwrap();
    drop(transaction);
    assert!(
        store
            .open(&body.request.case, StoreAccess::Read)
            .unwrap()
            .replay(&body.request)
            .unwrap()
            .is_some()
    );
}
#[test]
fn killed_before_link_preserves_orphan_and_refuses_empty_history() {
    let root = root();
    let child = spawn(&root, "before-link");
    terminate(child, &root);
    let case_directory = fs::read_dir(root.join("cases"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let records = case_directory.join("transactions");
    let files: Vec<_> = fs::read_dir(&records)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].extension().unwrap(), "tmp");
    let before = fs::read(&files[0]).unwrap();
    assert!(!before.is_empty());
    let store = UserReviewFiles::open(&root, true).unwrap();
    assert!(
        matches!(store.open(&body().request.case,StoreAccess::Write),Err(e) if e.code==ErrorCode::NeedsInput)
    );
    assert_eq!(fs::read(&files[0]).unwrap(), before);
}
#[test]
fn killed_after_publication_reopens_original_receipt_without_duplicate() {
    let root = root();
    let child = spawn(&root, "after-publication");
    terminate(child, &root);
    let expected: MutationReceipt =
        serde_json::from_slice(&fs::read(root.join("child-receipt.json")).unwrap()).unwrap();
    let store = UserReviewFiles::open(&root, true).unwrap();
    let body = body();
    let mut transaction = store.open(&body.request.case, StoreAccess::Write).unwrap();
    assert_eq!(
        transaction.replay(&body.request).unwrap(),
        Some(expected.clone())
    );
    assert_eq!(transaction.publish(&body, &Permit).unwrap(), expected);
    drop(transaction);
    let case_directory = fs::read_dir(root.join("cases"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(
        fs::read_dir(case_directory.join("transactions"))
            .unwrap()
            .count(),
        1
    );
}
