//! Platform a registry selects for: OS and (for GUI adapters) the display server.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsId {
    Windows,
    Linux,
    MacOs,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayServer {
    Win32,
    X11,
    Wayland,
    Quartz,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TargetPlatform {
    pub os: OsId,
    /// `None` for headless sessions (CLI without a display).
    pub display: Option<DisplayServer>,
}

impl OsId {
    /// OS of this build, from `std::env::consts::OS` (no cfg branch needed).
    pub fn of_build() -> Self {
        Self::parse(std::env::consts::OS)
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "windows" => Self::Windows,
            "linux" => Self::Linux,
            "macos" => Self::MacOs,
            _ => Self::Other,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::MacOs => "macos",
            Self::Other => "other",
        }
    }
}

impl DisplayServer {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Win32 => "win32",
            Self::X11 => "x11",
            Self::Wayland => "wayland",
            Self::Quartz => "quartz",
        }
    }
}

impl TargetPlatform {
    pub const fn headless(os: OsId) -> Self {
        Self { os, display: None }
    }

    pub const fn with_display(os: OsId, display: DisplayServer) -> Self {
        Self {
            os,
            display: Some(display),
        }
    }
}
