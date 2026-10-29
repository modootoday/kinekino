use super::super::RootedJobPaths;
use super::{digest, invalid};
use crate::{
    application::ExecutionControl,
    domain::{MAX_SOURCE_BYTES, VerifiedJobOutput},
};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_subtitles::application::storage::SubtitleStorage;
use std::{
    cell::RefCell,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub(super) struct SelectedFile {
    pub(super) path: PathBuf,
    pub(super) file: File,
    _guards: Vec<File>,
}
pub(super) fn selected_path(
    paths: &dyn RootedJobPaths,
    raw: &str,
) -> Result<(PathBuf, Vec<File>), ApiError> {
    let path = Path::new(raw);
    if !path.is_absolute() || raw.len() > 32768 || raw.chars().any(char::is_control) {
        return Err(invalid("Job files require absolute bounded local paths"));
    }
    let name = path
        .file_name()
        .ok_or_else(|| invalid("Job path must name a file"))?;
    let (parent, guards) = paths.root(
        path.parent()
            .ok_or_else(|| invalid("Missing job file parent"))?,
    )?;
    Ok((parent.join(name), guards))
}
pub(super) fn open_source(paths: &dyn RootedJobPaths, raw: &str) -> Result<SelectedFile, ApiError> {
    let (path, guards) = selected_path(paths, raw)?;
    let file = paths.file(&path)?;
    Ok(SelectedFile {
        path,
        file,
        _guards: guards,
    })
}
pub(super) fn read_source(file: &mut File) -> Result<String, ApiError> {
    let mut bytes = Vec::new();
    Read::by_ref(file)
        .take(MAX_SOURCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(kinekino_core::file_publication::io_error)?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Subtitle job source exceeds8MiB",
        ));
    }
    if bytes.is_empty() {
        return Err(invalid("Job source must contain at most8MiB UTF8 text"));
    }
    String::from_utf8(bytes).map_err(|_| invalid("Job source must use UTF8"))
}
pub(super) struct SnapshotStorage<'a> {
    pub(super) source_path: &'a str,
    pub(super) output_path: &'a str,
    pub(super) snapshot: &'a str,
    pub(super) control: &'a dyn ExecutionControl,
    pub(super) paths: &'a dyn RootedJobPaths,
    pub(super) verified: RefCell<Option<VerifiedJobOutput>>,
}
impl SubtitleStorage for SnapshotStorage<'_> {
    fn read_utf8(&self, path: &str) -> Result<String, ApiError> {
        if path != self.source_path {
            return Err(invalid("Converter requested a different source"));
        }
        Ok(self.snapshot.into())
    }
    fn publish_new(&self, path: &str, contents: &str) -> Result<(), ApiError> {
        if path != self.output_path {
            return Err(invalid("Converter requested a different output"));
        }
        self.control.reserve_publication()?;
        kinekino_core::file_publication::publish_before(
            path,
            contents.as_bytes(),
            Some(self.control.deadline()),
        )?;
        self.control.begin_verification()?;
        let mut file = self.paths.file(Path::new(path))?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(contents.len() as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(kinekino_core::file_publication::io_error)?;
        let sha = digest(contents.as_bytes());
        if bytes.len() != contents.len() || digest(&bytes) != sha {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Published subtitle failed byte verification; preserve the file",
            ));
        }
        *self.verified.borrow_mut() = Some(VerifiedJobOutput {
            path: path.into(),
            sha256: sha,
            bytes: DecimalU64(bytes.len() as u64),
            receipt: serde_json::json!({}),
        });
        Ok(())
    }
}
