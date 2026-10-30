//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// UI-thread adapter drives the existing original child coordinator, not a global shutdown.
pub trait RouteMediaPort<S>: 'static {
    fn begin_retire(&self, owner: &S) -> Result<RetireTicket<S>, RouteMediaError>;
    fn poll_retire(&self, ticket: &RetireTicket<S>) -> Result<RouteMediaProgress, RouteMediaError>;
    fn cancel_retire(&self, ticket: &RetireTicket<S>) -> Result<(), RouteMediaError>;
    /// Called only after the original Viewing DOM commit; registration needs its real video node.
    fn begin_viewing(
        &self,
        owner: &S,
        retired: &RetireTicket<S>,
    ) -> Result<ViewingTicket<S>, RouteMediaError>;
    fn poll_viewing(
        &self,
        ticket: &ViewingTicket<S>,
    ) -> Result<RouteMediaProgress, RouteMediaError>;
}
