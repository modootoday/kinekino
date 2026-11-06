use super::*;
use crate::port::credential::Secret;
use crate::port::registry::tests_support::{Caps, Kind};
use std::collections::HashMap;

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

/// In-memory store used only to drive the harness. Starts without empty-key
/// rejection so the expanded contract fails until adapters (and this fake)
/// call `require_credential_key`.
#[derive(Default)]
struct MemStore {
    entries: HashMap<(String, String), Secret>,
    reject_empty: bool,
}

impl CredentialStore for MemStore {
    fn get(&self, service: &str, account: &str) -> PortResult<Option<Secret>> {
        if self.reject_empty {
            crate::port::credential::require_credential_key(service, account)?;
        }
        Ok(self.entries.get(&(service.into(), account.into())).cloned())
    }
    fn set(&mut self, service: &str, account: &str, secret: Secret) -> PortResult<()> {
        if self.reject_empty {
            crate::port::credential::require_credential_key(service, account)?;
        }
        self.entries
            .insert((service.into(), account.into()), secret);
        Ok(())
    }
    fn delete(&mut self, service: &str, account: &str) -> PortResult<bool> {
        if self.reject_empty {
            crate::port::credential::require_credential_key(service, account)?;
        }
        Ok(self
            .entries
            .remove(&(service.into(), account.into()))
            .is_some())
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

#[test]
fn secret_debug_never_prints_plaintext() {
    let plain = "super-secret-token-value";
    let rendered = format!("{:?}", Secret::new(plain));
    assert!(
        !rendered.contains(plain),
        "Debug leaked plaintext: {rendered}"
    );
    assert_eq!(rendered, "Secret(***)");
}

#[test]
fn credential_store_harness_rejects_store_that_allows_empty_keys() {
    let err = std::panic::catch_unwind(|| {
        check_credential_store(&mut MemStore {
            reject_empty: false,
            ..MemStore::default()
        })
        .unwrap();
    });
    assert!(
        err.is_err(),
        "harness must fail when empty keys are accepted"
    );
}

#[test]
fn credential_store_harness_accepts_conforming_mem_store() {
    check_credential_store(&mut MemStore {
        reject_empty: true,
        ..MemStore::default()
    })
    .unwrap();
}
