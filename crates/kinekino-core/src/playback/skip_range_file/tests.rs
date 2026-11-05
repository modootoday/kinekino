use super::*;
use crate::playback::store::check_skip_range_store;
use serde_json::{Value, json};

const NOW: u64 = 1_800_000_000_000;

fn now() -> u64 {
    NOW
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("kk-skip-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d.join("nested").join("skip-ranges.json")
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

fn range(start: f64, end: f64) -> SkipRange {
    SkipRange::new(start, end).expect("valid range")
}

fn entry(updated_at_ms: u64) -> Value {
    json!({"start_ms": 10_000, "end_ms": 90_000, "updated_at_ms": updated_at_ms})
}

#[test]
fn file_store_meets_contract_and_persists() {
    let path = temp("contract");
    check_skip_range_store(&mut FileSkipRangeStore::with_clock(&path, now)).unwrap();
    let reopened = FileSkipRangeStore::at(&path);
    assert!(
        reopened.get("b").unwrap().is_some(),
        "entries persist across instances"
    );
    cleanup(&path);
}

#[test]
fn writes_version_1_format_in_milliseconds() {
    let path = temp("format");
    let mut s = FileSkipRangeStore::with_clock(&path, now);
    s.put("file:/m/a.mkv", range(42.5, 120.0)).unwrap();
    let v = read_json(&path);
    let back = FileSkipRangeStore::with_clock(&path, now).get("file:/m/a.mkv");
    cleanup(&path);
    assert_eq!(
        v,
        json!({"version": 1, "entries": {"file:/m/a.mkv": {
            "start_ms": 42_500, "end_ms": 120_000, "updated_at_ms": NOW
        }}})
    );
    assert_eq!(back.unwrap(), Some(range(42.5, 120.0)));
}

#[test]
fn write_is_atomic_and_creates_parents() {
    let path = temp("atomic");
    FileSkipRangeStore::with_clock(&path, now)
        .put("k", range(10.0, 90.0))
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
    assert_eq!(names, vec!["skip-ranges.json".to_owned()]);
}

fn assert_corrupt_is_moved_aside(name: &str, original: &[u8]) {
    let path = temp(name);
    write_raw(&path, original);
    let mut s = FileSkipRangeStore::with_clock(&path, now);
    let got = s.get("k");
    let put = s.put("k", range(10.0, 90.0));
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
    assert_eq!(v["entries"]["k"]["start_ms"], 10_000, "{name}");
}

#[test]
fn corrupt_file_is_moved_aside_and_replaced() {
    assert_corrupt_is_moved_aside("corrupt-text", b"not json");
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
    let mut s = FileSkipRangeStore::with_clock(&path, now);
    let got = s.get("k");
    let put = s.put("k", range(10.0, 90.0));
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
    let put = FileSkipRangeStore::with_clock(&path, now).put("new", range(10.0, 90.0));
    let body = std::fs::read(&path).unwrap();
    cleanup(&path);
    put.unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let e = v["entries"].as_object().unwrap();
    assert_eq!(e.len(), MAX_ENTRIES);
    assert!(!e.contains_key("k1000"), "oldest-updated entry dropped");
    assert!(e.contains_key("k0999"), "second oldest kept");
    assert!(e.contains_key("new"));
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
    let put = FileSkipRangeStore::with_clock(&path, now).put("other", range(10.0, 90.0));
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
fn temp_file_name_is_per_process() {
    let path = temp("tmp-name");
    let s = FileSkipRangeStore::with_clock(&path, now);
    let expected = sibling(&path, &format!(".{}.tmp", std::process::id()));
    assert_eq!(s.temp_path(), expected);
}
