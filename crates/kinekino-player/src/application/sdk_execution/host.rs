//! Host-owned references never reconstruct SDK OperationId or recovery authority.
use super::*;
use std::collections::BTreeMap;
pub mod schema;
mod submission;
mod wire;
pub use submission::{SubmissionIdentity, SubmissionReceipt};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostOperationReference {
    pub host_scope: String,
    pub ordinal: String,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct HostReceiptRecovery {
    pub generation: u8,
    pub retry_requested: bool,
    pub retry_consumed: bool,
    pub attempt_budget_exhausted: bool,
}
struct Retained {
    next: u64,
    operations: BTreeMap<u64, OperationId>,
    submissions: BTreeMap<String, submission::Record>,
}
/// Assembly supplies one fresh unpredictable scope per host. Keep until explicit
/// close/join; a transport disconnect must not replace this original owner.
pub struct HostExecution {
    attachment: ExecutionAttachment,
    _owner: ExecutionOwner,
    scope: String,
    capacity: usize,
    max_request_bytes: usize,
    retained: Mutex<Retained>,
}
impl HostExecution {
    pub fn start(
        scope: String,
        client: ApplicationClientHandle,
        budget: ExecutionBudget,
        notifier: Arc<dyn CompletionNotifier>,
    ) -> Result<Self, ApiError> {
        if scope.len() != 32
            || !scope
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Fresh lowercase host scope required",
            ));
        }
        let owner = ExecutionOwner::start(client, budget, notifier)?;
        let attachment = owner.handle().attach()?;
        Ok(Self {
            attachment,
            _owner: owner,
            scope,
            capacity: budget.max_operations,
            max_request_bytes: budget.max_request_bytes,
            retained: Mutex::new(Retained {
                next: 1,
                operations: BTreeMap::new(),
                submissions: BTreeMap::new(),
            }),
        })
    }
    pub fn submit(&self, request: Request) -> Result<HostOperationReference, ApiError> {
        let mut state = self
            .retained
            .lock()
            .map_err(|_| internal("Host operation ownership unavailable"))?;
        if state.operations.len() >= self.capacity {
            return Err(capacity());
        }
        let ordinal = state.next;
        let next = ordinal.checked_add(1).ok_or_else(capacity)?;
        let id = self.attachment.submit(request)?;
        state.operations.insert(ordinal, id);
        state.next = next;
        Ok(HostOperationReference {
            host_scope: self.scope.clone(),
            ordinal: ordinal.to_string(),
        })
    }
    pub fn host_scope(&self) -> &str {
        &self.scope
    }
    fn resolve(&self, reference: &HostOperationReference) -> Result<OperationId, ApiError> {
        if reference.host_scope != self.scope {
            return Err(stale());
        }
        let ordinal: u64 = reference.ordinal.parse().map_err(|_| stale())?;
        if ordinal.to_string() != reference.ordinal {
            return Err(stale());
        }
        self.retained
            .lock()
            .map_err(|_| internal("Host operation ownership unavailable"))?
            .operations
            .get(&ordinal)
            .cloned()
            .ok_or_else(stale)
    }
    pub fn peek(&self, reference: &HostOperationReference) -> Result<ExecutionSnapshot, ApiError> {
        self.attachment.peek(&self.resolve(reference)?)
    }
    pub fn cancel(
        &self,
        reference: &HostOperationReference,
    ) -> Result<CancelDisposition, ApiError> {
        self.attachment.cancel(&self.resolve(reference)?)
    }
    pub fn receipt_recovery(
        &self,
        reference: &HostOperationReference,
    ) -> Result<Option<HostReceiptRecovery>, ApiError> {
        Ok(self
            .attachment
            .receipt_recovery(&self.resolve(reference)?)?
            .map(|snapshot| HostReceiptRecovery {
                generation: snapshot.generation(),
                retry_requested: snapshot.retry_requested,
                retry_consumed: snapshot.retry_consumed,
                attempt_budget_exhausted: snapshot.attempt_budget_exhausted,
            }))
    }
    pub fn retry_receipt(
        &self,
        reference: &HostOperationReference,
        expected_generation: u8,
    ) -> Result<CleanupRetryDisposition, ApiError> {
        let id = self.resolve(reference)?;
        let snapshot = self.attachment.receipt_recovery(&id)?.ok_or_else(pending)?;
        if snapshot.generation() != expected_generation {
            return Err(stale());
        }
        self.attachment.retry_receipt(&id, &snapshot.token)
    }
    pub fn acknowledge(&self, reference: &HostOperationReference) -> Result<AckReceipt, ApiError> {
        let id = self.resolve(reference)?;
        let mut state = self
            .retained
            .lock()
            .map_err(|_| internal("Host operation ownership unavailable"))?;
        if !state.operations.values().any(|retained| retained == &id) {
            return Err(stale());
        }
        let receipt = self.attachment.acknowledge(&id)?;
        state.operations.retain(|_, retained| retained != &id);
        Ok(receipt)
    }
    pub fn poll_close(&self) -> Result<ExecutionCloseProgress, ApiError> {
        self.attachment.poll_close()
    }
    pub fn retained_references(&self) -> Result<Vec<HostOperationReference>, ApiError> {
        Ok(self
            .retained
            .lock()
            .map_err(|_| internal("Host operation ownership unavailable"))?
            .operations
            .keys()
            .map(|ordinal| HostOperationReference {
                host_scope: self.scope.clone(),
                ordinal: ordinal.to_string(),
            })
            .collect())
    }
}
