#[path = "ports.rs"]
mod ports;
pub use ports::SkipHostApi;
mod contracts;
mod dispatch;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub(crate) mod late_routed;
mod registry;
#[cfg(all(windows, feature = "native-proposal-window"))]
mod routed_owner;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub(crate) use routed_owner::{PreparedRoutedSkipOwner, RoutedSkipSetupFailure};
pub mod schema;
use crate::application::api::{ApiError, ErrorCode, Request};
pub use contracts::*;
pub use registry::{SkipHostOwner, SkipHostRegistry};
use serde_json::Value;
pub struct UnavailableSkipHost;
impl SkipHostApi for UnavailableSkipHost {
    fn capability(&self) -> SkipHostCapability {
        SkipHostCapability {
            sessions: false,
            seek_submission: false,
        }
    }
    fn execute(&self, request: &Request) -> Result<Value, ApiError> {
        if is_export_command(&request.command) {
            dispatch::export_preflight(request)?;
        }
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Explicit skip session host unavailable",
        ))
    }
}
pub const COMMANDS: &[&str] = &[
    "segments.open",
    "segments.inspect",
    "segments.edit",
    "segments.accept_skip",
    "segments.set_skip_policy",
    "segments.evaluate",
    "segments.skip",
    "segments.override",
    "segments.close",
];
pub fn is_command(command: &str) -> bool {
    COMMANDS.contains(&command) || is_export_command(command)
}
pub const SKIP_EXPORT: &str = "segments.skip_export";
pub const EXPORT_LIST: &str = "segments.export_list";
pub const EXPORT_PEEK: &str = "segments.export_peek";
pub const EXPORT_CONSUME: &str = "segments.export_consume";
pub const EXPORT_COMMANDS: &[&str] = &[SKIP_EXPORT, EXPORT_LIST, EXPORT_PEEK, EXPORT_CONSUME];
pub fn is_export_command(command: &str) -> bool {
    EXPORT_COMMANDS.contains(&command)
}
