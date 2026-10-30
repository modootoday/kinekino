use super::super::StoredProposal;
use super::{CriteriaDocument, CriteriaIdentity, OutputReviewTarget, text};
use crate::domain::{api_values::DecimalU64, identifiers::valid_id};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputVerdict {
    Usable,
    NeedsChanges,
    Deferred,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputDecisionScope {
    AcceptedOutputOnly,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredReviewer {
    pub label: String,
    pub reviewer_id: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordClock {
    LocalSystemUnixMs,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedRecordTime {
    pub unix_ms: DecimalU64,
    pub clock: RecordClock,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputDecisionInput {
    pub decision_id: String,
    pub target: OutputReviewTarget,
    pub criteria: CriteriaIdentity,
    pub verdict: OutputVerdict,
    pub note: String,
    pub declared_reviewer: DeclaredReviewer,
    pub claimed_review_unix_ms: Option<DecimalU64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputDecision {
    pub schema_version: String,
    pub input: OutputDecisionInput,
    pub criteria_document: CriteriaDocument,
    pub observed_record_time: ObservedRecordTime,
    pub scope: OutputDecisionScope,
    pub automatic_application_enabled: bool,
    pub counts_as_model_sample: bool,
}
impl OutputDecisionInput {
    pub fn validate_stored(&self, criteria: &CriteriaDocument) -> Result<(), &'static str> {
        self.target.validate_stored()?;
        if self.criteria != criteria.identity()? || criteria.task != self.target.task {
            return Err("Output criteria identity or task differs");
        }
        if !valid_id(&self.decision_id)
            || !text(&self.note, 16_384)
            || !text(&self.declared_reviewer.label, 256)
            || self
                .declared_reviewer
                .reviewer_id
                .as_ref()
                .is_some_and(|id| !valid_id(id))
        {
            return Err("Invalid explicit output decision metadata");
        }
        Ok(())
    }
    pub fn validate(
        &self,
        state: &StoredProposal,
        criteria: &CriteriaDocument,
    ) -> Result<(), &'static str> {
        self.validate_stored(criteria)?;
        self.target.validate_against(state)
    }
}
impl OutputDecision {
    /// Does not verify external evidence, authenticate a reviewer, read a clock or publish.
    pub fn prepare(
        input: OutputDecisionInput,
        state: &StoredProposal,
        criteria_document: CriteriaDocument,
        observed_record_time: ObservedRecordTime,
    ) -> Result<Self, &'static str> {
        let decision = Self {
            schema_version: "1".into(),
            input,
            criteria_document,
            observed_record_time,
            scope: OutputDecisionScope::AcceptedOutputOnly,
            automatic_application_enabled: false,
            counts_as_model_sample: false,
        };
        decision.validate_against(state)?;
        Ok(decision)
    }
    /// Offline record structure/integrity only, not current evidence verification.
    pub fn validate_stored(&self) -> Result<(), &'static str> {
        if self.schema_version != "1"
            || self.automatic_application_enabled
            || self.counts_as_model_sample
        {
            return Err("Invalid output decision schema or scope");
        }
        self.input.validate_stored(&self.criteria_document)
    }
    pub fn validate_against(&self, state: &StoredProposal) -> Result<(), &'static str> {
        self.validate_stored()?;
        self.input.target.validate_against(state)
    }
    /// Request equality excludes adapter-observed time; stale checks remain application work.
    pub fn matches_request(&self, input: &OutputDecisionInput) -> bool {
        self.input == *input
    }
}
