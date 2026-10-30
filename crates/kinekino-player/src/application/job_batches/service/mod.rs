//! Shared bounded SDK/CLI use cases; no request-selected root or scope.
#[path = "ports.rs"]
mod ports;
use super::*;
use crate::{application::jobs::JobsApi, domain::api_values::DecimalU64};
pub use ports::BatchApi;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
pub const MAX_PAGE_BYTES: usize = 768 * 1024;
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateBatch {
    pub batch_id: String,
    pub items: Vec<BatchJobInput>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectBatch {
    pub batch_id: String,
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "page_limit")]
    pub limit: usize,
}
fn page_limit() -> usize {
    8
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchIdentity {
    pub batch_id: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchReceipt {
    pub schema_version: String,
    pub batch_id: String,
    pub revision: DecimalU64,
    pub total: usize,
    pub states: BTreeMap<String, usize>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchPage {
    pub receipt: BatchReceipt,
    pub entries: Vec<BatchEntry>,
    pub next_offset: Option<usize>,
}
pub struct BatchService {
    store: Arc<dyn BatchStore>,
    jobs: Arc<dyn JobsApi>,
}
impl BatchService {
    /// Trusted host composition must supply the same original Jobs root binding.
    pub fn new(store: Arc<dyn BatchStore>, jobs: Arc<dyn JobsApi>) -> Self {
        Self { store, jobs }
    }
}
fn receipt(document: &BatchDocument) -> BatchReceipt {
    let mut states = BTreeMap::new();
    for entry in &document.entries {
        let name = match &entry.progress {
            BatchProgress::Waiting => "waiting".into(),
            BatchProgress::SubmitPending => "submit_pending".into(),
            BatchProgress::SubmitError { .. } => "submit_error".into(),
            BatchProgress::RetryPending { .. } => "retry_pending".into(),
            BatchProgress::Observed { job, .. } => format!(
                "observed_{}",
                serde_json::to_value(job.state)
                    .expect("Static JobState serializes")
                    .as_str()
                    .expect("JobState is a string")
            ),
        };
        *states.entry(name).or_insert(0) += 1;
    }
    BatchReceipt {
        schema_version: "1".into(),
        batch_id: document.plan.batch_id.clone(),
        revision: document.revision,
        total: document.entries.len(),
        states,
    }
}
impl BatchApi for BatchService {
    fn create(&self, input: CreateBatch) -> Result<BatchReceipt, ApiError> {
        if !crate::domain::identifiers::valid_id(&input.batch_id)
            || input.items.is_empty()
            || input.items.len() > crate::domain::job_batch::MAX_BATCH_ITEMS
        {
            return Err(document::invalid("Invalid batch identity or item count"));
        }
        let bytes =
            serde_json::to_vec(&input).map_err(|_| document::invalid("Invalid batch input"))?;
        if bytes.len() > MAX_BATCH_INPUT_BYTES {
            return Err(ApiError::new(
                crate::application::api::ErrorCode::ResourceExhausted,
                "Batch capture input byte limit exceeded",
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for item in &input.items {
            if !crate::domain::identifiers::valid_id(&item.item_id) || !seen.insert(&item.item_id) {
                return Err(document::invalid(
                    "Invalid or duplicate batch item identity",
                ));
            }
        }
        let captured = input
            .items
            .into_iter()
            .map(|item| CapturedBatchInput::new(item.item_id, self.jobs.capture(item.request)?))
            .collect::<Result<Vec<_>, ApiError>>()?;
        let document = BatchDocument::from_captured(
            self.store.scope_sha256().into(),
            input.batch_id,
            captured,
        )?;
        self.store.create(&document)?;
        Ok(receipt(&document))
    }
    fn inspect(&self, input: InspectBatch, expected: Option<u64>) -> Result<BatchPage, ApiError> {
        if input.limit == 0 || input.limit > 8 {
            return Err(document::invalid("Batch page limit must be1..8"));
        }
        let lease = self.store.acquire(&input.batch_id)?;
        let document = lease.load()?;
        document.validate()?;
        if document.plan.batch_id != input.batch_id
            || document.plan.scope_sha256 != self.store.scope_sha256()
            || expected.is_some_and(|revision| document.revision.0 != revision)
        {
            return Err(ApiError::new(
                crate::application::api::ErrorCode::RevisionConflict,
                "Batch page revision or binding changed",
            ));
        }
        if input.offset > document.entries.len() {
            return Err(document::invalid("Batch page offset exceeds item count"));
        }
        let mut page = BatchPage {
            receipt: receipt(&document),
            entries: vec![],
            next_offset: None,
        };
        for entry in document
            .entries
            .into_iter()
            .skip(input.offset)
            .take(input.limit)
        {
            page.entries.push(entry);
            if serde_json::to_vec(&page)
                .map_err(|_| document::invalid("Invalid batch page"))?
                .len()
                > MAX_PAGE_BYTES
            {
                page.entries.pop();
                break;
            }
        }
        if page.entries.is_empty() && input.offset < page.receipt.total {
            return Err(ApiError::new(
                crate::application::api::ErrorCode::ResourceExhausted,
                "Batch entry exceeds the wire page byte limit; original journal is retained",
            ));
        }
        let end = input.offset + page.entries.len();
        page.next_offset = (end < page.receipt.total).then_some(end);
        Ok(page)
    }
    fn execute(
        &self,
        input: BatchIdentity,
        expected: u64,
        action: BatchAction,
    ) -> Result<BatchReceipt, ApiError> {
        let document = run(
            self.store.as_ref(),
            self.jobs.as_ref(),
            &input.batch_id,
            expected,
            action,
        )?;
        Ok(receipt(&document))
    }
}
