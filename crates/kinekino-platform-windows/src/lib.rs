//! Windows adapter (stub). Win32 window, mpv parent surface and WebView2 code move here in step 3+.
//! Windows-only crate: empty on other targets so `--workspace` builds stay green.
#![cfg(windows)]
pub mod fs;
pub mod id;
pub mod io;
pub mod ipc;
pub mod process;
pub mod publication;
pub mod session;
pub mod window;

use kinekino_core::platform::{OsFamily, PlatformHost};

/// Windows platform host. Capabilities stay false until adapters are migrated.
#[derive(Debug, Default)]
pub struct WindowsHost;

impl PlatformHost for WindowsHost {
    fn os_family(&self) -> OsFamily {
        OsFamily::Windows
    }
    fn has_embedded_browser(&self) -> bool {
        false
    }
    fn has_native_video_surface(&self) -> bool {
        false
    }
    fn has_native_desktop_window(&self) -> bool {
        true
    }
    fn has_mandatory_share_modes(&self) -> bool {
        true
    }
}
