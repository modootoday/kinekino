//! UI-owned transition progress. Adjacency, replacement and cancellation policy remain SDK-owned.
mod automatic;
pub(crate) mod callbacks;
pub(super) mod codec;
mod port;
mod state;
mod view;
pub(crate) use state::{HandleInput, PlayingContext, TransitionRequest, TransitionUi};
