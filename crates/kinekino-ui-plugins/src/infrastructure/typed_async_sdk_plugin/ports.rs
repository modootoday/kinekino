//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Called only on the owning UI thread. Implementations retain results until acknowledge.
/// The product assembly maps actual tickets to its opaque application operations.
pub trait AsyncSdkPort<I, O> {
    fn start(&mut self, ticket: &PluginTicket, input: I) -> Result<(), PluginError>;
    fn peek_terminal(
        &mut self,
        ticket: &PluginTicket,
    ) -> Result<Option<Arc<Result<O, ApiError>>>, PluginError>;
    fn acknowledge(&mut self, ticket: &PluginTicket) -> Result<(), PluginError>;
    /// Terminal-ready must remain Requested until registry acceptance and owner ACK.
    fn cancel(&mut self, ticket: &PluginTicket) -> Result<CancelReceipt, PluginError>;
    /// Busy retains the same owner for a subsequent close attempt.
    fn shutdown(&mut self) -> Result<(), PluginError>;
}
