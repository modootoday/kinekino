use super::*;
use crate::domain::identifiers::valid_sha256;
use std::io::Read;
impl ProjectFiles {
    pub(super) fn scan(
        &self,
        id: &str,
        dir: &Path,
        requested: Option<u64>,
    ) -> Result<StoredProjectRevision, ApiError> {
        let _guard = paths::directory(dir)?;
        let mut numbers = Vec::new();
        for entry in fs::read_dir(dir).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            let name = entry.file_name();
            let name = name.to_str().ok_or_else(corrupt)?;
            let n = name
                .strip_prefix("r-")
                .and_then(|n| n.parse::<u64>().ok())
                .filter(|n| *n > 0)
                .ok_or_else(corrupt)?;
            if name != format!("r-{n}") {
                return Err(corrupt());
            }
            numbers.push(n);
        }
        numbers.sort_unstable();
        if numbers.is_empty()
            || numbers
                .iter()
                .enumerate()
                .any(|(index, n)| *n != index as u64 + 1)
        {
            return Err(corrupt());
        }
        let latest_revision = DecimalU64(numbers.len() as u64);
        let mut selected = None;
        for n in numbers {
            let directory = dir.join(format!("r-{n}"));
            let _guard = paths::directory(&directory)?;
            let files = fs::read_dir(&directory)
                .map_err(io_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(io_error)?;
            if files.len() != 1 {
                return Err(corrupt());
            }
            let path = files[0].path();
            let sha = path
                .file_stem()
                .and_then(|name| name.to_str())
                .filter(|s| valid_sha256(s))
                .ok_or_else(corrupt)?;
            if path.extension().and_then(|name| name.to_str()) != Some("json") {
                return Err(corrupt());
            }
            let mut file = paths::file(&path)?;
            let mut bytes = Vec::new();
            Read::by_ref(&mut file)
                .take(MAX_RECORD_BYTES as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(io_error)?;
            if bytes.len() > MAX_RECORD_BYTES {
                return Err(corrupt());
            }
            let project: ProjectRevision = serde_json::from_slice(&bytes).map_err(|_| corrupt())?;
            project.validate().map_err(|_| corrupt())?;
            if project.project_id != id
                || project.revision.0 != n
                || canonical_sha256(&project).map_err(|_| corrupt())? != sha
                || serde_json::to_vec(&project).map_err(|_| corrupt())? != bytes
            {
                return Err(corrupt());
            }
            if n == requested.unwrap_or(latest_revision.0) {
                selected = Some(StoredProjectRevision {
                    revision_sha256: sha.into(),
                    latest_revision,
                    project,
                });
            }
        }
        selected.ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Project revision not found"))
    }
}
