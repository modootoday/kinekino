//! Whether an engine can run here: compiled in and its runtime present.
use crate::engine::port::EngineKind;
use std::path::{Path, PathBuf};

/// Directory of an unpacked CEF binary distribution (holds `libcef.so` / `libcef.dll`).
pub const CEF_DIR_ENV: &str = "KINEKINO_CEF_DIR";
/// Installed Google Chrome executable; defaults to `/usr/bin/google-chrome`.
pub const CHROME_PATH_ENV: &str = "KINEKINO_CHROME_PATH";
const DEFAULT_CHROME: &str = "/usr/bin/google-chrome";

pub trait Availability {
    fn is_available(&self, kind: EngineKind) -> bool;
}

/// Probes the real machine through the environment variables above.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemAvailability;

impl SystemAvailability {
    pub fn chrome_executable() -> PathBuf {
        std::env::var_os(CHROME_PATH_ENV).map_or_else(|| DEFAULT_CHROME.into(), PathBuf::from)
    }

    pub fn cef_dir() -> Option<PathBuf> {
        std::env::var_os(CEF_DIR_ENV).map(PathBuf::from)
    }

    fn has_libcef(dir: &Path) -> bool {
        dir.join("libcef.so").exists() || dir.join("libcef.dll").exists()
    }
}

impl Availability for SystemAvailability {
    fn is_available(&self, kind: EngineKind) -> bool {
        match kind {
            EngineKind::WebView2 => cfg!(all(windows, feature = "webview2")),
            EngineKind::Cef => {
                cfg!(feature = "cef") && Self::cef_dir().is_some_and(|d| Self::has_libcef(&d))
            }
            EngineKind::SystemChrome => {
                cfg!(all(target_os = "linux", feature = "chrome-embed"))
                    && Self::chrome_executable().exists()
            }
            EngineKind::ChromeCdp => cfg!(feature = "cdp") && Self::chrome_executable().exists(),
        }
    }
}
