//! Root-scope read callbacks and their retained exclusive SDK consumer.
pub(crate) mod catalog;
pub mod chooser;
pub mod codec;
pub(crate) mod episode_transition;
mod lifecycle;
mod native_playback;
pub(crate) mod progression;
mod selection;
pub use lifecycle::{ViewerCallbacks, export_callback_bindings};
#[cfg(test)]
mod tests;
