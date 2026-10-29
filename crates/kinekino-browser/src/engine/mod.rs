//! Engine-neutral port, capability-based selection, the contract suite, and the adapters
//! behind cfg and feature gates (ADR 0002).
#[cfg(any(feature = "cdp", feature = "cef"))]
pub mod chromium;
pub mod contract;
#[cfg(all(target_os = "linux", feature = "x11-embed"))]
pub mod embed;
pub mod port;
pub mod registry;
pub mod select;
#[cfg(all(windows, feature = "webview2"))]
pub mod webview2;
