//! Platform support matrix: exactly one default and one fallback engine per platform.
use crate::engine::port::EngineKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    /// Linux with an X11 (or XWayland) display.
    LinuxX11,
    /// Anything else (macOS, native Wayland): no officially supported GUI engine.
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Default,
    Fallback,
}

impl Platform {
    /// Platform of this build and session (Linux needs `DISPLAY` for X11 embedding).
    pub fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "linux") && std::env::var_os("DISPLAY").is_some() {
            Self::LinuxX11
        } else {
            Self::Unsupported
        }
    }

    /// Officially supported engines in preference order (ADR 0002).
    pub const fn supported(self) -> &'static [(EngineKind, Role)] {
        match self {
            Self::Windows => &[
                (EngineKind::WebView2, Role::Default),
                (EngineKind::Cef, Role::Fallback),
            ],
            Self::LinuxX11 => &[
                (EngineKind::Cef, Role::Default),
                (EngineKind::SystemChrome, Role::Fallback),
            ],
            Self::Unsupported => &[],
        }
    }
}
