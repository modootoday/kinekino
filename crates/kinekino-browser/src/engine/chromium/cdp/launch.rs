//! Launch settings for installed Google Chrome; path override via KINEKINO_CHROME_PATH.
use crate::engine::port::{EngineError, EngineResult};
use crate::engine::select::SystemAvailability;
use chromiumoxide::BrowserConfig;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CdpLaunch {
    pub headless: bool,
    pub profile_dir: PathBuf,
    pub window: (u32, u32),
    /// `--app=<url>` chromeless app window (headed only).
    pub app_url: Option<String>,
}

impl CdpLaunch {
    pub fn executable() -> PathBuf {
        SystemAvailability::chrome_executable()
    }

    pub(super) fn config(&self) -> EngineResult<BrowserConfig> {
        let mut b = BrowserConfig::builder()
            .chrome_executable(Self::executable())
            .user_data_dir(&self.profile_dir)
            .window_size(self.window.0, self.window.1)
            .arg("autoplay-policy=no-user-gesture-required");
        b = if self.headless {
            b.new_headless_mode()
        } else {
            // Headed: no device-metrics emulation, so the viewport follows the real
            // window size (required when the window is embedded and resized).
            b.with_head().viewport(None)
        };
        if let Some(url) = &self.app_url {
            b = b.arg(format!("app={url}"));
        }
        b.build().map_err(EngineError::Launch)
    }
}
