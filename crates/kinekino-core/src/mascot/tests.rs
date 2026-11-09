use super::*;
use std::sync::Mutex;

/// Wire names the external preset state machine listens for (guide mascot-preset-reference,
/// "앱 이벤트 → 프리셋"). Pointer, drag, typing and timer events stay with the mascot itself.
const PRESET_APP_EVENTS: &[&str] = &[
    "app_start",
    "app_exit",
    "load_start",
    "load_end",
    "success",
    "error",
    "warning",
    "notification",
    "music_start",
    "music_pause",
    "offline",
    "online",
    "task_complete",
];

#[derive(Default)]
struct Recorder(Mutex<Vec<MascotEvent>>);

impl MascotEventPort for Recorder {
    fn notify(&self, event: MascotEvent) {
        self.0.lock().unwrap().push(event);
    }
}

#[test]
fn every_event_has_a_distinct_preset_wire_name() {
    let names: Vec<&str> = MascotEvent::ALL.iter().map(|e| e.wire_name()).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), names.len(), "duplicate wire names: {names:?}");
    for n in &names {
        assert!(
            PRESET_APP_EVENTS.contains(n),
            "`{n}` is not a preset app event"
        );
    }
    for n in PRESET_APP_EVENTS {
        assert!(
            names.contains(n),
            "preset app event `{n}` has no MascotEvent"
        );
    }
}

#[test]
fn variant_payload_does_not_change_the_wire_name() {
    let a = MascotEvent::AppStart { first_run: true };
    let b = MascotEvent::AppStart { first_run: false };
    assert_eq!(a.wire_name(), b.wire_name());
    let f = MascotEvent::Error { fatal: true };
    assert_eq!(f.wire_name(), "error");
}

#[test]
fn no_mascot_accepts_every_event_without_effect() {
    let port: &dyn MascotEventPort = &NoMascot;
    for e in MascotEvent::ALL {
        port.notify(*e);
    }
    port.notify(MascotEvent::AppStart { first_run: true });
    port.notify(MascotEvent::Error { fatal: true });
}

#[test]
fn port_is_object_safe_and_shareable_across_threads() {
    fn assert_send_sync<T: Send + Sync + ?Sized>() {}
    assert_send_sync::<dyn MascotEventPort>();
    let rec = std::sync::Arc::new(Recorder::default());
    let port: std::sync::Arc<dyn MascotEventPort> = rec.clone();
    std::thread::spawn(move || port.notify(MascotEvent::LoadStart))
        .join()
        .unwrap();
    assert_eq!(*rec.0.lock().unwrap(), vec![MascotEvent::LoadStart]);
}
