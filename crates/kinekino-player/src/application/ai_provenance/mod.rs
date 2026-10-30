//! Historical proposal binding. Registration and persistence adapters are SDK-owned.
//! Evidence verification is distinct from model admission and human approval.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        proposals::CreateProposal,
    },
    domain::{
        identifiers::{valid_id, valid_sha256},
        proposals::{ProposalTask, StoredProposal},
    },
};
use sha2::{Digest, Sha256};
mod import;
pub use import::{ImportHistoricalProposal, import};

pub use crate::domain::proposals::CompanionReference;

/// An infrastructure port must reconstruct this from pinned originals and the
/// complete companion, rather than trusting a caller's assertions or a path.
#[derive(Clone)]
pub struct VerifiedHistoricalSample {
    pub reference: CompanionReference,
    pub task: ProposalTask,
    pub source_sha256: String,
    pub source_bytes: u64,
    pub source_request_utf8: String,
    pub candidate_sha256: String,
    pub candidate_bytes: u64,
    pub candidate_raw_utf8: String,
}

#[cfg(test)]
mod integration_tests;

pub trait HistoricalEvidenceReader: Send + Sync {
    fn verify(&self, reference: &CompanionReference) -> Result<VerifiedHistoricalSample, ApiError>;
}

/// Construction requires full evidence verification and exact source/raw binding.
/// Persistence must atomically retain this reference with the original proposal;
/// this module does not add an unsafe two-step create-then-sidecar operation.
pub struct PreparedHistoricalProposal {
    input: CreateProposal,
    evidence: CompanionReference,
}
impl PreparedHistoricalProposal {
    pub fn into_parts(self) -> (CreateProposal, CompanionReference) {
        (self.input, self.evidence)
    }
}

pub fn prepare(
    reader: &dyn HistoricalEvidenceReader,
    reference: CompanionReference,
    input: CreateProposal,
) -> Result<PreparedHistoricalProposal, ApiError> {
    validate_reference(&reference)?;
    input.source.validate().map_err(invalid)?;
    if !input.source.supports_task(input.generated.task) {
        return Err(invalid("Historical source task mismatch"));
    }
    let verified = reader.verify(&reference)?;
    matches_source(&verified, &input.source, input.generated.task)?;
    matches_input(
        &reference,
        &verified,
        input.generated.task,
        input.source.source_sha256(),
        &input.generated.raw_text,
    )?;
    Ok(PreparedHistoricalProposal {
        input,
        evidence: reference,
    })
}

/// Used after inspect/edit/accept/restart; the original generated text is checked,
/// never the editable draft or accepted output. Missing evidence stays an error.
pub fn verify_stored(
    reader: &dyn HistoricalEvidenceReader,
    reference: &CompanionReference,
    stored: &StoredProposal,
) -> Result<(), ApiError> {
    validate_reference(reference)?;
    stored.validate().map_err(invalid)?;
    let sample = reader.verify(reference)?;
    matches_source(
        &sample,
        &stored.document.source,
        stored.document.generated.task,
    )?;
    matches_input(
        reference,
        &sample,
        stored.document.generated.task,
        stored.document.source.source_sha256(),
        &stored.document.generated.raw_text,
    )
}

fn matches_source(
    sample: &VerifiedHistoricalSample,
    source: &crate::domain::proposals::SourceSnapshot,
    task: ProposalTask,
) -> Result<(), ApiError> {
    if sample.source_bytes == 0
        || sample.source_bytes > 1_048_576
        || sample.source_request_utf8.len() as u64 != sample.source_bytes
    {
        return Err(invalid("Historical source length differs"));
    }
    let reconstructed = crate::application::proposals::source_import::from_request_asset(
        source.source_id().into(),
        source.source_revision(),
        sample.source_sha256.clone(),
        sample.source_request_utf8.clone(),
        task,
    )?;
    if serde_json::to_value(reconstructed).map_err(|_| invalid("Source serialization failed"))?
        != serde_json::to_value(source).map_err(|_| invalid("Source serialization failed"))?
    {
        return Err(invalid(
            "Historical source snapshot differs from request asset",
        ));
    }
    Ok(())
}

pub fn inspect_source_context(
    reader: &dyn HistoricalEvidenceReader,
    stored: &StoredProposal,
) -> Result<crate::application::proposals::source_import::ImportedRequestContext, ApiError> {
    stored.validate().map_err(invalid)?;
    let binding = stored
        .historical
        .as_ref()
        .ok_or_else(crate::application::proposals::source_context::unavailable)?;
    validate_reference(&binding.reference)?;
    let sample = reader.verify(&binding.reference)?;
    matches_source(
        &sample,
        &stored.document.source,
        stored.document.generated.task,
    )?;
    matches_input(
        &binding.reference,
        &sample,
        stored.document.generated.task,
        stored.document.source.source_sha256(),
        &stored.document.generated.raw_text,
    )?;
    crate::application::proposals::source_import::from_request_asset_with_context(
        stored.document.source.source_id().into(),
        stored.document.source.source_revision(),
        sample.source_sha256,
        sample.source_request_utf8,
        stored.document.generated.task,
    )
}

fn matches_input(
    reference: &CompanionReference,
    sample: &VerifiedHistoricalSample,
    task: ProposalTask,
    source_sha256: &str,
    raw: &str,
) -> Result<(), ApiError> {
    if reference != &sample.reference
        || reference.bytes == 0
        || reference.bytes > 262_144
        || !valid_id(&reference.item_id)
        || !valid_sha256(&reference.sha256)
        || !valid_sha256(&sample.source_sha256)
        || !valid_sha256(&sample.candidate_sha256)
        || sample.source_bytes == 0
        || sample.source_bytes > 1_048_576
        || sample.task != task
        || sample.source_sha256 != source_sha256
        || sample.candidate_bytes == 0
        || sample.candidate_bytes > 262_144
        || sample.candidate_bytes != raw.len() as u64
        || sample.candidate_raw_utf8 != raw
        || sample.candidate_sha256 != format!("{:x}", Sha256::digest(raw.as_bytes()))
    {
        return Err(invalid("Historical evidence does not bind this proposal"));
    }
    Ok(())
}
fn validate_reference(reference: &CompanionReference) -> Result<(), ApiError> {
    reference.validate().map_err(invalid)
}

pub fn verify_bound(
    reader: Option<&dyn HistoricalEvidenceReader>,
    stored: &StoredProposal,
) -> Result<(), ApiError> {
    stored.validate().map_err(invalid)?;
    let Some(binding) = &stored.historical else {
        return Ok(());
    };
    binding.validate().map_err(invalid)?;
    let reader = reader.ok_or_else(|| {
        ApiError::new(
            ErrorCode::Unsupported,
            "Historical evidence reader is required for this bound proposal",
        )
    })?;
    verify_stored(reader, &binding.reference, stored)
}
fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_and_oversized_references_are_rejected() {
        let mut reference = CompanionReference {
            sha256: "a".repeat(64),
            bytes: 100,
            item_id: "valid-item".into(),
        };
        validate_reference(&reference).unwrap();
        for item in ["", "..", "../objects", "a/b"] {
            reference.item_id = item.into();
            assert!(validate_reference(&reference).is_err());
        }
        reference.item_id = "valid-item".into();
        reference.bytes = 0;
        assert!(validate_reference(&reference).is_err());
        reference.bytes = 262_145;
        assert!(validate_reference(&reference).is_err());
        reference.bytes = 100;
        reference.sha256 = "A".repeat(64);
        assert!(validate_reference(&reference).is_err());
    }

    #[test]
    fn binding_preserves_malformed_raw_and_rejects_swapped_identity() {
        let raw = "{\"unfinished\":";
        let reference = CompanionReference {
            sha256: "a".repeat(64),
            bytes: 100,
            item_id: "old-correction".into(),
        };
        let mut sample = VerifiedHistoricalSample {
            reference: reference.clone(),
            task: ProposalTask::Correction,
            source_sha256: "b".repeat(64),
            source_bytes: 1232,
            source_request_utf8: String::new(),
            candidate_sha256: format!("{:x}", Sha256::digest(raw.as_bytes())),
            candidate_bytes: raw.len() as u64,
            candidate_raw_utf8: raw.into(),
        };
        matches_input(
            &reference,
            &sample,
            ProposalTask::Correction,
            &"b".repeat(64),
            raw,
        )
        .unwrap();
        assert!(
            matches_input(
                &reference,
                &sample,
                ProposalTask::Correction,
                &"b".repeat(64),
                "repaired"
            )
            .is_err()
        );
        assert!(
            matches_input(
                &reference,
                &sample,
                ProposalTask::Correction,
                &"c".repeat(64),
                raw
            )
            .is_err()
        );
        sample.reference.item_id = "different-item".into();
        assert!(
            matches_input(
                &reference,
                &sample,
                ProposalTask::Correction,
                &"b".repeat(64),
                raw
            )
            .is_err()
        );
    }
}
