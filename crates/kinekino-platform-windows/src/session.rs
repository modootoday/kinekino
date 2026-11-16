//! Session detection and credential storage for Windows (compile-gated; tests run only on Windows).
use kinekino_core::port::credential::{CredentialStore, Secret, require_credential_key};
use kinekino_core::port::error::PortResult;
use kinekino_core::port::{DisplayServer, OsId, TargetPlatform};
use std::collections::HashMap;

pub fn target() -> TargetPlatform {
    TargetPlatform::with_display(OsId::Windows, DisplayServer::Win32)
}

/// Process-lifetime store, as on Unix. The Credential Manager adapter is not written yet, so
/// `kinekino_platform::credential::open_os_store` reports `Unavailable` and the product falls
/// back here with the reason shown.
#[derive(Debug, Default)]
pub struct SessionCredentialStore {
    entries: HashMap<(String, String), Secret>,
}

impl SessionCredentialStore {
    /// Same constructor shape on every OS adapter, so callers stay OS-neutral.
    pub fn new() -> Self {
        Self::default()
    }
}

impl CredentialStore for SessionCredentialStore {
    fn get(&self, service: &str, account: &str) -> PortResult<Option<Secret>> {
        require_credential_key(service, account)?;
        Ok(self.entries.get(&(service.into(), account.into())).cloned())
    }
    fn set(&mut self, service: &str, account: &str, secret: Secret) -> PortResult<()> {
        require_credential_key(service, account)?;
        self.entries
            .insert((service.into(), account.into()), secret);
        Ok(())
    }
    fn delete(&mut self, service: &str, account: &str) -> PortResult<bool> {
        require_credential_key(service, account)?;
        Ok(self
            .entries
            .remove(&(service.into(), account.into()))
            .is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_core::port::contract::check_credential_store;

    #[test]
    fn session_store_meets_contract() {
        check_credential_store(&mut SessionCredentialStore::new()).unwrap();
    }
}
