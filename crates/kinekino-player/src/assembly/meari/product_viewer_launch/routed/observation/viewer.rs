use crate::{
    application::{
        sdk_execution::{ExecutionCloseProgress, ExecutionCloseReceipt, WorkerCloseState},
        ui_observation::*,
    },
    assembly::meari::{
        native_execution_startup::NativeExecutionStartError,
        playback_child::routed::RegisteredRoutedChild,
        product_viewer_launch::{ProductViewerOutcome, returned_closed},
        viewer_cleanup::{TypedViewerOwners, ViewerCleanupError, ViewerNativeState},
        viewer_host::{ViewerCloseError, ViewerCloseProgress, ViewerHostError},
    },
    domain::api_values::DecimalU64,
    infrastructure::local_host::{DispatchState, LocalHostShutdownStatus, WorkerJoin},
};

pub(super) fn frame_snapshot(
    original: &ProductViewerOutcome<RegisteredRoutedChild>,
) -> Result<
    Option<crate::assembly::meari::player_shell_host::RoutedFrameObservation>,
    crate::assembly::meari::player_shell_host::FrameHookError,
> {
    let cleanup = match original {
        ProductViewerOutcome::StartFailure(_) => return Ok(None),
        ProductViewerOutcome::Finished(cleanup) => cleanup,
    };
    match &cleanup.owners {
        TypedViewerOwners::Returned(outcome) => outcome.controller.routed_frame_observation(),
        TypedViewerOwners::Unrun { controller, .. } => controller.routed_frame_observation(),
        _ => Ok(None),
    }
}

fn join(value: WorkerJoin) -> JoinObservation {
    match value {
        WorkerJoin::Pending => JoinObservation::Pending,
        WorkerJoin::Joined => JoinObservation::Joined,
        WorkerJoin::Panicked => JoinObservation::Panicked,
    }
}
fn endpoint(value: &LocalHostShutdownStatus, record: &mut NativeRoutedFinalV2) {
    record.local_host = LocalHostObservation {
        pipe: join(value.pipe),
        sdk: join(value.sdk),
        dispatch: match value.dispatch {
            None => DispatchObservation::Poisoned,
            Some(DispatchState::Idle) => DispatchObservation::Idle,
            Some(DispatchState::Accepted) => DispatchObservation::Accepted,
            Some(DispatchState::Running) => DispatchObservation::Running,
            Some(DispatchState::Completed) => DispatchObservation::Completed,
        },
    };
}
fn receipt(value: &ExecutionCloseReceipt, record: &mut NativeRoutedFinalV2) {
    record.execution.worker_stopped = Some(value.worker_stopped);
    record.execution.unacknowledged = Some(DecimalU64(value.unacknowledged as u64));
}
fn cleanup_sdk(error: &ViewerCleanupError) -> Option<crate::application::api::ApiError> {
    match error {
        ViewerCleanupError::Host(
            ViewerHostError::Sdk(error) | ViewerHostError::Close(ViewerCloseError::Sdk(error)),
        ) => Some(super::snapshot_api_error(error.as_ref())),
        _ => None,
    }
}
pub(super) fn observe(
    original: &ProductViewerOutcome<RegisteredRoutedChild>,
    record: &mut NativeRoutedFinalV2,
) {
    let cleanup = match original {
        ProductViewerOutcome::StartFailure(failure) => {
            record
                .first_error
                .get_or_insert_with(|| FailureObservation {
                    phase: "viewer_execution_start".into(),
                    message: format!("{:?}", failure.execution.cause),
                    sdk: match &failure.execution.cause {
                        NativeExecutionStartError::Sdk(error) => {
                            Some(super::snapshot_api_error(error))
                        }
                        _ => None,
                    },
                });
            return;
        }
        ProductViewerOutcome::Finished(cleanup) => cleanup,
    };
    record.native_closed = cleanup
        .evidence
        .last_native
        .map(|native| native == ViewerNativeState::Closed && returned_closed(&cleanup.owners));
    if let Some(error) = &cleanup.evidence.error {
        record
            .first_error
            .get_or_insert_with(|| FailureObservation {
                phase: "viewer_cleanup".into(),
                message: format!("{error:?}"),
                sdk: cleanup_sdk(error),
            });
    }
    match &cleanup.evidence.last_workers {
        Some(ViewerCloseProgress::Complete {
            endpoint: status,
            execution,
        }) => {
            endpoint(status, record);
            receipt(execution, record);
            record.viewer_workers_quiescent = Some(
                status.is_quiescent() && execution.worker_stopped && execution.unacknowledged == 0,
            );
        }
        Some(ViewerCloseProgress::EndpointPending(status)) => {
            endpoint(status, record);
            record.viewer_workers_quiescent = Some(false);
        }
        Some(ViewerCloseProgress::ExecutionPending(progress)) => {
            record.viewer_workers_quiescent = Some(false);
            match progress {
                ExecutionCloseProgress::Closed(value) => receipt(value, record),
                ExecutionCloseProgress::TerminalAckPending {
                    unacknowledged,
                    worker,
                }
                | ExecutionCloseProgress::Faulted {
                    unacknowledged,
                    worker,
                    ..
                } => {
                    record.execution.unacknowledged = Some(DecimalU64(*unacknowledged as u64));
                    record.execution.worker_stopped =
                        Some(matches!(worker, WorkerCloseState::Joined));
                }
                ExecutionCloseProgress::WorkerExitPending { .. } => {
                    record.execution.worker_stopped = Some(false)
                }
            }
            if let ExecutionCloseProgress::Faulted { error, .. } = progress {
                record
                    .first_error
                    .get_or_insert_with(|| FailureObservation {
                        phase: "viewer_execution_close".into(),
                        message: error.message.clone(),
                        sdk: Some(super::snapshot_api_error(error.as_ref())),
                    });
            }
        }
        Some(
            ViewerCloseProgress::CallbackPending { .. }
            | ViewerCloseProgress::CardScopePending
            | ViewerCloseProgress::JobsScopePending,
        ) => record.viewer_workers_quiescent = Some(false),
        None => (),
    }
    let observation = match &cleanup.owners {
        TypedViewerOwners::Returned(outcome) => Some(outcome.controller.routed_frame_observation()),
        TypedViewerOwners::Unrun { controller, .. } => Some(controller.routed_frame_observation()),
        _ => None,
    };
    record.diagnostic_write_error = observation.and_then(|result| match result {
        Ok(snapshot) => {
            snapshot.and_then(|snapshot| snapshot.diagnostic_error.map(|error| error.to_string()))
        }
        Err(error) => Some(error.to_string()),
    });
}
