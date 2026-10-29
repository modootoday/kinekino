use super::{changed, io};
use crate::staged_files::immutable_storage_paths;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad_executor::application::WorkerCancellation;
use std::{
    cell::RefCell,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub struct GuardedCancellation {
    _directories: Vec<File>,
    path: PathBuf,
    observed: RefCell<Option<File>>,
}
impl GuardedCancellation {
    pub fn open(original_root: &Path, signal: &Path) -> Result<Self, ApiError> {
        if signal != original_root.join("cancel.signal") {
            return Err(changed("Cancellation path differs from original root"));
        }
        let (_, directories) = immutable_storage_paths::root(original_root)?;
        Ok(Self {
            _directories: directories,
            path: signal.to_owned(),
            observed: RefCell::new(None),
        })
    }
}
impl WorkerCancellation for GuardedCancellation {
    fn requested(&self) -> Result<bool, ApiError> {
        if let Some(file) = self.observed.borrow().as_ref() {
            file.metadata().map_err(io)?;
            return Ok(true);
        }
        let mut file = match immutable_storage_paths::file(&self.path) {
            Ok(file) => file,
            Err(error) if error.code == ErrorCode::NotFound => return Ok(false),
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(8)
            .read_to_end(&mut bytes)
            .map_err(io)?;
        if bytes != b"cancel\n" {
            return Err(changed("Original cancellation marker is malformed"));
        }
        *self.observed.borrow_mut() = Some(file);
        Ok(true)
    }
}
