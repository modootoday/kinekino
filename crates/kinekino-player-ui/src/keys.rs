//! Window keyboard shortcuts, by physical key so a Korean IME layout still matches.
use crate::controller::{SEEK_STEP_1, SEEK_STEP_2, SEEK_STEP_3, SEEK_STEP_4};
use crate::session::Command;
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

/// The command a key press asks for (start screen hint: `Ctrl+O로 파일 열기`).
/// Cmd+O counts on macOS; Space plays/pauses, F toggles and Esc leaves fullscreen.
/// As in PotPlayer's default layout, Up/Down step the volume and M mutes;
/// Left/Right (and Ctrl/Shift/Ctrl+Alt variants) seek; PageUp/PageDown change item;
/// Backspace goes to the start. X/C step the speed down/up and Z resets it.
/// Alt+A and Alt+L cycle the audio and subtitle tracks (PotPlayer's "Cycle audio stream"
/// and "Cycle subtitle"); other Alt combinations never fire.
/// Auto-repeat only keeps stepping the volume, speed, and seek arrows.
/// Ctrl+[ / Ctrl+] mark the skip span; S skips inside it; Ctrl+\ clears it
/// (confirmed by the shortcut-table ADR, adr:kinekino/20261105151827).
pub fn shortcut(mods: ModifiersState, key: PhysicalKey, repeat: bool) -> Option<Command> {
    let primary = (mods.control_key() || mods.meta_key()) && !mods.alt_key();
    let plain = mods.is_empty();
    let alt = mods == ModifiersState::ALT;
    let shift = mods == ModifiersState::SHIFT;
    // Exact chords so Ctrl+Shift+← is not a seek (Shift and Ctrl steps are distinct).
    let ctrl_only = mods == ModifiersState::CONTROL || mods == ModifiersState::META;
    let ctrl_alt = mods == (ModifiersState::CONTROL | ModifiersState::ALT);
    match key {
        PhysicalKey::Code(KeyCode::ArrowUp) if plain => return Some(Command::VolumeUp),
        PhysicalKey::Code(KeyCode::ArrowDown) if plain => return Some(Command::VolumeDown),
        PhysicalKey::Code(KeyCode::KeyX) if plain => return Some(Command::SpeedDown),
        PhysicalKey::Code(KeyCode::KeyC) if plain => return Some(Command::SpeedUp),
        PhysicalKey::Code(KeyCode::ArrowLeft) if plain => {
            return Some(Command::SeekBy(-SEEK_STEP_1));
        }
        PhysicalKey::Code(KeyCode::ArrowRight) if plain => {
            return Some(Command::SeekBy(SEEK_STEP_1));
        }
        PhysicalKey::Code(KeyCode::ArrowLeft) if ctrl_only => {
            return Some(Command::SeekBy(-SEEK_STEP_2));
        }
        PhysicalKey::Code(KeyCode::ArrowRight) if ctrl_only => {
            return Some(Command::SeekBy(SEEK_STEP_2));
        }
        PhysicalKey::Code(KeyCode::ArrowLeft) if shift => {
            return Some(Command::SeekBy(-SEEK_STEP_3));
        }
        PhysicalKey::Code(KeyCode::ArrowRight) if shift => {
            return Some(Command::SeekBy(SEEK_STEP_3));
        }
        PhysicalKey::Code(KeyCode::ArrowLeft) if ctrl_alt => {
            return Some(Command::SeekBy(-SEEK_STEP_4));
        }
        PhysicalKey::Code(KeyCode::ArrowRight) if ctrl_alt => {
            return Some(Command::SeekBy(SEEK_STEP_4));
        }
        _ if repeat => return None,
        _ => {}
    }
    match key {
        PhysicalKey::Code(KeyCode::KeyO) if primary => Some(Command::Open),
        PhysicalKey::Code(KeyCode::Space) if plain => Some(Command::PlayPause),
        PhysicalKey::Code(KeyCode::KeyF) if plain => Some(Command::ToggleFullscreen),
        PhysicalKey::Code(KeyCode::Escape) if plain => Some(Command::ExitFullscreen),
        PhysicalKey::Code(KeyCode::KeyM) if plain => Some(Command::Mute),
        PhysicalKey::Code(KeyCode::KeyZ) if plain => Some(Command::SpeedReset),
        PhysicalKey::Code(KeyCode::KeyA) if alt => Some(Command::CycleAudio),
        PhysicalKey::Code(KeyCode::KeyL) if alt => Some(Command::CycleSubtitle),
        PhysicalKey::Code(KeyCode::BracketLeft) if primary => Some(Command::MarkSkipStart),
        PhysicalKey::Code(KeyCode::BracketRight) if primary => Some(Command::MarkSkipEnd),
        PhysicalKey::Code(KeyCode::KeyS) if plain => Some(Command::SkipSegment),
        PhysicalKey::Code(KeyCode::Backslash) if primary => Some(Command::ClearSkip),
        PhysicalKey::Code(KeyCode::PageUp) if plain => Some(Command::Previous),
        PhysicalKey::Code(KeyCode::PageDown) if plain => Some(Command::Next),
        PhysicalKey::Code(KeyCode::Backspace) if plain => Some(Command::Seek(0.0)),
        _ => None,
    }
}
