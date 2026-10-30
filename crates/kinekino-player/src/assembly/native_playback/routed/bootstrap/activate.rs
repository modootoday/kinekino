use super::super::{OriginalRoute, ReadyRoutedNative, RoutedExportReadOwner, RoutedSetupError};
use super::PendingRoutedActivation;
use crate::{
    application::{api::ApplicationClientHandle, skip_segments::host::PreparedRoutedSkipOwner},
    infrastructure::skip_session_identity::new_host_generation,
};
use std::sync::Arc;

#[must_use = "Same client and original owners remain explicit"]
pub(crate) struct ActivatedRoutedClient<R: ReadyRoutedNative> {
    pub _retained_client: ApplicationClientHandle,
    pub ready: R,
    pub _retained_read_owner: RoutedExportReadOwner,
    reactivation: Option<Reactivation>,
}
struct Reactivation {
    client: ApplicationClientHandle,
    router: Arc<crate::infrastructure::mpv::ParentEngineRouter>,
    sessions: Arc<crate::application::playback_session::EpisodePlaybackSessions>,
    rearm: crate::application::skip_segments::host::late_routed::LateRoutedRearm,
}
impl<R: ReadyRoutedNative> ActivatedRoutedClient<R> {
    pub(crate) fn suspend_original(&self) -> Result<(), crate::application::api::ApiError> {
        self.reactivation
            .as_ref()
            .ok_or_else(|| {
                crate::application::api::ApiError::new(
                    crate::application::api::ErrorCode::RevisionConflict,
                    "Original native reactivation was consumed",
                )
            })?
            .rearm
            .suspend()
    }
    pub(crate) fn prepare_reactivation(
        &mut self,
    ) -> Result<PendingRoutedActivation, crate::application::api::ApiError> {
        let original = self.reactivation.as_ref().ok_or_else(|| {
            crate::application::api::ApiError::new(
                crate::application::api::ErrorCode::RevisionConflict,
                "Original native reactivation was consumed",
            )
        })?;
        let installer = original.rearm.resume()?;
        let activation = PendingRoutedActivation {
            client: original.client.clone(),
            router: original.router.clone(),
            sessions: original.sessions.clone(),
            installer,
        };
        self.reactivation = None;
        Ok(activation)
    }
}
#[must_use = "Preserve first error, original R, activation and any prepared original owner"]
pub(crate) struct RoutedActivationFailure<R: ReadyRoutedNative> {
    pub error: RoutedSetupError<R::Error>,
    pub _retained_ready: R,
    pub activation: PendingRoutedActivation,
    pub _retained_revocation_error: Option<crate::application::api::ApiError>,
    _pending: Option<(PreparedRoutedSkipOwner, OriginalRoute)>,
}
impl PendingRoutedActivation {
    pub(crate) fn activate<R: ReadyRoutedNative>(
        self,
        ready: R,
    ) -> Result<ActivatedRoutedClient<R>, Box<RoutedActivationFailure<R>>> {
        let preparation = ready.with_ready_route(|router, route| {
            if !Arc::ptr_eq(router, &self.router) {
                return Err(crate::application::api::ApiError::new(
                    crate::application::api::ErrorCode::RevisionConflict,
                    "Ready parent belongs to a different original client router",
                ));
            }
            let issuer = router.capture_active_native_export_issuer(route)?;
            Ok((
                PreparedRoutedSkipOwner::new(self.sessions.clone(), issuer),
                OriginalRoute {
                    _router: router.clone(),
                    _route: route.clone(),
                },
            ))
        });
        let (prepared, original) = match preparation {
            Err(error) => {
                return Err(Box::new(RoutedActivationFailure {
                    error: RoutedSetupError::Ready(error),
                    _retained_ready: ready,
                    activation: self,
                    _retained_revocation_error: None,
                    _pending: None,
                }));
            }
            Ok(Err(error)) => {
                return Err(Box::new(RoutedActivationFailure {
                    error: RoutedSetupError::Api(error),
                    _retained_ready: ready,
                    activation: self,
                    _retained_revocation_error: None,
                    _pending: None,
                }));
            }
            Ok(Ok(prepared)) => prepared,
        };
        let generation = match new_host_generation() {
            Ok(generation) => generation,
            Err(error) => {
                let revocation_error = original
                    ._router
                    .revoke_route_admission(&original._route)
                    .err();
                return Err(Box::new(RoutedActivationFailure {
                    error: RoutedSetupError::Api(error),
                    _retained_ready: ready,
                    activation: self,
                    _retained_revocation_error: revocation_error,
                    _pending: Some((prepared, original)),
                }));
            }
        };
        let issuer = prepared.issuer();
        let installed = match self.installer.install(generation, prepared) {
            Ok(installed) => installed,
            Err(failure) => {
                let revocation_error = original
                    ._router
                    .revoke_route_admission(&original._route)
                    .err();
                let activation = PendingRoutedActivation {
                    installer: failure.installer,
                    ..self
                };
                return Err(Box::new(RoutedActivationFailure {
                    error: RoutedSetupError::Api(failure.error),
                    _retained_ready: ready,
                    activation,
                    _retained_revocation_error: revocation_error,
                    _pending: Some((failure.prepared, original)),
                }));
            }
        };
        let read_owner = RoutedExportReadOwner {
            _skip_owner: installed.owner,
            _retained_issuer: issuer,
            _original: original,
        };
        // `installed.registry` is already owned by the port in this exact existing SDK.
        drop(installed.registry);
        Ok(ActivatedRoutedClient {
            _retained_client: self.client.clone(),
            ready,
            _retained_read_owner: read_owner,
            reactivation: Some(Reactivation {
                client: self.client,
                router: self.router,
                sessions: self.sessions,
                rearm: installed.rearm,
            }),
        })
    }
}
