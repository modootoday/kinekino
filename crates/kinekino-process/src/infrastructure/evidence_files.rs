use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::infrastructure::os_policy::FileIdentity;
/// Handle identity through the platform port (volume+index / dev+inode); never path-inferred.
fn identity(file: &File) -> io::Result<FileIdentity> {
    crate::infrastructure::os_policy::file_identity(file)
}
#[derive(Clone, Debug)]
pub struct RawFile {
    path: PathBuf,
    identity: FileIdentity,
}
impl RawFile {
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Opens independently: never DuplicateHandle/try_clone a child's file cursor.
    pub fn open_read(&self) -> io::Result<File> {
        let file = File::open(&self.path)?;
        if identity(&file)? != self.identity {
            return Err(io::Error::other("raw evidence file identity changed"));
        }
        Ok(file)
    }
}

#[derive(Clone, Debug)]
pub struct EvidencePaths {
    pub stdout: RawFile,
    pub stderr: RawFile,
    pub input: Option<PathBuf>,
}
impl EvidencePaths {
    pub fn check_limit(&self, maximum: u64) -> io::Result<()> {
        if maximum == 0 {
            return Err(io::Error::other("positive raw-output bound required"));
        }
        for path in [&self.stdout, &self.stderr] {
            if path.open_read()?.metadata()?.len() > maximum {
                return Err(io::Error::other("raw-output limit exceeded"));
            }
        }
        Ok(())
    }
}
pub struct PreparedEvidence {
    pub paths: EvidencePaths,
    pub(crate) stdout: File,
    pub(crate) stderr: File,
    pub(crate) input: Option<File>,
}
impl PreparedEvidence {
    /// Consume an independently opened regular reader as the existing child stdin.
    /// Caller retains its source identity/write-delete guards until physical close.
    /// This resets offset to zero. No other clone may read/seek the shared cursor.
    /// No media bytes or invented stdin.raw path are copied into evidence.
    pub fn with_input_file(mut self, input: File) -> io::Result<Self> {
        if self.input.is_some() || self.paths.input.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Evidence input is already bound",
            ));
        }
        // The OS adapter returns a private duplicate without write/delete rights
        // (ADR 0006); the original caller-owned reader is closed here.
        self.input = Some(super::os_policy::process().read_only_duplicate(&input)?);
        Ok(self)
    }

    /// Caller owns the directory. Existing evidence is never overwritten or removed on failure.
    pub fn create(root: &Path, input: Option<&[u8]>, input_limit: usize) -> io::Result<Self> {
        if !root.is_absolute() || !root.is_dir() {
            return Err(io::Error::other(
                "existing absolute evidence directory required",
            ));
        }
        if input.is_some_and(|bytes| bytes.len() > input_limit) {
            return Err(io::Error::other("request input exceeds bound"));
        }
        let stdout_path = root.join("stdout.raw");
        let stderr_path = root.join("stderr.raw");
        let stdout = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&stdout_path)?;
        let stderr = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&stderr_path)?;
        let (input_path, input) = match input {
            Some(bytes) => {
                let path = root.join("stdin.raw");
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)?;
                file.write_all(bytes)?;
                file.sync_all()?;
                (Some(path.clone()), Some(File::open(path)?))
            }
            None => (None, None),
        };
        let paths = EvidencePaths {
            stdout: RawFile {
                path: stdout_path,
                identity: identity(&stdout)?,
            },
            stderr: RawFile {
                path: stderr_path,
                identity: identity(&stderr)?,
            },
            input: input_path,
        };
        Ok(Self {
            paths,
            stdout,
            stderr,
            input,
        })
    }
}

#[cfg(test)]
mod input_file_tests {
    use super::*;
    use std::{
        io::{Read, Seek, SeekFrom},
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "owned-input-file-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&root).unwrap();
            std::fs::create_dir(root.join("evidence")).unwrap();
            std::fs::write(root.join("source.bin"), b"original-reader").unwrap();
            Self(root)
        }
        fn evidence(&self, bytes: Option<&[u8]>) -> PreparedEvidence {
            PreparedEvidence::create(&self.0.join("evidence"), bytes, 1024).unwrap()
        }
        fn reader(&self) -> File {
            File::open(self.0.join("source.bin")).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn original_reader_starts_at_zero_and_child_capability_is_read_only() {
        let fixture = Fixture::new();
        let mut reader = fixture.reader();
        reader.seek(SeekFrom::Start(4)).unwrap();
        let mut prepared = fixture.evidence(None).with_input_file(reader).unwrap();
        assert!(prepared.paths.input.is_none());
        assert!(!fixture.0.join("evidence/stdin.raw").exists());
        let input = prepared.input.as_mut().unwrap();
        let mut bytes = Vec::new();
        input.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"original-reader");
        assert!(input.write_all(b"cannot-write").is_err());
        assert_eq!(std::fs::read(fixture.0.join("source.bin")).unwrap(), bytes);
    }
    #[test]
    fn bound_bytes_cannot_be_replaced_by_a_new_reader() {
        let fixture = Fixture::new();
        let prepared = fixture.evidence(Some(b"original-wire"));
        let error = prepared.with_input_file(fixture.reader()).err().unwrap();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(
            std::fs::read(fixture.0.join("evidence/stdin.raw")).unwrap(),
            b"original-wire"
        );
    }
    #[test]
    fn write_only_file_is_rejected_before_input_admission() {
        let fixture = Fixture::new();
        let writer = OpenOptions::new()
            .write(true)
            .open(fixture.0.join("source.bin"))
            .unwrap();
        assert!(fixture.evidence(None).with_input_file(writer).is_err());
    }
}
