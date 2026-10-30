use super::timeline_evidence;
use super::*;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{api_values::DecimalU64, identifiers::valid_sha256, review_source::*},
};

struct Guard<'a> {
    case: &'a dyn CaseLease,
    media: &'a dyn MediaLease,
    control: &'a dyn MutationControl,
    proof: Option<&'a timeline_evidence::RetainedTimelineEvidence<'a>>,
    association: &'a AssociationRevision,
}
impl PublicationGuard for Guard<'_> {
    fn before_publish(&self) -> Result<(), ApiError> {
        self.control.checkpoint()?;
        self.case.check_current()?;
        self.media.check_current()?;
        if let Some(proof) = self.proof {
            let TimelineStatus::Verified(record) = &self.association.timeline_status else {
                return Err(conflict("Verified publication record missing"));
            };
            proof.check_record(record)?;
        }
        Ok(())
    }
}
fn checked_receipt(
    receipt: MutationReceipt,
    request: &MutationRequest,
) -> Result<MutationReceipt, ApiError> {
    if &receipt.body.request != request {
        return Err(conflict("Stored replay identity differs"));
    }
    if !valid_sha256(&receipt.envelope_sha256) || receipt.body.validate().is_err() {
        return Err(uncertain_receipt());
    }
    Ok(receipt)
}
fn uncertain_receipt() -> ApiError {
    let mut error = ApiError::new(
        ErrorCode::EngineFailed,
        "Review publication receipt is uncertain; query the same key",
    );
    error.details = serde_json::json!({"delivery":"execution_unknown","publication_may_exist":true,"automatic_retry":false});
    error
}
pub fn mutate(
    cases: &dyn CaseResolver,
    media: &dyn MediaResolver,
    store: &dyn UserReviewStore,
    control: &dyn MutationControl,
    request: MutationRequest,
) -> Result<MutationReceipt, ApiError> {
    mutate_with_evidence(cases, media, store, control, request, None)
}

/// Exact replay precedes fresh evidence admission. New verified publications
/// keep the original proof alive through the storage adapter's final guard.
pub fn mutate_with_evidence(
    cases: &dyn CaseResolver,
    media: &dyn MediaResolver,
    store: &dyn UserReviewStore,
    control: &dyn MutationControl,
    request: MutationRequest,
    evidence: Option<&dyn timeline_evidence::TimelineEvidenceResolver>,
) -> Result<MutationReceipt, ApiError> {
    request.validate()?;
    let case = cases.retain(&request.case)?;
    if case.case() != &request.case {
        return Err(conflict("Case resolver returned a foreign case"));
    }
    case.check_current()?;
    let mut transaction = store.open(&request.case, StoreAccess::Write)?;
    if let Some(receipt) = transaction.replay(&request)? {
        return checked_receipt(receipt, &request);
    }
    control.checkpoint()?;
    let mut proof = None;
    let association = match &request.mutation {
        Mutation::VerifyAssociation {
            association_id,
            media: reference_media,
            evidence: reference,
            note,
        } => {
            let previous = transaction
                .association(association_id, None)?
                .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Association missing"))?;
            if request.expected_revision != Some(previous.revision) {
                return Err(conflict("Association expected revision differs"));
            }
            let resolver = evidence.ok_or_else(|| {
                ApiError::new(
                    ErrorCode::Unsupported,
                    "Timeline evidence adapter unavailable",
                )
            })?;
            let retained = timeline_evidence::retain(
                cases,
                media,
                resolver,
                &request.case,
                reference_media,
                reference,
            )?;
            let record = retained.mapping_record()?;
            let association = AssociationRevision {
                association_id: association_id.clone(),
                case: request.case.clone(),
                media: reference_media.clone(),
                revision: next_revision(request.expected_revision).map_err(validate)?,
                timeline_status: TimelineStatus::Verified(Box::new(record)),
                evidence_refs: vec![reference.clone()],
                note: note.clone(),
            };
            association
                .validate_successor(&previous)
                .map_err(validate)?;
            proof = Some(retained);
            association
        }
        Mutation::CreateAssociation(a) => {
            if transaction.association(&a.association_id, None)?.is_some() {
                return Err(conflict("Association already exists"));
            }
            a.as_ref().clone()
        }
        Mutation::ReviseAssociation(a) => {
            let previous = transaction
                .association(&a.association_id, None)?
                .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Association missing"))?;
            a.validate_successor(&previous).map_err(validate)?;
            a.as_ref().clone()
        }
        Mutation::SaveDraft {
            draft,
            create_association,
        } => {
            if let Some(a) = create_association {
                if transaction.association(&a.association_id, None)?.is_some() {
                    return Err(conflict("Combined association already exists"));
                }
                a.as_ref().clone()
            } else {
                transaction
                    .association(&draft.association_id, Some(draft.association_revision))?
                    .ok_or_else(|| {
                        ApiError::new(ErrorCode::NotFound, "Exact draft association missing")
                    })?
            }
        }
    };
    if association.case != request.case {
        return Err(conflict("Association belongs to a different case"));
    }
    association.validate().map_err(validate)?;
    let lease = media.retain(&request.case, &association.media)?;
    if lease.identity() != &association.media {
        return Err(conflict("Media lease identity differs"));
    }
    lease.check_current()?;
    let draft = if let Mutation::SaveDraft { draft, .. } = &request.mutation {
        match transaction.draft(&draft.draft_id, None)? {
            Some(previous) => {
                if request.expected_revision != Some(previous.draft_revision) {
                    return Err(conflict("Draft expected revision differs"));
                }
                draft
                    .validate_successor(&previous, &association, lease.verified_duration_us())
                    .map_err(validate)?;
            }
            None => {
                if request.expected_revision.is_some() {
                    return Err(conflict("Draft revision missing"));
                }
                draft
                    .validate(&association, lease.verified_duration_us())
                    .map_err(validate)?;
            }
        }
        Some(draft.as_ref().clone())
    } else {
        None
    };
    let body = TransactionBody {
        schema_version: "1".into(),
        request: request.clone(),
        association,
        draft,
    };
    body.validate()?;
    let guard = Guard {
        case: case.as_ref(),
        media: lease.as_ref(),
        control,
        proof: proof.as_ref(),
        association: &body.association,
    };
    guard.before_publish()?;
    let receipt = transaction.publish(&body, &guard)?;
    if receipt.body != body || !valid_sha256(&receipt.envelope_sha256) {
        return Err(uncertain_receipt());
    }
    Ok(receipt)
}
pub fn inspect_association(
    cases: &dyn CaseResolver,
    store: &dyn UserReviewStore,
    case: &ReviewCase,
    id: &str,
    revision: Option<DecimalU64>,
) -> Result<AssociationRevision, ApiError> {
    case.validate().map_err(validate)?;
    if !valid_review_object_id(id) || revision.is_some_and(|r| r.0 == 0) {
        return Err(invalid("Invalid association query"));
    }
    let lease = cases.retain(case)?;
    if lease.case() != case {
        return Err(conflict("Foreign case lease"));
    }
    lease.check_current()?;
    let transaction = store.open(case, StoreAccess::Read)?;
    let association = transaction
        .association(id, revision)?
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Association missing"))?;
    association.validate().map_err(validate)?;
    if association.case != *case
        || association.association_id != id
        || revision.is_some_and(|r| r != association.revision)
    {
        return Err(conflict("Stored association query mismatch"));
    }
    Ok(association)
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct DraftView {
    pub draft: RangeDraft,
    pub association: AssociationRevision,
}
pub fn read_draft(
    cases: &dyn CaseResolver,
    store: &dyn UserReviewStore,
    case: &ReviewCase,
    id: &str,
    revision: Option<DecimalU64>,
) -> Result<DraftView, ApiError> {
    case.validate().map_err(validate)?;
    if !valid_review_object_id(id) || revision.is_some_and(|r| r.0 == 0) {
        return Err(invalid("Invalid draft query"));
    }
    let lease = cases.retain(case)?;
    if lease.case() != case {
        return Err(conflict("Foreign case lease"));
    }
    lease.check_current()?;
    let transaction = store.open(case, StoreAccess::Read)?;
    let draft = transaction
        .draft(id, revision)?
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Draft missing"))?;
    let association = transaction
        .association(&draft.association_id, Some(draft.association_revision))?
        .ok_or_else(|| conflict("Stored draft association missing"))?;
    draft.validate(&association, None).map_err(validate)?;
    if association.case != *case
        || draft.draft_id != id
        || revision.is_some_and(|r| r != draft.draft_revision)
    {
        return Err(conflict("Stored draft query mismatch"));
    }
    Ok(DraftView { draft, association })
}
