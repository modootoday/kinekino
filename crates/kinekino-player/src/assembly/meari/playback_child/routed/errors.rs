use super::super::cleanup::PlaybackCleanupError;
use crate::{
    application::api::ApiError,
    infrastructure::mpv::{ParentMediaSurfaceError, SurfaceInputError},
};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub(crate) enum RoutedChildError {
    Surface(ParentMediaSurfaceError),
    Input(Arc<SurfaceInputError>),
    Api(Arc<ApiError>),
    Cleanup(PlaybackCleanupError),
    Borrowed,
    Expired,
    NotReady,
    UnboundEngine,
    Identity,
}
impl std::fmt::Display for RoutedChildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Surface(error) => write!(f, "registered routed child: {error}"),
            Self::Input(error) => write!(f, "registered routed input: {error}"),
            Self::Api(error) => write!(f, "registered routed SDK: {}", error.message),
            Self::Cleanup(error) => write!(f, "registered routed cleanup: {error}"),
            _ => write!(f, "registered routed child: {self:?}"),
        }
    }
}
impl std::error::Error for RoutedChildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Surface(error) => Some(error),
            Self::Input(error) => Some(error.as_ref()),
            Self::Cleanup(error) => Some(error),
            _ => None,
        }
    }
}

pub(super) fn retain_api_error(error: &ApiError) -> Arc<ApiError> {
    Arc::new(ApiError {
        code: error.code,
        message: error.message.clone(),
        retryable: error.retryable,
        details: error.details.clone(),
        remediation: error.remediation.clone(),
        request_id: error.request_id.clone(),
    })
}
