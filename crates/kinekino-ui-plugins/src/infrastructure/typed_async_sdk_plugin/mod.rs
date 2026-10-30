//! Terminal-only typed adapter. Execution and publication belong to the injected SDK owner.
#[path = "ports.rs"]
mod ports;
pub use ports::AsyncSdkPort;
mod codec;
mod lifecycle;
pub use codec::{AsyncSdkBinding, async_binding};
use kinekino_media_contracts::errors::ApiError;
pub use lifecycle::{AsyncCodecBudget, AsyncSdkPump, TerminalAcceptance, TypedAsyncSdkPlugin};
use meari::application::plugins::{CancelReceipt, PluginError, PluginTicket};
use std::sync::Arc;

#[cfg(test)]
mod tests;
