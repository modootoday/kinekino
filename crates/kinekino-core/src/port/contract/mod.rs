//! Reusable contract harness: adapter crates call these from their tests so every
//! adapter of a port is held to the same conventions.
use super::Adapter;
use super::capability::{AdapterKind, CapabilityTable};
use super::credential::{CredentialStore, Secret};
use super::error::{ErrorCode, PortResult, ToPortError};

#[cfg(test)]
mod tests;

/// Identity: kind name is stable lower-kebab, actual capabilities ⊆ declared.
pub fn check_identity<A: Adapter>(adapter: &A, declared: A::Caps) {
    let name = adapter.kind().name();
    assert!(
        is_kebab(name),
        "adapter kind name `{name}` must be lower-kebab"
    );
    let caps = adapter.capabilities();
    assert!(
        declared.contains_all(caps),
        "{name}: capabilities {caps} exceed declared {declared}"
    );
}

/// Unsupported calls must fail with `ErrorCode::Unsupported`, not panic or succeed.
pub fn check_unsupported<T: std::fmt::Debug, E: ToPortError + std::fmt::Display>(
    what: &str,
    result: Result<T, E>,
) {
    match result {
        Err(e) => assert_eq!(e.error_code(), ErrorCode::Unsupported, "{what}: {e}"),
        Ok(v) => panic!("{what}: unsupported call returned Ok({v:?})"),
    }
}

/// Every port error variant maps to a code; `Unsupported`/`InvalidInput` are not retryable.
pub fn check_error_mapping<E: ToPortError + std::fmt::Display>(samples: &[E]) {
    for e in samples {
        let p = e.to_port_error();
        if matches!(p.code, ErrorCode::Unsupported | ErrorCode::InvalidInput) {
            assert!(!p.retryable, "{} must not be retryable: {e}", p.code);
        }
        assert!(!p.message.is_empty(), "{}: empty message", p.code);
    }
}

/// Kind names are unique across a port's adapters.
pub fn check_unique_kinds<K: AdapterKind>(kinds: &[K]) {
    let mut names: Vec<&str> = kinds.iter().map(|k| k.name()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "duplicate adapter kind names");
}

/// Capability table algebra the registry relies on.
pub fn check_capability_table<C: CapabilityTable>(all: C) {
    let none = C::empty();
    assert!(none.is_empty());
    assert!(all.contains_all(none));
    assert!(all.missing(all).is_empty());
    assert_eq!(none.missing(all), all);
    assert_eq!(
        all.names().len(),
        all.names()
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    );
}

/// CredentialStore contract on a fresh store: round-trip, key isolation, overwrite,
/// delete-absent, and empty service/account → non-retryable InvalidInput.
pub fn check_credential_store(store: &mut dyn CredentialStore) -> PortResult<()> {
    let (svc, acct) = ("kinekino.contract", "harness");
    assert_eq!(store.get(svc, acct)?, None);
    store.set(svc, acct, Secret::new("s3cret"))?;
    assert_eq!(
        store.get(svc, acct)?.as_ref().map(Secret::expose),
        Some("s3cret")
    );
    assert!(store.delete(svc, acct)?);
    assert!(!store.delete(svc, acct)?);
    assert_eq!(store.get(svc, acct)?, None);

    // Key isolation: distinct (service, account) pairs stay independent.
    store.set("svc.a", "acct.1", Secret::new("a1"))?;
    store.set("svc.a", "acct.2", Secret::new("a2"))?;
    store.set("svc.b", "acct.1", Secret::new("b1"))?;
    assert_eq!(
        store.get("svc.a", "acct.1")?.as_ref().map(Secret::expose),
        Some("a1")
    );
    assert_eq!(
        store.get("svc.a", "acct.2")?.as_ref().map(Secret::expose),
        Some("a2")
    );
    assert_eq!(
        store.get("svc.b", "acct.1")?.as_ref().map(Secret::expose),
        Some("b1")
    );

    // Overwrite: second set on the same key replaces the value.
    store.set("svc.a", "acct.1", Secret::new("a1-new"))?;
    assert_eq!(
        store.get("svc.a", "acct.1")?.as_ref().map(Secret::expose),
        Some("a1-new")
    );
    assert_eq!(
        store.get("svc.a", "acct.2")?.as_ref().map(Secret::expose),
        Some("a2")
    );

    // Delete absent leaves other keys intact.
    assert!(!store.delete("svc.missing", "acct.missing")?);
    assert_eq!(
        store.get("svc.a", "acct.1")?.as_ref().map(Secret::expose),
        Some("a1-new")
    );
    assert_eq!(
        store.get("svc.b", "acct.1")?.as_ref().map(Secret::expose),
        Some("b1")
    );
    let _ = store.delete("svc.a", "acct.1")?;
    let _ = store.delete("svc.a", "acct.2")?;
    let _ = store.delete("svc.b", "acct.1")?;

    // Empty service or account → InvalidInput (not retryable).
    for (label, service, account) in [
        ("empty service", "", "acct"),
        ("empty account", "svc", ""),
        ("both empty", "", ""),
    ] {
        match store.get(service, account) {
            Err(e) => {
                assert_eq!(e.code, ErrorCode::InvalidInput, "{label} get: {e}");
                assert!(!e.retryable, "{label} get must not be retryable");
            }
            Ok(v) => panic!("{label} get returned Ok({v:?})"),
        }
        match store.set(service, account, Secret::new("x")) {
            Err(e) => {
                assert_eq!(e.code, ErrorCode::InvalidInput, "{label} set: {e}");
                assert!(!e.retryable, "{label} set must not be retryable");
            }
            Ok(()) => panic!("{label} set returned Ok(())"),
        }
        match store.delete(service, account) {
            Err(e) => {
                assert_eq!(e.code, ErrorCode::InvalidInput, "{label} delete: {e}");
                assert!(!e.retryable, "{label} delete must not be retryable");
            }
            Ok(v) => panic!("{label} delete returned Ok({v})"),
        }
    }
    Ok(())
}

fn is_kebab(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && !s.ends_with('-')
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
