//! Thin production wiring over supplied owners. Creates no SDK, LocalHost, worker or loop.
pub mod no_media;
#[path = "viewer_native_launch/product_frame.rs"]
mod product_frame;
pub(crate) mod routed;
use super::{
    native_cleanup_driver::{CleanupDriverOutcome, drive_cleanup},
    native_execution_startup::PreparedNativeExecution,
    player_shell_document::{
        DocumentOptions, PreparedShellDocument, mount_prepared_product_shell_with_callbacks,
        mount_prepared_shell_with_callbacks,
    },
    player_shell_host::{ShellHostConfig, ShellHostController, run_shell_host},
    player_shell_startup::{ShellInputs, prepare_shell},
    viewer_cleanup::{
        TypedViewerCleanup, TypedViewerController, TypedViewerOwners, UnrunNative, ViewerCleanup,
        ViewerCleanupLimits, ViewerNoChild, ViewerOwners,
    },
    viewer_host::ViewerHost,
};
use crate::{
    application::api::ApplicationClientHandle, infrastructure::local_host::LocalHost,
    presentation::web_ui_adapter::player_shell::document::StartLabels,
};
use meari_native::application::window_lifecycle::NativeChild;
use std::time::Duration;

pub struct ViewerLaunchInput {
    pub client: ApplicationClientHandle,
    pub endpoint: LocalHost,
    pub execution: PreparedNativeExecution,
    pub labels: StartLabels,
    pub document: DocumentOptions,
    pub host: ShellHostConfig,
}
#[must_use = "Retain the unrun Native owners or execute the same startup once"]
pub struct TypedPreparedViewerLaunch<C: NativeChild> {
    controller: TypedViewerController<C>,
    native: UnrunNative,
}
pub type PreparedViewerLaunch = TypedPreparedViewerLaunch<ViewerNoChild>;
impl<C: NativeChild> TypedPreparedViewerLaunch<C> {
    pub fn into_unrun(self) -> TypedViewerOwners<C> {
        TypedViewerOwners::Unrun {
            controller: Box::new(self.controller),
            native: Box::new(self.native),
        }
    }
    pub fn run(self) -> TypedViewerOwners<C> {
        TypedViewerOwners::Returned(Box::new(run_shell_host(
            self.native.owner,
            self.native.startup,
            self.controller,
        )))
    }
}
pub fn prepare_viewer_launch(
    input: ViewerLaunchInput,
) -> Result<PreparedViewerLaunch, Box<ViewerOwners>> {
    let prepared = prepare_viewer_document::<ViewerNoChild>(input)?;
    Ok(PreparedViewerLaunch {
        controller: ShellHostController::new(prepared.document, prepared.host),
        native: prepared.native,
    })
}
/// Product caller uses one fixed frame graph; standalone/NoMedia callers retain their profile.
pub fn prepare_product_viewer_launch(
    input: ViewerLaunchInput,
) -> Result<PreparedViewerLaunch, Box<ViewerOwners>> {
    let prepared = prepare_viewer_document_profile::<ViewerNoChild>(
        input,
        ViewerDocumentMode::ProductFrame,
        None,
    )?;
    Ok(PreparedViewerLaunch {
        controller: ShellHostController::new_with_route_binding(
            prepared.document,
            prepared.host,
            |host, route| host.bind_route_observation(route),
        ),
        native: prepared.native,
    })
}
/// Typed choices already inspected by the original project caller; no store scan or auto item.
pub struct ProductViewerChoices {
    pub project: crate::domain::projects::ProjectRevision,
    pub generation: std::num::NonZeroU64,
    pub capacity: std::num::NonZeroUsize,
}
pub struct CapturedProductViewerChoices {
    pub stored: crate::domain::projects::StoredProjectRevision,
    pub generation: std::num::NonZeroU64,
    pub capacity: std::num::NonZeroUsize,
}
enum ViewerChoices {
    Live(super::viewer_host::TrustedLiveChoices),
    Legacy(ProductViewerChoices),
    Captured(
        CapturedProductViewerChoices,
        Option<std::sync::Arc<dyn crate::application::catalog::CatalogArtifacts>>,
    ),
}
/// R1 host-held selections only. This does not install an editor or a user chooser.
pub fn prepare_product_viewer_launch_with_live_choices(
    input: ViewerLaunchInput,
    choices: super::viewer_host::TrustedLiveChoices,
) -> Result<PreparedViewerLaunch, Box<ViewerOwners>> {
    let prepared = prepare_viewer_document_profile::<ViewerNoChild>(
        input,
        ViewerDocumentMode::ProductFrame,
        Some(ViewerChoices::Live(choices)),
    )?;
    Ok(PreparedViewerLaunch {
        controller: ShellHostController::new_with_route_binding(
            prepared.document,
            prepared.host,
            |host, route| host.bind_route_observation(route),
        ),
        native: prepared.native,
    })
}
/// Opt-in with the exact SDK-returned stored revision. Does not create a live origin.
pub fn prepare_product_viewer_launch_with_captured_choices(
    input: ViewerLaunchInput,
    choices: CapturedProductViewerChoices,
) -> Result<PreparedViewerLaunch, Box<ViewerOwners>> {
    let prepared = prepare_viewer_document_profile::<ViewerNoChild>(
        input,
        ViewerDocumentMode::ProductFrame,
        Some(ViewerChoices::Captured(choices, None)),
    )?;
    Ok(PreparedViewerLaunch {
        controller: ShellHostController::new_with_route_binding(
            prepared.document,
            prepared.host,
            |host, route| host.bind_route_observation(route),
        ),
        native: prepared.native,
    })
}
pub fn prepare_product_viewer_launch_with_choices(
    input: ViewerLaunchInput,
    choices: ProductViewerChoices,
) -> Result<PreparedViewerLaunch, Box<ViewerOwners>> {
    let prepared = prepare_viewer_document_profile::<ViewerNoChild>(
        input,
        ViewerDocumentMode::ProductFrame,
        Some(ViewerChoices::Legacy(choices)),
    )?;
    Ok(PreparedViewerLaunch {
        controller: ShellHostController::new_with_route_binding(
            prepared.document,
            prepared.host,
            |host, route| host.bind_route_observation(route),
        ),
        native: prepared.native,
    })
}
/// The original product caller supplies the existing immutable artifact adapter.
pub fn prepare_product_viewer_launch_with_captured_evidence(
    input: ViewerLaunchInput,
    choices: CapturedProductViewerChoices,
    artifacts: std::sync::Arc<dyn crate::application::catalog::CatalogArtifacts>,
) -> Result<PreparedViewerLaunch, Box<ViewerOwners>> {
    let prepared = prepare_viewer_document_profile::<ViewerNoChild>(
        input,
        ViewerDocumentMode::ProductFrame,
        Some(ViewerChoices::Captured(choices, Some(artifacts))),
    )?;
    Ok(PreparedViewerLaunch {
        controller: ShellHostController::new_with_route_binding(
            prepared.document,
            prepared.host,
            |host, route| host.bind_route_observation(route),
        ),
        native: prepared.native,
    })
}
#[derive(Clone, Copy)]
enum ViewerDocumentMode {
    Standalone,
    ProductFrame,
}
struct PreparedViewerDocument {
    document: PreparedShellDocument<ViewerHost>,
    host: ShellHostConfig,
    native: UnrunNative,
}
fn prepare_viewer_document<C: NativeChild>(
    input: ViewerLaunchInput,
) -> Result<PreparedViewerDocument, Box<TypedViewerOwners<C>>> {
    prepare_viewer_document_profile(input, ViewerDocumentMode::Standalone, None)
}
fn prepare_viewer_document_profile<C: NativeChild>(
    input: ViewerLaunchInput,
    mode: ViewerDocumentMode,
    choices: Option<ViewerChoices>,
) -> Result<PreparedViewerDocument, Box<TypedViewerOwners<C>>> {
    let (owner, startup, execution) = input.execution.into_parts();
    let completion = std::sync::Arc::new(
        crate::infrastructure::meari::completion_notifier::NativeStartupCompletionNotifier::new(
            startup.wake_handle(),
        ),
    );
    let native = Box::new(UnrunNative { owner, startup });
    let host = ViewerHost::new_with_completion(
        input.client.clone(),
        execution,
        input.endpoint,
        completion,
    );
    let mut props =
        crate::presentation::web_ui_adapter::player_shell::document::read_props(&input.labels);
    let mut prepared = match prepare_shell(
        ShellInputs {
            client: input.client,
            host_owner: host,
        },
        input.labels,
    ) {
        Ok(prepared) => prepared,
        Err(failure) => {
            return Err(Box::new(TypedViewerOwners::ShellFailure {
                failure,
                native,
            }));
        }
    };
    if let Err(error) =
        crate::presentation::web_ui_adapter::player_shell::document::enable_read_panel(
            &mut prepared.request.document,
            props.clone(),
        )
    {
        let failure = Box::new(super::player_shell_startup::ShellPreparationFailure {
            error: super::player_shell_startup::ShellStartupError::Start(crate::presentation::web_ui_adapter::player_shell::controller::StartPreparationError::Document(error)),
            inputs: prepared.inputs,
            state: prepared.state,
        });
        return Err(Box::new(TypedViewerOwners::ShellFailure {
            failure,
            native,
        }));
    }
    let configured = match choices {
        Some(ViewerChoices::Live(choices)) => {
            prepared.inputs.host_owner.configure_live_choices(choices)
        }
        Some(ViewerChoices::Legacy(choices)) => prepared.inputs.host_owner.configure_card_choices(
            choices.project,
            choices.generation,
            choices.capacity,
        ),
        Some(ViewerChoices::Captured(choices, artifacts)) => {
            let configured = prepared
                .inputs
                .host_owner
                .callbacks()
                .borrow_mut()
                .configure_captured_edit_choices(
                    choices.stored,
                    choices.generation,
                    choices.capacity,
                )
                .map_err(|error| {
                    super::viewer_host::ViewerHostError::Callback(std::sync::Arc::new(error))
                });
            configured.and_then(|()| match artifacts {
                Some(artifacts) => prepared
                    .inputs
                    .host_owner
                    .install_capture_evidence(artifacts),
                None => Ok(()),
            })
        }
        None if matches!(mode, ViewerDocumentMode::ProductFrame) => prepared
            .inputs
            .host_owner
            .configure_default_chooser(&mut props),
        None => Ok(()),
    };
    if let Err(error) = configured {
        return Err(Box::new(TypedViewerOwners::DocumentFailure {
            failure: Box::new(super::player_shell_document::DocumentFailure {
                stage: super::player_shell_document::DocumentStage::Configuration,
                diagnostic: error.to_string(),
                inputs: prepared.inputs,
                state: prepared.state,
                ticket: prepared.request.ticket,
            }),
            native,
        }));
    }
    let callbacks = prepared.inputs.host_owner.callbacks();
    props.extend(callbacks.borrow().configured_card_props());
    prepared.request.document.props = props.clone();
    #[cfg(test)]
    if let Err(error) = no_media::prepare_live_fixture_source(&mut prepared.request.document) {
        let failure = Box::new(super::player_shell_startup::ShellPreparationFailure {
            error: super::player_shell_startup::ShellStartupError::Start(crate::presentation::web_ui_adapter::player_shell::controller::StartPreparationError::Document(error)),
            inputs: prepared.inputs, state: prepared.state,
        });
        return Err(Box::new(TypedViewerOwners::ShellFailure {
            failure,
            native,
        }));
    }
    #[cfg(test)]
    {
        // The static fixture supplies declared callback references through the original export.
        // Keep the same props for the initial slot and later original Viewing return.
        props = prepared.request.document.props.clone();
    }
    let frame = if matches!(mode, ViewerDocumentMode::ProductFrame) {
        match product_frame::prepare(&prepared.request.document) {
            Ok(frame) => Some(frame),
            Err(error) => {
                let failure = Box::new(super::player_shell_startup::ShellPreparationFailure {
                    error: super::player_shell_startup::ShellStartupError::Start(crate::presentation::web_ui_adapter::player_shell::controller::StartPreparationError::Document(error)),
                    inputs: prepared.inputs, state: prepared.state,
                });
                return Err(Box::new(TypedViewerOwners::ShellFailure {
                    failure,
                    native,
                }));
            }
        }
    } else {
        None
    };
    let export = super::viewer_callbacks::export_callback_bindings(callbacks);
    let mounted = match frame.as_ref() {
        Some(frame) => {
            mount_prepared_product_shell_with_callbacks(prepared, input.document, frame, export)
        }
        None => mount_prepared_shell_with_callbacks(prepared, input.document, export),
    };
    let mut document = match mounted {
        Ok(document) => document,
        Err(failure) => {
            return Err(Box::new(TypedViewerOwners::DocumentFailure {
                failure,
                native,
            }));
        }
    };
    if let Some(frame) = frame {
        document
            .inputs
            .host_owner
            .bind_product_document(document.document.clone(), props, frame);
    } else {
        document
            .inputs
            .host_owner
            .bind_document(document.document.clone(), props);
    }
    Ok(PreparedViewerDocument {
        document,
        host: input.host,
        native: *native,
    })
}
pub fn finish_viewer(
    owners: ViewerOwners,
    limits: ViewerCleanupLimits,
) -> CleanupDriverOutcome<ViewerCleanup> {
    finish_typed_viewer(owners, limits)
}
pub(crate) fn finish_typed_viewer<C: NativeChild>(
    owners: TypedViewerOwners<C>,
    limits: ViewerCleanupLimits,
) -> CleanupDriverOutcome<TypedViewerCleanup<C>> {
    drive_cleanup(
        TypedViewerCleanup::new(owners, limits),
        Duration::from_millis(5),
        std::thread::sleep,
    )
}

#[cfg(test)]
mod tests;
