use super::*;
use crate::{application::clip_projects::ClipPublication, domain::identifiers::valid_sha256};
use std::io::Read;
pub(super) fn scan(
    directory: &Path,
    id: &str,
    requested: Option<DecimalU64>,
) -> Result<StoredClipProject, ApiError> {
    let mut events = Vec::new();
    for (index, entry) in fs::read_dir(directory)
        .map_err(kinekino_core::file_publication::io_error)?
        .enumerate()
    {
        if index >= MAX_ENTRIES {
            return Err(corrupt());
        }
        let entry = entry.map_err(kinekino_core::file_publication::io_error)?;
        let name = entry.file_name().into_string().map_err(|_| corrupt())?;
        if name.starts_with(".pending-") {
            let _guard = paths::file(&entry.path())?;
            continue;
        }
        let (number, sha) = name
            .strip_prefix("r-")
            .and_then(|s| s.strip_suffix(".json"))
            .and_then(|s| s.split_once('-'))
            .ok_or_else(corrupt)?;
        let revision = number.parse::<u64>().map_err(|_| corrupt())?;
        if revision == 0 || number != format!("{revision:020}") || !valid_sha256(sha) {
            return Err(corrupt());
        }
        events.push((revision, sha.to_owned(), entry.path()));
    }
    events.sort_by_key(|entry| entry.0);
    if events.is_empty() {
        return Err(corrupt());
    }
    let latest = DecimalU64(events.len() as u64);
    let mut previous: Option<StoredClipProject> = None;
    let mut selected = None;
    for (index, (revision, sha, path)) in events.into_iter().enumerate() {
        if revision != index as u64 + 1 {
            return Err(corrupt());
        }
        let mut file = paths::file(&path)?;
        let mut bytes = Vec::new();
        file.by_ref()
            .take(MAX_RECORD_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(kinekino_core::file_publication::io_error)?;
        if bytes.len() > MAX_RECORD_BYTES {
            return Err(corrupt());
        }
        let publication: ClipPublication = serde_json::from_slice(&bytes).map_err(|_| corrupt())?;
        if publication.document.document_id != id {
            return Err(corrupt());
        }
        let expected = clip_projects::prepare_publication(previous.as_ref(), &publication.document)
            .map_err(|_| corrupt())?;
        if publication.publication_revision.0 != revision
            || serde_json::to_vec(&expected).map_err(|_| corrupt())? != bytes
            || clip_projects::publication_sha256(&publication).map_err(|_| corrupt())? != sha
        {
            return Err(corrupt());
        }
        let current = StoredClipProject {
            publication_sha256: sha,
            latest_publication_revision: DecimalU64(revision),
            publication,
        };
        if revision == requested.unwrap_or(latest).0 {
            selected = Some(current.clone());
        }
        previous = Some(current);
    }
    let mut selected = selected
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Clip publication revision not found"))?;
    selected.latest_publication_revision = latest;
    Ok(selected)
}
