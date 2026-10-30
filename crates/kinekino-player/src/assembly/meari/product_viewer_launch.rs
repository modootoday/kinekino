//! Moves the configured product client and endpoint through the existing Viewer owners once.
use super::{
    native_cleanup_driver::CleanupDriverOutcome,
    native_execution_startup::{NativeExecutionFailure, prepare_native_execution},
    player_shell_document::DocumentOptions,
    player_shell_host::ShellHostConfig,
    viewer_cleanup::{
        TypedViewerCleanup, TypedViewerOwners, ViewerCleanupLimits, ViewerCleanupStatus,
        ViewerNativeState, ViewerNoChild,
    },
    viewer_native_launch::{ViewerLaunchInput, finish_viewer, prepare_product_viewer_launch},
};
mod project;
pub mod routed;
use crate::{
    application::{api::ApplicationClientHandle, sdk_execution::ExecutionBudget},
    infrastructure::local_host::LocalHost,
    presentation::web_ui_adapter::player_shell::document::StartLabels,
};
use meari_native::{
    application::{
        WindowOptions,
        window_lifecycle::{LifecycleLimits, NativeChild},
    },
    domain::Viewport,
    infrastructure::native_host::{HostLifecycle, HostScheduleLimits},
};
pub use project::{SelectedViewerProject, prepare_project};
use serde_json::{Value, json};
use std::{
    io::{self, Write},
    time::Duration,
};

#[derive(Debug, Default)]
pub struct ViewerDiagnosticFailures {
    pub stdout: Option<io::Error>,
    pub stderr: Option<io::Error>,
}
/// Secondary output only: this function never consumes owners or decides the primary exit.
pub fn write_viewer_event(
    kind: &str,
    event: &Value,
    recording: bool,
    elapsed_ms: u128,
    elapsed_scope: &str,
    stdout: &mut dyn Write,
) -> Option<io::Error> {
    if !recording {
        return None;
    }
    (|| -> io::Result<()> {
        serde_json::to_writer(
            &mut *stdout,
            &json!({
                "kind":kind,"elapsed_ms":elapsed_ms,"elapsed_scope":elapsed_scope,"data":event,
            }),
        )
        .map_err(io::Error::from)?;
        stdout.write_all(b"\n")?;
        stdout.flush()
    })()
    .err()
}
/// Secondary output only: this function never consumes owners or decides the primary exit.
pub fn write_viewer_report(
    report: &Value,
    succeeded: bool,
    recording: bool,
    elapsed_ms: u128,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> ViewerDiagnosticFailures {
    let stdout_error = write_viewer_event(
        "native-viewer-outcome",
        report,
        recording,
        elapsed_ms,
        "native-viewer",
        stdout,
    );
    let stderr_error = if succeeded {
        None
    } else {
        writeln!(stderr, "{report}")
            .and_then(|()| stderr.flush())
            .err()
    };
    ViewerDiagnosticFailures {
        stdout: stdout_error,
        stderr: stderr_error,
    }
}

pub struct ViewerStartFailure {
    endpoint: LocalHost,
    execution: Box<NativeExecutionFailure>,
}
#[must_use = "Retain original endpoint/native/Viewer owners through the explicit exit decision"]
pub enum ProductViewerOutcome<C: NativeChild = ViewerNoChild> {
    StartFailure(Box<ViewerStartFailure>),
    Finished(Box<CleanupDriverOutcome<TypedViewerCleanup<C>>>),
}
impl<C: NativeChild> ProductViewerOutcome<C> {
    pub fn succeeded(&self) -> bool {
        match self {
            Self::StartFailure(_) => false,
            Self::Finished(cleanup) => success_gate(
                &cleanup.status,
                returned_closed(&cleanup.owners),
                cleanup.evidence.error.is_some(),
                cleanup.acceptance_error.is_some(),
            ),
        }
    }
    pub fn report(&self) -> Value {
        match self {
            Self::StartFailure(failure) => json!({
                "success":false,"stage":"execution_start","cause":format!("{:?}",failure.execution.cause),
                "endpoint_retained":true,"endpoint_running":failure.endpoint.is_running(),
                "loop_retained":failure.execution.loop_owner.is_some(),
                "startup_retained":failure.execution.startup.is_some(),"normal_os_exit":false,
            }),
            Self::Finished(cleanup) => {
                let capture_evidence = match &cleanup.owners {
                    TypedViewerOwners::Returned(outcome) => outcome.runtime.as_ref().or(outcome.controller.runtime.as_ref())
                        .and_then(|runtime| runtime.try_borrow().ok().and_then(|runtime| runtime.host().host().captured_evidence_status()))
                        .map(|status| json!({"saved":status.saved,"pending":status.pending,"last_saved":status.last_saved,"first_error":status.first_error.as_deref()})),
                    _ => None,
                };
                let owners = match &cleanup.owners {
                    TypedViewerOwners::ShellFailure { .. } => {
                        json!({"stage":"shell_prepare","owners_retained":true})
                    }
                    TypedViewerOwners::DocumentFailure { failure, .. } => {
                        json!({"stage":"document_prepare","document_stage":format!("{:?}",failure.stage),"owners_retained":true})
                    }
                    TypedViewerOwners::Unrun { .. } => {
                        json!({"stage":"unrun","owners_retained":true})
                    }
                    TypedViewerOwners::Returned(outcome) => json!({
                        "stage":"native_return","result":format!("{:?}",outcome.result),
                        "loop_retained":outcome.loop_owner.is_some(),"startup_retained":outcome.startup.is_some(),
                        "runtime_retained":outcome.runtime.is_some(),"recovery_retained":outcome.recovery.is_some(),
                    }),
                };
                json!({"success":self.succeeded(),"owners":owners,"cleanup":format!("{:?}",cleanup.status),
                    "observations":cleanup.evidence.observations,
                    "workers":format!("{:?}",cleanup.evidence.last_workers),
                    "error":format!("{:?}",cleanup.evidence.error),
                    "acceptance_error":format!("{:?}",cleanup.acceptance_error),
                    "normal_os_exit":returned_closed(&cleanup.owners),"media_started":false,"capture_evidence":capture_evidence})
            }
        }
    }
}
fn returned_closed<C: NativeChild>(owners: &TypedViewerOwners<C>) -> bool {
    let TypedViewerOwners::Returned(outcome) = owners else {
        return false;
    };
    outcome.result.is_ok()
        && outcome
            .runtime
            .as_ref()
            .or(outcome.controller.runtime.as_ref())
            .and_then(|runtime| {
                runtime
                    .try_borrow()
                    .ok()
                    .map(|runtime| runtime.document().lifecycle())
            })
            == Some(HostLifecycle::Closed)
}
fn success_gate(
    status: &ViewerCleanupStatus,
    returned_closed: bool,
    error: bool,
    acceptance_error: bool,
) -> bool {
    *status == ViewerCleanupStatus::WorkersQuiescent(ViewerNativeState::Closed)
        && returned_closed
        && !error
        && !acceptance_error
}

pub fn run_product_viewer(
    client: ApplicationClientHandle,
    endpoint: LocalHost,
) -> ProductViewerOutcome {
    run_product_viewer_with_project(client, endpoint, None)
}
pub fn run_product_viewer_with_project(
    client: ApplicationClientHandle,
    endpoint: LocalHost,
    project: Option<SelectedViewerProject>,
) -> ProductViewerOutcome {
    let input = match prepare_product_input(client, endpoint) {
        Ok(input) => input,
        Err(failure) => return ProductViewerOutcome::StartFailure(failure),
    };
    run_prepared_product(input, project)
}
fn prepare_product_input(
    client: ApplicationClientHandle,
    endpoint: LocalHost,
) -> Result<ViewerLaunchInput, Box<ViewerStartFailure>> {
    let schedule =
        HostScheduleLimits::new(Duration::from_millis(2), Duration::from_millis(250), 1024)
            .expect("fixed Viewer schedule limits");
    let execution = match prepare_native_execution(
        client.clone(),
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 256 * 1024,
            max_total_request_bytes: 8 * 256 * 1024,
            max_terminal_bytes: 256 * 1024,
            max_total_terminal_bytes: 8 * 256 * 1024,
        },
        schedule,
    ) {
        Ok(execution) => execution,
        Err(execution) => {
            return Err(Box::new(ViewerStartFailure {
                endpoint,
                execution,
            }));
        }
    };
    Ok(ViewerLaunchInput {
        client,
        endpoint,
        execution,
        labels: StartLabels {
            connection: "연결 상태 관측 없음".into(),
            time: "시간 관측 없음".into(),
        },
        document: DocumentOptions {
            namespace: "product-viewer".into(),
            epoch: 1,
            source_generation: 1,
            viewport: Viewport {
                width: 1180,
                height: 760,
                dark: true,
            },
        },
        host: ShellHostConfig {
            options: WindowOptions::default(),
            schedule,
            children: LifecycleLimits::new(1, Duration::from_secs(2))
                .expect("fixed Viewer child limits"),
        },
    })
}
fn run_prepared_product(
    input: ViewerLaunchInput,
    project: Option<SelectedViewerProject>,
) -> ProductViewerOutcome {
    let prepared = match project {
        Some(project) => {
            super::viewer_native_launch::prepare_product_viewer_launch_with_captured_evidence(
                input,
                super::viewer_native_launch::CapturedProductViewerChoices {
                    stored: project.stored,
                    generation: std::num::NonZeroU64::new(1).expect("initial choice generation"),
                    capacity: std::num::NonZeroUsize::new(8).expect("fixed choice capacity"),
                },
                project.artifacts,
            )
        }
        None => prepare_product_viewer_launch(input),
    };
    let owners = match prepared {
        Ok(prepared) => prepared.run(),
        Err(owners) => *owners,
    };
    ProductViewerOutcome::Finished(Box::new(finish_viewer(
        owners,
        ViewerCleanupLimits::new(Duration::from_secs(2), 500, Duration::from_millis(2))
            .expect("fixed Viewer cleanup limits"),
    )))
}

#[cfg(test)]
mod tests;
