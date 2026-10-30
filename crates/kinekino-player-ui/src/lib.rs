//! Plan P6 player window on Meari: one native code path for Linux and Windows (macOS via CI).
//! Frames come from any `VideoSurface` (libmpv in production) and are uploaded into the
//! document's `<video id="video">` as a raster texture; no WebView, no GL interop.
mod controller;
mod port;
mod session;
mod source;
mod styles;
mod texture;
mod window;

pub use controller::{Controller, Effect, Engine, siblings};
pub use session::{Command, Session, VIEWPORT, activate, meari_props, path_of};
pub use source::{Episode, PRODUCT, View, clock};
pub use texture::{VIDEO_ID, VideoTexture};
pub use window::{Options, Report, run};
