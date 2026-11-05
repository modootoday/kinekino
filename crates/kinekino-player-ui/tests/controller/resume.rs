//! 이어보기 in the player controller (ADR 20261103115459): a local file restores its saved
//! point once the open finishes, positions are saved at the decided moments, remote items
//! never touch the store, and a failing store never blocks playback.
use super::support::{FakeEngine, go};
use kinekino_core::playback::{MemoryResumeStore, ResumeKey, ResumePoint, ResumeStore};
use kinekino_core::port::error::{ErrorCode, PortError, PortResult};
use kinekino_core::source::{
    SourceError, SourceItem, SourceItemKind, SourcePage, SourceRef, StreamCandidate,
};
use kinekino_player_ui::{Catalog, Command, Controller, Effect};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const S: u64 = 1_000_000;
/// FakeEngine reports this duration for every item.
const DURATION_US: u64 = 1500 * S;

/// A memory store the test can still read after handing it to the controller; counts calls.
#[derive(Clone, Default)]
struct Shared {
    store: Arc<Mutex<MemoryResumeStore>>,
    calls: Arc<AtomicUsize>,
}

impl Shared {
    fn saved(&self, key: &str) -> Option<ResumePoint> {
        self.store.lock().unwrap().get(key).unwrap()
    }
    fn seed(&self, key: &str, position_us: u64, duration_us: u64) {
        let point = ResumePoint {
            position_us,
            duration_us,
        };
        self.store.lock().unwrap().put(key, point).unwrap();
    }
    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl ResumeStore for Shared {
    fn get(&self, key: &str) -> PortResult<Option<ResumePoint>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.store.lock().unwrap().get(key)
    }
    fn put(&mut self, key: &str, point: ResumePoint) -> PortResult<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.store.lock().unwrap().put(key, point)
    }
    fn clear(&mut self, key: &str) -> PortResult<bool> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.store.lock().unwrap().clear(key)
    }
}

/// Every call fails, like a store on a broken disk.
struct Failing;
impl ResumeStore for Failing {
    fn get(&self, _: &str) -> PortResult<Option<ResumePoint>> {
        Err(PortError::new(ErrorCode::Io, "store unavailable"))
    }
    fn put(&mut self, _: &str, _: ResumePoint) -> PortResult<()> {
        Err(PortError::new(ErrorCode::Io, "store unavailable"))
    }
    fn clear(&mut self, _: &str) -> PortResult<bool> {
        Err(PortError::new(ErrorCode::Io, "store unavailable"))
    }
}

/// Real files (keys canonicalize existing paths), unique per test and process.
fn folder(tag: &str) -> (PathBuf, Vec<PathBuf>) {
    let dir = std::env::temp_dir().join(format!("kk-player-resume-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let files = ["01.mkv", "02.mkv", "03.mkv"]
        .iter()
        .map(|n| {
            let p = dir.join(n);
            std::fs::write(&p, b"x").unwrap();
            p
        })
        .collect();
    (dir, files)
}

/// The key the controller builds for the source string it loads.
fn key(path: &Path) -> String {
    ResumeKey::for_source(&path.to_string_lossy())
        .expect("a local file has a key")
        .as_str()
        .to_owned()
}

fn point(secs: u64) -> Option<ResumePoint> {
    Some(ResumePoint {
        position_us: secs * S,
        duration_us: DURATION_US,
    })
}

fn seeks(c: &Controller<FakeEngine>) -> Vec<String> {
    c.engine
        .commands
        .iter()
        .filter(|c| c.starts_with("seek"))
        .cloned()
        .collect()
}

/// Opens `file` with a fresh shared store; plays to `secs` and polls (saved by interval).
fn playing_at(file: &Path, secs: f64) -> (Controller<FakeEngine>, Shared) {
    let store = Shared::default();
    let mut c = Controller::new(FakeEngine::default()).with_resume(Box::new(store.clone()));
    c.open(file);
    c.finish();
    c.engine.time = secs;
    c.poll();
    (c, store)
}

#[test]
fn saved_local_position_restores_rewound_with_a_notice() {
    let (dir, files) = folder("restore");
    let store = Shared::default();
    store.seed(&key(&files[0]), 600 * S, DURATION_US);
    let mut c = Controller::new(FakeEngine::default()).with_resume(Box::new(store.clone()));
    c.open(&files[0]);
    c.finish();
    assert!(c.view.has_media && c.view.playing);
    assert_eq!(seeks(&c), ["seek 597.000 absolute"], "3 s before the point");
    assert_eq!(c.view.position, 597.0);
    assert!(c.view.notice.contains("이어서 재생"), "notice says why");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn saved_point_is_revalidated_against_the_opened_duration() {
    let (dir, files) = folder("revalidate");
    let store = Shared::default();
    // Valid for the stored 3000 s, but within the last 60 s of the 1500 s the engine reports.
    store.seed(&key(&files[0]), 1480 * S, 3000 * S);
    // Valid for both lengths: restored.
    store.seed(&key(&files[1]), 600 * S, 3000 * S);
    let mut c = Controller::new(FakeEngine::default()).with_resume(Box::new(store.clone()));
    c.open(&files[0]);
    c.finish();
    assert!(c.view.has_media && c.view.playing);
    assert!(
        seeks(&c).is_empty(),
        "finished point: start from the beginning"
    );
    assert_eq!(c.view.position, 0.0);
    assert!(!c.view.notice.contains("이어서 재생"));
    go(&mut c, Command::Item(1));
    assert_eq!(seeks(&c), ["seek 597.000 absolute"]);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn playing_saves_every_ten_seconds_of_media_time() {
    let (dir, files) = folder("interval");
    let k = key(&files[0]);
    let (mut c, store) = playing_at(&files[0], 100.0);
    assert_eq!(store.saved(&k), point(100));
    c.engine.time = 105.0;
    c.poll();
    assert_eq!(
        store.saved(&k),
        point(100),
        "under 10 s since the last save"
    );
    c.engine.time = 111.0;
    c.poll();
    assert_eq!(store.saved(&k), point(111));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn pausing_saves_the_position() {
    let (dir, files) = folder("pause");
    let k = key(&files[0]);
    let (mut c, store) = playing_at(&files[0], 100.0);
    c.engine.time = 105.0;
    c.poll();
    c.apply(Command::PlayPause);
    assert!(!c.view.playing);
    assert_eq!(store.saved(&k), point(105));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn switching_items_saves_the_old_item() {
    let (dir, files) = folder("switch");
    let (mut c, store) = playing_at(&files[0], 100.0);
    c.engine.time = 105.0;
    c.poll();
    go(&mut c, Command::Next);
    assert_eq!(c.view.current, Some(1));
    assert_eq!(store.saved(&key(&files[0])), point(105), "Next");

    c.engine.time = 200.0;
    c.poll();
    c.engine.time = 205.0;
    c.poll();
    go(&mut c, Command::Item(2));
    assert_eq!(c.view.current, Some(2));
    assert_eq!(store.saved(&key(&files[1])), point(205), "Item");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn close_command_saves_and_still_closes() {
    let (dir, files) = folder("close");
    let k = key(&files[0]);
    let (mut c, store) = playing_at(&files[0], 100.0);
    c.engine.time = 105.0;
    c.poll();
    assert_eq!(c.apply(Command::Close), Some(Effect::Close));
    assert_eq!(store.saved(&k), point(105));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn save_resume_saves_when_the_window_run_ends() {
    let (dir, files) = folder("runend");
    let k = key(&files[0]);
    let (mut c, store) = playing_at(&files[0], 100.0);
    c.engine.time = 107.0;
    c.poll();
    c.save_resume();
    assert_eq!(store.saved(&k), point(107));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn near_the_end_clears_the_entry() {
    let (dir, files) = folder("nearend");
    let k = key(&files[0]);
    let (mut c, store) = playing_at(&files[0], 100.0);
    assert_eq!(store.saved(&k), point(100));
    c.engine.time = 1460.0;
    c.poll();
    assert_eq!(store.saved(&k), None, "within the last 60 s: finished");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn reaching_the_end_clears_the_entry() {
    let (dir, files) = folder("eof");
    let k = key(&files[0]);
    let (mut c, store) = playing_at(&files[0], 100.0);
    assert_eq!(store.saved(&k), point(100));
    c.engine.eof = true;
    c.poll();
    assert!(!c.view.playing);
    assert_eq!(store.saved(&k), None);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn auto_next_at_the_end_clears_the_finished_item() {
    let (dir, files) = folder("autonext");
    let k = key(&files[0]);
    let (mut c, store) = playing_at(&files[0], 100.0);
    c.apply(Command::ToggleAutoNext);
    c.engine.eof = true;
    c.poll();
    c.finish();
    assert_eq!(c.view.current, Some(1));
    assert_eq!(store.saved(&k), None, "played to its end");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn closing_in_the_first_seconds_clears_the_entry() {
    let (dir, files) = folder("early");
    let k = key(&files[0]);
    let (mut c, store) = playing_at(&files[0], 100.0);
    assert_eq!(store.saved(&k), point(100));
    // Back to the start (e.g. a seek) without a poll in between.
    c.engine.time = 6.0;
    c.view.position = 6.0;
    assert_eq!(c.apply(Command::Close), Some(Effect::Close));
    assert_eq!(store.saved(&k), None, "under 10 s: nothing to resume");
    std::fs::remove_dir_all(dir).unwrap();
}

/// One source whose root lists two playables resolved to remote streams.
struct Remote;
impl Catalog for Remote {
    fn sources(&self) -> Vec<SourceItem> {
        vec![remote_item("", "Fixture", SourceItemKind::Container)]
    }
    fn browse(&self, _: &SourceRef, _: u32) -> Result<SourcePage, SourceError> {
        Ok(SourcePage {
            items: vec![
                remote_item("e1", "Episode 1", SourceItemKind::Playable),
                remote_item("e2", "Episode 2", SourceItemKind::Playable),
            ],
            next_page: None,
        })
    }
    fn resolve(&self, at: &SourceRef) -> Result<Vec<StreamCandidate>, SourceError> {
        Ok(vec![StreamCandidate {
            url: format!("http://127.0.0.1:9/{}.mp4", at.id),
            headers: vec![],
        }])
    }
}

fn remote_item(id: &str, title: &str, kind: SourceItemKind) -> SourceItem {
    SourceItem {
        reference: SourceRef {
            provider: "com.example.fixture".into(),
            id: id.into(),
        },
        title: title.into(),
        kind,
    }
}

#[test]
fn remote_items_never_touch_the_store() {
    let store = Shared::default();
    let mut c = Controller::new(FakeEngine::default())
        .with_catalog(Box::new(Remote))
        .with_resume(Box::new(store.clone()));
    go(&mut c, Command::Browse);
    go(&mut c, Command::BrowseItem(0)); // source root
    go(&mut c, Command::BrowseItem(0)); // episode 1
    assert!(c.view.has_media && c.view.playing);
    c.engine.time = 100.0;
    c.poll();
    c.engine.time = 200.0;
    c.poll();
    c.apply(Command::PlayPause);
    c.apply(Command::PlayPause);
    go(&mut c, Command::Next);
    assert_eq!(c.view.current, Some(1));
    c.engine.time = 300.0;
    c.poll();
    c.save_resume();
    c.engine.eof = true;
    c.poll();
    assert_eq!(c.apply(Command::Close), Some(Effect::Close));
    assert!(seeks(&c).is_empty());
    assert_eq!(store.calls(), 0, "no key for plugin items");
}

#[test]
fn failing_store_never_blocks_playback() {
    let (dir, files) = folder("failing");
    let mut c = Controller::new(FakeEngine::default()).with_resume(Box::new(Failing));
    c.open(&files[0]);
    c.finish();
    assert!(c.view.has_media && c.view.playing, "open still plays");
    assert!(c.view.status.is_empty(), "no error on screen");
    c.engine.time = 100.0;
    assert!(c.poll());
    assert_eq!(c.view.position, 100.0);
    c.apply(Command::PlayPause);
    assert!(!c.view.playing, "pause still works");
    c.apply(Command::PlayPause);
    go(&mut c, Command::Next);
    assert_eq!(c.view.current, Some(1), "switching still works");
    c.save_resume();
    assert_eq!(c.apply(Command::Close), Some(Effect::Close));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn restore_survives_polls_before_the_engine_reaches_the_point() {
    let (dir, files) = folder("pending");
    let k = key(&files[0]);
    let store = Shared::default();
    store.seed(&k, 600 * S, DURATION_US);
    let mut c = Controller::new(FakeEngine::default()).with_resume(Box::new(store.clone()));
    c.open(&files[0]);
    c.finish();
    // The engine still reports the old time: the seek has not landed yet.
    c.poll();
    assert_eq!(
        store.saved(&k),
        point(600),
        "a stale time is not a position"
    );
    assert_eq!(c.apply(Command::Close), Some(Effect::Close));
    assert_eq!(store.saved(&k), point(600), "closing before the seek lands");
    // Once the engine is at the point, saving goes on as usual.
    c.engine.time = 597.0;
    c.poll();
    c.engine.time = 610.0;
    c.poll();
    assert_eq!(store.saved(&k), point(610));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_seek_before_the_restore_lands_counts_as_the_position() {
    let (dir, files) = folder("pendingseek");
    let k = key(&files[0]);
    let store = Shared::default();
    store.seed(&k, 600 * S, DURATION_US);
    let mut c = Controller::new(FakeEngine::default()).with_resume(Box::new(store.clone()));
    c.open(&files[0]);
    c.finish();
    c.apply(Command::Seek(0.2)); // 300 s of 1500
    c.engine.time = 300.0;
    c.poll();
    c.engine.time = 305.0;
    c.poll();
    assert_eq!(c.apply(Command::Close), Some(Effect::Close));
    assert_eq!(store.saved(&k), point(305));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_file_opened_by_a_relative_path_resumes() {
    // Missing on purpose: the key of a missing file is its normalized absolute path.
    let relative = Path::new("kk-player-resume-relative/01.mkv");
    let absolute = std::path::absolute(relative).unwrap();
    let store = Shared::default();
    store.seed(&key(&absolute), 600 * S, DURATION_US);
    let mut c = Controller::new(FakeEngine::default()).with_resume(Box::new(store.clone()));
    c.open(relative);
    c.finish();
    assert_eq!(seeks(&c), ["seek 597.000 absolute"]);
}
