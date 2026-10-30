use super::*;
use crate::{application::card_projects::CardPublication, domain::identifiers::valid_sha256};
use std::io::Read;

impl CardProjectFiles {
    pub(super) fn scan(
        &self,
        id: &str,
        dir: &Path,
        requested: Option<DecimalU64>,
    ) -> Result<StoredCardProject, ApiError> {
        let _guard = paths::directory(dir)?;
        let mut numbers = Vec::new();
        for entry in fs::read_dir(dir).map_err(io_error)? {
            let name = entry.map_err(io_error)?.file_name();
            let name = name.to_str().ok_or_else(corrupt)?;
            let n = name
                .strip_prefix("r-")
                .and_then(|s| s.parse::<u64>().ok())
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
        let latest = DecimalU64(numbers.len() as u64);
        let mut previous: Option<StoredCardProject> = None;
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
                .and_then(|s| s.to_str())
                .filter(|s| valid_sha256(s))
                .ok_or_else(corrupt)?;
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                return Err(corrupt());
            }
            let mut file = paths::file(&path)?;
            let mut bytes = Vec::new();
            Read::by_ref(&mut file)
                .take((MAX_DOCUMENT_BYTES + 4096) as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(io_error)?;
            if bytes.len() > MAX_DOCUMENT_BYTES + 4096 {
                return Err(corrupt());
            }
            let publication: CardPublication =
                serde_json::from_slice(&bytes).map_err(|_| corrupt())?;
            if publication.document.document_id != id {
                return Err(corrupt());
            }
            let expected =
                card_projects::prepare_publication(previous.as_ref(), &publication.document)
                    .map_err(|_| corrupt())?;
            if publication.publication_revision.0 != n
                || serde_json::to_vec(&expected).map_err(|_| corrupt())? != bytes
                || card_projects::publication_sha256(&publication).map_err(|_| corrupt())? != sha
            {
                return Err(corrupt());
            }
            let current = StoredCardProject {
                publication_sha256: sha.into(),
                latest_publication_revision: latest,
                publication,
            };
            if n == requested.unwrap_or(latest).0 {
                selected = Some(current.clone());
            }
            previous = Some(current);
        }
        selected.ok_or_else(|| {
            ApiError::new(ErrorCode::NotFound, "Card publication revision not found")
        })
    }
}
