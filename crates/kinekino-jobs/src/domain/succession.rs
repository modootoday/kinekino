use super::{JobRecord, JobState};
impl JobRecord {
    pub fn validate_successor(&self, previous: &Self) -> Result<(), &'static str> {
        previous.validate()?;
        self.validate()?;
        if self.job_id != previous.job_id
            || serde_json::to_value(&self.submission).ok()
                != serde_json::to_value(&previous.submission).ok()
            || self.input_sha256 != previous.input_sha256
            || previous.revision.0.checked_add(1) != Some(self.revision.0)
        {
            return Err("Job successor changed immutable input or revision");
        }
        let retry = self.state == JobState::Queued
            && matches!(
                previous.state,
                JobState::NeedsInput
                    | JobState::Failed
                    | JobState::Cancelled
                    | JobState::Interrupted
            );
        if retry {
            if previous.publication_started
                || previous.attempt.0.checked_add(1) != Some(self.attempt.0)
            {
                return Err("Unsafe retry of a possibly published attempt");
            }
            return Ok(());
        }
        if self.attempt != previous.attempt
            || previous.publication_started && !self.publication_started
        {
            return Err("Job successor changed attempt or publication history");
        }
        let allowed = match (previous.state, self.state) {
            (
                JobState::Queued,
                JobState::Running | JobState::Cancelling | JobState::Interrupted,
            ) => true,
            (JobState::Running, JobState::Running) => {
                !previous.publication_started && self.publication_started
            }
            (
                JobState::Running,
                JobState::Cancelling
                | JobState::Verifying
                | JobState::Failed
                | JobState::NeedsInput
                | JobState::Interrupted,
            ) => true,
            (
                JobState::Cancelling,
                JobState::Cancelled
                | JobState::Failed
                | JobState::NeedsInput
                | JobState::Interrupted,
            ) => true,
            (
                JobState::Verifying,
                JobState::Completed
                | JobState::Failed
                | JobState::NeedsInput
                | JobState::Interrupted,
            ) => true,
            _ => false,
        };
        if !allowed {
            return Err("Invalid durable job transition");
        }
        Ok(())
    }
}
