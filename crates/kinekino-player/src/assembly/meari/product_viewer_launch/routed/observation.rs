//! Original cleanup carriers projected to data only; no owner is consumed or polled here.
mod provider;
mod viewer;
use super::ProductPlaybackOutcome;
use crate::{
    application::ui_observation::*, domain::api_values::DecimalU64,
    infrastructure::thread_completion::DispatcherShutdown,
};
use std::io::Write;

/// An observation owns exact field values; the original non-Clone SDK error stays in its owner.
fn snapshot_api_error(
    original: &crate::application::api::ApiError,
) -> crate::application::api::ApiError {
    crate::application::api::ApiError {
        code: original.code,
        message: original.message.clone(),
        retryable: original.retryable,
        details: original.details.clone(),
        remediation: original.remediation.clone(),
        request_id: original.request_id.clone(),
    }
}

impl ProductPlaybackOutcome {
    pub(crate) fn native_routed_observation(&self) -> NativeRoutedFinalV2 {
        let transition = self.original_transition_lifetime();
        let ended = transition.end_result();
        let mut record = NativeRoutedFinalV2 {
            schema_version: 2,
            native_closed: None,
            viewer_workers_quiescent: None,
            execution: ExecutionObservation {
                worker_stopped: None,
                unacknowledged: None,
            },
            local_host: LocalHostObservation {
                pipe: JoinObservation::Unobserved,
                sdk: JoinObservation::Unobserved,
                dispatch: DispatchObservation::Unobserved,
            },
            transition: TransitionObservation {
                configured: true,
                admission_closed: Some(transition.admission_closed()),
                ended: ended.map(Result::is_ok),
                remaining_job_id: ended
                    .and_then(|result| result.as_ref().ok())
                    .and_then(|job| job.as_ref())
                    .map(|job| job.as_str().to_owned()),
                error: ended
                    .and_then(|result| result.as_ref().err())
                    .map(snapshot_api_error),
            },
            provider: provider::observe(self.original_provider_shutdown()),
            dispatcher: DispatcherObservation {
                joined: None,
                worker_panicked: None,
                faults: None,
            },
            cleanup_complete: self.succeeded(),
            first_error: None,
            diagnostic_write_error: None,
        };
        if let Some(retained) = &self.retained {
            record.first_error = Some(FailureObservation {
                phase: "routed_setup".into(),
                message: retained.diagnostic.clone(),
                sdk: None,
            });
        } else if let Some(error) = &self.pending_close_error {
            record.first_error = Some(FailureObservation {
                phase: "pending_native_close".into(),
                message: error.clone(),
                sdk: None,
            });
        }
        viewer::observe(self.original_viewer_outcome(), &mut record);
        if let Some(dispatcher) = self.original_dispatcher_shutdown() {
            match dispatcher {
                DispatcherShutdown::Joined {
                    worker_panicked,
                    faults,
                } => {
                    record.dispatcher.joined = Some(true);
                    record.dispatcher.worker_panicked = Some(*worker_panicked);
                    record.dispatcher.faults = Some(DecimalU64(*faults as u64));
                }
                DispatcherShutdown::Pending | DispatcherShutdown::AdmissionOpen => {
                    record.dispatcher.joined = Some(false)
                }
            }
        }
        if record.first_error.is_none() {
            let original = record
                .transition
                .error
                .as_ref()
                .map(|error| ("transition_end", error))
                .or_else(|| {
                    record
                        .provider
                        .first_error
                        .as_ref()
                        .map(|error| ("provider_shutdown", error))
                });
            if let Some((phase, error)) = original {
                record.first_error = Some(FailureObservation {
                    phase: phase.into(),
                    message: error.message.clone(),
                    sdk: Some(snapshot_api_error(error)),
                });
            }
        }
        record
    }
    /// Same original report; typed V2 is borrowed into the bounded final envelope.
    pub fn write_report(
        &self,
        recording: bool,
        elapsed_ms: u128,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> super::super::ViewerDiagnosticFailures {
        let mut report = self.report();
        let mut failures = super::super::write_viewer_report(
            &report,
            self.succeeded(),
            false,
            elapsed_ms,
            stdout,
            stderr,
        );
        if recording {
            failures.stdout = (|| {
                let legacy = report.as_object_mut().ok_or_else(|| {
                    std::io::Error::other("Original viewer report is not an object")
                })?;
                match viewer::frame_snapshot(self.original_viewer_outcome()) {
                    Ok(snapshot) => {
                        let trace = snapshot.and_then(|snapshot| snapshot.trace);
                        legacy.insert(
                            "routed_frame_trace".into(),
                            serde_json::to_value(trace.as_deref()).map_err(std::io::Error::from)?,
                        );
                    }
                    Err(error) => {
                        legacy.insert(
                            "routed_frame_trace_error".into(),
                            serde_json::Value::String(error.to_string()),
                        );
                    }
                }
                let encoded = encoding::final_outcome(
                    &report,
                    &self.native_routed_observation(),
                    elapsed_ms,
                )?;
                encoding::write_line(&encoded, stdout)
            })()
            .err();
        }
        failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observation_snapshot_preserves_all_six_fields_and_keeps_original_error_owned() {
        let original = std::sync::Arc::new(crate::application::api::ApiError {
            code: crate::application::api::ErrorCode::InvalidArgument,
            message: "원본\nmessage".into(),
            retryable: true,
            details: serde_json::json!({"nested":[true, {"source":"original"}]}),
            remediation: "원본 remediation".into(),
            request_id: "retained-original-request".into(),
        });
        let original_alias = original.clone();
        let mut snapshot = snapshot_api_error(original.as_ref());
        assert_eq!(
            serde_json::to_value(&snapshot).unwrap(),
            serde_json::json!({
                "code":"invalid_argument", "message":"원본\nmessage", "retryable":true,
                "details":{"nested":[true,{"source":"original"}]},
                "remediation":"원본 remediation", "request_id":"retained-original-request"
            })
        );
        snapshot.details["nested"][0] = serde_json::Value::Bool(false);
        snapshot.message.push('!');
        assert_eq!(original.details["nested"][0], true);
        assert_eq!(original.message, "원본\nmessage");
        assert!(std::sync::Arc::ptr_eq(&original, &original_alias));
        assert_eq!(original.request_id, "retained-original-request");
    }
}
