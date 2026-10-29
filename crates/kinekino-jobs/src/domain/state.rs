use super::{JobId, PreparedJobInput, valid_sha256};
use kinekino_media_contracts::{api_values::DecimalU64, errors::ErrorCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Cancelling,
    Verifying,
    Completed,
    NeedsInput,
    Failed,
    Cancelled,
    Interrupted,
}
impl JobState {
    pub fn terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::NeedsInput | Self::Failed | Self::Cancelled | Self::Interrupted
        )
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobFailure {
    pub code: ErrorCode,
    pub message: String,
    pub remediation: String,
    pub details: Value,
    pub retryable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedJobOutput {
    pub path: String,
    pub sha256: String,
    pub bytes: DecimalU64,
    pub receipt: Value,
}
impl VerifiedJobOutput {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.path.trim().is_empty()
            || self.path.len() > 32768
            || self.path.chars().any(char::is_control)
            || !valid_sha256(&self.sha256)
            || !self.receipt.is_object()
            || serde_json::to_vec(&self.receipt)
                .map_err(|_| "Invalid job receipt")?
                .len()
                > 32768
        {
            return Err("Invalid verified output receipt");
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobRecord {
    pub schema_version: String,
    pub job_id: JobId,
    pub revision: DecimalU64,
    pub attempt: DecimalU64,
    pub input: PreparedJobInput,
    pub input_sha256: String,
    pub submission: Option<SubmissionRef>,
    pub state: JobState,
    pub cancel_requested: bool,
    pub publication_started: bool,
    pub output: Option<VerifiedJobOutput>,
    pub failure: Option<JobFailure>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmissionRef {
    pub key_sha256: String,
    pub request_sha256: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancelDisposition {
    Requested,
    AlreadyTerminal,
    PublicationInProgress,
}
impl JobRecord {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != "1"
            || self.revision.0 == 0
            || self.attempt.0 == 0
            || self.attempt.0 > self.revision.0
            || self.input.sha256()? != self.input_sha256
            || self.submission.as_ref().is_some_and(|submission| {
                !valid_sha256(&submission.key_sha256) || !valid_sha256(&submission.request_sha256)
            })
            || (self.state == JobState::Completed) != self.output.is_some()
            || matches!(
                self.state,
                JobState::NeedsInput | JobState::Failed | JobState::Interrupted
            ) != self.failure.is_some()
            || matches!(self.state, JobState::Cancelling | JobState::Cancelled)
                && !self.cancel_requested
            || matches!(
                self.state,
                JobState::Queued | JobState::Running | JobState::Verifying | JobState::Completed
            ) && self.cancel_requested
            || matches!(self.state, JobState::Verifying | JobState::Completed)
                && !self.publication_started
            || matches!(
                self.state,
                JobState::Queued | JobState::Cancelling | JobState::Cancelled
            ) && self.publication_started
        {
            return Err("Invalid durable job state");
        }
        if let Some(output) = &self.output {
            output.validate()?;
        }
        if self.failure.as_ref().is_some_and(|failure| {
            failure.message.len() > 4096
                || failure.message.trim().is_empty()
                || failure.remediation.len() > 256
                || !serde_json::to_vec(&failure.details).is_ok_and(|bytes| bytes.len() <= 16384)
        }) || serde_json::to_vec(self)
            .map_err(|_| "Invalid durable job JSON")?
            .len()
            > 131072
        {
            return Err("Job record exceeds storage limits");
        }
        Ok(())
    }
    pub fn queued(job_id: JobId, input: PreparedJobInput) -> Result<Self, &'static str> {
        let input_sha256 = input.sha256()?;
        Ok(Self {
            schema_version: "1".into(),
            job_id,
            revision: DecimalU64(1),
            attempt: DecimalU64(1),
            input,
            input_sha256,
            submission: None,
            state: JobState::Queued,
            cancel_requested: false,
            publication_started: false,
            output: None,
            failure: None,
        })
    }
    fn advance(&mut self) -> Result<(), &'static str> {
        self.revision.0 = self
            .revision
            .0
            .checked_add(1)
            .ok_or("Job revision overflow")?;
        Ok(())
    }
    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.state != JobState::Queued {
            return Err("Only queued jobs can start");
        }
        self.advance()?;
        self.state = JobState::Running;
        Ok(())
    }
    pub fn request_cancel(&mut self) -> Result<CancelDisposition, &'static str> {
        if self.state.terminal() {
            return Ok(CancelDisposition::AlreadyTerminal);
        }
        if self.publication_started {
            return Ok(CancelDisposition::PublicationInProgress);
        }
        if !self.cancel_requested {
            self.advance()?;
            self.cancel_requested = true;
            self.state = JobState::Cancelling;
        }
        Ok(CancelDisposition::Requested)
    }
    pub fn acknowledge_cancel(&mut self) -> Result<(), &'static str> {
        if self.state != JobState::Cancelling || !self.cancel_requested || self.publication_started
        {
            return Err("Cancellation requires worker acknowledgement before publication");
        }
        self.advance()?;
        self.state = JobState::Cancelled;
        Ok(())
    }
    /// Called under the same application gate as cancellation, before publishing any output.
    pub fn begin_publication(&mut self) -> Result<(), &'static str> {
        if self.state != JobState::Running || self.cancel_requested || self.publication_started {
            return Err("Job cannot enter publication");
        }
        self.advance()?;
        self.publication_started = true;
        Ok(())
    }
    pub fn begin_verification(&mut self) -> Result<(), &'static str> {
        if self.state != JobState::Running || !self.publication_started {
            return Err("Verification requires publication");
        }
        self.advance()?;
        self.state = JobState::Verifying;
        Ok(())
    }
    pub fn complete(&mut self, output: VerifiedJobOutput) -> Result<(), &'static str> {
        output.validate()?;
        if self.state != JobState::Verifying
            || !self.publication_started
            || !valid_sha256(&output.sha256)
            || output.path.trim().is_empty()
            || output.path.len() > 32768
            || !output.receipt.is_object()
        {
            return Err("Completion requires a validated output receipt");
        }
        self.advance()?;
        self.output = Some(output);
        self.state = JobState::Completed;
        Ok(())
    }
    pub fn fail(&mut self, failure: JobFailure, needs_input: bool) -> Result<(), &'static str> {
        if !matches!(
            self.state,
            JobState::Running | JobState::Cancelling | JobState::Verifying
        ) {
            return Err("Only executing jobs can fail");
        }
        self.advance()?;
        self.failure = Some(failure);
        self.state = if needs_input {
            JobState::NeedsInput
        } else {
            JobState::Failed
        };
        Ok(())
    }
    /// Only an execution owner that acquired exclusive startup ownership may call this.
    pub fn interrupt(&mut self) -> Result<bool, &'static str> {
        if self.state.terminal() {
            return Ok(false);
        }
        self.advance()?;
        self.state = JobState::Interrupted;
        self.failure = Some(JobFailure {
            code: ErrorCode::EngineFailed,
            message: "Execution host stopped before a terminal receipt".into(),
            remediation: "inspect_interrupted_job".into(),
            details: serde_json::json!({"automatic_retry":false}),
            retryable: false,
        });
        Ok(true)
    }
    pub fn retry(&mut self) -> Result<(), &'static str> {
        if !matches!(
            self.state,
            JobState::NeedsInput | JobState::Failed | JobState::Cancelled | JobState::Interrupted
        ) || self.publication_started
        {
            return Err("Retry requires an explicitly recoverable non-completed attempt");
        }
        let attempt = self
            .attempt
            .0
            .checked_add(1)
            .ok_or("Job attempt overflow")?;
        self.advance()?;
        self.attempt = DecimalU64(attempt);
        self.state = JobState::Queued;
        self.cancel_requested = false;
        self.failure = None;
        self.output = None;
        Ok(())
    }
    pub fn recovery_required(&self) -> bool {
        self.publication_started
            && matches!(
                self.state,
                JobState::Failed | JobState::NeedsInput | JobState::Interrupted
            )
    }
}
