use super::*;
use crate::domain::{
    api_values::DecimalU64,
    review_source::{AssociationRevision, RangeDraft},
};
use std::{cell::Cell, io::Cursor};
pub(super) struct Transaction {
    pub root: Arc<Root>,
    pub case: ReviewCase,
    pub directory: PathBuf,
    pub _reservation: File,
    pub _guards: Vec<File>,
    pub _files: Vec<File>,
    pub records: Vec<MutationReceipt>,
    pub access: StoreAccess,
}
impl CaseTransaction for Transaction {
    fn replay(&self, request: &MutationRequest) -> Result<Option<MutationReceipt>, ApiError> {
        if request.case != self.case {
            return Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Foreign review case",
            ));
        }
        for receipt in &self.records {
            let old = &receipt.body.request;
            if old.scope() == request.scope() && old.idempotency_key == request.idempotency_key {
                return if old == request {
                    Ok(Some(receipt.clone()))
                } else {
                    Err(ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Review key payload differs",
                    ))
                };
            }
        }
        Ok(None)
    }
    fn association(
        &self,
        id: &str,
        revision: Option<DecimalU64>,
    ) -> Result<Option<AssociationRevision>, ApiError> {
        Ok(self
            .records
            .iter()
            .map(|r| &r.body.association)
            .filter(|a| a.association_id == id && revision.is_none_or(|r| r == a.revision))
            .max_by_key(|a| a.revision)
            .cloned())
    }
    fn draft(
        &self,
        id: &str,
        revision: Option<DecimalU64>,
    ) -> Result<Option<RangeDraft>, ApiError> {
        Ok(self
            .records
            .iter()
            .filter_map(|r| r.body.draft.as_ref())
            .filter(|d| d.draft_id == id && revision.is_none_or(|r| r == d.draft_revision))
            .max_by_key(|d| d.draft_revision)
            .cloned())
    }
    fn publish(
        &mut self,
        body: &TransactionBody,
        guard: &dyn PublicationGuard,
    ) -> Result<MutationReceipt, ApiError> {
        if self.access != StoreAccess::Write || !self.root.writable {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Read-only transaction",
            ));
        }
        body.validate()?;
        if body.request.case != self.case {
            return Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Foreign review transaction",
            ));
        }
        if let Some(receipt) = self.replay(&body.request)? {
            return Ok(receipt);
        }
        if self.records.len() >= MAX_CASE_TRANSACTIONS {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Case history capacity reached; preserve existing keys",
            ));
        }
        self.check_next(body)?;
        let bytes = serde_json::to_vec(body).map_err(|_| incomplete("encode"))?;
        let (_, sha) = content_digest::sha256(&mut Cursor::new(&bytes), MAX_TRANSACTION_BYTES)?;
        let target = self.directory.join(format!("{sha}.json"));
        let attempted = Cell::new(false);
        crate::infrastructure::new_file::publish_guarded(target.to_str().ok_or_else(||incomplete("path"))?,&bytes,||{guard.before_publish()?;attempted.set(true);Ok(())})
            .map_err(|mut error|{if attempted.get(){error.details=serde_json::json!({"phase":"publication","delivery":"execution_unknown","publication_may_exist":true,"automatic_retry":false,"cause":error.details});}error})?;
        #[cfg(test)]
        if self
            .root
            .lose_ack
            .swap(false, std::sync::atomic::Ordering::SeqCst)
        {
            return Err(unknown("ack_lost"));
        }
        let (receipt, file) =
            records::read(&target, &self.case).map_err(|_| unknown("readback"))?;
        if receipt.body != *body {
            return Err(unknown("readback_identity"));
        }
        let mut next = self.records.clone();
        next.push(receipt.clone());
        records::validate(&next).map_err(|_| unknown("history_readback"))?;
        self.records = next;
        self._files.push(file);
        Ok(receipt)
    }
}
impl Transaction {
    fn check_next(&self, body: &TransactionBody) -> Result<(), ApiError> {
        let conflict = || {
            ApiError::new(
                ErrorCode::RevisionConflict,
                "Review transaction CAS/binding differs",
            )
        };
        let a = &body.association;
        match &body.request.mutation {
            Mutation::CreateAssociation(_) => {
                if self.association(&a.association_id, None)?.is_some() {
                    return Err(conflict());
                }
            }
            Mutation::ReviseAssociation(_) | Mutation::VerifyAssociation { .. } => {
                let old = self
                    .association(&a.association_id, None)?
                    .ok_or_else(conflict)?;
                a.validate_successor(&old).map_err(|_| conflict())?;
            }
            Mutation::SaveDraft {
                draft,
                create_association,
            } => {
                let existing = self.association(
                    &a.association_id,
                    if create_association.is_some() {
                        None
                    } else {
                        Some(a.revision)
                    },
                )?;
                if create_association.is_some() && existing.is_some()
                    || create_association.is_none() && existing.as_ref() != Some(a)
                {
                    return Err(conflict());
                }
                match self.draft(&draft.draft_id, None)? {
                    Some(old) => {
                        if body.request.expected_revision != Some(old.draft_revision) {
                            return Err(conflict());
                        }
                        draft
                            .validate_successor(&old, a, None)
                            .map_err(|_| conflict())?;
                    }
                    None => {
                        if body.request.expected_revision.is_some() {
                            return Err(conflict());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
