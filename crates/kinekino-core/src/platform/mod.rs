//! Platform port: capabilities the core needs from an OS adapter crate.
pub mod clock;
pub mod ipc;
pub mod process;
pub mod settings_window;
pub mod window;

/// OS family reported by an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsFamily {
    Windows,
    Unix,
}

/// Implemented by `kinekino-platform-*`; selected once in `kinekino-platform`.
pub trait PlatformHost {
    fn os_family(&self) -> OsFamily;
    /// Whether an embedded browser engine adapter is available in this build.
    fn has_embedded_browser(&self) -> bool;
    /// Whether the native mpv video surface adapter is available in this build.
    fn has_native_video_surface(&self) -> bool;
    /// Whether this OS adapter can host a native desktop (proposal/viewer) window.
    fn has_native_desktop_window(&self) -> bool;
    /// Whether opens honour mandatory share modes, so a held [`ShareMode::ReadOnly`] handle makes
    /// a concurrent writer open fail. POSIX hosts only offer advisory locks.
    fn has_mandatory_share_modes(&self) -> bool;
}

/// How other handles may share a file opened through the platform port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareMode {
    /// No concurrent handle may open the file (Windows `share_mode(0)`).
    Exclusive,
    /// Concurrent readers only (Windows `FILE_SHARE_READ`).
    ReadOnly,
    /// Concurrent readers and writers (`FILE_SHARE_READ | FILE_SHARE_WRITE`).
    ReadWrite,
    /// Readers, writers and delete/rename (`FILE_SHARE_READ | WRITE | DELETE`).
    All,
}

/// Extra open semantics expressed OS-neutrally (Win32 `FILE_FLAG_*`; no-ops where absent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OpenFlags {
    /// Open a reparse point/symlink itself instead of following it.
    pub no_follow: bool,
    /// Allow opening directories as handles.
    pub directory_handle: bool,
    /// Delete the file once the last handle closes.
    pub delete_on_close: bool,
    /// Request read + write + delete access (Win32 `DELETE` right; POSIX needs none).
    pub delete_access: bool,
}

/// Failure to close an original handle; the raw value is retained only as evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseFailure {
    pub raw: usize,
    pub code: i32,
}

/// Stable identity of an open file (volume + file index / dev + inode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileIdentity {
    pub volume: u64,
    pub index: u64,
}

/// 128-bit file identity (volume serial + full file id / dev + inode), for persisted origins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileIdentity128 {
    pub volume: u64,
    pub file: [u8; 16],
}

/// OS file/process/handle policy port. Implemented by `kinekino-platform` (`CurrentOs`) and
/// installed once per process by its entry (`kinekino_platform::install`). Domain crates call
/// [`os`] instead of depending on an adapter crate.
pub trait OsPolicy: Send + Sync {
    /// Applies a share policy to `options` (no-op where the OS has no mandatory share modes).
    fn apply_share_mode(&self, options: &mut std::fs::OpenOptions, mode: ShareMode);
    /// Applies OS-neutral open flags to `options`.
    fn apply_open_flags(&self, options: &mut std::fs::OpenOptions, flags: OpenFlags);
    /// Opens an existing file for reading without following a final link.
    fn open_reserved_read(&self, path: &std::path::Path) -> std::io::Result<std::fs::File>;
    /// True for reparse points / symlinks.
    fn is_reparse_point(&self, metadata: &std::fs::Metadata) -> bool;
    /// True when `error` is a share-mode conflict.
    fn is_sharing_violation(&self, error: &std::io::Error) -> bool;
    /// Stable identity of an open file.
    fn file_identity(&self, file: &std::fs::File) -> std::io::Result<FileIdentity>;
    /// Prevents a console window for a child process (no-op where none exists).
    fn hide_console_window(&self, command: &mut std::process::Command);
    /// Closes exactly the original handle of `file` once.
    fn close_original(&self, file: std::fs::File) -> Result<(), CloseFailure>;
    /// 128-bit identity of an open file or directory handle.
    fn file_identity_128(&self, file: &std::fs::File) -> std::io::Result<FileIdentity128>;
    /// True for an absolute path on a local fixed volume (Windows: drive-letter disk prefix).
    fn is_local_fixed_path(&self, path: &std::path::Path) -> bool;
    /// Normalizes an absolute local-disk path (Windows: strips a `\\?\` verbatim-disk prefix;
    /// rejects UNC/device paths). `None` when the path is not an ordinary local absolute path.
    fn normalize_local_path(&self, path: &std::path::Path) -> Option<std::path::PathBuf>;
    /// Takes a non-blocking exclusive lock on `file`; `ErrorKind::WouldBlock` when held elsewhere.
    /// No-op where an exclusive share mode already excludes other handles.
    fn try_lock_exclusive(&self, file: &std::fs::File) -> std::io::Result<()>;
    /// Shared counterpart of [`Self::try_lock_exclusive`]: no-op where mandatory share modes
    /// already exclude writers; `WouldBlock` while an exclusive holder exists elsewhere.
    fn try_lock_shared(&self, file: &std::fs::File) -> std::io::Result<()>;
    /// 16 OS-random bytes in GUID display order (`data1` BE, `data2` BE, `data3` BE, `data4`).
    fn fresh_guid(&self) -> std::io::Result<[u8; 16]>;
    /// Creates a new staging file for complete-file publication: write access (plus read when
    /// `readable`), the right to rename the open file, readers-only sharing, no final-link follow.
    fn create_publication_stage(
        &self,
        path: &std::path::Path,
        readable: bool,
    ) -> std::io::Result<std::fs::File>;
    /// Renames the open staging `file` (created at `staged`) to `target`, failing with
    /// `AlreadyExists` instead of replacing an existing target.
    fn rename_no_replace(
        &self,
        file: &std::fs::File,
        staged: &std::path::Path,
        target: &std::path::Path,
    ) -> std::io::Result<()>;
    /// Acquires a process-exclusive lock file at `path` that is released when the returned handle
    /// is dropped. A held lock fails with `AlreadyExists`.
    fn create_transient_lock(&self, path: &std::path::Path) -> std::io::Result<std::fs::File>;
    /// Capability table of the OS adapter this policy delegates to.
    fn host(&self) -> &dyn PlatformHost;
    /// Process-family ownership adapter (ADR 0006).
    fn process(&self) -> &dyn process::ProcessOwnership;
    /// Local IPC transport adapter (ADR 0007).
    fn ipc(&self) -> &dyn ipc::IpcTransport;
    /// Parent-anchored monotonic counter adapter (ADR 0008).
    fn counter(&self) -> &dyn clock::ParentCounter;
    /// Native window port (ADR 0009).
    fn window(&self) -> &dyn window::WindowPort;
}

static OS: std::sync::OnceLock<&'static dyn OsPolicy> = std::sync::OnceLock::new();

/// Registers the process-wide [`OsPolicy`]. First install wins; later calls are ignored.
pub fn install(policy: &'static dyn OsPolicy) {
    let _ = OS.set(policy);
}

/// Whether an [`OsPolicy`] has been installed in this process.
pub fn is_installed() -> bool {
    OS.get().is_some()
}

/// The installed [`OsPolicy`].
///
/// # Panics
/// When no entry installed one; every process entry must call `kinekino_platform::install()`.
pub fn os() -> &'static dyn OsPolicy {
    *OS.get()
        .expect("OsPolicy not installed: the process entry must call kinekino_platform::install()")
}
