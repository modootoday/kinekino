//! Windows child-process policy adapter.
pub mod clock;
pub mod ownership;
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Prevents a console window from appearing for the child.
pub fn hide_console_window(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}
