//! Keep the verified local files and their ancestor directories guarded for the adapter lifetime.
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::identifiers::valid_sha256,
    infrastructure::{content_digest, immutable_storage_paths as paths},
};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub(crate) fn open(path: &Path, expected: &str) -> Result<(PathBuf, Vec<File>), ApiError> {
    if !valid_sha256(expected) || !path.is_absolute() {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Frame tool requires an absolute local path and SHA256",
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Frame tool parent is missing"))?;
    let name = path.file_name().ok_or_else(|| {
        ApiError::new(ErrorCode::InvalidArgument, "Frame tool filename is missing")
    })?;
    let (parent, mut guards) = paths::root(parent)?;
    let path = parent.join(name);
    let mut file = paths::file(&path)?;
    let (_, actual) = content_digest::sha256(&mut file, 512 * 1024 * 1024)?;
    if actual != expected {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Frame tool SHA256 differs from the explicit host configuration",
        ));
    }
    guards.push(file);
    Ok((path, guards))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_pin_rejects_mismatch_and_holds_verified_file_against_writes() {
        let root = std::env::temp_dir().join(format!("frame-pin-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("tool.exe");
        std::fs::write(&path, b"not executed").unwrap();
        let wrong = open(&path, &"0".repeat(64)).err().unwrap();
        assert_eq!(wrong.code, ErrorCode::SourceExpired);
        let (_, expected) = content_digest::sha256(&mut File::open(&path).unwrap(), 4096).unwrap();
        let (_, guards) = open(&path, &expected).unwrap();
        let mandatory = crate::infrastructure::os_policy::has_mandatory_share_modes();
        assert_eq!(
            std::fs::OpenOptions::new().write(true).open(&path).is_err(),
            mandatory
        );
        // Mandatory share modes keep the verified file from being removed while guarded; POSIX
        // unlink only drops the name, and the guarded original stays readable.
        assert_eq!(std::fs::remove_file(&path).is_err(), mandatory);
        if !mandatory {
            use std::io::{Read, Seek};
            let mut original = &guards[guards.len() - 1];
            original.seek(std::io::SeekFrom::Start(0)).unwrap();
            let mut held = Vec::new();
            original.read_to_end(&mut held).unwrap();
            assert_eq!(held, b"not executed");
            std::fs::write(&path, b"replacement").unwrap();
        }
        drop(guards);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
