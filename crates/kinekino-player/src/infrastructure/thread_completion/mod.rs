//! Host-owned bounded join service. Child completion and host shutdown are distinct.
mod completion;
mod dispatcher;
pub use completion::{CompletionWake, JoinFailure, JoinRequest, JoinTicket, NotificationState};
pub use dispatcher::{
    DispatcherHandle, DispatcherHost, DispatcherShutdown, DispatcherStatus, SubmitFailure,
    SubmitRejection,
};
#[cfg(test)]
mod tests;
