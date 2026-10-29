//! Browser port on the shared `Registry`: registrations are generated from the ADR 0002
//! platform matrix, so `Registry::select` and [`crate::engine::select::select`] agree.
mod factory;
#[cfg(test)]
mod tests;

use crate::engine::port::{BrowserEngine, CapabilitySet, EngineError, EngineKind};
use crate::engine::select::{
    Availability as EngineAvailability, Platform, Role as MatrixRole, SystemAvailability,
};
use kinekino_core::port::registry::{Availability, PlatformRule, Registration, Registry, Role};
use kinekino_core::port::{
    AdapterKind, CapabilityTable, DisplayServer, ErrorCode, OsId, TargetPlatform, ToPortError,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::rc::Rc;

pub const PORT: &str = "browser";

/// Build settings shared by all browser adapters. Holds no secrets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BrowserBuildConfig {
    pub headless: bool,
    pub profile_dir: PathBuf,
    pub window: (u32, u32),
    /// First document / `--app` URL.
    pub initial_url: Option<String>,
    /// Native host window (X11 XID / HWND) for embedded adapters.
    pub parent: Option<u64>,
}

impl Default for BrowserBuildConfig {
    fn default() -> Self {
        Self {
            headless: true,
            profile_dir: std::env::temp_dir().join("kinekino-browser-profile"),
            window: (1280, 720),
            initial_url: None,
            parent: None,
        }
    }
}

pub type BrowserRegistry =
    Registry<dyn BrowserEngine, EngineKind, CapabilitySet, BrowserBuildConfig>;

impl CapabilityTable for CapabilitySet {
    fn empty() -> Self {
        Self::EMPTY
    }
    fn contains_all(self, required: Self) -> bool {
        CapabilitySet::contains_all(self, required)
    }
    fn missing(self, required: Self) -> Self {
        CapabilitySet::missing(self, required)
    }
    fn is_empty(self) -> bool {
        CapabilitySet::is_empty(self)
    }
    fn names(self) -> Vec<&'static str> {
        self.iter().map(|c| c.name()).collect()
    }
}

impl AdapterKind for EngineKind {
    fn name(self) -> &'static str {
        EngineKind::name(self)
    }
}

impl ToPortError for EngineError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::Launch(_) => ErrorCode::Unavailable,
            Self::Call(_) => ErrorCode::Internal,
            Self::Unsupported(_) => ErrorCode::Unsupported,
        }
    }
}

/// Matrix platform a target maps to (Linux needs X11/XWayland; Wayland-only is unsupported).
pub fn matrix_platform(t: TargetPlatform) -> Platform {
    match (t.os, t.display) {
        (OsId::Windows, _) => Platform::Windows,
        (OsId::Linux, Some(DisplayServer::X11)) => Platform::LinuxX11,
        _ => Platform::Unsupported,
    }
}

/// Platform rules of one engine, derived from the matrix. `ChromeCdp` (headless/CLI) is
/// not in the matrix and is selectable only by policy.
pub fn rules(kind: EngineKind) -> Vec<PlatformRule> {
    if kind == EngineKind::ChromeCdp {
        return [OsId::Linux, OsId::Windows]
            .map(|os| PlatformRule::new(os, None, Role::Explicit))
            .to_vec();
    }
    let targets = [
        (Platform::Windows, OsId::Windows, None),
        (Platform::LinuxX11, OsId::Linux, Some(DisplayServer::X11)),
    ];
    targets
        .into_iter()
        .flat_map(|(p, os, display)| {
            p.supported()
                .iter()
                .filter(move |(k, _)| *k == kind)
                .map(move |(_, r)| {
                    let role = match r {
                        MatrixRole::Default => Role::Default,
                        MatrixRole::Fallback => Role::Fallback,
                    };
                    PlatformRule::new(os, display, role)
                })
        })
        .collect()
}

/// Registry over every engine kind with the given availability probe.
pub fn registry_with(availability: Rc<dyn EngineAvailability>) -> BrowserRegistry {
    let mut reg = BrowserRegistry::new(PORT);
    for kind in EngineKind::ALL {
        let probe = Rc::clone(&availability);
        reg.register(Registration::new(
            kind,
            kind.declared(),
            rules(kind),
            move || {
                if probe.is_available(kind) {
                    Availability::Available
                } else {
                    Availability::Missing(factory::missing_reason(kind))
                }
            },
            move |cfg: &BrowserBuildConfig| {
                factory::build(kind, cfg).map_err(|e| e.to_port_error())
            },
        ));
    }
    reg
}

/// Registry probing the real machine (`KINEKINO_CEF_DIR`, `KINEKINO_CHROME_PATH`, features).
pub fn registry() -> BrowserRegistry {
    registry_with(Rc::new(SystemAvailability))
}
