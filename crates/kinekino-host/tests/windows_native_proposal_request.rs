//! Windows share-mode and mount-point semantics of the native proposal request reader.
//! These fixtures need Win32 share modes and `FSCTL_SET_REPARSE_POINT`, so they live in the
//! host adapter crate rather than in the OS-neutral player crate.
#![cfg(windows)]
use kinekino_player::{
    application::api::ErrorCode,
    infrastructure::native_proposal_request::read_native_proposal_request,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let path = std::env::temp_dir().join(format!(
            "player-native-request-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn read_handle_guards_prevent_opening_a_file_with_exclusive_writer() {
    // Process entry for this test: the OS policy backs the file guards.
    kinekino_platform::install();
    use std::os::windows::fs::OpenOptionsExt;
    let fixture = Fixture::new();
    let path = fixture.0.join("request.json");
    fs::write(&path, b"{}").unwrap();
    let writer = fs::OpenOptions::new()
        .write(true)
        .share_mode(0)
        .open(&path)
        .unwrap();
    assert!(read_native_proposal_request(&path).is_err());
    drop(writer);
    assert_eq!(read_native_proposal_request(&path).unwrap(), b"{}");
}

#[test]
fn junction_ancestor_and_reparse_final_entry_are_rejected_without_following_them() {
    // Process entry for this test: the OS policy backs the file guards.
    kinekino_platform::install();
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    use windows::Win32::{Foundation::HANDLE, System::IO::DeviceIoControl};

    let fixture = Fixture::new();
    let target = fixture.0.join("target");
    let junction = fixture.0.join("junction");
    fs::create_dir(&target).unwrap();
    fs::create_dir(&junction).unwrap();
    fs::write(target.join("request.json"), b"{}").unwrap();
    // Windows mount-point fixture through the OS adapter, without shell or symlink privileges.
    let substitute: Vec<_> = format!("\\??\\{}", target.display())
        .encode_utf16()
        .collect();
    let print: Vec<_> = target.to_string_lossy().encode_utf16().collect();
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&0xa0000003_u32.to_le_bytes());
    let data_len = 8 + (substitute.len() + print.len() + 2) * 2;
    buffer.extend_from_slice(&u16::try_from(data_len).unwrap().to_le_bytes());
    buffer.extend_from_slice(&0_u16.to_le_bytes());
    for field in [
        0,
        substitute.len() * 2,
        (substitute.len() + 1) * 2,
        print.len() * 2,
    ] {
        buffer.extend_from_slice(&u16::try_from(field).unwrap().to_le_bytes());
    }
    for character in substitute.into_iter().chain([0]).chain(print).chain([0]) {
        buffer.extend_from_slice(&character.to_le_bytes());
    }
    let directory = fs::OpenOptions::new()
        .write(true)
        .share_mode(3)
        .custom_flags(0x02200000)
        .open(&junction)
        .unwrap();
    let mut returned = 0;
    // SAFETY: directory is live; the mount-point buffer has checked lengths and remains live
    // for this synchronous DeviceIoControl call. No output buffer or OVERLAPPED is supplied.
    unsafe {
        DeviceIoControl(
            HANDLE(directory.as_raw_handle()),
            0x000900a4,
            Some(buffer.as_ptr().cast()),
            u32::try_from(buffer.len()).unwrap(),
            None,
            0,
            Some(&mut returned),
            None,
        )
        .unwrap();
    }
    drop(directory);
    assert_eq!(
        read_native_proposal_request(&junction.join("request.json"))
            .unwrap_err()
            .code,
        ErrorCode::PermissionDenied
    );
    assert_eq!(
        read_native_proposal_request(&junction).unwrap_err().code,
        ErrorCode::PermissionDenied
    );
    fs::remove_dir(&junction).unwrap();
}
