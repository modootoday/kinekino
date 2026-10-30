use super::{ApiError, ProposalFiles, WriterReservation, incomplete};
use crate::{
    domain::identifiers::valid_id,
    infrastructure::{immutable_storage_paths as rooted, proposals::io_error},
};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

pub(super) struct Directory {
    pub path: PathBuf,
    _guards: Vec<File>,
}
fn directory(path: &Path, create: bool) -> Result<Option<File>, ApiError> {
    match fs::symlink_metadata(path) {
        Ok(_) => rooted::directory(path).map(Some),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if !create {
                return Ok(None);
            }
            fs::create_dir(path).map_err(io_error)?;
            rooted::directory(path).map(Some)
        }
        Err(error) => Err(io_error(error)),
    }
}
pub(super) fn history(
    store: &ProposalFiles,
    writer: &WriterReservation<'_>,
    id: &str,
    create: bool,
) -> Result<Option<Directory>, ApiError> {
    writer.check(store)?;
    store.identity(id)?;
    let root = store.path.join("output-decisions");
    let Some(root_guard) = directory(&root, create)? else {
        return Ok(None);
    };
    let path = root.join(format!("p-{id}"));
    let Some(proposal_guard) = directory(&path, create)? else {
        return Ok(None);
    };
    Ok(Some(Directory {
        path,
        _guards: vec![root_guard, proposal_guard],
    }))
}
pub(super) fn decision_id(path: &Path) -> Result<String, ApiError> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(incomplete)?;
    let id = name
        .strip_prefix("d-")
        .filter(|id| valid_id(id))
        .ok_or_else(incomplete)?;
    Ok(id.into())
}
pub(super) fn decision_path(directory: &Directory, id: &str) -> Result<PathBuf, ApiError> {
    if !valid_id(id) {
        return Err(incomplete());
    }
    Ok(directory.path.join(format!("d-{id}")))
}
