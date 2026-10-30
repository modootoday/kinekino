//! Window keyboard shortcuts, by physical key so a Korean IME layout still matches.
use crate::session::Command;
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

/// The command a key press asks for (start screen hint: `Ctrl+O로 파일 열기`).
/// Cmd+O counts on macOS; Space plays/pauses, F toggles and Esc leaves fullscreen.
/// Alt combinations and auto-repeat never fire.
pub fn shortcut(mods: ModifiersState, key: PhysicalKey, repeat: bool) -> Option<Command> {
    let primary = (mods.control_key() || mods.meta_key()) && !mods.alt_key();
    let plain = mods.is_empty();
    if repeat {
        return None;
    }
    match key {
        PhysicalKey::Code(KeyCode::KeyO) if primary => Some(Command::Open),
        PhysicalKey::Code(KeyCode::Space) if plain => Some(Command::PlayPause),
        PhysicalKey::Code(KeyCode::KeyF) if plain => Some(Command::ToggleFullscreen),
        PhysicalKey::Code(KeyCode::Escape) if plain => Some(Command::ExitFullscreen),
        _ => None,
    }
}
