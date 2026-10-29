//! Playback context (P1 #32): OS-neutral commands, the backend port that platform adapters
//! (Windows native surface, Unix mpv IPC) implement, and resume policy (#9).
pub mod backend;
pub mod control;
pub mod resume;
pub mod session;
pub mod store;
pub mod surface;

pub use backend::{BackendStatus, FakeBackend, PlaybackBackend, check_backend};
pub use control::{MAX_SEEK_US, PlaybackControl};
pub use resume::{ResumePoint, ResumePolicy};
pub use session::ResumingPlayback;
pub use store::{FileResumeStore, MemoryResumeStore, ResumeStore, check_resume_store};
pub use surface::{FakeSurface, RgbaFrame, VideoSurface, check_surface};
