use super::{
    ApiError, Cursor, MAX_STATE, Path, ProposalFiles, StoredProposal, content_digest, corrupt, fs,
    immutable_equal, io_error, paths, reservation::WriterReservation,
};
use crate::domain::identifiers::valid_sha256;
use std::io::Read;

/// Only the optional list path opts into aggregate accounting. Legacy reads keep
/// their original contract and do not inherit the list's smaller store budget.
pub(super) struct ReadBudget {
    pub revision_entries_left: usize,
    pub per_proposal_revisions: usize,
    pub state_bytes_left: u64,
}

impl ProposalFiles {
    pub(super) fn revisions(&self, dir: &Path) -> Result<Vec<u64>, ApiError> {
        self.revisions_with_budget(dir, None)
    }
    fn revisions_with_budget(
        &self,
        dir: &Path,
        mut budget: Option<&mut ReadBudget>,
    ) -> Result<Vec<u64>, ApiError> {
        let _guard = paths::directory(dir)?;
        let mut revisions = Vec::new();
        for entry in fs::read_dir(dir).map_err(io_error)? {
            if let Some(b) = budget.as_mut() {
                if revisions.len() >= b.per_proposal_revisions || b.revision_entries_left == 0 {
                    return Err(crate::application::proposals::list::exhausted());
                }
                b.revision_entries_left -= 1;
            }
            let entry = entry.map_err(io_error)?;
            let name = entry.file_name();
            let name = name.to_str().ok_or_else(corrupt)?;
            let n = name
                .strip_prefix("r-")
                .and_then(|s| s.parse::<u64>().ok())
                .filter(|n| *n > 0)
                .ok_or_else(corrupt)?;
            if name != format!("r-{n}") {
                return Err(corrupt());
            }
            revisions.push(n);
        }
        revisions.sort_unstable();
        if revisions
            .iter()
            .enumerate()
            .any(|(i, n)| *n != i as u64 + 1)
        {
            return Err(corrupt());
        }
        Ok(revisions)
    }
    pub(super) fn read(&self, id: &str, dir: &Path, n: u64) -> Result<StoredProposal, ApiError> {
        self.read_with_budget(id, dir, n, None)
            .map(|(state, _)| state)
    }
    fn read_with_budget(
        &self,
        id: &str,
        dir: &Path,
        n: u64,
        mut budget: Option<&mut ReadBudget>,
    ) -> Result<(StoredProposal, String), ApiError> {
        let revision = dir.join(format!("r-{n}"));
        let _guard = paths::directory(&revision)?;
        let entries = fs::read_dir(&revision).map_err(io_error)?;
        let files = if budget.is_some() {
            entries.take(2).collect::<Result<Vec<_>, _>>()
        } else {
            entries.collect::<Result<Vec<_>, _>>()
        }
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
        let maximum = budget
            .as_ref()
            .map_or(MAX_STATE, |b| b.state_bytes_left.min(MAX_STATE));
        Read::by_ref(&mut file)
            .take(maximum + 1)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        if let Some(b) = budget.as_mut() {
            if bytes.len() as u64 > maximum {
                return Err(crate::application::proposals::list::exhausted());
            }
            b.state_bytes_left -= bytes.len() as u64;
        }
        if content_digest::sha256(&mut Cursor::new(&bytes), MAX_STATE)?.1 != sha {
            return Err(corrupt());
        }
        let state: StoredProposal = serde_json::from_slice(&bytes).map_err(|_| corrupt())?;
        state.validate().map_err(|_| corrupt())?;
        if state.document.proposal_id != id || state.document.draft_revision.0 != n {
            return Err(corrupt());
        }
        state.document.source.validate().map_err(|_| corrupt())?;
        if let Some(draft) = &state.document.draft {
            draft
                .validate(&state.document.source, state.document.generated.task)
                .map_err(|_| corrupt())?;
        }
        Ok((state, sha.to_owned()))
    }
    pub(super) fn read_current_locked(
        &self,
        writer: &WriterReservation<'_>,
        id: &str,
    ) -> Result<StoredProposal, ApiError> {
        writer.check(self)?;
        let dir = self.identity(id)?;
        let revisions = self.revisions(&dir)?;
        let latest = revisions.last().copied().ok_or_else(corrupt)?;
        let state = self.read(id, &dir, latest)?;
        if latest > 1 && !immutable_equal(&self.read(id, &dir, 1)?, &state) {
            return Err(corrupt());
        }
        Ok(state)
    }
    pub(super) fn read_current_bounded(
        &self,
        writer: &WriterReservation<'_>,
        id: &str,
        budget: &mut ReadBudget,
    ) -> Result<(StoredProposal, String), ApiError> {
        writer.check(self)?;
        let dir = self.identity(id)?;
        let revisions = self.revisions_with_budget(&dir, Some(&mut *budget))?;
        let latest = revisions.last().copied().ok_or_else(corrupt)?;
        let (state, sha) = self.read_with_budget(id, &dir, latest, Some(&mut *budget))?;
        if latest > 1
            && !immutable_equal(
                &self.read_with_budget(id, &dir, 1, Some(&mut *budget))?.0,
                &state,
            )
        {
            return Err(corrupt());
        }
        Ok((state, sha))
    }
}
