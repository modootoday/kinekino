//! Calls into the installed `kinekino_core::platform::OsPolicy` port. This crate never names an
//! OS adapter; process entries install one (`kinekino_platform::install`), unit tests inject
//! `kinekino_platform::CurrentOs` explicitly (dev-dependency only).
pub use kinekino_core::platform::{
    CloseFailure, FileIdentity, FileIdentity128, OpenFlags, ShareMode,
};
use kinekino_core::platform::{OsPolicy, os as installed};
use std::{
    fs::{File, Metadata, OpenOptions},
    io,
    path::Path,
    process::Command,
};

/// Local IPC transport port (ADR 0007).
pub fn ipc() -> &'static dyn kinekino_core::platform::ipc::IpcTransport {
    os().ipc()
}
/// Process-family ownership port (ADR 0006).
pub fn process() -> &'static dyn kinekino_core::platform::process::ProcessOwnership {
    os().process()
}
/// Native window port (ADR 0009).
pub fn window() -> &'static dyn kinekino_core::platform::window::WindowPort {
    os().window()
}
/// Maps a toolkit window handle to the port's OS-neutral handle.
pub fn native_window(
    parent: &impl raw_window_handle::HasWindowHandle,
) -> anyhow::Result<kinekino_core::platform::window::NativeWindow> {
    use kinekino_core::platform::window::NativeWindow;
    use raw_window_handle::RawWindowHandle;
    Ok(match parent.window_handle()?.as_raw() {
        RawWindowHandle::Win32(h) => NativeWindow::Win32(h.hwnd.get() as u64),
        RawWindowHandle::Xlib(h) => {
            // `c_ulong` is 64-bit on LP64 Unix and 32-bit on Windows; widen without loss.
            let window: std::ffi::c_ulong = h.window;
            NativeWindow::X11(window as u64)
        }
        RawWindowHandle::Xcb(h) => NativeWindow::X11(u64::from(h.window.get())),
        _ => anyhow::bail!("Unsupported native window handle"),
    })
}
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

/// Capability table of the installed OS adapter.
pub fn host() -> &'static dyn kinekino_core::platform::PlatformHost {
    os().host()
}

/// Native proposal/viewer windows: this build carries the feature and the OS adapter can host one.
pub fn native_proposal_window_available() -> bool {
    cfg!(feature = "native-proposal-window") && host().has_native_desktop_window()
}

pub fn file_identity_128(file: &File) -> io::Result<FileIdentity128> {
    os().file_identity_128(file)
}
pub fn is_local_fixed_path(path: &Path) -> bool {
    os().is_local_fixed_path(path)
}
pub fn try_lock_exclusive(file: &File) -> io::Result<()> {
    os().try_lock_exclusive(file)
}
pub fn try_lock_shared(file: &File) -> io::Result<()> {
    os().try_lock_shared(file)
}
/// Fresh OS-random 128-bit identifier (GUID display byte order).
pub fn fresh_guid() -> io::Result<[u8; 16]> {
    os().fresh_guid()
}
/// Fresh OS-random 128-bit identifier as 32 lowercase hex digits (GUID display order).
pub fn fresh_guid_hex() -> io::Result<String> {
    Ok(os()
        .fresh_guid()?
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
pub fn create_publication_stage(path: &Path, readable: bool) -> io::Result<File> {
    os().create_publication_stage(path, readable)
}
pub fn rename_no_replace(file: &File, staged: &Path, target: &Path) -> io::Result<()> {
    os().rename_no_replace(file, staged, target)
}
pub fn create_transient_lock(path: &Path) -> io::Result<File> {
    os().create_transient_lock(path)
}
/// Builder-chain form of [`apply_share_mode`] / [`apply_open_flags`].
pub trait OpenOptionsPolicy {
    fn share(&mut self, mode: ShareMode) -> &mut Self;
    fn flags(&mut self, flags: OpenFlags) -> &mut Self;
}
impl OpenOptionsPolicy for OpenOptions {
    fn share(&mut self, mode: ShareMode) -> &mut Self {
        apply_share_mode(self, mode);
        self
    }
    fn flags(&mut self, flags: OpenFlags) -> &mut Self {
        apply_open_flags(self, flags);
        self
    }
}
/// Final-component no-follow open.
pub const NO_FOLLOW: OpenFlags = OpenFlags {
    no_follow: true,
    directory_handle: false,
    delete_on_close: false,
    delete_access: false,
};
/// No-follow directory handle.
pub const NO_FOLLOW_DIRECTORY: OpenFlags = OpenFlags {
    no_follow: true,
    directory_handle: true,
    delete_on_close: false,
    delete_access: false,
};
pub fn normalize_local_path(path: &Path) -> Option<std::path::PathBuf> {
    os().normalize_local_path(path)
}
/// Whether a held read-shared handle makes concurrent writer opens fail on this host.
pub fn has_mandatory_share_modes() -> bool {
    host().has_mandatory_share_modes()
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
