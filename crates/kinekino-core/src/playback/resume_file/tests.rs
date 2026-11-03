use super::*;
use crate::playback::store::check_resume_store;
use serde_json::{Value, json};

const NOW: u64 = 1_800_000_000_000;

fn now() -> u64 {
    NOW
}

/// Unique temp dir per test; returns `<dir>/nested/resume.json`.
fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("kk-resume-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d.join("nested").join("resume.json")
}

fn cleanup(path: &Path) {
    let _ = std::fs::remove_dir_all(path.parent().unwrap().parent().unwrap());
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let name = path.file_name().unwrap().to_str().unwrap();
    path.with_file_name(format!("{name}{suffix}"))
}

fn write_raw(path: &Path, body: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn point(position_us: u64) -> ResumePoint {
    ResumePoint {
        position_us,
        duration_us: 1_200_000_000,
    }
}

fn entry(updated_at_ms: u64) -> Value {
    json!({"position_ms": 60_000, "duration_ms": 1_200_000, "updated_at_ms": updated_at_ms})
}

#[test]
fn file_store_meets_contract_and_persists() {
    let path = temp("contract");
    check_resume_store(&mut FileResumeStore::with_clock(&path, now)).unwrap();
    let reopened = FileResumeStore::at(&path);
    assert!(
        reopened.get("b").unwrap().is_some(),
        "entries persist across instances"
    );
    cleanup(&path);
}

#[test]
fn writes_version_1_format_in_milliseconds() {
    let path = temp("format");
    let mut s = FileResumeStore::with_clock(&path, now);
    s.put(
        "file:/m/a.mkv",
        ResumePoint {
            position_us: 42_500_999,
            duration_us: 1_200_000_000,
        },
    )
    .unwrap();
    let v = read_json(&path);
    let back = FileResumeStore::with_clock(&path, now).get("file:/m/a.mkv");
    cleanup(&path);
    assert_eq!(
        v,
        json!({"version": 1, "entries": {"file:/m/a.mkv": {
            "position_ms": 42_500, "duration_ms": 1_200_000, "updated_at_ms": NOW
        }}})
    );
    assert_eq!(
        back.unwrap(),
        Some(ResumePoint {
            position_us: 42_500_000,
            duration_us: 1_200_000_000
        })
    );
}

#[test]
fn write_is_atomic_and_creates_parents() {
    let path = temp("atomic");
    FileResumeStore::with_clock(&path, now)
        .put("k", point(60_000_000))
        .unwrap();
    let exists = path.exists();
    let tmp_left = sibling(&path, ".tmp").exists();
    let names: Vec<_> = std::fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    cleanup(&path);
    assert!(exists);
    assert!(!tmp_left, "temp file must be renamed away");
    assert_eq!(names, vec!["resume.json".to_owned()]);
}

fn assert_corrupt_is_moved_aside(name: &str, original: &[u8]) {
    let path = temp(name);
    write_raw(&path, original);
    let mut s = FileResumeStore::with_clock(&path, now);
    let got = s.get("k");
    let put = s.put("k", point(60_000_000));
    let corrupt = std::fs::read(sibling(&path, ".corrupt"));
    let body = std::fs::read(&path);
    cleanup(&path);
    assert_eq!(got.unwrap(), None, "{name}: corrupt file reads as empty");
    put.unwrap();
    assert_eq!(
        corrupt.unwrap(),
        original,
        "{name}: original bytes preserved"
    );
    let v: Value = serde_json::from_slice(&body.unwrap()).unwrap();
    assert_eq!(v["version"], 1, "{name}");
    assert_eq!(v["entries"]["k"]["position_ms"], 60_000, "{name}");
    assert_eq!(v["entries"].as_object().unwrap().len(), 1, "{name}");
}

#[test]
fn corrupt_file_is_moved_aside_and_replaced() {
    assert_corrupt_is_moved_aside("corrupt-text", b"not json");
}

#[test]
fn legacy_unversioned_file_is_treated_as_corrupt() {
    assert_corrupt_is_moved_aside("corrupt-legacy", br#"{"k":[1,2]}"#);
}

#[test]
fn unknown_fields_in_v1_are_treated_as_corrupt() {
    assert_corrupt_is_moved_aside(
        "corrupt-unknown",
        br#"{"version":1,"entries":{},"extra":true}"#,
    );
}

#[test]
fn newer_version_is_read_as_empty_and_never_overwritten() {
    let path = temp("newer");
    let original = br#"{"version":2,"entries":{},"future":{"x":1}}"#;
    write_raw(&path, original);
    let mut s = FileResumeStore::with_clock(&path, now);
    let got = s.get("k");
    let put = s.put("k", point(60_000_000));
    let body = std::fs::read(&path).unwrap();
    let corrupt_exists = sibling(&path, ".corrupt").exists();
    cleanup(&path);
    assert_eq!(got.unwrap(), None);
    assert_eq!(put.unwrap_err().code, ErrorCode::Unsupported);
    assert_eq!(body, original);
    assert!(!corrupt_exists, "newer files are not moved aside");
}

#[test]
fn cap_drops_least_recently_updated() {
    let path = temp("cap");
    // Key order runs opposite to age order, so dropping by key order would fail here.
    let entries: serde_json::Map<String, Value> = (0..MAX_ENTRIES)
        .map(|i| {
            (
                format!("k{:04}", MAX_ENTRIES - i),
                entry(NOW - (MAX_ENTRIES - i) as u64 * 1000),
            )
        })
        .collect();
    write_raw(
        &path,
        &serde_json::to_vec(&json!({"version": 1, "entries": entries})).unwrap(),
    );
    let put = FileResumeStore::with_clock(&path, now).put("new", point(60_000_000));
    let body = std::fs::read(&path).unwrap();
    cleanup(&path);
    put.unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let e = v["entries"].as_object().unwrap();
    assert_eq!(e.len(), MAX_ENTRIES);
    assert!(!e.contains_key("k1000"), "oldest-updated entry dropped");
    assert!(e.contains_key("k0999"), "second oldest kept");
    assert!(e.contains_key("k0001"), "first key in key order kept");
    assert!(e.contains_key("new"));
}

#[test]
fn previous_corrupt_copy_is_replaced() {
    let path = temp("corrupt-replace");
    write_raw(&sibling(&path, ".corrupt"), b"older corrupt copy");
    write_raw(&path, b"newer corrupt body");
    let put = FileResumeStore::with_clock(&path, now).put("k", point(60_000_000));
    let corrupt = std::fs::read(sibling(&path, ".corrupt"));
    cleanup(&path);
    put.unwrap();
    assert_eq!(corrupt.unwrap(), b"newer corrupt body");
}

#[test]
fn clear_leaves_newer_and_corrupt_files_untouched() {
    for (name, original) in [
        ("clear-newer", &br#"{"version":2,"entries":{"k":1}}"#[..]),
        ("clear-corrupt", &b"not json"[..]),
    ] {
        let path = temp(name);
        write_raw(&path, original);
        let cleared = FileResumeStore::with_clock(&path, now).clear("k");
        let body = std::fs::read(&path);
        let corrupt_exists = sibling(&path, ".corrupt").exists();
        cleanup(&path);
        assert!(!cleared.unwrap(), "{name}");
        assert_eq!(body.unwrap(), original, "{name}");
        assert!(
            !corrupt_exists,
            "{name}: nothing moved aside without a write"
        );
    }
}

#[test]
fn entries_older_than_max_age_are_dropped_on_write() {
    let path = temp("age");
    write_raw(
        &path,
        &serde_json::to_vec(&json!({"version": 1, "entries": {
            "old": entry(NOW - MAX_AGE_MS - 1),
            "edge": entry(NOW - MAX_AGE_MS),
            "recent": entry(NOW - 1000),
        }}))
        .unwrap(),
    );
    let put = FileResumeStore::with_clock(&path, now).put("other", point(60_000_000));
    let v = read_json(&path);
    cleanup(&path);
    put.unwrap();
    let e = v["entries"].as_object().unwrap();
    assert!(!e.contains_key("old"));
    assert!(e.contains_key("edge"), "exactly MAX_AGE_MS old is kept");
    assert!(e.contains_key("recent"));
    assert!(e.contains_key("other"));
}

#[test]
fn errors_never_contain_the_media_key() {
    let path = temp("err");
    let blocker = path.parent().unwrap();
    std::fs::create_dir_all(blocker.parent().unwrap()).unwrap();
    std::fs::write(blocker, b"regular file").unwrap();
    let store_path = blocker.join("resume.json");
    let err = FileResumeStore::with_clock(&store_path, now)
        .put("file:/secret/movie.mkv", point(60_000_000))
        .unwrap_err();
    cleanup(&path);
    assert!(!err.message.contains("secret"), "{err}");
    assert!(!err.to_json().contains("secret"), "{err}");
}

#[test]
fn temp_file_name_is_per_process() {
    let path = temp("tmp-name");
    let s = FileResumeStore::with_clock(&path, now);
    let expected = sibling(&path, &format!(".{}.tmp", std::process::id()));
    assert_eq!(s.temp_path(), expected);
}

#[test]
fn failed_write_removes_its_temp_file() {
    let path = temp("tmp-cleanup");
    // A non-empty directory at the store path makes the final rename fail.
    std::fs::create_dir_all(path.join("occupied")).unwrap();
    let s = FileResumeStore::with_clock(&path, now);
    let saved = s.save(&mut Entries::new());
    let names: Vec<_> = std::fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    cleanup(&path);
    assert_eq!(saved.unwrap_err().code, ErrorCode::Io);
    assert_eq!(names, vec!["resume.json".to_owned()], "temp file removed");
}
