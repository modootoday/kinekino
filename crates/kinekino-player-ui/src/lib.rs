//! Plan P6 player window on Meari: one native code path for Linux and Windows (macOS via CI).
//! Frames come from any `VideoSurface` (libmpv in production) and are uploaded into the
//! document's `<video id="video">` as a raster texture; no WebView, no GL interop.
mod catalog;
mod controller;
mod keys;
mod port;
mod session;
mod source;
mod styles;
mod texture;
mod window;

pub use catalog::{BROWSE_ROWS, Catalog};
pub use controller::{Controller, Effect, Engine, HIDE_AFTER, NOTICE_FOR, siblings};
pub use keys::shortcut;
pub use session::{Command, Session, VIEWPORT, activate, meari_props, path_of};
pub use source::{PlaylistItem, PRODUCT, View, clock};
pub use texture::{VIDEO_ID, VideoTexture};
pub use window::actions::{PortRefusal, WindowOps, perform};
pub use window::{Options, Report, run};
