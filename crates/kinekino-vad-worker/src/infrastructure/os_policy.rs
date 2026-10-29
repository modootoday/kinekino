//! Calls into the installed `kinekino_core::platform::OsPolicy` port for the
//! shared player storage module. This crate never names an OS adapter; the
//! process entry installs one (`kinekino_platform::install`).
pub use kinekino_core::platform::{OpenFlags, ShareMode};
use kinekino_core::platform::{OsPolicy, os as installed};
use std::{
    fs::{File, Metadata, OpenOptions},
    io,
    path::Path,
};

fn os() -> &'static dyn OsPolicy {
    #[cfg(test)]
    kinekino_platform::install();
    installed()
}
pub fn apply_share_mode(options: &mut OpenOptions, mode: ShareMode) {
    os().apply_share_mode(options, mode);
}
pub fn apply_open_flags(options: &mut OpenOptions, flags: OpenFlags) {
    os().apply_open_flags(options, flags);
}
pub fn is_reparse_point(metadata: &Metadata) -> bool {
    os().is_reparse_point(metadata)
}
pub fn is_sharing_violation(error: &io::Error) -> bool {
    os().is_sharing_violation(error)
}
pub fn is_local_fixed_path(path: &Path) -> bool {
    os().is_local_fixed_path(path)
}
pub fn try_lock_exclusive(file: &File) -> io::Result<()> {
    os().try_lock_exclusive(file)
}

/// Test oracle for retained guards: Windows share modes make `mutation` fail;
/// POSIX sharing is advisory, so it succeeds and identity/digest re-checks are
/// the guard there. Returns whether the mutation was applied.
#[cfg(test)]
pub fn guarded_mutation(mutation: io::Result<()>) -> bool {
    let mandatory = os().host().has_mandatory_share_modes();
    assert_eq!(mutation.is_err(), mandatory, "{mutation:?}");
    !mandatory
}
