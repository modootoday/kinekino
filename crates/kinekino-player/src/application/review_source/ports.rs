use super::{MutationReceipt, MutationRequest, TransactionBody};
use crate::{
    application::api::ApiError,
    domain::{
        api_values::DecimalU64,
        review_source::{AssociationRevision, MediaReference, RangeDraft, ReviewCase},
    },
};

/// Retains verified package/item/request membership guards for the entire operation.
pub trait CaseLease {
    fn case(&self) -> &ReviewCase;
    fn check_current(&self) -> Result<(), ApiError>;
}
pub trait CaseResolver: Send + Sync {
    fn retain(&self, case: &ReviewCase) -> Result<Box<dyn CaseLease + '_>, ApiError>;
}
pub trait MediaLease {
    fn identity(&self) -> &MediaReference;
    fn check_current(&self) -> Result<(), ApiError>;
    fn verified_duration_us(&self) -> Option<u64>;
}
pub trait MediaResolver: Send + Sync {
    fn retain(
        &self,
        case: &ReviewCase,
        media: &MediaReference,
    ) -> Result<Box<dyn MediaLease + '_>, ApiError>;
}
pub trait MutationControl {
    fn checkpoint(&self) -> Result<(), ApiError>;
    fn native_processes(&self) -> Option<&super::NativeProcessLedger> {
        None
    }
}
/// Storage invokes this immediately before its one atomic publication, never afterwards.
pub trait PublicationGuard {
    fn before_publish(&self) -> Result<(), ApiError>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreAccess {
    Read,
    Write,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct StoreLimits {
    pub max_case_transactions: DecimalU64,
    pub max_transaction_bytes: DecimalU64,
}
pub trait UserReviewStore: Send + Sync {
    /// None means this adapter has not declared its storage limits, never unlimited.
    fn limits(&self) -> Option<StoreLimits> {
        None
    }
    /// Write access reserves the original case exclusively through read/check/publish.
    /// Read access must not create records or require current media availability.
    fn open(
        &self,
        case: &ReviewCase,
        access: StoreAccess,
    ) -> Result<Box<dyn CaseTransaction + '_>, ApiError>;
}
pub trait CaseTransaction {
    /// Same case/operation/object/key and different request must return Conflict.
    fn replay(&self, request: &MutationRequest) -> Result<Option<MutationReceipt>, ApiError>;
    fn association(
        &self,
        id: &str,
        revision: Option<DecimalU64>,
    ) -> Result<Option<AssociationRevision>, ApiError>;
    fn draft(&self, id: &str, revision: Option<DecimalU64>)
    -> Result<Option<RangeDraft>, ApiError>;
    /// CAS state is held by this write reservation. Publish one complete immutable
    /// envelope and read it back. Post-publication uncertainty must remain
    /// execution_unknown/publication_may_exist, never Cancelled or false no-write.
    fn publish(
        &mut self,
        body: &TransactionBody,
        guard: &dyn PublicationGuard,
    ) -> Result<MutationReceipt, ApiError>;
}
