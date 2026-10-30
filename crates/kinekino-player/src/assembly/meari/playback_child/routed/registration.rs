use super::admission::OriginalAdmission;
use super::{
    RegisteredRoutedChild, RegistrationError,
    engine_cleanup::EnginePart,
    errors::RoutedChildError,
    ready::RoutedMediaCheckpoint,
    state::{State, SurfaceOwner},
    surface_cleanup::SurfacePart,
};
use crate::{
    assembly::meari::playback_surface::{PlaybackSurfaceChild, PlaybackSurfaceError},
    infrastructure::mpv::{
        NativeSurfaceInput, ParentEngineRouter, ParentMediaFailure, ParentMediaSurface,
    },
};
use meari_native::{
    application::window_lifecycle::{ChildCreationFailure, ChildLease},
    infrastructure::window::{
        NativeSurfaceRegistrar, NativeSurfaceRegistrationFailure, NativeSurfaceReservation,
    },
};
use std::{cell::RefCell, rc::Rc, sync::Arc};
pub(crate) struct RoutedMediaRegistration {
    pub _retained_registrar: NativeSurfaceRegistrar<RegisteredRoutedChild>,
    pub lease: ChildLease,
    pub checkpoint: RoutedMediaCheckpoint,
}
pub(crate) struct RoutedMediaRegistrationFailure {
    pub _retained_registrar: NativeSurfaceRegistrar<RegisteredRoutedChild>,
    pub _retained_failure: NativeSurfaceRegistrationFailure<RegistrationError>,
    pub checkpoint: Option<RoutedMediaCheckpoint>,
}
#[expect(
    clippy::result_large_err,
    reason = "NativeSurfaceRegistrar requires the original inline ChildCreationFailure with partial cleanup owners; boxing changes its lifecycle callback contract"
)]
pub(crate) fn register_routed_media(
    registrar: NativeSurfaceRegistrar<RegisteredRoutedChild>,
    reservation: NativeSurfaceReservation,
    router: Arc<ParentEngineRouter>,
) -> Result<RoutedMediaRegistration, Box<RoutedMediaRegistrationFailure>> {
    let mut checkpoint = None;
    let result = registrar.register_reserved(reservation, |_, parent, slot| {
        let (surface, error) = match NativeSurfaceInput::new(parent, slot) {
            Ok(input) => match ParentMediaSurface::prepare(input) {
                Ok(surface) => (SurfaceOwner::Ready(Box::new(surface)), None),
                Err(failure) => {
                    let ParentMediaFailure { error, input } = *failure;
                    (
                        SurfaceOwner::Prepare(Box::new(input)),
                        Some(RoutedChildError::Surface(error)),
                    )
                }
            },
            Err(failure) => {
                let error = RoutedChildError::Input(Arc::new(failure.error.clone()));
                (SurfaceOwner::Rejected(failure), Some(error))
            }
        };
        let admission = Rc::new(OriginalAdmission::new(router.clone()));
        let state = Rc::new(RefCell::new(State {
            admission: admission.clone(),
            surface,
            router,
            route: None,
            original_engine: None,
            cleanup: None,
            unbound_cleanup: None,
            revoked: false,
            first_error: error.clone(),
        }));
        checkpoint = Some(RoutedMediaCheckpoint {
            state: Rc::downgrade(&state),
            admission: admission.clone(),
        });
        let (window, child) = state.borrow().identity();
        let child = PlaybackSurfaceChild::new(
            window,
            child,
            EnginePart {
                admission,
                state: state.clone(),
                diagnostic: None,
            },
            SurfacePart {
                state,
                request: None,
                signal: None,
                diagnostic: error.clone(),
            },
        );
        match error {
            Some(error) => Err(ChildCreationFailure {
                error: PlaybackSurfaceError::Surface(error),
                partial: Some(child),
            }),
            None => Ok(child),
        }
    });
    match result {
        Ok(lease) => Ok(RoutedMediaRegistration {
            _retained_registrar: registrar,
            lease,
            checkpoint: checkpoint.expect("registered factory retains checkpoint"),
        }),
        Err(failure) => Err(Box::new(RoutedMediaRegistrationFailure {
            _retained_registrar: registrar,
            _retained_failure: failure,
            checkpoint,
        })),
    }
}
