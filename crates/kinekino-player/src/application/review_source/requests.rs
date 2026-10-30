use super::{invalid, validate};
use crate::{
    application::api::ApiError,
    domain::{api_values::DecimalU64, review_source::*},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "payload",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Mutation {
    CreateAssociation(Box<AssociationRevision>),
    ReviseAssociation(Box<AssociationRevision>),
    VerifyAssociation {
        association_id: String,
        media: MediaReference,
        evidence: EvidenceReference,
        note: String,
    },
    SaveDraft {
        draft: Box<RangeDraft>,
        create_association: Option<Box<AssociationRevision>>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationRequest {
    pub case: ReviewCase,
    pub idempotency_key: String,
    pub expected_revision: Option<DecimalU64>,
    pub mutation: Mutation,
}
impl MutationRequest {
    pub fn scope(&self) -> (&'static str, &str) {
        match &self.mutation {
            Mutation::CreateAssociation(a) => ("association_create", &a.association_id),
            Mutation::ReviseAssociation(a) => ("association_revise", &a.association_id),
            Mutation::VerifyAssociation { association_id, .. } => {
                ("association_revise", association_id)
            }
            Mutation::SaveDraft { draft, .. } => ("range_draft_save", &draft.draft_id),
        }
    }
    pub fn validate(&self) -> Result<(), ApiError> {
        self.case.validate().map_err(validate)?;
        if !valid_review_object_id(&self.idempotency_key) {
            return Err(invalid("Review mutation key required"));
        }
        let next = next_revision(self.expected_revision).map_err(validate)?;
        match &self.mutation {
            Mutation::CreateAssociation(a) | Mutation::ReviseAssociation(a) => {
                if a.timeline_status != TimelineStatus::Unverified {
                    return Err(invalid(
                        "Caller timeline summaries are not publication authority",
                    ));
                }
                a.validate().map_err(validate)?;
                if a.case != self.case || a.revision != next {
                    return Err(invalid("Association case/revision does not match request"));
                }
                if matches!(&self.mutation, Mutation::CreateAssociation(_))
                    != self.expected_revision.is_none()
                {
                    return Err(invalid("Create/revise revision context differs"));
                }
            }
            Mutation::VerifyAssociation {
                association_id,
                media,
                evidence,
                note,
            } => {
                media.validate().map_err(validate)?;
                if self.expected_revision.is_none()
                    || !valid_review_object_id(association_id)
                    || !valid_review_object_id(&evidence.asset_id)
                    || !crate::domain::identifiers::valid_sha256(&evidence.sha256)
                    || note.len() > 8192
                {
                    return Err(invalid("Invalid verified association request"));
                }
            }
            Mutation::SaveDraft {
                draft,
                create_association,
            } => {
                draft.validate_shape().map_err(validate)?;
                if draft.draft_revision != next {
                    return Err(invalid("Invalid range draft revision"));
                }
                if let Some(a) = create_association {
                    if a.timeline_status != TimelineStatus::Unverified {
                        return Err(invalid("Combined create cannot supply verified metadata"));
                    }
                    if self.expected_revision.is_some()
                        || a.case != self.case
                        || a.revision != DecimalU64(1)
                    {
                        return Err(invalid(
                            "Combined create requires a new association and draft",
                        ));
                    }
                    draft.validate(a, None).map_err(validate)?;
                }
            }
        }
        Ok(())
    }
}
/// This single immutable body is the publication authority, including exact replay identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransactionBody {
    pub schema_version: String,
    pub request: MutationRequest,
    pub association: AssociationRevision,
    pub draft: Option<RangeDraft>,
}
impl TransactionBody {
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.schema_version != "1" {
            return Err(invalid("Unknown review transaction schema"));
        }
        self.request.validate()?;
        self.association.validate().map_err(validate)?;
        if self.association.case != self.request.case {
            return Err(invalid("Transaction case differs"));
        }
        match &self.request.mutation {
            Mutation::VerifyAssociation {
                association_id,
                media,
                evidence,
                note,
            } => {
                if self.association.association_id != *association_id
                    || self.association.media != *media
                    || self.association.note != *note
                    || self.association.revision
                        != next_revision(self.request.expected_revision).map_err(validate)?
                    || self.association.evidence_refs.as_slice() != [evidence.clone()]
                    || !matches!(
                        self.association.timeline_status,
                        TimelineStatus::Verified(_)
                    )
                    || self.draft.is_some()
                {
                    return Err(invalid("Verified transaction payload differs"));
                }
            }
            Mutation::CreateAssociation(a) | Mutation::ReviseAssociation(a) => {
                if a.as_ref() != &self.association || self.draft.is_some() {
                    return Err(invalid("Association transaction payload differs"));
                }
            }
            Mutation::SaveDraft {
                draft,
                create_association,
            } => {
                if self.draft.as_ref() != Some(draft.as_ref())
                    || create_association
                        .as_ref()
                        .is_some_and(|a| a.as_ref() != &self.association)
                {
                    return Err(invalid("Draft transaction payload differs"));
                }
                draft.validate(&self.association, None).map_err(validate)?;
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationReceipt {
    pub envelope_sha256: String,
    pub body: TransactionBody,
}
