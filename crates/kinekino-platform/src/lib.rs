//! Platform facade: the only place that selects an OS adapter with `cfg`.
pub use kinekino_core::platform::{
    CloseFailure, FileIdentity, FileIdentity128, OpenFlags, OsFamily, PlatformHost, ShareMode,
};

#[cfg(not(windows))]
use kinekino_platform_unix as adapter;
#[cfg(windows)]
use kinekino_platform_windows as adapter;

/// Returns the adapter for the OS this binary was built for.
pub fn current() -> impl PlatformHost {
    HOST
}

#[cfg(windows)]
const HOST: adapter::WindowsHost = adapter::WindowsHost;
#[cfg(not(windows))]
const HOST: adapter::UnixHost = adapter::UnixHost;
#[cfg(windows)]
static PROCESS: adapter::process::ownership::WindowsOwnership =
    adapter::process::ownership::WindowsOwnership;
#[cfg(not(windows))]
static PROCESS: adapter::process::ownership::UnixOwnership =
    adapter::process::ownership::UnixOwnership;

#[cfg(windows)]
static IPC: adapter::ipc::WindowsIpc = adapter::ipc::WindowsIpc;
#[cfg(not(windows))]
static IPC: adapter::ipc::UnixIpc = adapter::ipc::UnixIpc { runtime_dir: None };
#[cfg(windows)]
static WINDOW: adapter::window::WindowsWindows = adapter::window::WindowsWindows;
// X11 adapter pending (ADR 0009); Unix reports `has_native_desktop_window() == false`.
#[cfg(not(windows))]
static WINDOW: kinekino_core::platform::window::UnsupportedWindows =
    kinekino_core::platform::window::UnsupportedWindows;

/// File-open policy port.
pub mod fs {
    pub use super::adapter::fs::{
        apply_open_flags, apply_share_mode, file_identity, is_reparse_point, is_sharing_violation,
        open_reserved_read, symlink_file,
    };
}

/// Child-process policy port.
pub mod process {
    pub use super::adapter::process::hide_console_window;
}

/// Raw-handle ownership port.
pub mod io {
    pub use super::adapter::io::close_original;
}

/// Session port: target platform detection and the credential store.
pub mod session {
    pub use super::adapter::session::{SessionCredentialStore, target};
}

/// [`kinekino_core::platform::OsPolicy`] backed by this build's OS adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct CurrentOs;

impl kinekino_core::platform::OsPolicy for CurrentOs {
    fn apply_share_mode(&self, options: &mut std::fs::OpenOptions, mode: ShareMode) {
        adapter::fs::apply_share_mode(options, mode);
    }
    fn apply_open_flags(&self, options: &mut std::fs::OpenOptions, flags: OpenFlags) {
        adapter::fs::apply_open_flags(options, flags);
    }
    fn open_reserved_read(&self, path: &std::path::Path) -> std::io::Result<std::fs::File> {
        adapter::fs::open_reserved_read(path)
    }
    fn is_reparse_point(&self, metadata: &std::fs::Metadata) -> bool {
        adapter::fs::is_reparse_point(metadata)
    }
    fn is_sharing_violation(&self, error: &std::io::Error) -> bool {
        adapter::fs::is_sharing_violation(error)
    }
    fn file_identity(&self, file: &std::fs::File) -> std::io::Result<FileIdentity> {
        adapter::fs::file_identity(file)
    }
    fn hide_console_window(&self, command: &mut std::process::Command) {
        adapter::process::hide_console_window(command);
    }
    fn close_original(&self, file: std::fs::File) -> Result<(), CloseFailure> {
        adapter::io::close_original(file)
    }
    fn file_identity_128(&self, file: &std::fs::File) -> std::io::Result<FileIdentity128> {
        adapter::fs::file_identity_128(file)
    }
    fn is_local_fixed_path(&self, path: &std::path::Path) -> bool {
        adapter::fs::is_local_fixed_path(path)
    }
    fn normalize_local_path(&self, path: &std::path::Path) -> Option<std::path::PathBuf> {
        adapter::fs::normalize_local_path(path)
    }
    fn try_lock_exclusive(&self, file: &std::fs::File) -> std::io::Result<()> {
        adapter::fs::try_lock_exclusive(file)
    }
    fn try_lock_shared(&self, file: &std::fs::File) -> std::io::Result<()> {
        adapter::fs::try_lock_shared(file)
    }
    fn fresh_guid(&self) -> std::io::Result<[u8; 16]> {
        adapter::id::fresh_guid()
    }
    fn create_publication_stage(
        &self,
        path: &std::path::Path,
        readable: bool,
    ) -> std::io::Result<std::fs::File> {
        adapter::publication::create_stage(path, readable)
    }
    fn rename_no_replace(
        &self,
        file: &std::fs::File,
        staged: &std::path::Path,
        target: &std::path::Path,
    ) -> std::io::Result<()> {
        adapter::publication::rename_no_replace(file, staged, target)
    }
    fn create_transient_lock(&self, path: &std::path::Path) -> std::io::Result<std::fs::File> {
        adapter::publication::create_transient_lock(path)
    }
    fn host(&self) -> &dyn PlatformHost {
        &HOST
    }
    fn process(&self) -> &dyn kinekino_core::platform::process::ProcessOwnership {
        &PROCESS
    }
    fn ipc(&self) -> &dyn kinekino_core::platform::ipc::IpcTransport {
        &IPC
    }
    fn counter(&self) -> &dyn kinekino_core::platform::clock::ParentCounter {
        &adapter::process::clock::COUNTER
    }
    fn window(&self) -> &dyn kinekino_core::platform::window::WindowPort {
        &WINDOW
    }
}

/// Installs [`CurrentOs`] as the process-wide OS policy. Idempotent; call from every entry.
pub fn install() {
    kinekino_core::platform::install(&CurrentOs);
}

#[cfg(test)]
mod install_tests {
    #[test]
    fn install_registers_current_os() {
        super::install();
        assert!(kinekino_core::platform::is_installed());
        let file = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
        let os = kinekino_core::platform::os();
        assert_eq!(
            os.file_identity(&file).unwrap(),
            super::fs::file_identity(&file).unwrap()
        );
    }
}
