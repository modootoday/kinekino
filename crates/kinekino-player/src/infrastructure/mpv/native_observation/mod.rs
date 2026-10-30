//! Actual IPC receive metadata only; ordinary cached publishing never enters this module.
mod state;
pub(super) use state::ReceiveState;
#[cfg(test)]
mod tests;
