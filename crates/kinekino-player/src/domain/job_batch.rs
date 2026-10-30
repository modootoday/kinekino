//! Immutable batch identity; execution and retry authority remain with Jobs.
use crate::domain::identifiers::{valid_id, valid_sha256};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const MAX_BATCH_ITEMS: usize = 128;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchItemPin {
    pub item_id: String,
    pub request_sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchPlan {
    pub schema_version: String,
    /// Supplied by the trusted store/host binding, never selected by a wire request.
    pub scope_sha256: String,
    pub batch_id: String,
    pub items: Vec<BatchItemPin>,
}
impl BatchPlan {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != "1"
            || !valid_id(&self.batch_id)
            || !valid_sha256(&self.scope_sha256)
        {
            return Err("Invalid batch identity or host scope");
        }
        if self.items.is_empty() || self.items.len() > MAX_BATCH_ITEMS {
            return Err("Batch requires1..128 items");
        }
        let mut ids = BTreeSet::new();
        for item in &self.items {
            if !valid_id(&item.item_id)
                || !valid_sha256(&item.request_sha256)
                || !ids.insert(&item.item_id)
            {
                return Err("Batch items require unique identities and pinned requests");
            }
        }
        Ok(())
    }
    /// Stable across lost responses/reopen; Jobs still normalizes and checks the request.
    pub fn submission_key(&self, item_id: &str) -> Result<String, &'static str> {
        self.validate()?;
        let item = self
            .items
            .iter()
            .find(|item| item.item_id == item_id)
            .ok_or("Unknown batch item")?;
        let mut hash = Sha256::new();
        for part in [
            "player.job-batch.submit/1",
            &self.scope_sha256,
            &self.batch_id,
            &item.item_id,
            &item.request_sha256,
        ] {
            hash.update(part.as_bytes());
            hash.update([0]);
        }
        Ok(format!("batch-{:x}", hash.finalize()))
    }
}
