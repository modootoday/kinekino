//! Registry ↔ legacy `select()` equivalence and browser contract checks.
use super::*;
use crate::engine::port::Capability;
use crate::engine::select::{self, Availability as EngineAvailability};
use kinekino_core::port::contract::{
    check_capability_table, check_error_mapping, check_unique_kinds,
};
use kinekino_core::port::registry::Policy;

struct Installed(Vec<EngineKind>);
impl EngineAvailability for Installed {
    fn is_available(&self, kind: EngineKind) -> bool {
        self.0.contains(&kind)
    }
}

fn subsets() -> Vec<Vec<EngineKind>> {
    (0..16u8)
        .map(|m| {
            EngineKind::ALL
                .into_iter()
                .enumerate()
                .filter(|(i, _)| m & (1 << i) != 0)
                .map(|(_, k)| k)
                .collect()
        })
        .collect()
}

fn targets() -> Vec<TargetPlatform> {
    use DisplayServer::*;
    use OsId::*;
    vec![
        TargetPlatform::headless(Windows),
        TargetPlatform::with_display(Windows, Win32),
        TargetPlatform::with_display(Linux, X11),
        TargetPlatform::with_display(Linux, Wayland),
        TargetPlatform::headless(Linux),
        TargetPlatform::headless(MacOs),
    ]
}

#[test]
fn registry_matches_legacy_select_exhaustively() {
    let requireds = [
        CapabilitySet::EMPTY,
        CapabilitySet::of(&[Capability::H264]),
        CapabilitySet::of(&[Capability::WindowEmbed, Capability::Eval]),
        CapabilitySet::of(&[Capability::RequestInterception]),
        CapabilitySet::of(&[Capability::Vp9, Capability::Widevine]),
    ];
    let mut n = 0;
    for installed in subsets() {
        let avail: Rc<dyn EngineAvailability> = Rc::new(Installed(installed.clone()));
        let reg = registry_with(Rc::clone(&avail));
        for t in targets() {
            for req in requireds {
                let legacy = select::select(matrix_platform(t), req, avail.as_ref())
                    .ok()
                    .map(|s| s.kind);
                let new = reg
                    .select(t, req, Policy::PlatformOrder)
                    .ok()
                    .map(|s| s.kind);
                assert_eq!(new, legacy, "{installed:?} {t:?} {req}");
                n += 1;
            }
        }
    }
    assert_eq!(n, 16 * 6 * 5);
}

#[test]
fn chrome_cdp_is_policy_only() {
    let reg = registry_with(Rc::new(Installed(EngineKind::ALL.to_vec())));
    let t = TargetPlatform::headless(OsId::Linux);
    assert!(
        reg.select(t, CapabilitySet::EMPTY, Policy::PlatformOrder)
            .is_err()
    );
    let s = reg
        .select(t, CapabilitySet::EMPTY, Policy::Only(EngineKind::ChromeCdp))
        .unwrap();
    assert_eq!((s.kind, s.role), (EngineKind::ChromeCdp, Role::Explicit));
}

#[test]
fn browser_port_meets_contract() {
    check_unique_kinds(&EngineKind::ALL);
    check_capability_table(CapabilitySet::of(&Capability::ALL));
    check_error_mapping(&[
        EngineError::Launch("x".into()),
        EngineError::Call("y".into()),
        EngineError::Unsupported("z"),
    ]);
    assert_eq!(
        EngineError::Unsupported("z").error_code(),
        ErrorCode::Unsupported
    );
}

#[test]
fn embedded_build_without_parent_is_rejected_not_panicking() {
    let reg = registry_with(Rc::new(Installed(EngineKind::ALL.to_vec())));
    let sel = reg
        .select(
            TargetPlatform::with_display(OsId::Linux, DisplayServer::X11),
            CapabilitySet::EMPTY,
            Policy::PlatformOrder,
        )
        .unwrap();
    let err = reg
        .build(sel, &BrowserBuildConfig::default())
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::Unavailable, "{err}");
    let wv = reg
        .build(
            kinekino_core::port::registry::Selection {
                kind: EngineKind::WebView2,
                role: Role::Default,
            },
            &BrowserBuildConfig::default(),
        )
        .err()
        .unwrap();
    assert_eq!(wv.code, ErrorCode::Unsupported);
}

#[test]
fn build_config_defaults_and_rejects_unknown() {
    let c: BrowserBuildConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(c, BrowserBuildConfig::default());
    assert!(serde_json::from_str::<BrowserBuildConfig>(r#"{"api_key":"x"}"#).is_err());
}
