//! Plan P6 player window on Meari: one native code path for Linux and Windows (macOS via CI).
//! Frames come from any `VideoSurface` (libmpv in production) and are uploaded into the
//! document's `<video id="video">` as a raster texture; no WebView, no GL interop.
mod catalog;
mod controller;
mod keys;
mod menu;
mod port;
mod session;
mod source;
mod styles;
mod texture;
mod window;

pub use catalog::{BROWSE_ROWS, Catalog};
pub use controller::{
    Controller, Effect, Engine, HIDE_AFTER, NOTICE_FOR, RESTORE_HOLD, SEEK_STEP_1, SEEK_STEP_2,
    SEEK_STEP_3, SEEK_STEP_4, SPEED_MAX, SPEED_MIN, SPEED_STEP, VOLUME_STEP, siblings,
};
pub use keys::shortcut;
pub use menu::{CONTEXT_MENU_SIZE, context_anchor};
pub use session::{Command, Mockup, Session, VIEWPORT, activate, clock, meari_props, path_of};
pub use source::{PRODUCT, PlaylistItem, View};
pub use texture::{VIDEO_ID, VideoTexture};
pub use window::actions::{PortRefusal, WindowOps, perform};
pub use window::{Options, Report, run};
