use super::*;
use crate::{application::owned_attempt::AttemptFailure, domain::JobFailure};
use sha2::{Digest, Sha256};
use std::io::{self, Write};

struct BoundedBytes {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for BoundedBytes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|end| end > self.limit)
        {
            return Err(io::Error::other("Owned attempt diagnostic byte limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn bounded_json(value: &serde_json::Value, limit: usize) -> bool {
    serde_json::to_writer(
        BoundedBytes {
            bytes: Vec::new(),
            limit,
        },
        value,
    )
    .is_ok()
}
fn same_output(left: &VerifiedJobOutput, right: &VerifiedJobOutput) -> bool {
    left.path == right.path
        && left.sha256 == right.sha256
        && left.bytes.0 == right.bytes.0
        && left.receipt == right.receipt
}
impl PendingOwnedAttempt {
    pub(super) fn verified(&self) -> Option<&VerifiedJobOutput> {
        match &self.stage {
            Stage::Output(output) => output.verified.as_ref(),
            _ => None,
        }
    }
    fn persist_diagnostic(&mut self) -> Result<(), ApiError> {
        if self.diagnostic_blob.is_some() {
            return Ok(());
        }
        let first = self
            .first_error
            .as_ref()
            .expect("original failure before persistence");
        let captured = AttemptFailure::capture(first);
        let first_details = if bounded_json(&first.details, 16384) {
            first.details.clone()
        } else {
            serde_json::json!({"details_omitted":true})
        };
        let secondary: Vec<_> = self
            .diagnostic
            .secondary()
            .iter()
            .map(|error| {
                serde_json::json!({
                    "code":error.code(),"message":error.message(),"remediation":error.remediation()
                })
            })
            .collect();
        let body = serde_json::json!({
            "schema_version":"owned-attempt-diagnostic-1",
            "job_id":self.control.identity.job_id(),"attempt":self.control.identity.attempt().to_string(),
            "input_sha256":self.control.identity.input_sha256(),"source_blob_sha256":self.control.identity.source_blob_sha256(),
            "submission_request_sha256":self.control.identity.submission_request_sha256(),
            "first_error":{"code":captured.code(),"message":captured.message(),
                "remediation":captured.remediation(),"retryable":first.retryable,"details":first_details,
                "reported_error_request_id":super::super::bounded_text(&first.request_id, 128)},
            "secondary":secondary,"secondary_overflow":self.diagnostic.secondary_overflow(),
            "verified_output":self.verified(),"original_deadline_exceeded":Instant::now() >= self.control.deadline,
            "panic_payload_retained":self.panic_payload.is_some(),
            "publication_permit_retained":self.rejected_permit.is_some(),
            "meaning":"diagnostic data; not settlement or publication authority"
        });
        let mut writer = BoundedBytes {
            bytes: Vec::new(),
            limit: 256 * 1024,
        };
        serde_json::to_writer(&mut writer, &body).map_err(|_| {
            ApiError::new(
                ErrorCode::ResourceExhausted,
                "Owned diagnostic exceeds bounded storage",
            )
        })?;
        let body = String::from_utf8(writer.bytes).map_err(|_| {
            ApiError::new(ErrorCode::Internal, "Owned diagnostic UTF8 encoding failed")
        })?;
        let expected = format!("{:x}", Sha256::digest(body.as_bytes()));
        let reference = self.blobs.put_utf8(&body)?;
        if reference.sha256 != expected
            || reference.bytes.0 != body.len() as u64
            || self.blobs.read_utf8(&reference)? != body
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original diagnostic bytes did not read back identically",
            ));
        }
        if !self
            .diagnostic_history
            .iter()
            .any(|old| old.sha256 == reference.sha256)
        {
            if self.diagnostic_history.len() >= 16 {
                return Err(ApiError::new(
                    ErrorCode::ResourceExhausted,
                    "Owned diagnostic revision bound reached",
                ));
            }
            self.diagnostic_history.push(reference.clone());
        }
        self.diagnostic_blob = Some(reference);
        Ok(())
    }
    pub(super) fn persist_failure(mut self: Box<Self>) -> OwnedRunOutcome {
        if !self.poll_candidate_release() {
            let result = catch_unwind(AssertUnwindSafe(|| self.persist_diagnostic()));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => self.record(error),
                Err(payload) => {
                    if self.panic_payload.is_none() {
                        self.panic_payload = Some(payload);
                    }
                    self.record(ApiError::new(
                        ErrorCode::Internal,
                        "Original pending release diagnostic persistence panicked",
                    ));
                }
            }
            return OwnedRunOutcome::Pending(self);
        }
        // Keep the original carrier outside the unwind boundary. Store ports are
        // adapters too: a panic must not drop its session or candidate.
        match catch_unwind(AssertUnwindSafe(|| self.persist_failure_step())) {
            Ok(Some(job)) => OwnedRunOutcome::Finished(Box::new(job)),
            Ok(None) => OwnedRunOutcome::Pending(self),
            Err(payload) => {
                if self.panic_payload.is_none() {
                    self.panic_payload = Some(payload);
                }
                self.record(ApiError::new(
                    ErrorCode::Internal,
                    "Original owned failure persistence panicked",
                ));
                OwnedRunOutcome::Pending(self)
            }
        }
    }
    fn persist_failure_step(&mut self) -> Option<JobRecord> {
        if let Err(error) = self.persist_diagnostic() {
            self.record(error);
            return None;
        }
        let current = match self.control.current() {
            Ok(current) => current,
            Err(error) => {
                self.record(error);
                return None;
            }
        };
        if current.state == JobState::Completed
            && current
                .output
                .as_ref()
                .zip(self.verified())
                .is_some_and(|(left, right)| same_output(left, right))
        {
            return Some(current);
        }
        if current.state.terminal() {
            let ours = current
                .failure
                .as_ref()
                .and_then(|failure| failure.details.get("owned_attempt_diagnostic"))
                .is_some_and(|reference| {
                    self.diagnostic_history.iter().any(|known| {
                        serde_json::to_value(known).is_ok_and(|value| &value == reference)
                    })
                });
            if ours {
                return Some(current);
            }
            self.record(ApiError::new(
                ErrorCode::RevisionConflict,
                "Terminal record does not match this original owned outcome",
            ));
            return None;
        }
        let first = self
            .first_error
            .as_ref()
            .expect("immutable original failure");
        let needs_input = matches!(
            first.code,
            ErrorCode::NeedsInput
                | ErrorCode::SourceExpired
                | ErrorCode::NotFound
                | ErrorCode::RevisionConflict
                | ErrorCode::PermissionDenied
                | ErrorCode::Unsupported
                | ErrorCode::InvalidArgument
        );
        let message = if first.message.trim().is_empty() {
            "Owned execution failed"
        } else {
            &first.message
        };
        let failure = JobFailure {
            code: first.code,
            message: super::super::bounded_text(message, 4096),
            remediation: super::super::bounded_text(&first.remediation, 256),
            details: serde_json::json!({"owned_attempt_diagnostic":self.diagnostic_blob.as_ref().expect("read-back diagnostic blob")}),
            retryable: first.retryable,
        };
        match self
            .control
            .change(|job| job.fail(failure.clone(), needs_input))
        {
            Ok(job) => Some(job),
            Err(error) => {
                self.record(error);
                None
            }
        }
    }
}
