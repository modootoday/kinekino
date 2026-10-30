use super::StagedDownloadOutput;
use crate::application::{
    api::{ApiError, ErrorCode},
    download::{DownloadOutput, DownloadReceipt, MediaIdentity},
    jobs::ExecutionControl,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
struct VerificationControl(std::time::Instant);
impl ExecutionControl for VerificationControl {
    fn cancellation_requested(&self) -> bool {
        false
    }
    fn deadline(&self) -> std::time::Instant {
        self.0
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        panic!("Low-level filesystem test cannot reserve job publication")
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        panic!("Low-level filesystem test cannot change job state")
    }
}

#[test]
fn exact_publication_name_survives_utf16_allocation_boundaries() {
    // Process entry for this test: the OS policy backs the file guards.
    kinekino_platform::install();
    use std::{collections::BTreeSet, os::windows::ffi::OsStrExt};
    use windows::Win32::Storage::FileSystem::FILE_RENAME_INFO;
    let alignment = std::mem::align_of::<FILE_RENAME_INFO>();
    for prefix in ["ascii-", "영상🐾-"] {
        let directory = Directory::new();
        let canonical = fs::canonicalize(&directory.0).unwrap();
        let mut residues = BTreeSet::new();
        let mut names = BTreeSet::new();
        for padding in 0..alignment / std::mem::size_of::<u16>() {
            let name = format!("{prefix}{}.mp4", "x".repeat(padding));
            let target = canonical.join(&name);
            residues.insert(
                (std::mem::offset_of!(FILE_RENAME_INFO, FileName)
                    + target.as_os_str().encode_wide().count() * 2)
                    % alignment,
            );
            let output = StagedDownloadOutput::new(&directory.0, &name).unwrap();
            let mut partial = output.create_owned().unwrap();
            let bytes = b"exact published target";
            partial.append(bytes).unwrap();
            partial.seal().unwrap();
            partial.publish_owned().unwrap();
            let receipt = DownloadReceipt {
                identity: MediaIdentity {
                    media: "boundary".into(),
                    tracks: "boundary".into(),
                },
                bytes: bytes.len() as u64,
                sha256: Sha256::digest(bytes).into(),
            };
            let verified = partial
                .verify_published(
                    &receipt,
                    &VerificationControl(
                        std::time::Instant::now() + std::time::Duration::from_secs(10),
                    ),
                )
                .unwrap();
            assert_eq!(PathBuf::from(verified.path), target);
            assert_eq!(fs::read(&target).unwrap(), bytes);
            names.insert(std::ffi::OsString::from(name));
            let actual: BTreeSet<_> = fs::read_dir(&directory.0)
                .unwrap()
                .map(|entry| entry.unwrap().file_name())
                .collect();
            assert_eq!(actual, names);
        }
        assert_eq!(residues, (0..alignment).step_by(2).collect());
    }
}
#[test]
fn existing_targets_are_never_replaced_at_utf16_allocation_boundaries() {
    // Process entry for this test: the OS policy backs the file guards.
    kinekino_platform::install();
    use std::{collections::BTreeSet, os::windows::ffi::OsStrExt};
    use windows::Win32::Storage::FileSystem::FILE_RENAME_INFO;
    let alignment = std::mem::align_of::<FILE_RENAME_INFO>();
    for prefix in ["existing-", "기존🐾-"] {
        let directory = Directory::new();
        let canonical = fs::canonicalize(&directory.0).unwrap();
        let mut residues = BTreeSet::new();
        let mut names = BTreeSet::new();
        for padding in 0..alignment / std::mem::size_of::<u16>() {
            let name = format!("{prefix}{}.mp4", "x".repeat(padding));
            let target = canonical.join(&name);
            residues.insert(
                (std::mem::offset_of!(FILE_RENAME_INFO, FileName)
                    + target.as_os_str().encode_wide().count() * 2)
                    % alignment,
            );
            fs::write(&target, b"original target").unwrap();
            let output = StagedDownloadOutput::new(&directory.0, &name).unwrap();
            let mut partial = output.create_owned().unwrap();
            partial.append(b"replacement").unwrap();
            partial.seal().unwrap();
            assert_eq!(
                partial.publish_owned().unwrap_err().code,
                ErrorCode::RevisionConflict
            );
            partial.abort_owned().unwrap();
            drop(partial);
            assert_eq!(fs::read(&target).unwrap(), b"original target");
            names.insert(std::ffi::OsString::from(name));
            let actual: BTreeSet<_> = fs::read_dir(&directory.0)
                .unwrap()
                .map(|entry| entry.unwrap().file_name())
                .collect();
            assert_eq!(actual, names);
        }
        assert_eq!(residues, (0..alignment).step_by(2).collect());
    }
}

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "player-download-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn original_partial_renames_without_replacing_existing_output() {
    // Process entry for this test: the OS policy backs the file guards.
    kinekino_platform::install();
    let directory = Directory::new();
    let output = StagedDownloadOutput::new(&directory.0, "episode.mp4").unwrap();
    let mut first = output.create().unwrap();
    first.append(b"first").unwrap();
    first.seal().unwrap();
    first.publish().unwrap();
    let mut second = output.create().unwrap();
    second.append(b"second").unwrap();
    second.seal().unwrap();
    assert_eq!(
        second.publish().unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(fs::read(directory.0.join("episode.mp4")).unwrap(), b"first");
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
}
#[test]
fn explicit_abort_removes_original_partial_and_allows_new_owner() {
    // Process entry for this test: the OS policy backs the file guards.
    kinekino_platform::install();
    let directory = Directory::new();
    let output = StagedDownloadOutput::new(&directory.0, "episode.mp4").unwrap();
    let mut partial = output.create().unwrap();
    partial.append(b"incomplete").unwrap();
    partial.abort().unwrap();
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 0);
    output.create().unwrap().abort().unwrap();
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 0);
}
#[test]
fn rejects_windows_reserved_and_path_filenames_before_creating_partial() {
    // Process entry for this test: the OS policy backs the file guards.
    kinekino_platform::install();
    let directory = Directory::new();
    for name in [
        "NUL.mp4",
        "COM1.mp4",
        "LPT¹.mp4",
        "../other.mp4",
        "file:stream",
        "trailing.",
    ] {
        assert!(
            StagedDownloadOutput::new(&directory.0, name).is_err(),
            "{name}"
        );
    }
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 0);
}

#[test]
fn unsealed_publication_fails_and_cleans_owned_partial() {
    // Process entry for this test: the OS policy backs the file guards.
    kinekino_platform::install();
    let directory = Directory::new();
    let output = StagedDownloadOutput::new(&directory.0, "episode.mp4").unwrap();
    let mut partial = output.create().unwrap();
    partial.append(b"not yet verified").unwrap();
    assert_eq!(
        partial.publish().unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 0);
}
