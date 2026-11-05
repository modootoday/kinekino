//! Player prefs in the controller (ADR 20261105092851): volume, mute, speed, and auto-next
//! load on attach and save when the user changes them. A failing store never blocks playback.
use super::support::FakeEngine;
use kinekino_core::playback::{MemoryPlayerPrefsStore, PlayerPrefs, PlayerPrefsStore};
use kinekino_core::port::error::{ErrorCode, PortError, PortResult};
use kinekino_player_ui::{Command, Controller};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct Shared {
    store: Arc<Mutex<MemoryPlayerPrefsStore>>,
    calls: Arc<AtomicUsize>,
}

impl Shared {
    fn with(prefs: PlayerPrefs) -> Self {
        Self {
            store: Arc::new(Mutex::new(MemoryPlayerPrefsStore::with(prefs))),
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }
    fn current(&self) -> PlayerPrefs {
        self.store.lock().unwrap().current()
    }
    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl PlayerPrefsStore for Shared {
    fn load(&self) -> PortResult<PlayerPrefs> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.store.lock().unwrap().load()
    }
    fn save(&mut self, prefs: &PlayerPrefs) -> PortResult<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.store.lock().unwrap().save(prefs)
    }
}

struct Failing;
impl PlayerPrefsStore for Failing {
    fn load(&self) -> PortResult<PlayerPrefs> {
        Err(PortError::new(ErrorCode::Io, "prefs unavailable"))
    }
    fn save(&mut self, _: &PlayerPrefs) -> PortResult<()> {
        Err(PortError::new(ErrorCode::Io, "prefs unavailable"))
    }
}

#[test]
fn with_prefs_applies_saved_volume_mute_speed_and_auto_next() {
    let saved = PlayerPrefs {
        volume: 40.0,
        muted: true,
        speed: 1.5,
        auto_next: true,
    };
    let shared = Shared::with(saved);
    let c = Controller::new(FakeEngine::default()).with_prefs(Box::new(shared.clone()));
    assert_eq!(c.view.volume, 40.0);
    assert!(c.view.muted);
    assert_eq!(c.view.speed, 1.5);
    assert!(c.view.auto_next);
    assert!(
        c.engine.commands.iter().any(|s| s == "set volume 40"),
        "engine volume applied: {:?}",
        c.engine.commands
    );
    assert!(c.engine.commands.iter().any(|s| s == "set mute yes"));
    assert!(c.engine.commands.iter().any(|s| s == "set speed 1.50"));
    assert!(shared.calls() >= 1, "load was called");
}

#[test]
fn volume_mute_speed_and_auto_next_changes_are_saved() {
    let shared = Shared::default();
    let mut c = Controller::new(FakeEngine::default()).with_prefs(Box::new(shared.clone()));
    c.apply(Command::VolumeDown);
    c.apply(Command::VolumeDown);
    assert_eq!(c.view.volume, 90.0);
    assert_eq!(shared.current().volume, 90.0);

    c.apply(Command::Mute);
    assert!(c.view.muted && shared.current().muted);

    c.apply(Command::SpeedUp);
    assert_eq!(c.view.speed, 1.1);
    assert_eq!(shared.current().speed, 1.1);

    c.apply(Command::ToggleAutoNext);
    assert!(c.view.auto_next && shared.current().auto_next);
}

#[test]
fn close_saves_current_prefs() {
    let shared = Shared::default();
    let mut c = Controller::new(FakeEngine::default()).with_prefs(Box::new(shared.clone()));
    c.view.volume = 33.0;
    c.view.muted = true;
    c.view.speed = 2.0;
    c.view.auto_next = true;
    c.apply(Command::Close);
    assert_eq!(
        shared.current(),
        PlayerPrefs {
            volume: 33.0,
            muted: true,
            speed: 2.0,
            auto_next: true,
        }
    );
}

#[test]
fn failing_prefs_store_does_not_block_commands() {
    let mut c = Controller::new(FakeEngine::default()).with_prefs(Box::new(Failing));
    // Defaults remain; volume still moves.
    assert_eq!(c.view.volume, 100.0);
    c.apply(Command::VolumeDown);
    assert_eq!(c.view.volume, 95.0);
    c.apply(Command::Mute);
    assert!(c.view.muted);
    c.apply(Command::Close);
}
