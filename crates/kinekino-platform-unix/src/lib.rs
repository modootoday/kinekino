//! Linux/macOS adapter. mpv playback backend (`mpv`) is real; Chromium browser surface lands later.
pub mod fs;
pub mod id;
pub mod io;
pub mod ipc;
pub mod mpv;
pub mod process;
pub mod publication;
pub mod session;
pub mod token_file;

use kinekino_core::platform::{OsFamily, PlatformHost};

/// Unix platform host. No GUI adapters yet; CLI-only capabilities.
#[derive(Debug, Default)]
pub struct UnixHost;

impl PlatformHost for UnixHost {
    fn os_family(&self) -> OsFamily {
        OsFamily::Unix
    }
    fn has_embedded_browser(&self) -> bool {
        false
    }
    fn has_native_video_surface(&self) -> bool {
        false
    }
    fn has_native_desktop_window(&self) -> bool {
        false
    }
    fn has_mandatory_share_modes(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_host_reports_unix_without_gui_adapters() {
        let host = UnixHost;
        assert_eq!(host.os_family(), OsFamily::Unix);
        assert!(!host.has_embedded_browser() && !host.has_native_video_surface());
    }
}
