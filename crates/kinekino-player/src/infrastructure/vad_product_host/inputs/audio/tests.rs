use super::*;
use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
fn root() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "product-vad-audio-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    path
}
fn wav(sample: i16) -> Vec<u8> {
    let mut raw = b"RIFF".to_vec();
    raw.extend_from_slice(&38u32.to_le_bytes());
    raw.extend_from_slice(b"WAVEfmt ");
    raw.extend_from_slice(&16u32.to_le_bytes());
    raw.extend_from_slice(&[1, 0, 1, 0, 128, 62, 0, 0, 0, 125, 0, 0, 2, 0, 16, 0]);
    raw.extend_from_slice(b"data");
    raw.extend_from_slice(&2u32.to_le_bytes());
    raw.extend_from_slice(&sample.to_le_bytes());
    raw
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[test]
fn distinct_requested_audio_ids_have_distinct_actual_pins_and_original_guards() {
    let root = root();
    let a = wav(1);
    let b = wav(2);
    fs::write(root.join("audio-first.wav"), &a).unwrap();
    fs::write(root.join("audio-second.wav"), &b).unwrap();
    let first = snapshot(&root, "first", &sha(&a), None).unwrap();
    let second = snapshot(&root, "second", &sha(&b), None).unwrap();
    assert_eq!(first.samples, DecimalU64(1));
    assert_ne!(first.source.sha256(), second.source.sha256());
    assert_ne!(first.pcm.sha256(), second.pcm.sha256());
    crate::infrastructure::os_policy::guarded_mutation(fs::write(
        root.join("audio-first.wav"),
        b"changed",
    ));
    drop(first);
    assert!(fs::write(root.join("audio-first.wav"), b"changed").is_ok());
    drop(second);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn wrong_original_source_sha_rejects_and_releases_failed_read_guards() {
    let root = root();
    let raw = wav(1);
    let path = root.join("audio-first.wav");
    fs::write(&path, &raw).unwrap();
    assert_eq!(
        snapshot(&root, "first", &"a".repeat(64), None)
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    assert!(fs::write(&path, b"changed").is_ok());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn invalid_id_and_non_wav_bytes_cannot_become_prepared_audio() {
    let root = root();
    assert_eq!(
        snapshot(&root, "../escape", &"a".repeat(64), None)
            .err()
            .unwrap()
            .code,
        ErrorCode::InvalidArgument
    );
    fs::write(root.join("audio-bad.wav"), b"not wav").unwrap();
    assert_eq!(
        snapshot(&root, "bad", &sha(b"not wav"), None)
            .err()
            .unwrap()
            .code,
        ErrorCode::InvalidArgument
    );
    fs::remove_dir_all(root).unwrap();
}
struct Cancelled(Instant);
impl AnalysisControl for Cancelled {
    fn deadline(&self) -> Instant {
        self.0
    }
    fn cancellation_requested(&self) -> bool {
        true
    }
}
#[test]
fn original_cancellation_precedes_even_missing_audio_open() {
    let root = root();
    let control = Cancelled(Instant::now() + Duration::from_secs(1));
    assert_eq!(
        snapshot(&root, "missing", &"a".repeat(64), Some(&control))
            .err()
            .unwrap()
            .code,
        ErrorCode::Cancelled
    );
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}
