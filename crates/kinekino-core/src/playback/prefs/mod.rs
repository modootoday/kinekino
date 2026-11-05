//! Player viewing preferences (#9 / M1-7): volume, mute, speed, and auto-next.
mod file;
mod store;
mod value;

pub use file::{FilePlayerPrefsStore, default_player_prefs_file};
pub use store::{MemoryPlayerPrefsStore, PlayerPrefsStore, check_player_prefs_store};
pub use value::PlayerPrefs;
