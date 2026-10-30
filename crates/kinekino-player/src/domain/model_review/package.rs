use super::{text, valid_id, valid_sha256};
use crate::domain::api_values::DecimalU64;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceAsset {
    pub asset_id: String,
    pub sha256: String,
    pub bytes: DecimalU64,
    pub media_type: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRange {
    pub start_us: DecimalU64,
    pub end_us: DecimalU64,
}
impl ReviewRange {
    pub fn valid(&self) -> bool {
        self.start_us.0 < self.end_us.0
    }
    pub fn contains(&self, other: &Self) -> bool {
        other.valid() && other.start_us.0 >= self.start_us.0 && other.end_us.0 <= self.end_us.0
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCondition {
    pub model_id: String,
    pub revision: String,
    pub precision: String,
    pub engine_id: String,
    pub engine_revision: String,
    pub input_conditions: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckOutcome {
    Passed,
    Failed,
    NotRun,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutomatedCheck {
    pub check_id: String,
    pub outcome: CheckOutcome,
    pub summary: String,
    pub evidence: Vec<EvidenceAsset>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewResult {
    pub condition: ModelCondition,
    pub assets: Vec<EvidenceAsset>,
    pub automated_checks: Vec<AutomatedCheck>,
    pub measured_resources: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewCriterion {
    pub criterion_id: String,
    pub how_to_review: String,
    pub expected: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewItem {
    pub item_id: String,
    pub task: String,
    pub source: EvidenceAsset,
    pub range: Option<ReviewRange>,
    pub baseline: Option<ReviewResult>,
    pub candidate: ReviewResult,
    pub criteria: Vec<ReviewCriterion>,
    pub known_issues: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewPackage {
    pub schema_version: String,
    pub package_id: String,
    pub items: Vec<ReviewItem>,
}
impl ReviewPackage {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != "1"
            || !valid_id(&self.package_id)
            || self.items.is_empty()
            || self.items.len() > 64
        {
            return Err("Invalid review package identity or item count");
        }
        let mut ids = BTreeSet::new();
        for item in &self.items {
            if !valid_id(&item.item_id)
                || !valid_id(&item.task)
                || !ids.insert(&item.item_id)
                || item.range.as_ref().is_some_and(|range| !range.valid())
                || item.criteria.is_empty()
                || item.criteria.len() > 32
                || item.known_issues.len() > 64
                || item.known_issues.iter().any(|i| !text(i, 4096))
            {
                return Err("Invalid review item, range or criteria");
            }
            let mut criteria = BTreeSet::new();
            if item.criteria.iter().any(|c| {
                !valid_id(&c.criterion_id)
                    || !criteria.insert(&c.criterion_id)
                    || !text(&c.how_to_review, 4096)
                    || !text(&c.expected, 4096)
            }) {
                return Err("Review methods and expected criteria are required");
            }
            for result in std::iter::once(&item.candidate).chain(item.baseline.iter()) {
                let c = &result.condition;
                if !text(&c.model_id, 256)
                    || !pinned(&c.revision)
                    || !text(&c.precision, 64)
                    || !text(&c.engine_id, 256)
                    || !pinned(&c.engine_revision)
                    || !fields(&c.input_conditions)
                    || !fields(&result.measured_resources)
                    || result.assets.is_empty()
                    || result.assets.len() > 16
                    || result.automated_checks.is_empty()
                    || result.automated_checks.len() > 64
                {
                    return Err(
                        "Pinned model/engine conditions, result assets and automated checks are required",
                    );
                }
                let mut checks = BTreeSet::new();
                if result.automated_checks.iter().any(|v| {
                    !valid_id(&v.check_id)
                        || !checks.insert(&v.check_id)
                        || !text(&v.summary, 4096)
                        || v.evidence.len() > 8
                }) {
                    return Err("Invalid automated check evidence");
                }
            }
        }
        let mut assets = BTreeMap::new();
        let mut asset_ids = BTreeMap::new();
        let mut total = 0u64;
        for asset in self.assets() {
            if let Some(previous) = asset_ids.insert(
                &asset.asset_id,
                (&asset.sha256, asset.bytes.0, &asset.media_type),
            ) && previous != (&asset.sha256, asset.bytes.0, &asset.media_type)
            {
                return Err("Conflicting asset identifier metadata");
            }
            if !valid_id(&asset.asset_id)
                || !valid_sha256(&asset.sha256)
                || asset.bytes.0 == 0
                || asset.bytes.0 > 1_073_741_824
                || !text(&asset.media_type, 128)
            {
                return Err("Invalid content-addressed evidence asset");
            }
            if let Some(bytes) = assets.insert(&asset.sha256, asset.bytes.0) {
                if bytes != asset.bytes.0 {
                    return Err("Conflicting evidence byte identities");
                }
            } else {
                total = total
                    .checked_add(asset.bytes.0)
                    .ok_or("Evidence size overflow")?;
            }
        }
        if total > 4_294_967_296 {
            return Err("Review evidence exceeds total size bound");
        }
        Ok(())
    }
    pub fn assets(&self) -> Vec<&EvidenceAsset> {
        self.items.iter().flat_map(ReviewItem::assets).collect()
    }
}
impl ReviewItem {
    /// Exact item references, including baseline and automated-check evidence.
    pub fn assets(&self) -> Vec<&EvidenceAsset> {
        let mut assets = vec![&self.source];
        for result in std::iter::once(&self.candidate).chain(self.baseline.iter()) {
            assets.extend(&result.assets);
            for check in &result.automated_checks {
                assets.extend(&check.evidence);
            }
        }
        assets
    }
}
fn pinned(value: &str) -> bool {
    text(value, 256)
        && !["main", "master", "latest", "head", "unknown"]
            .contains(&value.to_ascii_lowercase().as_str())
}
fn fields(values: &BTreeMap<String, String>) -> bool {
    !values.is_empty()
        && values.len() <= 32
        && values.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>() <= 4096
        && values.iter().all(|(k, v)| valid_id(k) && text(v, 1024))
}
