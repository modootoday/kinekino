//! One existing configured client, one execution owner and one Native loop.
use super::{
    product_native_cleanup::{ProductCleanupOutcome, finish_owned_product},
    proposal_window_startup::{ProposalExecutionFailure, prepare_proposal_execution},
};
use crate::{
    application::{
        api::ApplicationClientHandle, launch::native_proposal::NativeProposalLaunchRequest,
        sdk_execution::ExecutionBudget,
    },
    infrastructure::{diagnostics, meari::product_styles::PreparedProductStyles},
    presentation::web_ui_adapter::proposals::{
        frame_composition::FrameComposition,
        inspector_state::{InspectorLayout, InspectorViewState},
        product_window::*,
        window_cleanup::*,
        window_startup::{ProposalRuntime, ProposalWindowStartup, WindowStartupError},
    },
};

pub struct PreparedNativeLaunch {
    request: NativeProposalLaunchRequest,
    styles: PreparedProductStyles,
}
impl PreparedNativeLaunch {
    pub fn run(self, client: ApplicationClientHandle) -> ProductNativeOutcome {
        run_product_native(client, self.request, self.styles)
    }
}
pub fn prepare_native_launch(
    args: &[String],
) -> Result<PreparedNativeLaunch, crate::application::api::ApiError> {
    use crate::application::{
        api::{ApiError, ErrorCode},
        launch::native_proposal::{decode_native_proposal_request, native_proposal_options},
    };
    let options = native_proposal_options(args)?.ok_or_else(|| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Native proposal options are required",
        )
    })?;
    let bytes = crate::infrastructure::native_proposal_request::read_native_proposal_request(
        &options.request_path,
    )?;
    let request = decode_native_proposal_request(&bytes)?;
    let styles = crate::infrastructure::meari::product_styles::prepare_product_styles()
        .map_err(|error| ApiError::new(ErrorCode::EngineFailed, error))?;
    diagnostics::record(
        "native-proposal-input",
        json!({"request_sha256":meari::domain::build::ContentHash::of(&bytes),"request_bytes":bytes.len(),"context":request.context,"selection":request.selection}),
    );
    Ok(PreparedNativeLaunch { request, styles })
}
use meari_native::{
    domain::Viewport,
    infrastructure::{
        native_host::{HostScheduleLimits, NativeDocumentHandle},
        window::*,
    },
};
use serde_json::{Value, json};
use std::{cell::RefCell, rc::Rc, time::Duration};

fn prepare_launch_window(
    client: ApplicationClientHandle,
    execution: crate::application::sdk_execution::ExecutionOwner,
    request: NativeProposalLaunchRequest,
    config: ProductWindowConfig,
) -> Result<ProposalWindowStartup, ProductWindowBuildFailure> {
    let frame = FrameComposition::new(
        crate::domain::product::PRODUCT.display_name.into(),
        "스튜디오".into(),
    )
    .with_inspector(
        "제안 검토".into(),
        InspectorViewState::new(InspectorLayout::Wide, false),
    );
    prepare_product_frame_window(client, execution, request, config, frame)
}

#[must_use = "Retain original failures and cleanup authority until the explicit process exit decision"]
pub enum ProductNativeOutcome {
    StartFailure(Box<ProposalExecutionFailure>),
    Finished {
        cleanup: Box<ProductCleanupOutcome>,
        notices: Vec<NativeWindowNoticeKind>,
        unrun_native: Option<Box<(NativeLoopOwner, NativeStartup)>>,
    },
}
impl ProductNativeOutcome {
    pub fn succeeded(&self) -> bool {
        match self {
            Self::Finished {
                cleanup, notices, ..
            } => {
                matches!(
                    cleanup.status,
                    CleanupStatus::SdkQuiescent {
                        native: NativeCleanupState::Closed
                    }
                ) && cleanup.acceptance_error.is_none()
                    && matches!(&cleanup.owners, ProductWindowOwners::Returned(outcome) if outcome.result.is_ok())
                    && notices.contains(&NativeWindowNoticeKind::FrameSubmitted)
            }
            Self::StartFailure(_) => false,
        }
    }
    pub fn report(&self) -> Value {
        match self {
            Self::StartFailure(failure) => {
                json!({"success":false,"stage":"execution_start","cause":format!("{:?}",failure.cause),"loop_retained":failure.loop_owner.is_some(),"startup_retained":failure.startup.is_some(),"normal_os_exit":false})
            }
            Self::Finished {
                cleanup,
                notices,
                unrun_native,
            } => {
                let mut original = match &cleanup.owners {
                    ProductWindowOwners::Returned(outcome) => {
                        json!({"stage":"native_return","result":format!("{:?}",outcome.result),"startup_retained":outcome.startup.is_some(),"runtime_retained":outcome.runtime.is_some(),"recovery_retained":outcome.recovery.is_some(),"loop_retained":outcome.loop_owner.is_some(),"sdk_error":outcome.controller.sdk_error,"mount_failure":outcome.controller.mount_failure.is_some(),"bridge_failure":outcome.controller.bridge_failure.is_some(),"attach_failure":outcome.controller.attach_failure.is_some(),"preparation_error":outcome.controller.preparation_error.is_some()})
                    }
                    ProductWindowOwners::BuildFailure(failure) => match failure.as_ref() {
                        ProductWindowBuildFailure::Input(failure) => {
                            json!({"stage":"ui_input","error":failure.error})
                        }
                        ProductWindowBuildFailure::Configuration(_) => {
                            json!({"stage":"ui_configuration","input_retained":true})
                        }
                    },
                    ProductWindowOwners::Headless(_) => json!({"stage":"headless"}),
                };
                if let ProductWindowOwners::Returned(outcome) = &cleanup.owners {
                    let controller = &outcome.controller;
                    original["initial_error"] = json!(
                        controller
                            .initial_failure
                            .as_deref()
                            .and_then(|result| result.as_ref().err())
                    );
                    original["mount_cause"] = json!(
                        controller
                            .mount_failure
                            .as_ref()
                            .map(|failure| format!("{:?}", failure.factory.cause))
                    );
                    original["bridge_cause"] = json!(
                        controller
                            .bridge_failure
                            .as_ref()
                            .map(|failure| format!("{:?}", failure.error))
                    );
                    original["attach_cause"] = json!(
                        controller
                            .attach_failure
                            .as_ref()
                            .map(|failure| format!("{:?}", failure.problem))
                    );
                    original["preparation_cause"] = json!(controller.preparation_error.as_ref().map(|error|{
                        use crate::presentation::web_ui_adapter::proposals::initial_panel::InitialPrepareError;
                        match error {InitialPrepareError::Sdk(error)=>json!({"sdk":error}),InitialPrepareError::Selection(error)=>json!({"selection":format!("{error:?}")}),InitialPrepareError::Panel(error)=>json!({"panel":format!("{error:?}")}),InitialPrepareError::MissingCorrectionLabels=>json!({"missing_correction_labels":true})}
                    }));
                }
                json!({"success":self.succeeded(),"original":original,"cleanup":format!("{:?}",cleanup.status),"observations":cleanup.evidence.observations,"sdk":format!("{:?}",cleanup.evidence.last_sdk),"cleanup_error":format!("{:?}",cleanup.evidence.last_error),"acceptance_error":cleanup.acceptance_error,"accepted_initial":cleanup.evidence.accepted_initial.is_some(),"unrun_native_retained":unrun_native.is_some(),"notices":notices.iter().map(|notice|format!("{notice:?}")).collect::<Vec<_>>()})
            }
        }
    }
}

struct ObservedController {
    inner: ProposalWindowStartup,
    document: Rc<RefCell<Option<NativeDocumentHandle>>>,
}
impl NativeStartupController for ObservedController {
    type Host = <ProposalWindowStartup as NativeStartupController>::Host;
    type Error = WindowStartupError;
    fn observe(
        &mut self,
        context: &mut NativeStartupContext<'_>,
    ) -> Result<NativeStartupPoll<Self::Host>, Self::Error> {
        let progress = self.inner.observe(context)?;
        // This launcher has no pre-window retry UI. Return the original partial owner for cleanup.
        if self.inner.mount_failure.is_some() {
            return Err(WindowStartupError::Mount);
        }
        Ok(progress)
    }
    fn attached(
        &mut self,
        runtime: Rc<RefCell<ProposalRuntime>>,
        wake: NativeWindowWake,
    ) -> Result<(), Self::Error> {
        *self.document.borrow_mut() = Some(runtime.borrow().document());
        self.inner.attached(runtime, wake)
    }
}

pub fn run_product_native(
    client: ApplicationClientHandle,
    request: NativeProposalLaunchRequest,
    styles: PreparedProductStyles,
) -> ProductNativeOutcome {
    let limits =
        HostScheduleLimits::new(Duration::from_millis(2), Duration::from_millis(250), 1024)
            .expect("fixed product schedule limits");
    diagnostics::record(
        "native-proposal-styles",
        json!({"css_hash":meari::domain::build::ContentHash::of(styles.css.as_bytes()),"sources":styles.source_hashes,"assets":styles.assets.manifest(),"references":styles.references,"font_licenses":styles.font_licenses.iter().map(|(path,contents)|json!({"path":path,"bytes":contents.len()})).collect::<Vec<_>>()}),
    );
    let prepared = match prepare_proposal_execution(
        client.clone(),
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 256 * 1024,
            max_total_request_bytes: 8 * 256 * 1024,
            max_terminal_bytes: 256 * 1024,
            max_total_terminal_bytes: 8 * 256 * 1024,
        },
        limits,
    ) {
        Ok(prepared) => prepared,
        Err(failure) => return ProductNativeOutcome::StartFailure(failure),
    };
    diagnostics::record(
        "native-proposal-execution-started",
        json!({"request_id":request.context.request_id,"selection":request.selection}),
    );
    let (owner, startup, execution) = prepared.into_parts();
    let notices = Rc::new(RefCell::new(Vec::new()));
    let observed = notices.clone();
    let document = Rc::new(RefCell::new(None::<NativeDocumentHandle>));
    let observed_document = document.clone();
    let config = ProductWindowConfig {
        labels: ProductWindowLabels::korean(),
        visuals: ProductWindowVisuals {
            css: styles.css,
            assets: Some(styles.assets),
        },
        viewport: Viewport {
            width: 1180,
            height: 760,
            dark: true,
        },
        limits,
        worker_exit_delay: Duration::from_millis(5),
        observer: Box::new(move |notice| {
            {
                // Retain at most one of each finite notice kind; full validation events stream out.
                let mut kinds = observed.borrow_mut();
                if !kinds.contains(&notice.kind) {
                    kinds.push(notice.kind);
                }
            }
            diagnostics::record(
                "native-proposal-window",
                json!({"kind":format!("{:?}",notice.kind),"revision":notice.revision,"host_id":notice.host_id,"frame":format!("{:?}",notice.frame)}),
            );
            if notice.kind == NativeWindowNoticeKind::FrameSubmitted
                && let Some(handle) = observed_document.borrow().as_ref()
            {
                match handle.try_with(|document| document.renderer.asset_report()) {
                    Ok(Ok(report)) => diagnostics::record("native-proposal-assets", report),
                    other => diagnostics::record(
                        "native-proposal-assets-unobserved",
                        format!("{other:?}"),
                    ),
                }
            }
            Ok(())
        }),
    };
    let owners = match prepare_launch_window(client, execution, request, config) {
        Ok(controller) => {
            let returned = owner.run_startup(
                startup,
                ObservedController {
                    inner: controller,
                    document,
                },
            );
            ProductWindowOwners::Returned(Box::new(NativeRunOutcome {
                controller: returned.controller.inner,
                startup: returned.startup,
                runtime: returned.runtime,
                recovery: returned.recovery,
                loop_owner: returned.loop_owner,
                result: returned.result,
            }))
        }
        Err(failure) => {
            // No run occurred. Keep the unused Native loop/startup alive through SDK cleanup.
            return finish_unrun_product(failure, owner, startup, notices.borrow().clone());
        }
    };
    ProductNativeOutcome::Finished {
        cleanup: Box::new(finish_owned_product(owners)),
        notices: notices.borrow().clone(),
        unrun_native: None,
    }
}

fn finish_unrun_product(
    failure: ProductWindowBuildFailure,
    owner: NativeLoopOwner,
    startup: NativeStartup,
    notices: Vec<NativeWindowNoticeKind>,
) -> ProductNativeOutcome {
    ProductNativeOutcome::Finished {
        cleanup: Box::new(finish_owned_product(ProductWindowOwners::BuildFailure(
            Box::new(failure),
        ))),
        notices,
        unrun_native: Some(Box::new((owner, startup))),
    }
}

#[cfg(test)]
mod tests;
