use crate::infrastructure::immutable_storage_paths as paths;
mod guarded;
mod list;
mod origin;
pub mod output_review_ids;
mod output_reviews;
mod publication;
mod reservation;
mod state_read;
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        proposals::{ProposalStore, StoredProposal},
    },
    domain::model_review::valid_id,
    infrastructure::content_digest,
};
use std::{
    fs::{self, File, OpenOptions},
    io::{Cursor, Write},
    path::{Path, PathBuf},
};
pub struct ProposalFiles {
    path: PathBuf,
    _guards: Vec<File>,
    origin: std::sync::OnceLock<origin::OriginSnapshot>,
    historical:
        Option<std::sync::Arc<dyn crate::application::ai_provenance::HistoricalEvidenceReader>>,
    record_clock:
        std::sync::Arc<dyn crate::application::proposals::output_review::OutputReviewClock>,
}
const MAX_STATE: u64 = 4_194_304;
pub fn open_configured(
    root: &Path,
    historical: Option<&crate::application::launch::HistoricalVerifierConfig>,
) -> Result<ProposalFiles, ApiError> {
    let store = ProposalFiles::open(root)?;
    match historical {
        Some(config) => Ok(store.with_historical_reader(std::sync::Arc::new(
            crate::infrastructure::historical_evidence::HistoricalVerifier::open(
                &config.executable,
                config.executable_sha256.clone(),
                &config.workspace,
                config.package.clone(),
            )?,
        ))),
        None => Ok(store),
    }
}
fn io_error(e: std::io::Error) -> ApiError {
    ApiError::new(
        match e.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::NotFound,
            std::io::ErrorKind::AlreadyExists => ErrorCode::RevisionConflict,
            std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            _ => ErrorCode::EngineFailed,
        },
        format!("Proposal storage I/O: {e}"),
    )
}
fn corrupt() -> ApiError {
    let mut e = ApiError::new(
        ErrorCode::NeedsInput,
        "Proposal publication is incomplete or corrupt; preserve it and recover manually",
    );
    e.remediation = "recover_proposal_publication".into();
    e
}
impl ProposalFiles {
    pub fn open(root: &Path) -> Result<Self, ApiError> {
        let (path, guards) = paths::root(root)?;
        Ok(Self {
            path,
            _guards: guards,
            origin: std::sync::OnceLock::new(),
            historical: None,
            record_clock: std::sync::Arc::new(output_reviews::clock::SystemRecordClock),
        })
    }
    pub fn with_historical_reader(
        mut self,
        reader: std::sync::Arc<dyn crate::application::ai_provenance::HistoricalEvidenceReader>,
    ) -> Self {
        self.historical = Some(reader);
        self
    }
    pub fn with_output_review_clock(
        mut self,
        clock: std::sync::Arc<dyn crate::application::proposals::output_review::OutputReviewClock>,
    ) -> Self {
        self.record_clock = clock;
        self
    }
    fn identity(&self, id: &str) -> Result<PathBuf, ApiError> {
        if !valid_id(id) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid proposal identity",
            ));
        }
        Ok(self.path.join(format!("p-{id}")))
    }
}
impl ProposalStore for ProposalFiles {
    fn source_context_available(&self) -> bool {
        self.historical.is_some()
    }
    fn inspect_source_context(
        &self,
        id: &str,
    ) -> Result<crate::application::proposals::source_context::SourceContextInspection, ApiError>
    {
        self.identity(id)?;
        let reader = self
            .historical_reader()
            .ok_or_else(crate::application::proposals::source_context::unavailable)?;
        let writer = reservation::WriterReservation::acquire(self)?;
        let state = self.read_current_locked(&writer, id)?;
        let context = crate::application::ai_provenance::inspect_source_context(reader, &state)?;
        writer.check(self)?;
        let after = self.read_current_locked(&writer, id)?;
        if crate::domain::projects::canonical_sha256(&state).map_err(|_| corrupt())?
            != crate::domain::projects::canonical_sha256(&after).map_err(|_| corrupt())?
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Proposal changed during source context inspection",
            ));
        }
        crate::application::proposals::source_context::SourceContextInspection::new(state, context)
    }
    fn list_available(&self) -> bool {
        true
    }
    fn list(
        &self,
        input: &crate::application::proposals::list::ListProposals,
    ) -> Result<crate::application::proposals::list::ProposalListPage, ApiError> {
        list::read(self, input)
    }
    fn capture_mutation_available(&self) -> bool {
        true
    }
    fn open_capture_mutation(
        &self,
        id: &str,
    ) -> Result<
        Option<Box<dyn crate::application::proposals::guarded::CaptureMutationSession + '_>>,
        ApiError,
    > {
        guarded::open(self, id).map(Some)
    }
    fn capture_current(
        &self,
        id: &str,
        expected_revision: u64,
    ) -> Result<crate::application::proposals::CapturedProposal, ApiError> {
        self.identity(id)?;
        let writer = reservation::WriterReservation::acquire(self)?;
        let state = self.read_current_locked(&writer, id)?;
        crate::application::ai_provenance::verify_bound(self.historical_reader(), &state)?;
        if state.document.draft_revision.0 != expected_revision {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Live proposal revision changed",
            ));
        }
        let origin = origin::capture_origin(self, &writer)?;
        Ok(crate::application::proposals::CapturedProposal {
            state,
            origin: Some(origin),
        })
    }
    fn output_review_available(&self) -> bool {
        true
    }
    fn historical_reader(
        &self,
    ) -> Option<&dyn crate::application::ai_provenance::HistoricalEvidenceReader> {
        self.historical.as_deref()
    }
    fn load(&self, id: &str) -> Result<StoredProposal, ApiError> {
        self.identity(id)?;
        let writer = reservation::WriterReservation::acquire(self)?;
        let state = self.read_current_locked(&writer, id)?;
        crate::application::ai_provenance::verify_bound(self.historical_reader(), &state)?;
        Ok(state)
    }
    fn publish(&self, expected: Option<u64>, state: &StoredProposal) -> Result<(), ApiError> {
        let prepared = self.prepare_publication(state)?;
        let writer = reservation::WriterReservation::acquire(self)?;
        self.publish_prepared_locked(&writer, expected, state, prepared)
    }
    fn open_output_review(
        &self,
        id: &str,
    ) -> Result<
        Option<Box<dyn crate::application::proposals::output_review::OutputReviewSession + '_>>,
        ApiError,
    > {
        output_reviews::open(self, id).map(Some)
    }
    fn output_decisions(
        &self,
        id: &str,
    ) -> Result<crate::application::proposals::output_review::OutputDecisionHistory, ApiError> {
        output_reviews::history(self, id)
    }
}

fn immutable_equal(previous: &StoredProposal, next: &StoredProposal) -> bool {
    previous.historical == next.historical
        && serde_json::to_vec(&previous.document.source).ok()
            == serde_json::to_vec(&next.document.source).ok()
        && serde_json::to_vec(&previous.document.generated).ok()
            == serde_json::to_vec(&next.document.generated).ok()
}
