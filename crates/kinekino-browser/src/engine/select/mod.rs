//! Engine selection: the app states required capabilities, the matrix picks the engine.
mod availability;
mod matrix;

pub use availability::{Availability, CEF_DIR_ENV, CHROME_PATH_ENV, SystemAvailability};
pub use matrix::{Platform, Role};

use crate::engine::port::{CapabilitySet, EngineKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub kind: EngineKind,
    pub role: Role,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectError {
    /// No officially supported engine on this platform.
    UnsupportedPlatform,
    /// Supported engines exist but none is installed and capable; lists what each lacks.
    NoCapableEngine {
        rejected: Vec<(EngineKind, Rejection)>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    NotAvailable,
    Missing(CapabilitySet),
}

impl std::fmt::Display for SelectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(f, "no supported browser engine on this platform"),
            Self::NoCapableEngine { rejected } => {
                write!(f, "no capable browser engine:")?;
                for (kind, why) in rejected {
                    match why {
                        Rejection::NotAvailable => write!(f, " {} not available;", kind.name())?,
                        Rejection::Missing(m) => write!(f, " {} lacks {m};", kind.name())?,
                    }
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for SelectError {}

/// First engine of the platform matrix (default, then fallback) that is available and
/// declares every required capability.
pub fn select(
    platform: Platform,
    required: CapabilitySet,
    availability: &dyn Availability,
) -> Result<Selection, SelectError> {
    let candidates = platform.supported();
    if candidates.is_empty() {
        return Err(SelectError::UnsupportedPlatform);
    }
    let mut rejected = Vec::new();
    for &(kind, role) in candidates {
        let missing = kind.declared().missing(required);
        if !missing.is_empty() {
            rejected.push((kind, Rejection::Missing(missing)));
        } else if !availability.is_available(kind) {
            rejected.push((kind, Rejection::NotAvailable));
        } else {
            return Ok(Selection { kind, role });
        }
    }
    Err(SelectError::NoCapableEngine { rejected })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::port::Capability::*;

    struct Installed(&'static [EngineKind]);
    impl Availability for Installed {
        fn is_available(&self, kind: EngineKind) -> bool {
            self.0.contains(&kind)
        }
    }

    const ALL: Installed = Installed(&EngineKind::ALL);

    #[test]
    fn linux_defaults_to_cef_and_falls_back_to_chrome_for_h264() {
        let embed = CapabilitySet::of(&[WindowEmbed]);
        let s = select(Platform::LinuxX11, embed, &ALL).unwrap();
        assert_eq!((s.kind, s.role), (EngineKind::Cef, Role::Default));
        let h264 = CapabilitySet::of(&[WindowEmbed, H264]);
        let s = select(Platform::LinuxX11, h264, &ALL).unwrap();
        assert_eq!((s.kind, s.role), (EngineKind::SystemChrome, Role::Fallback));
    }

    #[test]
    fn linux_without_cef_uses_chrome() {
        let s = select(
            Platform::LinuxX11,
            CapabilitySet::EMPTY,
            &Installed(&[EngineKind::SystemChrome]),
        )
        .unwrap();
        assert_eq!(s.kind, EngineKind::SystemChrome);
    }

    #[test]
    fn windows_defaults_to_webview2_then_cef() {
        let s = select(Platform::Windows, CapabilitySet::EMPTY, &ALL).unwrap();
        assert_eq!(s.kind, EngineKind::WebView2);
        let s = select(Platform::Windows, CapabilitySet::of(&[Eval]), &ALL).unwrap();
        assert_eq!((s.kind, s.role), (EngineKind::Cef, Role::Fallback));
    }

    #[test]
    fn reports_missing_capabilities() {
        let req = CapabilitySet::of(&[H264, WindowEmbed]);
        let err = select(Platform::LinuxX11, req, &Installed(&[EngineKind::Cef])).unwrap_err();
        assert_eq!(
            err,
            SelectError::NoCapableEngine {
                rejected: vec![
                    (
                        EngineKind::Cef,
                        Rejection::Missing(CapabilitySet::of(&[H264]))
                    ),
                    (EngineKind::SystemChrome, Rejection::NotAvailable),
                ]
            }
        );
        assert_eq!(
            select(Platform::Unsupported, req, &ALL),
            Err(SelectError::UnsupportedPlatform)
        );
    }

    #[test]
    fn cdp_engine_is_never_a_gui_choice() {
        for p in [Platform::Windows, Platform::LinuxX11] {
            assert!(
                p.supported()
                    .iter()
                    .all(|(k, _)| *k != EngineKind::ChromeCdp)
            );
        }
    }
}
