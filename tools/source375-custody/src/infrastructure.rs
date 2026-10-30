use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
pub fn hash(path: &Path) -> Result<String, String> {
    let output = Command::new("C:/Windows/System32/certutil.exe")
        .arg("-hashfile")
        .arg(path)
        .arg("SHA256")
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!("certutil failed: {}", path.display()));
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(str::to_ascii_uppercase)
        .ok_or("SHA256 line missing".into())
}
pub fn ordinary(path: &Path) -> Result<PathBuf, String> {
    for ancestor in path.ancestors() {
        let metadata =
            fs::symlink_metadata(ancestor).map_err(|e| format!("{}: {e}", ancestor.display()))?;
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err("reparse path refused".into());
        }
    }
    path.canonicalize().map_err(|e| e.to_string())
}
pub fn identity(path: &Path) -> Result<String, String> {
    use std::os::windows::{fs::MetadataExt, io::AsRawHandle};
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandle(handle: *mut std::ffi::c_void, information: *mut u32) -> i32;
    }
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut information = [0_u32; 13];
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) } == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    Ok(format!(
        "{:08X}-{:08X}{:08X}\t{}\t{}\t{}",
        information[7],
        information[11],
        information[12],
        metadata.len(),
        metadata.creation_time(),
        metadata.last_write_time()
    ))
}
pub fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())
}
pub fn copy_verified(from: &Path, to: &Path, expected: &str) -> Result<(), String> {
    ordinary(from)?;
    write_new(to, &fs::read(from).map_err(|e| e.to_string())?)?;
    if hash(from)? != expected || hash(to)? != expected {
        return Err(format!("copy hash changed: {}", from.display()));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn os_hash_vector_no_overwrite_and_identity() {
        let path =
            std::env::temp_dir().join(format!("custody-hash-test-{}.txt", std::process::id()));
        write_new(&path, b"abc").unwrap();
        assert!(write_new(&path, b"replacement").is_err());
        assert_eq!(
            hash(&path).unwrap(),
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
        );
        assert!(!identity(&path).unwrap().is_empty());
        fs::remove_file(path).unwrap();
    }
}
