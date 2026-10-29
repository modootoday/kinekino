//! Upload consent: what (segment/frame), where (HF + provider), scope. Withdrawn = refuse.
use crate::port::InferenceError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentScope {
    Task,
    #[default]
    Project,
    Always,
}

/// Runtime consent state for one project.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Consent {
    granted_project: Option<String>,
    granted_tasks: Vec<String>,
    always: bool,
}

impl Consent {
    pub fn grant(&mut self, scope: ConsentScope, project: &str, job: &str) {
        match scope {
            ConsentScope::Always => self.always = true,
            ConsentScope::Project => self.granted_project = Some(project.into()),
            ConsentScope::Task => self.granted_tasks.push(job.into()),
        }
    }
    pub fn withdraw(&mut self) {
        *self = Self::default();
    }
    pub fn check(&self, project: &str, job: &str) -> Result<(), InferenceError> {
        if self.always
            || self.granted_project.as_deref() == Some(project)
            || self.granted_tasks.iter().any(|t| t == job)
        {
            Ok(())
        } else {
            Err(InferenceError::ConsentRequired(format!(
                "project {project}, job {job}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scopes_and_withdraw() {
        let mut c = Consent::default();
        assert!(c.check("p", "j").is_err());
        c.grant(ConsentScope::Task, "p", "j");
        assert!(c.check("p", "j").is_ok());
        assert!(c.check("p", "k").is_err());
        c.grant(ConsentScope::Project, "p", "j");
        assert!(c.check("p", "k").is_ok());
        assert!(c.check("q", "k").is_err());
        c.withdraw();
        assert!(c.check("p", "j").is_err());
    }
}
