//! Session detection and credential storage for Unix desktops.
use kinekino_core::port::credential::{CredentialStore, Secret};
use kinekino_core::port::error::PortResult;
use kinekino_core::port::{DisplayServer, OsId, TargetPlatform};
use std::collections::HashMap;

/// Display server from the session env: Wayland wins when both are set (XWayland).
pub fn detect(os: OsId, display: Option<&str>, wayland: Option<&str>) -> TargetPlatform {
    let set = |v: Option<&str>| v.is_some_and(|s| !s.is_empty());
    let display = if set(wayland) {
        Some(DisplayServer::Wayland)
    } else if set(display) {
        Some(DisplayServer::X11)
    } else {
        None
    };
    TargetPlatform { os, display }
}

/// Current process' platform.
pub fn target() -> TargetPlatform {
    let d = std::env::var("DISPLAY").ok();
    let w = std::env::var("WAYLAND_DISPLAY").ok();
    detect(OsId::of_build(), d.as_deref(), w.as_deref())
}

/// Process-lifetime store. Used until a Secret Service (D-Bus) adapter is registered.
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
        Ok(self.entries.get(&(service.into(), account.into())).cloned())
    }
    fn set(&mut self, service: &str, account: &str, secret: Secret) -> PortResult<()> {
        self.entries
            .insert((service.into(), account.into()), secret);
        Ok(())
    }
    fn delete(&mut self, service: &str, account: &str) -> PortResult<bool> {
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
    fn detection_table() {
        let l = OsId::Linux;
        assert_eq!(detect(l, None, None).display, None);
        assert_eq!(
            detect(l, Some(":0"), None).display,
            Some(DisplayServer::X11)
        );
        assert_eq!(detect(l, Some(""), None).display, None);
        assert_eq!(
            detect(l, Some(":0"), Some("wayland-0")).display,
            Some(DisplayServer::Wayland)
        );
    }

    #[test]
    fn session_store_meets_contract() {
        check_credential_store(&mut SessionCredentialStore::default()).unwrap();
    }
}
