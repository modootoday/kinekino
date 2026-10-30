use super::*;
use crate::infrastructure::os_policy::{OpenOptionsPolicy, ShareMode};

pub(super) struct PreparedPublication {
    bytes: Vec<u8>,
    sha: String,
    dir: PathBuf,
}

impl ProposalFiles {
    pub(super) fn prepare_publication(
        &self,
        state: &StoredProposal,
    ) -> Result<PreparedPublication, ApiError> {
        state
            .validate()
            .map_err(|m| ApiError::new(ErrorCode::InvalidArgument, m))?;
        crate::application::ai_provenance::verify_bound(self.historical_reader(), state)?;
        let bytes = serde_json::to_vec(state).map_err(|_| corrupt())?;
        let sha = content_digest::sha256(&mut Cursor::new(&bytes), MAX_STATE)?.1;
        let dir = self.identity(&state.document.proposal_id)?;
        Ok(PreparedPublication { bytes, sha, dir })
    }

    pub(super) fn publish_locked(
        &self,
        writer: &reservation::WriterReservation<'_>,
        expected: Option<u64>,
        state: &StoredProposal,
    ) -> Result<(), ApiError> {
        writer.check(self)?;
        let prepared = self.prepare_publication(state)?;
        self.publish_prepared_locked(writer, expected, state, prepared)
    }

    pub(super) fn publish_prepared_locked(
        &self,
        writer: &reservation::WriterReservation<'_>,
        expected: Option<u64>,
        state: &StoredProposal,
        prepared: PreparedPublication,
    ) -> Result<(), ApiError> {
        writer.check(self)?;
        let PreparedPublication { bytes, sha, dir } = prepared;
        let document = &state.document;
        let revision = match expected {
            None => {
                fs::create_dir(&dir).map_err(io_error)?;
                1
            }
            Some(expected) => {
                if self.revisions(&dir)?.last().copied() != Some(expected) {
                    return Err(ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Proposal revision changed",
                    ));
                }
                let previous = self.read_current_locked(writer, &document.proposal_id)?;
                if !immutable_equal(&previous, state) {
                    return Err(ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Immutable proposal source, generation or historical binding changed",
                    ));
                }
                expected.checked_add(1).ok_or_else(corrupt)?
            }
        };
        if document.draft_revision.0 != revision {
            return Err(corrupt());
        }
        let _guard = paths::directory(&dir)?;
        let revision = dir.join(format!("r-{revision}"));
        fs::create_dir(&revision).map_err(io_error)?;
        let _revision_guard = paths::directory(&revision)?;
        // An incomplete publication remains visible and blocks reads for recovery.
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .share(ShareMode::ReadOnly)
            .open(revision.join(format!("{sha}.json")))
            .map_err(io_error)?;
        file.write_all(&bytes).map_err(io_error)?;
        file.sync_all().map_err(io_error)?;
        Ok(())
    }
}
