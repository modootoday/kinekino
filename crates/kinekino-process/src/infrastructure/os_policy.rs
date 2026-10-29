//! Calls into the installed `kinekino_core::platform::OsPolicy` port. This crate never names an
//! OS adapter; process entries install one (`kinekino_platform::install`), unit tests inject
//! `kinekino_platform::CurrentOs` explicitly (dev-dependency only).
pub use kinekino_core::platform::{CloseFailure, FileIdentity, OpenFlags, ShareMode};
use kinekino_core::platform::{OsPolicy, os as installed};
use std::{
    fs::{File, Metadata, OpenOptions},
    io,
    path::Path,
    process::Command,
};

fn os() -> &'static dyn OsPolicy {
    #[cfg(test)]
    kinekino_core::platform::install(&kinekino_platform::CurrentOs);
    installed()
}

pub fn apply_share_mode(options: &mut OpenOptions, mode: ShareMode) {
    os().apply_share_mode(options, mode);
}
pub fn apply_open_flags(options: &mut OpenOptions, flags: OpenFlags) {
    os().apply_open_flags(options, flags);
}
pub fn open_reserved_read(path: &Path) -> io::Result<File> {
    os().open_reserved_read(path)
}
pub fn is_reparse_point(metadata: &Metadata) -> bool {
    os().is_reparse_point(metadata)
}
pub fn is_sharing_violation(error: &io::Error) -> bool {
    os().is_sharing_violation(error)
}
pub fn file_identity(file: &File) -> io::Result<FileIdentity> {
    os().file_identity(file)
}
pub fn hide_console_window(command: &mut Command) {
    os().hide_console_window(command);
}
pub fn close_original(file: File) -> Result<(), CloseFailure> {
    os().close_original(file)
}
/// Process-family ownership adapter of the installed OS policy (ADR 0006).
pub fn process() -> &'static dyn kinekino_core::platform::process::ProcessOwnership {
    os().process()
}
