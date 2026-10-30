use crate::{
    application::{
        api::{ApiError, ErrorCode},
        download::{DownloadOutput, PartialOutput, VerificationLease},
    },
    infrastructure::{
        immutable_storage_paths, storage_directory_identity::StorageDirectoryIdentity,
    },
};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    os::windows::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
        io::AsRawHandle,
    },
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use windows::Win32::{
    Foundation::HANDLE,
    Storage::FileSystem::{
        FILE_DISPOSITION_INFO, FILE_RENAME_INFO, FileDispositionInfo, FileRenameInfo,
        SetFileInformationByHandle,
    },
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Root {
    path: PathBuf,
    guards: Vec<File>,
    identity: StorageDirectoryIdentity,
}
impl Root {
    fn check(&self) -> Result<(), ApiError> {
        let current = immutable_storage_paths::directory(&self.path)?;
        let original = self
            .guards
            .last()
            .ok_or_else(|| error(ErrorCode::Internal, "Missing output root guard"))?;
        if StorageDirectoryIdentity::observe(original)? != self.identity
            || StorageDirectoryIdentity::observe(&current)? != self.identity
        {
            return Err(error(
                ErrorCode::SourceExpired,
                "Download output directory changed",
            ));
        }
        Ok(())
    }
}

pub struct StagedDownloadOutput {
    root: Arc<Root>,
    name: String,
}
impl StagedDownloadOutput {
    /// Caller supplies an existing ordinary local directory and one portable filename.
    pub fn new(directory: &Path, name: &str) -> Result<Self, ApiError> {
        let stem = name
            .split('.')
            .next()
            .unwrap_or_default()
            .trim_end_matches(' ')
            .to_ascii_uppercase();
        if name.is_empty()
            || name.encode_utf16().count() > 200
            || name.ends_with([' ', '.'])
            || name
                .chars()
                .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
            || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || matches!(
                stem.as_str(),
                "COM¹" | "COM²" | "COM³" | "LPT¹" | "LPT²" | "LPT³"
            )
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && stem.as_bytes()[3].is_ascii_digit()
                && stem.as_bytes()[3] != b'0')
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Choose a portable download filename",
            ));
        }
        let (path, guards) = immutable_storage_paths::root(directory)?;
        let identity = StorageDirectoryIdentity::observe(
            guards
                .last()
                .ok_or_else(|| error(ErrorCode::Internal, "Missing output root guard"))?,
        )?;
        Ok(Self {
            root: Arc::new(Root {
                path,
                guards,
                identity,
            }),
            name: name.into(),
        })
    }
}
struct Staged {
    guard: Arc<SealedGuard>,
    target: PathBuf,
    sealed: bool,
    published: bool,
}
struct SealedGuard {
    file: File,
    root: Arc<Root>,
    path: PathBuf,
    identity: StorageDirectoryIdentity,
    aborted: std::sync::atomic::AtomicBool,
}
impl VerificationLease for SealedGuard {
    fn path(&self) -> &Path {
        &self.path
    }
    fn check_current(&self) -> Result<(), ApiError> {
        if self.aborted.load(Ordering::Acquire) {
            return Err(error(
                ErrorCode::Cancelled,
                "Original output verification was aborted",
            ));
        }
        self.root.check()?;
        if StorageDirectoryIdentity::observe(&self.file)? != self.identity {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original output file identity changed",
            ));
        }
        Ok(())
    }
}
impl StagedDownloadOutput {
    fn create_staged(&self) -> Result<Staged, ApiError> {
        self.root.check()?;
        let path = self.root.path.join(format!(
            ".download-{}-{}.part",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        // GENERIC_READ | GENERIC_WRITE | DELETE; deny competing write and delete.
        // Verifiers reopening this file must share original write/delete access.
        let file = OpenOptions::new()
            .write(true)
            .access_mode(0xc0010000)
            .share_mode(1)
            .create_new(true)
            .custom_flags(0x00200000)
            .open(&path)
            .map_err(io_error)?;
        let identity = match StorageDirectoryIdentity::observe(&file) {
            Ok(identity) => identity,
            Err(mut primary) => {
                if let Err(cleanup) = disposition(&file, true) {
                    primary.details = serde_json::json!({"primary_details": primary.details, "cleanup_failure": cleanup});
                }
                return Err(primary);
            }
        };
        Ok(Staged {
            guard: Arc::new(SealedGuard {
                file,
                root: self.root.clone(),
                path,
                identity,
                aborted: std::sync::atomic::AtomicBool::new(false),
            }),
            target: self.root.path.join(&self.name),
            sealed: false,
            published: false,
        })
    }
}
impl DownloadOutput for StagedDownloadOutput {
    fn create(&self) -> Result<Box<dyn PartialOutput>, ApiError> {
        Ok(Box::new(self.create_staged()?))
    }
    fn create_owned(&self) -> Result<Box<dyn PartialOutput + Send>, ApiError> {
        Ok(Box::new(self.create_staged()?))
    }
}
impl PartialOutput for Staged {
    fn publish_owned(&mut self) -> Result<(), ApiError> {
        if self.published {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Output is already published",
            ));
        }
        if Arc::strong_count(&self.guard) != 1 {
            return Err(error(
                ErrorCode::NeedsInput,
                "Probe still retains original output guards",
            ));
        }
        if !self.sealed {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Download partial is not sealed",
            ));
        }
        self.guard.check_current()?;
        rename_new(&self.guard.file, &self.target)?;
        self.check_target_identity()?;
        self.published = true;
        Ok(())
    }
    fn abort_owned(&mut self) -> Result<(), ApiError> {
        if self.published {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Published output cannot be deleted by cleanup",
            ));
        }
        self.guard.aborted.store(true, Ordering::Release);
        if Arc::strong_count(&self.guard) != 1 {
            return Err(error(
                ErrorCode::NeedsInput,
                "Probe cleanup is pending; original output guards retained",
            ));
        }
        disposition(&self.guard.file, true)
    }
    fn verify_published(
        &mut self,
        receipt: &crate::application::download::DownloadReceipt,
        control: &dyn crate::application::jobs::ExecutionControl,
    ) -> Result<crate::application::jobs::VerifiedJobOutput, ApiError> {
        use sha2::{Digest, Sha256};
        use std::io::{Read, Seek, SeekFrom};
        if !self.published {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Output is not published",
            ));
        }
        self.guard.check_current()?;
        if self.guard.file.metadata().map_err(io_error)?.len() != receipt.bytes {
            return Err(error(
                ErrorCode::SourceExpired,
                "Published output byte count changed",
            ));
        }
        let mut reader = &self.guard.file;
        reader.seek(SeekFrom::Start(0)).map_err(io_error)?;
        let mut digest = Sha256::new();
        let mut bytes = 0_u64;
        let mut buffer = [0_u8; 65536];
        loop {
            if std::time::Instant::now() >= control.deadline() {
                return Err(error(
                    ErrorCode::Timeout,
                    "Original publication verification deadline expired",
                ));
            }
            let count = reader.read(&mut buffer).map_err(io_error)?;
            if count == 0 {
                break;
            }
            bytes = bytes
                .checked_add(count as u64)
                .ok_or_else(|| error(ErrorCode::ResourceExhausted, "Output byte count overflow"))?;
            if bytes > receipt.bytes {
                return Err(error(
                    ErrorCode::SourceExpired,
                    "Published output byte count changed",
                ));
            }
            digest.update(&buffer[..count]);
        }
        let actual: [u8; 32] = digest.finalize().into();
        if bytes != receipt.bytes || actual != receipt.sha256 {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original published output bytes changed",
            ));
        }
        self.guard.check_current()?;
        let selection = serde_json::to_vec(&(&receipt.identity.media, &receipt.identity.tracks))
            .map_err(|_| error(ErrorCode::Internal, "Invalid selected media identity"))?;
        self.check_target_identity()?;
        Ok(crate::application::jobs::VerifiedJobOutput {
            path: self
                .target
                .to_str()
                .ok_or_else(|| {
                    error(
                        ErrorCode::InvalidArgument,
                        "Output path cannot be represented by this API",
                    )
                })?
                .into(),
            sha256: actual.iter().map(|byte| format!("{byte:02x}")).collect(),
            bytes: kinekino_media_contracts::api_values::DecimalU64(bytes),
            receipt: serde_json::json!({"operation":"production.download_media","source_selection_sha256":format!("{:x}",Sha256::digest(selection)),"original_output_guard_retained":true}),
        })
    }
    fn verification_lease(&self) -> Result<Arc<dyn VerificationLease>, ApiError> {
        if !self.sealed {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Output must be sealed before probe admission",
            ));
        }
        self.guard.check_current()?;
        Ok(self.guard.clone())
    }
    fn append(&mut self, bytes: &[u8]) -> Result<(), ApiError> {
        if self.sealed {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Download partial is sealed",
            ));
        }
        (&self.guard.file).write_all(bytes).map_err(io_error)
    }
    fn seal(&mut self) -> Result<(), ApiError> {
        self.guard.file.sync_all().map_err(io_error)?;
        self.sealed = true;
        Ok(())
    }
    fn path(&self) -> &Path {
        &self.guard.path
    }
    fn abort(mut self: Box<Self>) -> Result<(), ApiError> {
        self.abort_owned()
    }
    fn publish(mut self: Box<Self>) -> Result<(), ApiError> {
        let result = self.publish_owned();
        if let Err(mut primary) = result {
            if Arc::strong_count(&self.guard) == 1
                && let Err(cleanup) = disposition(&self.guard.file, true)
            {
                primary.details = serde_json::json!({"primary_details": primary.details, "cleanup_failure": cleanup});
            }
            return Err(primary);
        }
        Ok(())
    }
}
impl Staged {
    fn check_target_identity(&self) -> Result<(), ApiError> {
        self.guard.check_current()?;
        // Witness only: READ access, sharing the original writer's READ/WRITE/DELETE.
        // The original handle/guards remain the authority and continue to deny
        // competing writes/deletes. Do not substitute this pathname lookup for them.
        let witness = OpenOptions::new()
            .read(true)
            .share_mode(7)
            .custom_flags(0x00200000)
            .open(&self.target)
            .map_err(io_error)?;
        let metadata = witness.metadata().map_err(io_error)?;
        if !metadata.is_file()
            || metadata.file_attributes() & 0x400 != 0
            || StorageDirectoryIdentity::observe(&witness)? != self.guard.identity
        {
            return Err(error(
                ErrorCode::SourceExpired,
                "Published target differs from the original file identity",
            ));
        }
        self.guard.check_current()
    }
}
impl Drop for Staged {
    fn drop(&mut self) {
        if !self.published {
            self.guard.aborted.store(true, Ordering::Release);
            if Arc::strong_count(&self.guard) == 1 {
                // Never remove by a reopenable pathname. OS failure leaves an owned .part;
                // it must not be reported as successful cleanup by the caller.
                let _ = disposition(&self.guard.file, true);
            }
        }
    }
}
fn disposition(file: &File, delete: bool) -> Result<(), ApiError> {
    let info = FILE_DISPOSITION_INFO { DeleteFile: delete };
    // Exact initialized repr(C) layout, original live handle, synchronous call.
    unsafe {
        SetFileInformationByHandle(
            HANDLE(file.as_raw_handle()),
            FileDispositionInfo,
            (&info as *const FILE_DISPOSITION_INFO).cast(),
            std::mem::size_of_val(&info) as u32,
        )
    }
    .map_err(|e| {
        io_error(std::io::Error::from_raw_os_error(
            (e.code().0 as u32 & 0xffff) as i32,
        ))
    })
}
fn rename_new(file: &File, target: &Path) -> Result<(), ApiError> {
    let mut name: Vec<u16> = target.as_os_str().encode_wide().collect();
    let name_bytes = name
        .len()
        .checked_mul(std::mem::size_of::<u16>())
        .ok_or_else(|| error(ErrorCode::InvalidArgument, "Download path too long"))?;
    let name_length = u32::try_from(name_bytes)
        .map_err(|_| error(ErrorCode::InvalidArgument, "Download path too long"))?;
    // Explicit terminator, independent of allocator/alignment padding. The native
    // logical filename length excludes it; the supplied storage includes it.
    name.push(0);
    let terminated_bytes = name_bytes
        .checked_add(std::mem::size_of::<u16>())
        .ok_or_else(|| error(ErrorCode::InvalidArgument, "Download path too long"))?;
    let offset = std::mem::offset_of!(FILE_RENAME_INFO, FileName);
    let bytes = offset
        .checked_add(terminated_bytes)
        .ok_or_else(|| error(ErrorCode::InvalidArgument, "Download path too long"))?;
    let length = u32::try_from(bytes)
        .map_err(|_| error(ErrorCode::InvalidArgument, "Download path too long"))?;
    // usize storage supplies FILE_RENAME_INFO alignment and initialized trailing bytes.
    let mut storage = vec![
        0_usize;
        bytes
            .max(std::mem::size_of::<FILE_RENAME_INFO>())
            .div_ceil(std::mem::size_of::<usize>())
    ];
    let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    unsafe {
        (*info).FileNameLength = name_length;
        // Zeroed ReplaceIfExists is false: an existing output is never replaced.
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            storage.as_mut_ptr().cast::<u8>().add(offset).cast::<u16>(),
            name.len(),
        );
        SetFileInformationByHandle(
            HANDLE(file.as_raw_handle()),
            FileRenameInfo,
            info.cast(),
            length,
        )
    }
    .map_err(|e| {
        io_error(std::io::Error::from_raw_os_error(
            (e.code().0 as u32 & 0xffff) as i32,
        ))
    })
}
fn error(code: ErrorCode, message: &str) -> ApiError {
    ApiError::new(code, message)
}
fn io_error(error: std::io::Error) -> ApiError {
    let kind = format!("{:?}", error.kind());
    let raw_os_error = error.raw_os_error();
    let mut result = kinekino_core::file_publication::io_error(error);
    result.details = serde_json::json!({"io_kind": kind, "raw_os_error": raw_os_error});
    result
}

#[cfg(test)]
mod verification_lease_tests {
    use super::*;
    #[test]
    fn same_bytes_at_another_target_cannot_verify_original_published_handle() {
        // Process entry for this test: the OS policy backs the file guards.
        kinekino_platform::install();
        use crate::application::{
            download::{DownloadReceipt, MediaIdentity},
            jobs::ExecutionControl,
        };
        use sha2::{Digest, Sha256};
        struct Control(std::time::Instant);
        impl ExecutionControl for Control {
            fn cancellation_requested(&self) -> bool {
                false
            }
            fn deadline(&self) -> std::time::Instant {
                self.0
            }
            fn reserve_publication(&self) -> Result<(), ApiError> {
                panic!("Adapter test cannot reserve publication")
            }
            fn begin_verification(&self) -> Result<(), ApiError> {
                panic!("Adapter test cannot change job state")
            }
        }
        let fixture = Fixture::new();
        let output = fixture.output();
        let mut partial = output.create_staged().unwrap();
        let data = b"same bytes are not file identity";
        partial.append(data).unwrap();
        partial.seal().unwrap();
        partial.publish_owned().unwrap();
        let original_target = partial.target.clone();
        let unrelated = fixture.0.join("unrelated.media");
        std::fs::write(&unrelated, data).unwrap();
        partial.target = unrelated.clone();
        let receipt = DownloadReceipt {
            identity: MediaIdentity {
                media: "fixture".into(),
                tracks: "fixture".into(),
            },
            bytes: data.len() as u64,
            sha256: Sha256::digest(data).into(),
        };
        assert_eq!(
            partial
                .verify_published(
                    &receipt,
                    &Control(std::time::Instant::now() + std::time::Duration::from_secs(10))
                )
                .unwrap_err()
                .code,
            ErrorCode::SourceExpired
        );
        assert_eq!(std::fs::read(&original_target).unwrap(), data);
        assert_eq!(std::fs::read(&unrelated).unwrap(), data);
    }
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "download-verification-lease-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn output(&self) -> StagedDownloadOutput {
            StagedDownloadOutput::new(&self.0, "episode.mp4").unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn sealed_lease_never_authorizes_appending() {
        // Process entry for this test: the OS policy backs the file guards.
        kinekino_platform::install();
        let fixture = Fixture::new();
        let output = fixture.output();
        let mut partial = output.create_owned().unwrap();
        assert_eq!(
            partial.verification_lease().err().unwrap().code,
            ErrorCode::RevisionConflict
        );
        partial.append(b"fixture").unwrap();
        partial.seal().unwrap();
        let lease = partial.verification_lease().unwrap();
        lease.check_current().unwrap();
        assert_eq!(
            partial.append(b"later").unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        drop(lease);
        partial.abort().unwrap();
    }
    #[test]
    fn retained_original_lease_denies_write_rename_and_delete() {
        // Process entry for this test: the OS policy backs the file guards.
        kinekino_platform::install();
        let fixture = Fixture::new();
        let output = fixture.output();
        let mut partial = output.create_owned().unwrap();
        partial.append(b"fixture").unwrap();
        partial.seal().unwrap();
        let lease = partial.verification_lease().unwrap();
        let path = lease.path().to_owned();
        assert_eq!(
            OpenOptions::new()
                .write(true)
                .share_mode(7)
                .open(&path)
                .unwrap_err()
                .raw_os_error(),
            Some(32)
        );
        assert_eq!(
            std::fs::rename(&path, fixture.0.join("moved.part"))
                .unwrap_err()
                .raw_os_error(),
            Some(32)
        );
        assert_eq!(
            std::fs::remove_file(&path).unwrap_err().raw_os_error(),
            Some(32)
        );
        lease.check_current().unwrap();
        drop(lease);
        partial.abort().unwrap();
        assert!(!path.exists());
    }
    #[test]
    fn premature_abort_retains_guard_and_reports_pending_cleanup() {
        // Process entry for this test: the OS policy backs the file guards.
        kinekino_platform::install();
        let fixture = Fixture::new();
        let output = fixture.output();
        let mut partial = output.create_owned().unwrap();
        partial.append(b"fixture").unwrap();
        partial.seal().unwrap();
        let lease = partial.verification_lease().unwrap();
        let path = lease.path().to_owned();
        assert_eq!(partial.abort().unwrap_err().code, ErrorCode::NeedsInput);
        assert_eq!(
            lease.check_current().unwrap_err().code,
            ErrorCode::Cancelled
        );
        assert_eq!(
            OpenOptions::new()
                .write(true)
                .share_mode(7)
                .open(&path)
                .unwrap_err()
                .raw_os_error(),
            Some(32)
        );
        drop(lease);
        std::fs::remove_file(&path).unwrap();
    }
    #[test]
    fn premature_publish_preserves_partial_and_creates_no_final_file() {
        // Process entry for this test: the OS policy backs the file guards.
        kinekino_platform::install();
        let fixture = Fixture::new();
        let output = fixture.output();
        let mut partial = output.create_owned().unwrap();
        partial.append(b"fixture").unwrap();
        partial.seal().unwrap();
        let lease = partial.verification_lease().unwrap();
        let path = lease.path().to_owned();
        assert_eq!(partial.publish().unwrap_err().code, ErrorCode::NeedsInput);
        assert!(!fixture.0.join("episode.mp4").exists());
        assert!(path.exists());
        assert_eq!(
            std::fs::remove_file(&path).unwrap_err().raw_os_error(),
            Some(32)
        );
        drop(lease);
        std::fs::remove_file(&path).unwrap();
    }
}
