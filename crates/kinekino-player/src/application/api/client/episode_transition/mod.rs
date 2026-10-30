//! Typed transition calls using the common response decoder.
#[path = "ports.rs"]
mod ports;
use super::{ApplicationClient, review_call};
use crate::{
    application::{
        api::{ApiError, RequestContext},
        playback_session::transition::{
            BeginAdjacentResolution, EpisodeDirection, ReleasedTransition, ReplacedEpisode,
            SubmittedTransition, TransitionObservation,
        },
    },
    domain::api_values::DecimalU64,
};
pub use ports::EpisodeTransitionClient;
use serde_json::json;

impl<T: ApplicationClient + ?Sized> EpisodeTransitionClient for T {}
