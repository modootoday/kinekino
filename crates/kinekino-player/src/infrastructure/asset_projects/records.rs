use super::*;
use crate::application::asset_projects::AssetPublication;
pub(super) struct RetainedAssetHistory {
    pub project: StoredAssetProject,
    _guards: Vec<File>,
}
fn pending(name: &str) -> bool {
    let Some(parts) = name
        .strip_prefix(".pending-")
        .and_then(|name| name.strip_suffix(".json"))
    else {
        return false;
    };
    let Some((pid, sequence)) = parts.split_once('-') else {
        return false;
    };
    pid.parse::<u32>()
        .ok()
        .is_some_and(|value| value > 0 && value.to_string() == pid)
        && sequence
            .parse::<u64>()
            .ok()
            .is_some_and(|value| value.to_string() == sequence)
}
impl AssetProjectFiles {
    pub(super) fn scan(
        &self,
        id: &str,
        directory: &Path,
        requested: Option<DecimalU64>,
    ) -> Result<StoredAssetProject, ApiError> {
        Ok(self.scan_retained(id, directory, requested)?.project)
    }
    pub(super) fn scan_retained(
        &self,
        id: &str,
        directory: &Path,
        requested: Option<DecimalU64>,
    ) -> Result<RetainedAssetHistory, ApiError> {
        let mut guards = vec![paths::directory(directory)?];
        let mut records = std::collections::BTreeMap::new();
        let mut pending_count = 0_usize;
        for entry in fs::read_dir(directory).map_err(io)? {
            let entry = entry.map_err(io)?;
            let name = entry.file_name();
            let name = name.to_str().ok_or_else(corrupt)?;
            if pending(name) {
                let file = paths::file(&entry.path())?;
                if file.metadata().map_err(io)?.len() > (MAX_ASSET_DOCUMENT_BYTES + 4096) as u64 {
                    return Err(corrupt());
                }
                pending_count += 1;
                if pending_count > MAX_PENDING {
                    return Err(corrupt());
                }
                guards.push(file);
                continue;
            }
            let (revision, sha) = name
                .strip_prefix("r-")
                .and_then(|name| name.strip_suffix(".json"))
                .and_then(|name| name.split_once('-'))
                .ok_or_else(corrupt)?;
            let revision = revision
                .parse::<u64>()
                .ok()
                .filter(|value| *value > 0)
                .ok_or_else(corrupt)?;
            if name != format!("r-{revision}-{sha}.json")
                || !valid_sha256(sha)
                || records
                    .insert(revision, (entry.path(), sha.to_string()))
                    .is_some()
                || records.len() > MAX_PUBLICATIONS
            {
                return Err(corrupt());
            }
        }
        if records.is_empty() {
            return Err(corrupt());
        }
        if records
            .keys()
            .enumerate()
            .any(|(index, revision)| *revision != index as u64 + 1)
        {
            return Err(corrupt());
        }
        let latest = DecimalU64(records.len() as u64);
        let mut previous = None;
        let mut selected = None;
        let mut total = 0_u64;
        for (revision, (path, sha)) in records {
            let mut file = paths::file(&path)?;
            let size = file.metadata().map_err(io)?.len();
            total = total
                .checked_add(size)
                .filter(|total| *total <= MAX_HISTORY_BYTES)
                .ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::ResourceExhausted,
                        "Asset verified history byte budget exceeded",
                    )
                })?;
            if size > (MAX_ASSET_DOCUMENT_BYTES + 4096) as u64 {
                return Err(corrupt());
            }
            let mut bytes = Vec::new();
            Read::by_ref(&mut file)
                .take((MAX_ASSET_DOCUMENT_BYTES + 4096 + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(io)?;
            let publication: AssetPublication =
                serde_json::from_slice(&bytes).map_err(|_| corrupt())?;
            let expected =
                asset_projects::prepare_publication(previous.as_ref(), &publication.document)
                    .map_err(|_| corrupt())?;
            if publication.document.document_id != id
                || publication.publication_revision.0 != revision
                || serde_json::to_vec(&expected).map_err(|_| corrupt())? != bytes
                || asset_projects::publication_sha256(&publication)? != sha
            {
                return Err(corrupt());
            }
            let current = StoredAssetProject {
                publication_sha256: sha,
                latest_publication_revision: publication.publication_revision,
                pending_publication_count: DecimalU64(pending_count as u64),
                history_bytes: DecimalU64(total),
                publication,
            };
            if requested.unwrap_or(latest).0 == revision {
                selected = Some(current.clone());
            }
            previous = Some(current);
            guards.push(file);
        }
        let mut selected = selected.ok_or_else(|| {
            ApiError::new(ErrorCode::NotFound, "Asset publication revision not found")
        })?;
        selected.latest_publication_revision = latest;
        selected.history_bytes = DecimalU64(total);
        Ok(RetainedAssetHistory {
            project: selected,
            _guards: guards,
        })
    }
}
