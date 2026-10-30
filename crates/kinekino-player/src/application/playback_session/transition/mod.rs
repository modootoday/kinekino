//! Verified adjacent-episode preparation and explicit owned replacement.
//! Rename pending sources to these module names; production must not include runtime paths.
mod adjacency;
mod coordinator;
mod resolve_job;
mod service;
mod slot;

pub use adjacency::EpisodeDirection;
pub use coordinator::{
    EpisodeTransitions, ReleasedTransition, SubmittedTransition, TransitionObservation,
};
pub use resolve_job::BeginAdjacentResolution;
pub use service::{EpisodeReplacementOwner, ReplacedEpisode};
#[cfg(feature = "native-proposal-window")]
pub(crate) use slot::RoutePause;
pub use slot::TransitionSlot;
