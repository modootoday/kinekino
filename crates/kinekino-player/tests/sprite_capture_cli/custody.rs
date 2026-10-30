use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    os::windows::{
        fs::{MetadataExt, OpenOptionsExt},
        io::AsRawHandle,
    },
    path::Path,
};
pub struct Guard {
    _directories: Vec<File>,
    file: File,
    pub sha256: String,
    before: String,
}
fn identity(file: &File) -> String {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandle(handle: *mut std::ffi::c_void, information: *mut u32) -> i32;
    }
    let mut information = [0u32; 13];
    assert_ne!(
        unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) },
        0
    );
    let metadata = file.metadata().unwrap();
    format!(
        "{:08X}-{:08X}{:08X}:{}:{}:{}",
        information[7],
        information[11],
        information[12],
        metadata.len(),
        metadata.creation_time(),
        metadata.last_write_time()
    )
}
impl Guard {
    pub fn acquire(path: &Path) -> Self {
        let canonical = path.canonicalize().unwrap();
        let mut parents: Vec<_> = path.ancestors().skip(1).collect();
        parents.reverse();
        let mut directories = vec![];
        for directory in parents {
            let file = OpenOptions::new()
                .read(true)
                .share_mode(3)
                .custom_flags(0x02200000)
                .open(directory)
                .unwrap();
            let metadata = file.metadata().unwrap();
            assert!(metadata.is_dir());
            assert_eq!(metadata.file_attributes() & 0x400, 0);
            directories.push(file);
        }
        let mut file = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .custom_flags(0x00200000)
            .open(path)
            .unwrap();
        assert!(file.metadata().unwrap().is_file());
        assert_eq!(file.metadata().unwrap().file_attributes() & 0x400, 0);
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
        let sha256 = format!("{:x}", hasher.finalize());
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
