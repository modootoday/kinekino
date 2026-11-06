//! Linux Secret Service (`org.freedesktop.Secret.Service`) credential adapter.
//!
//! Uses an already-unlocked collection only so headless sessions do not block on
//! a login-keyring prompt. Product assembly may fall back to an in-memory store
//! when [`SecretServiceCredentialStore::try_open`] returns `Unavailable`; that
//! wiring is intentionally not done here.
use kinekino_core::port::credential::{CredentialStore, Secret, require_credential_key};
use kinekino_core::port::error::{ErrorCode, PortError, PortResult};

/// Secret Service–backed [`CredentialStore`]. Construct with [`Self::try_open`].
#[derive(Debug, Default)]
pub struct SecretServiceCredentialStore;

impl SecretServiceCredentialStore {
    /// Connect and ensure at least one unlocked collection exists.
    pub fn try_open() -> PortResult<Self> {
        #[cfg(target_os = "linux")]
        {
            linux::probe()?;
            Ok(Self)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(unavailable(
                "Secret Service credential store is only available on Linux",
            ))
        }
    }

    /// Same as [`Self::try_open`].
    pub fn open() -> PortResult<Self> {
        Self::try_open()
    }
}

fn unavailable(message: impl Into<String>) -> PortError {
    PortError::new(ErrorCode::Unavailable, message)
}

impl CredentialStore for SecretServiceCredentialStore {
    fn get(&self, service: &str, account: &str) -> PortResult<Option<Secret>> {
        require_credential_key(service, account)?;
        #[cfg(target_os = "linux")]
        {
            linux::get(service, account)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (service, account);
            Err(unavailable(
                "Secret Service credential store is only available on Linux",
            ))
        }
    }

    fn set(&mut self, service: &str, account: &str, secret: Secret) -> PortResult<()> {
        require_credential_key(service, account)?;
        #[cfg(target_os = "linux")]
        {
            linux::set(service, account, secret.expose())
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (service, account, secret);
            Err(unavailable(
                "Secret Service credential store is only available on Linux",
            ))
        }
    }

    fn delete(&mut self, service: &str, account: &str) -> PortResult<bool> {
        require_credential_key(service, account)?;
        #[cfg(target_os = "linux")]
        {
            linux::delete(service, account)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (service, account);
            Err(unavailable(
                "Secret Service credential store is only available on Linux",
            ))
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::{PortResult, unavailable};
    use kinekino_core::port::credential::Secret;
    use kinekino_core::port::error::{ErrorCode, PortError};
    use secret_service::EncryptionType;
    use secret_service::blocking::{Collection, SecretService};
    use std::collections::HashMap;

    fn map_ss(err: secret_service::Error) -> PortError {
        unavailable(format!("secret service: {err}"))
    }

    fn with_unlocked<R>(f: impl FnOnce(&Collection<'_>) -> PortResult<R>) -> PortResult<R> {
        let ss = SecretService::connect(EncryptionType::Dh).map_err(map_ss)?;
        let cols = ss.get_all_collections().map_err(map_ss)?;
        let Some(col) = cols.into_iter().find(|c| !c.is_locked().unwrap_or(true)) else {
            return Err(unavailable(
                "secret service has no unlocked collection (unlock the login keyring or use a session collection)",
            ));
        };
        f(&col)
    }

    pub(super) fn probe() -> PortResult<()> {
        with_unlocked(|_| Ok(()))
    }

    fn attrs<'a>(service: &'a str, account: &'a str) -> HashMap<&'a str, &'a str> {
        HashMap::from([("service", service), ("account", account)])
    }

    pub(super) fn get(service: &str, account: &str) -> PortResult<Option<Secret>> {
        with_unlocked(|col| {
            let items = col.search_items(attrs(service, account)).map_err(map_ss)?;
            let Some(item) = items.into_iter().next() else {
                return Ok(None);
            };
            let bytes = item.get_secret().map_err(map_ss)?;
            let text = std::str::from_utf8(&bytes).map_err(|e| {
                PortError::new(
                    ErrorCode::Io,
                    format!("secret service value is not utf-8: {e}"),
                )
            })?;
            Ok(Some(Secret::new(text)))
        })
    }

    pub(super) fn set(service: &str, account: &str, secret: &str) -> PortResult<()> {
        with_unlocked(|col| {
            let label = format!("{service}/{account}");
            col.create_item(
                &label,
                attrs(service, account),
                secret.as_bytes(),
                true,
                "text/plain",
            )
            .map_err(map_ss)?;
            Ok(())
        })
    }

    pub(super) fn delete(service: &str, account: &str) -> PortResult<bool> {
        with_unlocked(|col| {
            let items = col.search_items(attrs(service, account)).map_err(map_ss)?;
            if items.is_empty() {
                return Ok(false);
            }
            for item in items {
                item.delete().map_err(map_ss)?;
            }
            Ok(true)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_core::port::contract::check_credential_store;
    use std::sync::Mutex;

    /// Serializes tests that touch `DBUS_SESSION_BUS_ADDRESS` or the live store.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn scrub(store: &mut SecretServiceCredentialStore) {
        let _ = store.delete("kinekino.contract", "harness");
        let _ = store.delete("svc.a", "acct.1");
        let _ = store.delete("svc.a", "acct.2");
        let _ = store.delete("svc.b", "acct.1");
        let _ = store.delete("svc.missing", "acct.missing");
    }

    #[test]
    fn secret_service_store_meets_contract_when_available() {
        let _guard = TEST_LOCK.lock().unwrap();
        let mut store = match SecretServiceCredentialStore::try_open() {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "skip secret_service_store_meets_contract_when_available: Secret Service unavailable ({e})"
                );
                return;
            }
        };
        scrub(&mut store);
        check_credential_store(&mut store)
            .expect("Secret Service adapter must meet CredentialStore contract");
        scrub(&mut store);
    }

    #[test]
    fn try_open_unavailable_when_session_bus_missing() {
        let _guard = TEST_LOCK.lock().unwrap();
        let prev = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
        // SAFETY: test-only env mutation; restored below.
        unsafe {
            std::env::set_var(
                "DBUS_SESSION_BUS_ADDRESS",
                "unix:path=/tmp/kinekino-m3-3-no-such-bus",
            );
        }
        let err = SecretServiceCredentialStore::try_open().expect_err("expected Unavailable");
        assert_eq!(err.code, ErrorCode::Unavailable);
        assert!(err.retryable, "Unavailable must be retryable");
        // SAFETY: restore prior test env.
        unsafe {
            match prev {
                Some(v) => std::env::set_var("DBUS_SESSION_BUS_ADDRESS", v),
                None => std::env::remove_var("DBUS_SESSION_BUS_ADDRESS"),
            }
        }
    }
}
