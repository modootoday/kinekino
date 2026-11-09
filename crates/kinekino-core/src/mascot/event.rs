//! App-level events the host emits for an optional mascot.
//!
//! Only facts the host observes are here. Pointer, drag, typing, voice and idle events, and
//! timers such as `load_long`, belong to the mascot itself and are not host events.

/// One app-level happening. Wire names match the preset state machine's event keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MascotEvent {
    /// The app opened; `first_run` is true on the first launch for this user.
    AppStart { first_run: bool },
    /// The app is closing; a mascot gets about one second, the host does not wait.
    AppExit,
    /// A load (media, page, job input) started.
    LoadStart,
    /// The load that started last ended, successfully or not.
    LoadEnd,
    /// A user-visible action succeeded.
    Success,
    /// A user-visible action failed; `fatal` when the app cannot continue it.
    Error { fatal: bool },
    /// Something needs the user's attention but nothing failed.
    Warning,
    /// A notification was shown.
    Notification,
    /// Playback started or resumed.
    PlaybackStart,
    /// Playback paused or stopped.
    PlaybackPause,
    /// The network became unavailable.
    Offline,
    /// The network came back.
    Online,
    /// A long-running task finished.
    TaskComplete,
}

impl MascotEvent {
    /// One value of every variant, for adapters and contract tests.
    pub const ALL: &'static [MascotEvent] = &[
        MascotEvent::AppStart { first_run: false },
        MascotEvent::AppExit,
        MascotEvent::LoadStart,
        MascotEvent::LoadEnd,
        MascotEvent::Success,
        MascotEvent::Error { fatal: false },
        MascotEvent::Warning,
        MascotEvent::Notification,
        MascotEvent::PlaybackStart,
        MascotEvent::PlaybackPause,
        MascotEvent::Offline,
        MascotEvent::Online,
        MascotEvent::TaskComplete,
    ];

    /// The event key a preset state machine listens for.
    pub fn wire_name(self) -> &'static str {
        match self {
            MascotEvent::AppStart { .. } => "app_start",
            MascotEvent::AppExit => "app_exit",
            MascotEvent::LoadStart => "load_start",
            MascotEvent::LoadEnd => "load_end",
            MascotEvent::Success => "success",
            MascotEvent::Error { .. } => "error",
            MascotEvent::Warning => "warning",
            MascotEvent::Notification => "notification",
            MascotEvent::PlaybackStart => "music_start",
            MascotEvent::PlaybackPause => "music_pause",
            MascotEvent::Offline => "offline",
            MascotEvent::Online => "online",
            MascotEvent::TaskComplete => "task_complete",
        }
    }
}
