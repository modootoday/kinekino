//! Native playback option policy. No UI, engine JSON, process or operating-system dependency.
mod rate;
mod tracks;
pub use rate::PlaybackRate;
pub use tracks::{
    MAX_TRACKS, NativeTrack, ObservedTrackChoice, ObservedTracks, TrackCatalog, TrackCatalogError,
    TrackId, TrackKind, TrackSelection,
};
