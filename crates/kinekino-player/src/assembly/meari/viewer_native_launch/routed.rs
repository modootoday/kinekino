//! Product-owned frame adapter over the original native SDK registration.
mod consumer;
use super::{
    TypedPreparedViewerLaunch, TypedViewerOwners, UnrunNative, ViewerDocumentMode, ViewerHost,
    ViewerLaunchInput, prepare_viewer_document_profile,
};
use crate::{
    application::{catalog::CatalogArtifacts, playback_session::transition::TransitionSlot},
    assembly::{
        meari::{
            playback_child::routed::RegisteredRoutedChild,
            player_shell_host::{ShellFrameConsumer, ShellHostController},
        },
        native_playback::routed::bootstrap::PendingRoutedActivation,
    },
    infrastructure::mpv::TrustedMpvExecutable,
};
use std::sync::Arc;

pub(crate) struct RoutedViewerInput {
    pub executable: TrustedMpvExecutable,
    pub artifacts: Arc<dyn CatalogArtifacts>,
    pub activation: PendingRoutedActivation,
    pub transitions: Arc<TransitionSlot>,
}
#[must_use = "Retain the original launch input and unique activation through cleanup"]
pub(crate) enum RoutedViewerFailure {
    Preparation {
        owners: Box<TypedViewerOwners<RegisteredRoutedChild>>,
        routed: RoutedViewerInput,
    },
    Declaration {
        diagnostic: String,
        owners: Box<TypedViewerOwners<RegisteredRoutedChild>>,
        routed: RoutedViewerInput,
    },
    Installation {
        owners: Box<TypedViewerOwners<RegisteredRoutedChild>>,
        consumer: Box<dyn ShellFrameConsumer<RegisteredRoutedChild>>,
    },
    MediaPort {
        owners: Box<TypedViewerOwners<RegisteredRoutedChild>>,
        consumer: Box<dyn ShellFrameConsumer<RegisteredRoutedChild>>,
        error: Arc<crate::application::api::ApiError>,
    },
}
pub(crate) struct RoutedFailureRetention {
    pub(crate) _retained_media_error: Option<Arc<crate::application::api::ApiError>>,
    pub diagnostic: String,
    pub routed: Option<RoutedViewerInput>,
    pub consumer: Option<Box<dyn ShellFrameConsumer<RegisteredRoutedChild>>>,
}
impl RoutedFailureRetention {
    pub(crate) fn close_pending(&self) -> Option<String> {
        if let Some(routed) = &self.routed {
            routed.transitions.close_admission();
        }
        self.routed
            .as_ref()
            .and_then(|routed| routed.activation.close_pending().err())
            .map(|error| error.message)
    }
}
impl RoutedViewerFailure {
    pub(crate) fn into_cleanup_parts(
        self,
    ) -> (
        Box<TypedViewerOwners<RegisteredRoutedChild>>,
        RoutedFailureRetention,
    ) {
        match self {
            Self::Preparation { owners, routed } => (
                owners,
                RoutedFailureRetention {
                    _retained_media_error: None,
                    diagnostic: "original routed document preparation failed".into(),
                    routed: Some(routed),
                    consumer: None,
                },
            ),
            Self::Declaration {
                diagnostic,
                owners,
                routed,
            } => (
                owners,
                RoutedFailureRetention {
                    _retained_media_error: None,
                    diagnostic,
                    routed: Some(routed),
                    consumer: None,
                },
            ),
            Self::Installation { owners, consumer } => (
                owners,
                RoutedFailureRetention {
                    _retained_media_error: None,
                    diagnostic: "original routed frame consumer installation rejected".into(),
                    routed: None,
                    consumer: Some(consumer),
                },
            ),
            Self::MediaPort {
                owners,
                consumer,
                error,
            } => (
                owners,
                RoutedFailureRetention {
                    diagnostic: error.message.clone(),
                    routed: None,
                    consumer: Some(consumer),
                    _retained_media_error: Some(error),
                },
            ),
        }
    }
}
pub(crate) fn prepare_routed_viewer(
    input: ViewerLaunchInput,
    routed: RoutedViewerInput,
    project: Option<crate::assembly::meari::product_viewer_launch::SelectedViewerProject>,
) -> Result<TypedPreparedViewerLaunch<RegisteredRoutedChild>, Box<RoutedViewerFailure>> {
    let choices = project.map(|project| {
        super::ViewerChoices::Captured(
            super::CapturedProductViewerChoices {
                stored: project.stored,
                generation: std::num::NonZeroU64::new(1).expect("initial choice generation"),
                capacity: std::num::NonZeroUsize::new(8).expect("fixed choice capacity"),
            },
            Some(project.artifacts),
        )
    });
    let mut prepared = match prepare_viewer_document_profile::<RegisteredRoutedChild>(
        input,
        ViewerDocumentMode::ProductFrame,
        choices,
    ) {
        Ok(prepared) => prepared,
        Err(owners) => {
            return Err(Box::new(RoutedViewerFailure::Preparation {
                owners,
                routed,
            }));
        }
    };
    let transition_bound = prepared
        .document
        .inputs
        .host_owner
        .bind_original_transition_slot(routed.transitions.clone())
        .is_ok();
    let declaration = prepared.document.inspect_viewing_video();
    let registration_redraw = prepared.document.inputs.host_owner.registration_redraw();
    let mut controller =
        ShellHostController::<ViewerHost, RegisteredRoutedChild>::new_with_route_binding(
            prepared.document,
            prepared.host,
            |host, route| host.bind_route_observation(route),
        );
    let wake_source = controller.frame_wake_source();
    let actions_source = controller.window_actions_source();
    if let Some(host) = controller.pending_owner_mut() {
        host.bind_automatic_wake_source(wake_source.clone());
        host.bind_viewing_window_actions(actions_source, wake_source);
    }
    if !transition_bound {
        return Err(Box::new(RoutedViewerFailure::Declaration {
            diagnostic: "Original transition slot binding rejected; owners retained".into(),
            owners: unrun(controller, prepared.native),
            routed,
        }));
    }
    let declaration = match declaration {
        Ok(declaration) => declaration,
        Err(error) => {
            return Err(Box::new(RoutedViewerFailure::Declaration {
                diagnostic: error.to_string(),
                owners: unrun(controller, prepared.native),
                routed,
            }));
        }
    };
    let consumer = Box::new(consumer::RoutedFrameConsumer::new(
        declaration.id().as_str().into(),
        declaration.origin().clone(),
        routed.executable,
        routed.artifacts,
        routed.activation,
        registration_redraw,
    ));
    let port = consumer.media_port();
    let installed = controller
        .pending_owner_mut()
        .ok_or_else(|| {
            Arc::new(crate::application::api::ApiError::new(
                crate::application::api::ErrorCode::Internal,
                "Original route media host owner is missing",
            ))
        })
        .and_then(|host| host.install_route_media_port(port));
    if let Err(error) = installed {
        return Err(Box::new(RoutedViewerFailure::MediaPort {
            owners: unrun(controller, prepared.native),
            consumer,
            error,
        }));
    }
    if let Err(consumer) = controller.install_frame_consumer(consumer) {
        return Err(Box::new(RoutedViewerFailure::Installation {
            owners: unrun(controller, prepared.native),
            consumer,
        }));
    }
    Ok(TypedPreparedViewerLaunch {
        controller,
        native: prepared.native,
    })
}
fn unrun(
    controller: ShellHostController<ViewerHost, RegisteredRoutedChild>,
    native: UnrunNative,
) -> Box<TypedViewerOwners<RegisteredRoutedChild>> {
    Box::new(TypedViewerOwners::Unrun {
        controller: Box::new(controller),
        native: Box::new(native),
    })
}
