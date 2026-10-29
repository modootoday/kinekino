//! Port conventions shared by every adapter port (P0, ADR cross-os foundation).
//!
//! - `kind()` + `capabilities()` on every adapter (see [`Adapter`]).
//! - port errors map to one [`ErrorCode`] with a `retryable` flag ([`ToPortError`]).
//! - events are pulled with `drain_events()` ([`EventSource`]), never pushed via callbacks.
//! - per-port `Config` structs are serde with defaults and carry no secrets; secrets go
//!   through [`credential::CredentialStore`] only.
//! - factories are registered in a [`registry::Registry`] and chosen by platform,
//!   required capabilities and policy.
pub mod capability;
pub mod contract;
pub mod credential;
pub mod error;
pub mod event;
pub mod registry;
pub mod target;

pub use capability::{AdapterKind, CapabilityTable};
pub use error::{ErrorCode, PortError, PortResult, ToPortError};
pub use event::EventSource;
pub use target::{DisplayServer, OsId, TargetPlatform};

/// Identity and capability declaration every adapter exposes.
pub trait Adapter {
    type Kind: AdapterKind;
    type Caps: CapabilityTable;
    fn kind(&self) -> Self::Kind;
    fn capabilities(&self) -> Self::Caps;
}
