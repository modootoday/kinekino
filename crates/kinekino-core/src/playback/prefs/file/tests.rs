use super::*;
use crate::playback::prefs::{PlayerPrefs, PlayerPrefsStore, check_player_prefs_store};
use crate::port::error::ErrorCode;
use std::sync::atomic::{AtomicU64, Ordering};

fn dir(tag: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "kk-player-prefs-{tag}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

static NEXT: AtomicU64 = AtomicU64::new(0);

fn sample() -> PlayerPrefs {
    PlayerPrefs {
        volume: 65.0,
        muted: true,
        speed: 1.5,
        auto_next: true,
    }
}

#[test]
fn file_store_meets_contract() {
    let path = dir("contract").join("player-prefs.json");
    check_player_prefs_store(&mut FilePlayerPrefsStore::at(path)).unwrap();
}

#[test]
fn missing_file_loads_defaults_and_save_creates_it() {
    let path = dir("missing").join("player-prefs.json");
    let mut store = FilePlayerPrefsStore::at(&path);
    assert_eq!(store.load().unwrap(), PlayerPrefs::default());
    store.save(&sample()).unwrap();
    assert_eq!(
        FilePlayerPrefsStore::at(&path).load().unwrap(),
        sample().clamped()
    );
}

#[test]
fn corrupt_file_loads_defaults_and_is_preserved_on_save() {
    let path = dir("corrupt").join("player-prefs.json");
    std::fs::write(&path, b"not-json").unwrap();
    let mut store = FilePlayerPrefsStore::at(&path);
    assert_eq!(store.load().unwrap(), PlayerPrefs::default());
    store.save(&sample()).unwrap();
    assert!(path.with_file_name("player-prefs.json.corrupt").is_file());
    assert_eq!(
        FilePlayerPrefsStore::at(&path).load().unwrap(),
        sample().clamped()
    );
}

#[test]
fn newer_version_is_not_overwritten() {
    let path = dir("newer").join("player-prefs.json");
    std::fs::write(
        &path,
        br#"{"version":9,"volume":10,"muted":false,"speed":1.0,"auto_next":false}"#,
    )
    .unwrap();
    let mut store = FilePlayerPrefsStore::at(&path);
    assert_eq!(store.load().unwrap(), PlayerPrefs::default());
    let err = store.save(&sample()).unwrap_err();
    assert_eq!(err.code, ErrorCode::Unsupported);
    let body = std::fs::read_to_string(&path).unwrap();
    assert!(body.contains("\"version\":9"));
}
