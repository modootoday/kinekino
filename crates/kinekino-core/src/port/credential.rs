//! Secrets port: the only path for API keys/tokens (never in config files or env-derived config).
use super::error::PortResult;

/// Opaque secret; `Debug` never prints the value.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(***)")
    }
}

/// Keyed by `service` (e.g. `"kinekino.openai"`) and `account`.
pub trait CredentialStore {
    fn get(&self, service: &str, account: &str) -> PortResult<Option<Secret>>;
    fn set(&mut self, service: &str, account: &str, secret: Secret) -> PortResult<()>;
    fn delete(&mut self, service: &str, account: &str) -> PortResult<bool>;
}
