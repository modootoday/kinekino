use super::{ApiError, ErrorCode, File, Path, ProposalFiles, paths};

pub(super) struct WriterReservation<'a> {
    root: &'a Path,
    _writer: File,
}
impl<'a> WriterReservation<'a> {
    pub(super) fn acquire(store: &'a ProposalFiles) -> Result<Self, ApiError> {
        Ok(Self {
            root: &store.path,
            _writer: paths::lock(&store.path.join("writer.lock"))?,
        })
    }
    pub(super) fn check(&self, store: &ProposalFiles) -> Result<(), ApiError> {
        if self.root != store.path.as_path() {
            return Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Proposal writer reservation belongs to another root",
            ));
        }
        Ok(())
    }
}
