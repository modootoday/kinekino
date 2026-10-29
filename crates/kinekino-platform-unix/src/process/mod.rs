//! Unix child-process policy adapter. Children never get a console window, so this is a no-op.
pub mod clock;
pub mod ownership;
use std::process::Command;

/// No-op on Unix.
pub fn hide_console_window(_command: &mut Command) {}
