use super::*;
use crate::domain::WorkerOperationCutoff;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
struct Original {
    end: Instant,
    cancel: bool,
}
impl RunControl for Original {
    fn deadline(&self) -> Instant {
        self.end
    }
    fn cancellation_requested(&self) -> bool {
        self.cancel
    }
}
fn writer(path: &Path) -> std::io::Result<File> {
    let os = kinekino_core::platform::os();
    let mut options = OpenOptions::new();
    os.apply_share_mode(options.write(true), ShareMode::ReadWrite);
    options.open(path)
}
/// Windows: the retained read-shared guard rejects writers (sharing violation).
/// POSIX has advisory sharing only: a writer may open, so the guard's evidence
/// is that the published bytes stay the exact original ones.
fn assert_writer_excluded(path: &Path) {
    let os = kinekino_core::platform::os();
    let before = std::fs::read(path).unwrap();
    match writer(path) {
        Err(error) => {
            assert!(os.host().has_mandatory_share_modes());
            assert!(os.is_sharing_violation(&error), "{error}");
        }
        Ok(_) => assert!(!os.host().has_mandatory_share_modes()),
    }
    assert_eq!(std::fs::read(path).unwrap(), before);
}
fn root() -> PathBuf {
    kinekino_platform::install();
    let root = std::env::temp_dir().join(format!(
        "vad-control-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    eprintln!("vad_control_fixture_root={}", root.display());
    root
}
#[test]
fn parent_sidecar_keeps_exact_request_pin_and_original_file_guard() {
    let root = root();
    let start = Instant::now();
    let deadline = Deadline::new(start, Duration::from_secs(10), Duration::from_secs(2)).unwrap();
    let caller = Original {
        end: start + Duration::from_secs(9),
        cancel: false,
    };
    let request = b"original stdin request\n";
    let prepared = prepare(&root, deadline, &caller, request).unwrap();
    let raw = std::fs::read(&prepared.path).unwrap();
    let cutoff: WorkerOperationCutoff = serde_json::from_slice(&raw).unwrap();
    cutoff.validate().unwrap();
    assert_eq!(
        cutoff.request_sha256,
        format!("{:x}", Sha256::digest(request))
    );
    assert_eq!(cutoff.clock.parent_pid, std::process::id());
    assert_writer_excluded(&prepared.path);
    assert!(prepare(&root, deadline, &caller, request).is_err());
    assert_eq!(std::fs::read(&prepared.path).unwrap(), raw);
    let path = prepared.path.clone();
    drop(prepared);
    writer(&path).unwrap();
}
#[test]
fn already_cancelled_parent_does_not_write_control_or_claim_child_cleanup() {
    let root = root();
    let start = Instant::now();
    let deadline = Deadline::new(start, Duration::from_secs(10), Duration::from_secs(2)).unwrap();
    let caller = Original {
        end: start + Duration::from_secs(9),
        cancel: true,
    };
    let error = match prepare(&root, deadline, &caller, b"request") {
        Err(error) => error,
        Ok(_) => panic!("Cancelled parent issued child control"),
    };
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert!(!root.join("worker-control.v1.json").exists());
}

#[test]
fn cancellation_marker_is_complete_guarded_and_never_overwrites_prior_bytes() {
    let root = root();
    let guard = publish_cancellation(&root).unwrap();
    let marker = root.join("cancel.signal");
    assert_eq!(std::fs::read(&marker).unwrap(), b"cancel\n");
    assert_writer_excluded(&marker);
    assert!(publish_cancellation(&root).is_err());
    assert_eq!(std::fs::read(&marker).unwrap(), b"cancel\n");
    drop(guard);
    let other = self::root();
    std::fs::write(other.join("cancel.signal"), b"prior evidence").unwrap();
    assert!(publish_cancellation(&other).is_err());
    assert_eq!(
        std::fs::read(other.join("cancel.signal")).unwrap(),
        b"prior evidence"
    );
}
