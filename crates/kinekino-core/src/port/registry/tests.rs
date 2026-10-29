//! Registry selection table.
use super::tests_support::{Caps, Kind};
use super::*;
use crate::port::target::{DisplayServer::*, OsId::*};

trait Port {
    fn tag(&self) -> &'static str;
}
struct Impl(&'static str);
impl Port for Impl {
    fn tag(&self) -> &'static str {
        self.0
    }
}

type Reg = Registry<dyn Port, Kind, Caps, ()>;

fn reg(beta_available: bool) -> Reg {
    let mut r = Reg::new("test");
    r.register(Registration::new(
        Kind::Alpha,
        Caps(0b001),
        vec![
            PlatformRule::new(Linux, Some(X11), Role::Fallback),
            PlatformRule::new(Windows, None, Role::Default),
        ],
        || Availability::Available,
        |_| Ok(Box::new(Impl("alpha")) as Box<dyn Port>),
    ));
    r.register(Registration::new(
        Kind::Beta,
        Caps(0b011),
        vec![PlatformRule::new(Linux, Some(X11), Role::Default)],
        move || {
            if beta_available {
                Availability::Available
            } else {
                Availability::Missing("no beta".into())
            }
        },
        |_| Ok(Box::new(Impl("beta")) as Box<dyn Port>),
    ));
    r.register(Registration::new(
        Kind::Gamma,
        Caps(0b111),
        vec![PlatformRule::new(Linux, None, Role::Explicit)],
        || Availability::Available,
        |_| Ok(Box::new(Impl("gamma")) as Box<dyn Port>),
    ));
    r
}

const X: TargetPlatform = TargetPlatform::with_display(Linux, X11);
const WL: TargetPlatform = TargetPlatform::with_display(Linux, Wayland);
const WIN: TargetPlatform = TargetPlatform::headless(Windows);
const MAC: TargetPlatform = TargetPlatform::headless(MacOs);

type Case = (bool, TargetPlatform, u8, Policy<Kind>, Option<(Kind, Role)>);

#[test]
fn selection_table() {
    use Policy::*;
    let cases: &[Case] = &[
        (true, X, 0, PlatformOrder, Some((Kind::Beta, Role::Default))),
        (
            false,
            X,
            0,
            PlatformOrder,
            Some((Kind::Alpha, Role::Fallback)),
        ),
        (
            true,
            X,
            0b010,
            PlatformOrder,
            Some((Kind::Beta, Role::Default)),
        ),
        (false, X, 0b010, PlatformOrder, None),
        (true, X, 0b100, PlatformOrder, None),
        (
            true,
            X,
            0b100,
            Prefer(Kind::Gamma),
            Some((Kind::Gamma, Role::Explicit)),
        ),
        (
            true,
            X,
            0,
            Prefer(Kind::Alpha),
            Some((Kind::Alpha, Role::Fallback)),
        ),
        (
            true,
            X,
            0b010,
            Prefer(Kind::Alpha),
            Some((Kind::Beta, Role::Default)),
        ),
        (
            true,
            X,
            0,
            Only(Kind::Gamma),
            Some((Kind::Gamma, Role::Explicit)),
        ),
        (false, X, 0, Only(Kind::Beta), None),
        (true, WL, 0, PlatformOrder, None),
        (
            true,
            WL,
            0,
            Prefer(Kind::Gamma),
            Some((Kind::Gamma, Role::Explicit)),
        ),
        (
            true,
            WIN,
            0,
            PlatformOrder,
            Some((Kind::Alpha, Role::Default)),
        ),
        (true, WIN, 0b010, PlatformOrder, None),
        (true, MAC, 0, PlatformOrder, None),
    ];
    for (i, (avail, p, req, pol, want)) in cases.iter().enumerate() {
        let got = reg(*avail)
            .select(*p, Caps(*req), *pol)
            .ok()
            .map(|s| (s.kind, s.role));
        assert_eq!(got, *want, "case {i}");
    }
}

#[test]
fn rejection_reasons_are_reported() {
    let err = reg(false)
        .select(X, Caps(0b010), Policy::PlatformOrder)
        .unwrap_err();
    assert_eq!(
        err,
        SelectError::NoCapableAdapter {
            rejected: vec![
                (Kind::Beta, Rejection::NotAvailable("no beta".into())),
                (Kind::Alpha, Rejection::Missing(Caps(0b010))),
                (Kind::Gamma, Rejection::ExplicitOnly),
            ]
        }
    );
    assert_eq!(
        reg(true).select(MAC, Caps(0), Policy::PlatformOrder),
        Err(SelectError::UnsupportedPlatform)
    );
    let pe = err.to_port_error("test");
    assert_eq!(pe.code, ErrorCode::Unavailable);
    assert!(pe.message.contains("beta not available: no beta"), "{pe}");
}

#[test]
fn build_uses_registered_factory() {
    let r = reg(true);
    let (sel, port) = r
        .select_and_build(X, Caps(0), Policy::PlatformOrder, &())
        .unwrap();
    assert_eq!((sel.kind, port.tag()), (Kind::Beta, "beta"));
    let e = r
        .select_and_build(MAC, Caps(0), Policy::PlatformOrder, &())
        .err()
        .unwrap();
    assert_eq!(e.code, ErrorCode::Unsupported);
}

#[test]
fn explain_json_lists_every_candidate() {
    let j = reg(false)
        .explain(X, Caps(0), Policy::PlatformOrder)
        .to_json();
    assert!(
        j.starts_with(r#"{"port":"test","platform":{"os":"linux","display":"x11"}"#),
        "{j}"
    );
    assert!(
        j.contains(r#""selected":{"kind":"alpha","role":"fallback"}"#),
        "{j}"
    );
    assert!(j.contains(r#"{"kind":"beta","role":"default","declared":["a","b"],"status":"not available: no beta"}"#), "{j}");
}

#[test]
fn re_registration_replaces() {
    let mut r = reg(true);
    r.register(Registration::new(
        Kind::Beta,
        Caps(0),
        vec![],
        || Availability::Available,
        |_| Ok(Box::new(Impl("b2")) as Box<dyn Port>),
    ));
    assert_eq!(r.kinds(), vec![Kind::Alpha, Kind::Gamma, Kind::Beta]);
    assert_eq!(r.declared(Kind::Beta), Some(Caps(0)));
}
