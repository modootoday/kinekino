use super::*;
use crate::port::registry::tests_support::{Caps, Kind};

struct Fake;
impl Adapter for Fake {
    type Kind = Kind;
    type Caps = Caps;
    fn kind(&self) -> Kind {
        Kind::Alpha
    }
    fn capabilities(&self) -> Caps {
        Caps(0b01)
    }
}

#[derive(Debug)]
struct E(ErrorCode);
impl std::fmt::Display for E {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "e {}", self.0)
    }
}
impl ToPortError for E {
    fn error_code(&self) -> ErrorCode {
        self.0
    }
}

#[test]
fn harness_accepts_conforming_fake() {
    check_identity(&Fake, Caps(0b11));
    check_unsupported::<(), _>("x", Err(E(ErrorCode::Unsupported)));
    check_error_mapping(&ErrorCode::ALL.map(E));
    check_unique_kinds(&[Kind::Alpha, Kind::Beta]);
    check_capability_table(Caps(0b111));
}

#[test]
#[should_panic(expected = "exceed declared")]
fn harness_rejects_undeclared_capability() {
    check_identity(&Fake, Caps(0b10));
}

#[test]
#[should_panic(expected = "unsupported call returned Ok")]
fn harness_rejects_silent_success() {
    check_unsupported::<u8, E>("x", Ok(1));
}
