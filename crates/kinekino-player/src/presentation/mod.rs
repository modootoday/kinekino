//! Meari-only presentation adapters for the native proposal/product windows. The former
//! eframe/egui player screens were removed; the plain player window is `kinekino-player-ui`.
#[cfg(all(windows, feature = "native-proposal-window"))]
pub(crate) mod playback_range;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub(crate) mod playback_time;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod web_ui_adapter;
