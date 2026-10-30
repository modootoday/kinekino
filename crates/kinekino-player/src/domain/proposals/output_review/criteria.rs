use super::super::ProposalTask;
use super::{hash::canonical_sha256, text};
use crate::domain::identifiers::valid_id;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputCriterion {
    pub criterion_id: String,
    pub how_to_review: String,
    pub expected: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriteriaDocument {
    pub schema_version: String,
    pub criteria_id: String,
    pub version: String,
    pub task: ProposalTask,
    pub purpose: String,
    pub clauses: Vec<OutputCriterion>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriteriaIdentity {
    pub criteria_id: String,
    pub version: String,
    pub task: ProposalTask,
    pub purpose: String,
    pub content_sha256: String,
}
impl CriteriaDocument {
    pub fn identity(&self) -> Result<CriteriaIdentity, &'static str> {
        self.validate()?;
        Ok(CriteriaIdentity {
            criteria_id: self.criteria_id.clone(),
            version: self.version.clone(),
            task: self.task,
            purpose: self.purpose.clone(),
            content_sha256: canonical_sha256(self)?,
        })
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != "1"
            || !valid_id(&self.criteria_id)
            || !valid_id(&self.version)
            || !text(&self.purpose, 4096)
            || self.clauses.is_empty()
            || self.clauses.len() > 32
        {
            return Err("Invalid versioned output criteria");
        }
        let mut ids = BTreeSet::new();
        for clause in &self.clauses {
            if !valid_id(&clause.criterion_id)
                || !ids.insert(&clause.criterion_id)
                || !text(&clause.how_to_review, 4096)
                || !text(&clause.expected, 4096)
            {
                return Err("Invalid or duplicate output criterion");
            }
        }
        Ok(())
    }
}
