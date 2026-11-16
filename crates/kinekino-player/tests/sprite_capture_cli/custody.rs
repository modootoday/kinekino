//! Holds an original file and its parent directories open, without following links, while a
//! CLI child runs, and proves afterwards that the same file is still in place. Opens go through
//! the installed OS policy port: Windows denies writers and renames while the guard is held
//! (share modes); POSIX keeps the descriptors and checks identity (dev + inode) afterwards.
use kinekino_player::infrastructure::os_policy::{self, OpenFlags, ShareMode};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    path::Path,
};
pub struct Guard {
    _directories: Vec<File>,
    file: File,
    pub sha256: String,
    before: String,
}
fn identity(file: &File) -> String {
    let id = os_policy::file_identity(file).unwrap();
    let metadata = file.metadata().unwrap();
    format!(
        "{:016X}-{:016X}:{}:{:?}",
        id.volume,
        id.index,
        metadata.len(),
        metadata.modified().unwrap()
    )
}
fn open(path: &Path, share: ShareMode, directory: bool) -> File {
    let mut options = OpenOptions::new();
    options.read(true);
    os_policy::apply_share_mode(&mut options, share);
    os_policy::apply_open_flags(
        &mut options,
        OpenFlags {
            no_follow: true,
            directory_handle: directory,
            ..OpenFlags::default()
        },
    );
    let file = options.open(path).unwrap();
    let metadata = file.metadata().unwrap();
    assert_eq!(metadata.is_dir(), directory, "{}", path.display());
    assert!(
        !os_policy::is_reparse_point(&metadata),
        "{}",
        path.display()
    );
    file
}
impl Guard {
    pub fn acquire(path: &Path) -> Self {
        let canonical = path.canonicalize().unwrap();
        let mut parents: Vec<_> = path.ancestors().skip(1).collect();
        parents.reverse();
        let directories = parents
            .into_iter()
            .filter(|directory| !directory.as_os_str().is_empty())
            .map(|directory| open(directory, ShareMode::ReadWrite, true))
            .collect();
        let mut file = open(path, ShareMode::ReadOnly, false);
        assert_eq!(path.canonicalize().unwrap(), canonical);
        let before = identity(&file);
        let mut hasher = Sha256::new();
        let mut buffer = [0; 65536];
        file.seek(SeekFrom::Start(0)).unwrap();
        loop {
            let count = file.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        let sha256 = kinekino_media_contracts::hex_lower(hasher.finalize());
        assert_eq!(identity(&file), before);
        Self {
            _directories: directories,
            file,
            sha256,
            before,
        }
    }
    pub fn assert_unchanged(&self) {
        assert_eq!(identity(&self.file), self.before);
    }
    pub fn evidence(&self) -> serde_json::Value {
        serde_json::json!({"sha256":self.sha256,"file_identity":self.before})
    }
}
