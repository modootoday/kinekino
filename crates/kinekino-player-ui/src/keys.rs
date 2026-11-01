//! Window keyboard shortcuts, by physical key so a Korean IME layout still matches.
use crate::session::Command;
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

/// The command a key press asks for (start screen hint: `Ctrl+O로 파일 열기`).
/// Cmd+O counts on macOS; Space plays/pauses, F toggles and Esc leaves fullscreen.
/// As in PotPlayer's default layout, Up/Down step the volume and M mutes.
/// Alt combinations never fire; auto-repeat only keeps stepping the volume.
pub fn shortcut(mods: ModifiersState, key: PhysicalKey, repeat: bool) -> Option<Command> {
    let primary = (mods.control_key() || mods.meta_key()) && !mods.alt_key();
    let plain = mods.is_empty();
    match key {
        PhysicalKey::Code(KeyCode::ArrowUp) if plain => return Some(Command::VolumeUp),
        PhysicalKey::Code(KeyCode::ArrowDown) if plain => return Some(Command::VolumeDown),
        _ if repeat => return None,
        _ => {}
    }
    match key {
        PhysicalKey::Code(KeyCode::KeyO) if primary => Some(Command::Open),
        PhysicalKey::Code(KeyCode::Space) if plain => Some(Command::PlayPause),
        PhysicalKey::Code(KeyCode::KeyF) if plain => Some(Command::ToggleFullscreen),
        PhysicalKey::Code(KeyCode::Escape) if plain => Some(Command::ExitFullscreen),
        PhysicalKey::Code(KeyCode::KeyM) if plain => Some(Command::Mute),
        _ => None,
    }
}
