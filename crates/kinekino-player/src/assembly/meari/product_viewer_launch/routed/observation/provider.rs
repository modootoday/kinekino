use crate::{
    application::ui_observation::{
        ProviderJobsObservation, ProviderObservationV2, ProviderShutdownDiagnostic,
    },
    domain::api_values::DecimalU64,
    infrastructure::browser_provider_jobs::ProviderShutdownOutcome,
};
use kinekino_jobs::infrastructure::RetainingWorkerHost;
pub(super) fn observe(
    original: Option<&ProviderShutdownOutcome<RetainingWorkerHost>>,
) -> ProviderObservationV2 {
    let mut value = ProviderObservationV2 {
        configured: true,
        completed: None,
        pending: None,
        jobs_receipt: None,
        browser_worker_exited: None,
        first_error: None,
        secondary_errors: Vec::new(),
        secondary_overflow: None,
    };
    let errors = match original {
        Some(ProviderShutdownOutcome::Complete(completed)) => {
            value.completed = Some(true);
            value.pending = Some(false);
            if let Some(receipt) = &completed.receipt {
                value.jobs_receipt = Some(ProviderJobsObservation {
                    completed_total: DecimalU64(receipt.jobs.completed_total as u64),
                    cancelled_total: DecimalU64(receipt.jobs.cancelled_total as u64),
                    interrupted_total: DecimalU64(receipt.jobs.interrupted_total as u64),
                });
                value.browser_worker_exited = Some(receipt.browser_worker_exited);
            }
            Some(&completed.errors)
        }
        Some(ProviderShutdownOutcome::Pending(pending)) => {
            value.completed = Some(false);
            value.pending = Some(true);
            value.first_error = pending
                .original_attempt_error()
                .map(super::snapshot_api_error);
            Some(pending.errors())
        }
        None => None,
    };
    if let Some(errors) = errors {
        value.first_error = errors
            .first()
            .map(super::snapshot_api_error)
            .or(value.first_error);
        value.secondary_errors = errors
            .secondary()
            .iter()
            .map(|error| ProviderShutdownDiagnostic {
                code: error.code(),
                message: error.message().to_owned(),
                remediation: error.remediation().to_owned(),
            })
            .collect();
        value.secondary_overflow = Some(errors.overflow());
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::api::{ApiError, ErrorCode},
        infrastructure::browser_provider_jobs::{
            ProviderShutdownComplete, ProviderShutdownReceipt,
        },
    };
    use kinekino_jobs::infrastructure::{ShutdownErrors, ShutdownReceipt};
    #[test]
    fn absent_original_carrier_never_fabricates_false_overflow_or_zero_receipts() {
        let value = observe(None);
        assert!(value.configured);
        assert_eq!(value.completed, None);
        assert_eq!(value.pending, None);
        assert_eq!(value.secondary_overflow, None);
        assert!(value.jobs_receipt.is_none());
        assert_eq!(value.browser_worker_exited, None);
    }
    #[test]
    fn original_first_error_and_bounded_secondary_fields_are_projected_without_reconstruction() {
        let mut first = ApiError::new(ErrorCode::Internal, "original first");
        first.details = serde_json::json!({"original":true});
        first.retryable = true;
        first.request_id = "original-request".into();
        let mut errors = ShutdownErrors::default();
        errors.record(super::super::snapshot_api_error(&first));
        errors.record(ApiError::new(
            ErrorCode::InvalidArgument,
            "original\nsecondary",
        ));
        let original = ProviderShutdownOutcome::Complete(ProviderShutdownComplete {
            receipt: None,
            errors,
        });
        let value = observe(Some(&original));
        let observed = value.first_error.as_ref().unwrap();
        assert_eq!(observed.details, first.details);
        assert_eq!(observed.request_id, first.request_id);
        assert!(observed.retryable);
        assert_eq!(value.secondary_overflow, Some(false));
        let secondary = serde_json::to_value(&value.secondary_errors[0]).unwrap();
        assert_eq!(secondary["message"], "original\nsecondary");
        assert!(secondary.get("details").is_none());
        assert!(secondary.get("retryable").is_none());
        assert!(value.jobs_receipt.is_none());
    }
    #[test]
    fn original_overflow_flag_and_receipt_totals_remain_distinct_observations() {
        let mut errors = ShutdownErrors::default();
        for _ in 0..10 {
            errors.record(ApiError::new(ErrorCode::Internal, "original"));
        }
        let original = ProviderShutdownOutcome::Complete(ProviderShutdownComplete {
            receipt: Some(ProviderShutdownReceipt {
                jobs: ShutdownReceipt {
                    completed_total: 3,
                    cancelled_total: 1,
                    interrupted_total: 2,
                },
                browser_worker_exited: true,
            }),
            errors,
        });
        let value = observe(Some(&original));
        assert_eq!(value.secondary_errors.len(), 8);
        assert_eq!(value.secondary_overflow, Some(true));
        assert_eq!(value.jobs_receipt.unwrap().completed_total.0, 3);
        assert_eq!(value.browser_worker_exited, Some(true));
        assert!(value.first_error.is_some()); // completion is not an error-free shutdown claim
    }
}
