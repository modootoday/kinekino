use super::*;
use kinekino_media_contracts::api_values::DecimalU64;
use sha2::{Digest, Sha256};
use std::{
    fs,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct Control {
    end: Instant,
    cancelled: bool,
}
impl AnalysisControl for Control {
    fn deadline(&self) -> Instant {
        self.end
    }
    fn cancellation_requested(&self) -> bool {
        self.cancelled
    }
}
fn control() -> Control {
    Control {
        end: Instant::now() + Duration::from_secs(5),
        cancelled: false,
    }
}
fn fixture(bytes: &[u8]) -> (PathBuf, PinnedFile) {
    let root = std::env::temp_dir().join(format!(
        "native-notice-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let path = root.join("notice.txt");
    fs::write(&path, bytes).unwrap();
    let pin = ArtifactPin::new(
        "native-notice".into(),
        format!("{:x}", Sha256::digest(bytes)),
        DecimalU64(bytes.len() as u64),
    )
    .unwrap();
    (root, PinnedFile { path, pin })
}
#[test]
fn full_notice_sized_file_keeps_original_guard_until_explicit_release() {
    let bytes = vec![b'n'; 338_088];
    let (root, expected) = fixture(&bytes);
    let (guard, actual) = file(&expected, MAX_RUNTIME_NOTICE_BYTES, false, &control()).unwrap();
    assert_eq!(actual, bytes);
    assert!(fs::remove_file(&expected.path).is_err());
    drop(guard);
    fs::remove_file(&expected.path).unwrap();
    fs::remove_dir(root).unwrap();
}
#[test]
fn actual_notice_reader_accepts_exact_bound_and_rejects_oversized_pin() {
    let (root, expected) = fixture(&vec![b'n'; MAX_RUNTIME_NOTICE_BYTES as usize]);
    let (guard, actual) = file(&expected, MAX_RUNTIME_NOTICE_BYTES, false, &control()).unwrap();
    assert_eq!(actual.len() as u64, MAX_RUNTIME_NOTICE_BYTES);
    drop(guard);
    fs::remove_file(&expected.path).unwrap();
    fs::remove_dir(root).unwrap();

    let (root, mut expected) = fixture(b"n");
    expected.pin = ArtifactPin::new(
        "native-notice".into(),
        "a".repeat(64),
        DecimalU64(MAX_RUNTIME_NOTICE_BYTES + 1),
    )
    .unwrap();
    assert_eq!(
        file(&expected, MAX_RUNTIME_NOTICE_BYTES, false, &control())
            .err()
            .unwrap()
            .code,
        ErrorCode::ResourceExhausted
    );
    fs::remove_file(&expected.path).unwrap();
    fs::remove_dir(root).unwrap();
}
#[test]
fn changed_notice_hash_fails_and_releases_failed_read_guards() {
    let (root, expected) = fixture(b"original");
    fs::write(&expected.path, b"modified").unwrap();
    assert_eq!(
        file(&expected, MAX_RUNTIME_NOTICE_BYTES, false, &control())
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    fs::remove_file(&expected.path).unwrap();
    fs::remove_dir(root).unwrap();
}
#[test]
fn original_cancellation_and_cutoff_precede_missing_notice_open() {
    let (root, expected) = fixture(b"n");
    fs::remove_file(&expected.path).unwrap();
    let cancelled = Control {
        end: Instant::now() + Duration::from_secs(5),
        cancelled: true,
    };
    assert_eq!(
        file(&expected, MAX_RUNTIME_NOTICE_BYTES, false, &cancelled)
            .err()
            .unwrap()
            .code,
        ErrorCode::Cancelled
    );
    let expired = Control {
        end: Instant::now(),
        cancelled: false,
    };
    assert_eq!(
        file(&expected, MAX_RUNTIME_NOTICE_BYTES, false, &expired)
            .err()
            .unwrap()
            .code,
        ErrorCode::Timeout
    );
    fs::remove_dir(root).unwrap();
}
