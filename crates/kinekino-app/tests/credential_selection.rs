//! Credential store selection: the OS store first, the session store only as an explicit,
//! reported fallback (M3 completion criterion 2).
use kinekino_app::Env;
use kinekino_app::settings::credentials::{CredentialBacking, select};
use kinekino_core::port::credential::{CredentialStore, Secret};
use kinekino_core::port::{ErrorCode, PortError, PortResult};
use std::cell::Cell;

/// Stand-in for an opened OS store; answers every lookup with a fixed secret.
struct FakeOs;

impl CredentialStore for FakeOs {
    fn get(&self, _: &str, _: &str) -> PortResult<Option<Secret>> {
        Ok(Some(Secret::new("from-os")))
    }
    fn set(&mut self, _: &str, _: &str, _: Secret) -> PortResult<()> {
        Ok(())
    }
    fn delete(&mut self, _: &str, _: &str) -> PortResult<bool> {
        Ok(true)
    }
}

fn env(mode: Option<&str>) -> Env {
    Env::from_pairs(mode.map(|m| ("KINEKINO_CREDENTIALS", m)))
}

fn os_ok() -> PortResult<Box<dyn CredentialStore>> {
    Ok(Box::new(FakeOs))
}

fn os_unavailable() -> PortResult<Box<dyn CredentialStore>> {
    Err(PortError::new(
        ErrorCode::Unavailable,
        "secret service has no unlocked collection",
    ))
}

fn read(store: &dyn CredentialStore) -> Option<String> {
    store
        .get("kinekino.test", "a")
        .unwrap()
        .map(|s| s.expose().to_owned())
}

#[test]
fn auto_uses_the_os_store_when_it_opens() {
    let sel = select(&env(None), os_ok).unwrap();
    assert_eq!(sel.backing, CredentialBacking::Os);
    assert_eq!(read(sel.store.as_ref()).as_deref(), Some("from-os"));
}

#[test]
fn auto_falls_back_to_session_and_keeps_the_reason() {
    let mut sel = select(&env(None), os_unavailable).unwrap();
    assert_eq!(
        sel.backing,
        CredentialBacking::SessionFallback {
            reason: "secret service has no unlocked collection".into()
        }
    );
    assert_eq!(sel.backing.name(), "session-fallback");
    assert_eq!(read(sel.store.as_ref()), None);
    sel.store
        .set("kinekino.test", "a", Secret::new("kept"))
        .unwrap();
    assert_eq!(read(sel.store.as_ref()).as_deref(), Some("kept"));
}

#[test]
fn auto_does_not_hide_errors_other_than_unavailable() {
    let err = select(&env(None), || {
        Err(PortError::new(ErrorCode::InvalidInput, "bad key"))
    })
    .err()
    .expect("non-unavailable errors propagate");
    assert_eq!(err.code, ErrorCode::InvalidInput);
}

#[test]
fn os_mode_requires_the_os_store() {
    let err = select(&env(Some("os")), os_unavailable)
        .err()
        .expect("os mode must not fall back");
    assert_eq!(err.code, ErrorCode::Unavailable);
    assert_eq!(
        select(&env(Some("os")), os_ok).unwrap().backing,
        CredentialBacking::Os
    );
}

#[test]
fn session_mode_never_opens_the_os_store() {
    let opened = Cell::new(false);
    let sel = select(&env(Some("session")), || {
        opened.set(true);
        os_ok()
    })
    .unwrap();
    assert!(!opened.get());
    assert_eq!(sel.backing, CredentialBacking::Session);
    assert_eq!(sel.backing.name(), "session");
}

#[test]
fn unknown_mode_is_invalid_input() {
    let err = select(&env(Some("keychain")), os_ok)
        .err()
        .expect("unknown mode");
    assert_eq!(err.code, ErrorCode::InvalidInput);
    assert!(
        err.message.contains("KINEKINO_CREDENTIALS"),
        "{}",
        err.message
    );
}
