//! Playback context (P1 #32): OS-neutral commands, the backend port that platform adapters
//! (Windows native surface, Unix mpv IPC) implement, and resume policy (#9).
pub mod backend;
pub mod control;
pub mod prefs;
pub mod resume;
pub mod resume_file;
pub mod resume_key;
pub mod session;
pub mod skip_range;
pub mod skip_range_file;
pub mod store;
pub mod surface;

pub use backend::{BackendStatus, FakeBackend, PlaybackBackend, check_backend};
pub use control::{MAX_SEEK_US, PlaybackControl};
pub use prefs::{
    FilePlayerPrefsStore, MemoryPlayerPrefsStore, PlayerPrefs, PlayerPrefsStore,
    check_player_prefs_store, default_player_prefs_file,
};
pub use resume::{ResumePoint, ResumePolicy};
pub use resume_file::{FileResumeStore, default_resume_file};
pub use resume_key::ResumeKey;
pub use session::ResumingPlayback;
pub use skip_range::SkipRange;
pub use skip_range_file::{FileSkipRangeStore, default_skip_ranges_file};
pub use store::{
    MemoryResumeStore, MemorySkipRangeStore, ResumeStore, SkipRangeStore, check_resume_store,
    check_skip_range_store,
};
pub use surface::{FakeSurface, RgbaFrame, VideoSurface, check_surface};
