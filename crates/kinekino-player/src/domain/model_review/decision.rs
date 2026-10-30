use super::{
    CheckOutcome, ModelCondition, ReviewPackage, ReviewRange, text, valid_id, valid_sha256,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    Usable,
    NeedsChanges,
    Deferred,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordReviewDecision {
    pub package_sha256: String,
    pub item_id: String,
    pub decision_id: String,
    pub verdict: ReviewVerdict,
    pub note: String,
    pub issue_ranges: Vec<ReviewRange>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewDecision {
    pub schema_version: String,
    pub package_sha256: String,
    pub package_id: String,
    pub task: String,
    pub item_id: String,
    pub decision_id: String,
    pub candidate_condition: ModelCondition,
    pub verdict: ReviewVerdict,
    pub note: String,
    pub issue_ranges: Vec<ReviewRange>,
    pub review_ready: bool,
    pub automated_prerequisites_passed: bool,
    pub sample_review_passed: bool,
    pub automatic_application_enabled: bool,
    pub scope: String,
}
impl RecordReviewDecision {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !valid_sha256(&self.package_sha256)
            || !valid_id(&self.item_id)
            || !valid_id(&self.decision_id)
            || !text(&self.note, 8192)
            || self.issue_ranges.len() > 64
            || self.issue_ranges.iter().any(|r| !r.valid())
        {
            return Err("Invalid evidence-bound review decision or note");
        }
        Ok(())
    }
    pub fn bind(self, package: &ReviewPackage) -> Result<ReviewDecision, &'static str> {
        self.validate()?;
        package.validate()?;
        let item = package
            .items
            .iter()
            .find(|i| i.item_id == self.item_id)
            .ok_or("Review item not found")?;
        if !self.issue_ranges.is_empty()
            && item
                .range
                .as_ref()
                .is_none_or(|range| self.issue_ranges.iter().any(|r| !range.contains(r)))
        {
            return Err("Issue range must be within the fixed sample");
        }
        let ready = item.baseline.is_some();
        let automatic = std::iter::once(&item.candidate)
            .chain(item.baseline.iter())
            .all(|result| {
                result
                    .automated_checks
                    .iter()
                    .all(|c| c.outcome == CheckOutcome::Passed)
            });
        Ok(ReviewDecision {
            schema_version: "1".into(),
            package_sha256: self.package_sha256,
            package_id: package.package_id.clone(),
            task: item.task.clone(),
            item_id: self.item_id,
            decision_id: self.decision_id,
            candidate_condition: item.candidate.condition.clone(),
            verdict: self.verdict,
            note: self.note,
            issue_ranges: self.issue_ranges,
            review_ready: ready,
            automated_prerequisites_passed: automatic,
            sample_review_passed: ready && automatic && self.verdict == ReviewVerdict::Usable,
            automatic_application_enabled: false,
            scope: "sample_only".into(),
        })
    }
}
