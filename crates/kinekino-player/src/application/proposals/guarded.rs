//! Capture authority comes from configured project storage and the owned writer.
mod requests;
use super::{ProposalStore, mutation_error};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        projects::ProjectStore,
    },
    domain::{api_values::DecimalU64, projects::canonical_sha256, proposals::*},
};
pub use requests::{AcceptCapturedProposal, EditCapturedProposal};
use serde::{Deserialize, Serialize};

/// The adapter retains its original writer through every read and publication.
pub trait CaptureMutationSession {
    fn origin(&self) -> &ProposalStoreOrigin;
    fn current(&self) -> &StoredProposal;
    fn revision(&self, revision: u64) -> Result<StoredProposal, ApiError>;
    fn publish(&mut self, expected: u64, state: &StoredProposal) -> Result<(), ApiError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureAnchor {
    pub project_id: String,
    pub project_revision: DecimalU64,
    pub project_revision_sha256: String,
    pub proposal_id: String,
    pub capture_sha256: String,
}

/// Nonsecret CAS expectation, never an origin grant or an implicit refresh.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureExpectation {
    pub draft_revision: DecimalU64,
    pub state_sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureMutationResult {
    pub state: StoredProposal,
    pub continuation: CaptureExpectation,
    pub acceptance: Option<AcceptanceReceipt>,
    pub reused_acceptance: bool,
}

fn conflict() -> ApiError {
    ApiError::new(
        ErrorCode::RevisionConflict,
        "Capture origin, immutable source or expected state changed",
    )
}
fn hash(state: &StoredProposal) -> Result<String, ApiError> {
    canonical_sha256(state)
        .map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Invalid captured state"))
}
fn same_immutable(a: &StoredProposal, b: &StoredProposal) -> Result<bool, ApiError> {
    let encode = |state: &StoredProposal| {
        serde_json::to_vec(&(
            &state.document.source,
            &state.document.generated,
            &state.historical,
        ))
        .map_err(|_| conflict())
    };
    Ok(encode(a)? == encode(b)?)
}
fn expected_state(state: &StoredProposal, expected: &CaptureExpectation) -> Result<(), ApiError> {
    if state.document.draft_revision != expected.draft_revision
        || hash(state)? != expected.state_sha256
    {
        return Err(conflict());
    }
    Ok(())
}

fn open<'a>(
    projects: &dyn ProjectStore,
    proposals: &'a dyn ProposalStore,
    anchor: &CaptureAnchor,
    expected: Option<&CaptureExpectation>,
) -> Result<(Box<dyn CaptureMutationSession + 'a>, CaptureExpectation), ApiError> {
    use crate::domain::identifiers::{valid_id, valid_sha256};
    if !valid_id(&anchor.project_id)
        || !valid_id(&anchor.proposal_id)
        || anchor.project_revision.0 == 0
        || !valid_sha256(&anchor.project_revision_sha256)
        || !valid_sha256(&anchor.capture_sha256)
        || expected.is_some_and(|e| e.draft_revision.0 == 0 || !valid_sha256(&e.state_sha256))
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid capture anchor or expectation",
        ));
    }
    let stored = projects.load(&anchor.project_id, Some(anchor.project_revision.0))?;
    stored.project.validate().map_err(|_| conflict())?;
    if stored.project.project_id != anchor.project_id
        || stored.project.revision != anchor.project_revision
        || stored.revision_sha256 != anchor.project_revision_sha256
        || canonical_sha256(&stored.project).map_err(|_| conflict())?
            != anchor.project_revision_sha256
    {
        return Err(conflict());
    }
    let snapshot = stored
        .project
        .proposals
        .iter()
        .find(|p| {
            p.captured.document.proposal_id == anchor.proposal_id
                && p.capture_sha256 == anchor.capture_sha256
        })
        .ok_or_else(conflict)?;
    let origin = snapshot.proposal_origin.as_ref().ok_or_else(|| {
        let mut error = ApiError::new(
            ErrorCode::NeedsInput,
            "Legacy capture has no verified proposal origin",
        );
        error.remediation = "create_explicit_live_capture".into();
        error
    })?;
    let session = proposals
        .open_capture_mutation(&anchor.proposal_id)?
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::Unsupported,
                "Capture mutation writer session unavailable",
            )
        })?;
    if session.origin() != origin
        || session.current().document.proposal_id != anchor.proposal_id
        || !same_immutable(&snapshot.captured, session.current())?
    {
        return Err(conflict());
    }
    crate::application::ai_provenance::verify_bound(
        proposals.historical_reader(),
        session.current(),
    )?;
    let expected = expected.cloned().unwrap_or(CaptureExpectation {
        draft_revision: snapshot.captured.document.draft_revision,
        state_sha256: snapshot.capture_sha256.clone(),
    });
    if expected.draft_revision.0 < snapshot.captured.document.draft_revision.0 {
        return Err(conflict());
    }
    Ok((session, expected))
}

fn result(
    state: StoredProposal,
    acceptance: Option<AcceptanceReceipt>,
    reused_acceptance: bool,
) -> Result<CaptureMutationResult, ApiError> {
    let continuation = CaptureExpectation {
        draft_revision: state.document.draft_revision,
        state_sha256: hash(&state)?,
    };
    Ok(CaptureMutationResult {
        state,
        continuation,
        acceptance,
        reused_acceptance,
    })
}

pub fn edit(
    projects: &dyn ProjectStore,
    proposals: &dyn ProposalStore,
    anchor: &CaptureAnchor,
    expected: Option<&CaptureExpectation>,
    draft: ProposalPayload,
) -> Result<CaptureMutationResult, ApiError> {
    let (mut session, expected) = open(projects, proposals, anchor, expected)?;
    expected_state(session.current(), &expected)?;
    let next = edit_state(session.current().clone(), draft, expected.draft_revision.0)
        .map_err(mutation_error)?;
    session.publish(expected.draft_revision.0, &next)?;
    result(next, None, false)
}

pub fn accept(
    projects: &dyn ProjectStore,
    proposals: &dyn ProposalStore,
    anchor: &CaptureAnchor,
    expected: Option<&CaptureExpectation>,
    target_source_revision: DecimalU64,
) -> Result<CaptureMutationResult, ApiError> {
    let (mut session, expected) = open(projects, proposals, anchor, expected)?;
    if session.current().document.draft_revision == expected.draft_revision {
        expected_state(session.current(), &expected)?;
        let next = accept_state(
            session.current().clone(),
            target_source_revision,
            expected.draft_revision.0,
        )
        .map_err(mutation_error)?;
        if !next.publication_required {
            return Err(conflict());
        }
        session.publish(expected.draft_revision.0, &next.state)?;
        return result(next.state, Some(next.receipt), false);
    }
    // Retry verifies the pre-accept full state and deterministic post-state
    // under the SAME writer before returning any existing receipt.
    if expected.draft_revision.0.checked_add(1) != Some(session.current().document.draft_revision.0)
    {
        return Err(conflict());
    }
    let before = session.revision(expected.draft_revision.0)?;
    expected_state(&before, &expected)?;
    if !same_immutable(&before, session.current())? {
        return Err(conflict());
    }
    let next = accept_state(before, target_source_revision, expected.draft_revision.0)
        .map_err(mutation_error)?;
    if !next.publication_required || hash(&next.state)? != hash(session.current())? {
        return Err(conflict());
    }
    result(session.current().clone(), Some(next.receipt), true)
}
