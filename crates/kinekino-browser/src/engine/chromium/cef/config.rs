//! CEF launch settings resolved from the host window and `KINEKINO_CEF_DIR`.
use crate::engine::port::{EngineError, EngineResult};
use crate::engine::select::{CEF_DIR_ENV, SystemAvailability};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CefLaunch {
    /// Native parent window: X11 XID (Linux) or HWND (Windows).
    pub parent: u64,
    /// Child size in pixels at (0, 0) of the parent.
    pub size: (i32, i32),
    /// Root cache; keep it stable so the runtime-downloaded Widevine CDM is reused.
    pub cache_dir: PathBuf,
    /// First document (fixture or app shell).
    pub initial_url: String,
}

impl CefLaunch {
    pub(super) fn cef_dir() -> EngineResult<PathBuf> {
        SystemAvailability::cef_dir()
            .ok_or_else(|| EngineError::Launch(format!("{CEF_DIR_ENV} is not set")))
    }

    pub(super) fn resources_dir() -> EngineResult<PathBuf> {
        let dir = Self::cef_dir()?;
        // Minimal distribution keeps resources flat; the full one has Resources/.
        let nested = dir.join("Resources");
        Ok(if nested.is_dir() { nested } else { dir })
    }
}
