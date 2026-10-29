//! Session detection and credential storage for Windows (compile-gated, unverified on Linux CI).
use kinekino_core::port::credential::{CredentialStore, Secret};
use kinekino_core::port::error::{ErrorCode, PortError, PortResult};
use kinekino_core::port::{DisplayServer, OsId, TargetPlatform};

pub fn target() -> TargetPlatform {
    TargetPlatform::with_display(OsId::Windows, DisplayServer::Win32)
}

/// Placeholder until the Credential Manager adapter lands: every call is `Unavailable`.
#[derive(Debug, Default)]
pub struct SessionCredentialStore;

impl SessionCredentialStore {
    /// Same constructor shape on every OS adapter, so callers stay OS-neutral.
    pub fn new() -> Self {
        Self
    }
}

fn unavailable() -> PortError {
    PortError::new(
        ErrorCode::Unavailable,
        "windows credential manager adapter not implemented",
    )
}

impl CredentialStore for SessionCredentialStore {
    fn get(&self, _: &str, _: &str) -> PortResult<Option<Secret>> {
        Err(unavailable())
    }
    fn set(&mut self, _: &str, _: &str, _: Secret) -> PortResult<()> {
        Err(unavailable())
    }
    fn delete(&mut self, _: &str, _: &str) -> PortResult<bool> {
        Err(unavailable())
    }
}
