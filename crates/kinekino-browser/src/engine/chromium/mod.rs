//! Chromium-family adapters.
//! - `cdp`: installed Google Chrome over DevTools (CLI/headless; control channel for embed).
//! - `chrome_embed`: installed Chrome `--app` window reparented into the host (Linux/X11).
//! - `cef`: official CEF as a native child window.
#[cfg(feature = "cdp")]
pub mod cdp;
#[cfg(feature = "cef")]
pub mod cef;
#[cfg(all(target_os = "linux", feature = "chrome-embed"))]
pub mod chrome_embed;
