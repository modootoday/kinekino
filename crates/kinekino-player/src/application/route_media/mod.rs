//! Same-owner route media lifetime. Tickets never authorize a new SDK or native parent.
#[path = "ports.rs"]
mod ports;
pub use ports::RouteMediaPort;
mod state;
use crate::application::api::{ApiError, ErrorCode};
pub use state::RouteMediaState;
use std::sync::Arc;

pub type RouteMediaError = Arc<ApiError>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteMediaProgress {
    Pending,
    Complete,
}
#[derive(Clone, Debug)]
pub struct RetireTicket<S> {
    owner: S,
    generation: u64,
    identity: Arc<()>,
}
#[derive(Clone, Debug)]
pub struct ViewingTicket<S> {
    owner: S,
    generation: u64,
    identity: Arc<()>,
}
impl<S> RetireTicket<S> {
    pub fn generation(&self) -> u64 {
        self.generation
    }
}
impl<S> ViewingTicket<S> {
    pub fn generation(&self) -> u64 {
        self.generation
    }
}
fn conflict() -> RouteMediaError {
    Arc::new(ApiError::new(
        ErrorCode::RevisionConflict,
        "Route media ticket or original owner differs",
    ))
}
fn unavailable() -> RouteMediaError {
    Arc::new(ApiError::new(
        ErrorCode::NeedsInput,
        "Original route media cleanup or registration is incomplete",
    ))
}
