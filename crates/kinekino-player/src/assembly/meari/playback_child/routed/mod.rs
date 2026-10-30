//! Original registered-child SDK adapter; ownership and cleanup responsibilities are separate.
mod admission;
mod engine_cleanup;
mod errors;
mod ready;
mod registration;
mod state;
mod surface_cleanup;
mod unbound_cleanup;
use crate::assembly::meari::playback_surface::{PlaybackSurfaceChild, PlaybackSurfaceError};
pub(crate) use errors::RoutedChildError;
pub(crate) use ready::{RoutedMediaReady, RoutedSurfaceObservation};
pub(crate) use registration::{
    RoutedMediaRegistration, RoutedMediaRegistrationFailure, register_routed_media,
};
pub(crate) type RegisteredRoutedChild =
    PlaybackSurfaceChild<engine_cleanup::EnginePart, surface_cleanup::SurfacePart>;
type RegistrationError = PlaybackSurfaceError<RoutedChildError, RoutedChildError>;
