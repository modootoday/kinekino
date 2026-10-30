//! Original routed ownership; UI consumes its checked transaction synchronously.
pub mod bootstrap;
use crate::{
    application::{api::ApiError, playback_session::v1, skip_segments::host::SkipHostOwner},
    infrastructure::mpv::{ParentEngineRouter, ParentRouteHandle},
};
#[cfg(test)]
use crate::{
    application::{
        api::PlayerSdk,
        catalog::CatalogArchive,
        playback_session::EpisodePlaybackSessions,
        skip_segments::host::{PreparedRoutedSkipOwner, SkipHostRegistry},
    },
    infrastructure::{
        catalog::CatalogFiles, skip_session_identity::new_host_generation,
        subtitle_files::SubtitleFiles,
    },
};
use std::sync::Arc;

pub(crate) mod ready_seal {
    pub trait Sealed {}
}

/// Implement only on the UI owner retaining the original lease and registered child.
/// Invoke `visit` inside the fresh checked-parent transaction; never retain its proof.
pub(crate) trait ReadyRoutedNative: ready_seal::Sealed {
    type Error;
    fn with_ready_route<T>(
        &self,
        visit: impl FnOnce(&Arc<ParentEngineRouter>, &ParentRouteHandle) -> T,
    ) -> Result<T, Self::Error>;
}

pub(crate) enum RoutedSetupError<E> {
    Ready(E),
    Api(ApiError),
}

struct OriginalRoute {
    _router: Arc<ParentEngineRouter>,
    _route: ParentRouteHandle,
}

#[must_use = "Retain the original ready owner and first error for its existing cleanup"]
#[cfg(test)]
pub(crate) struct RoutedAssemblyFailure<R: ReadyRoutedNative> {
    pub error: RoutedSetupError<R::Error>,
    pub ready: R,
    _pending: Option<(PreparedRoutedSkipOwner, OriginalRoute)>,
}

/// Explicit review lifetime. Parent retirement does not itself close result reads.
#[must_use = "Retain until explicit result review ends"]
pub(crate) struct RoutedExportReadOwner {
    // Drop closes registry admission before releasing original strong owners.
    _skip_owner: SkipHostOwner,
    _retained_issuer: v1::NativeExportIssuer,
    _original: OriginalRoute,
}
// Product reads use the configured client; these conveniences exercise the same issuer in tests.
#[cfg(test)]
impl RoutedExportReadOwner {
    pub fn list(
        &self,
        origin: &v1::NativeExportOriginV1,
    ) -> Result<v1::NativeExportListV1, ApiError> {
        self._retained_issuer.list(origin)
    }
    pub fn peek(
        &self,
        reference: v1::NativeResultRefV1,
        origin: &v1::NativeExportOriginV1,
    ) -> Result<v1::NativeExportResultV1, ApiError> {
        self._retained_issuer.peek(reference, origin)
    }
    pub fn consume(
        &self,
        reference: v1::NativeResultRefV1,
        origin: &v1::NativeExportOriginV1,
    ) -> Result<v1::NativeExportResultV1, ApiError> {
        self._retained_issuer.consume(reference, origin)
    }
}

#[must_use = "Keep original UI ownership and result review ownership explicit"]
#[cfg(test)]
pub(crate) struct RoutedNativeAssembly<R: ReadyRoutedNative> {
    pub ready: R,
    pub client: Arc<PlayerSdk<SubtitleFiles>>,
    pub read_owner: RoutedExportReadOwner,
}

#[cfg(test)]
pub(crate) fn assemble_routed<R: ReadyRoutedNative>(
    ready: R,
    files: Arc<CatalogFiles>,
    base: impl FnOnce() -> PlayerSdk<SubtitleFiles>,
) -> Result<RoutedNativeAssembly<R>, Box<RoutedAssemblyFailure<R>>> {
    let preparation = ready.with_ready_route(|router, route| {
        let issuer = router.capture_active_native_export_issuer(route)?;
        let sessions = Arc::new(EpisodePlaybackSessions::new(files.clone(), router.clone()));
        Ok::<_, ApiError>((
            PreparedRoutedSkipOwner::new(sessions, issuer),
            OriginalRoute {
                _router: router.clone(),
                _route: route.clone(),
            },
        ))
    });
    let (prepared, original) = match preparation {
        Ok(Ok(value)) => value,
        Err(error) => {
            return Err(Box::new(RoutedAssemblyFailure {
                error: RoutedSetupError::Ready(error),
                ready,
                _pending: None,
            }));
        }
        Ok(Err(error)) => {
            return Err(Box::new(RoutedAssemblyFailure {
                error: RoutedSetupError::Api(error),
                ready,
                _pending: None,
            }));
        }
    };
    let generation = match new_host_generation() {
        Ok(value) => value,
        Err(error) => {
            return Err(Box::new(RoutedAssemblyFailure {
                error: RoutedSetupError::Api(error),
                ready,
                _pending: Some((prepared, original)),
            }));
        }
    };
    let sessions = prepared.sessions();
    let issuer = prepared.issuer();
    let skip = match SkipHostRegistry::new_routed_export_host(generation, prepared, 1) {
        Ok(value) => Arc::new(value),
        Err(failure) => {
            return Err(Box::new(RoutedAssemblyFailure {
                error: RoutedSetupError::Api(failure.error),
                ready,
                _pending: Some((failure.prepared, original)),
            }));
        }
    };
    let read_owner = RoutedExportReadOwner {
        _skip_owner: skip.owner(),
        _retained_issuer: issuer,
        _original: original,
    };
    let catalog = CatalogArchive::new(files.clone(), files.clone()).with_episode_results(files);
    let client = Arc::new(
        base()
            .with_catalog(catalog)
            .with_shared_episode_playback(sessions)
            .with_skip_host(skip),
    );
    Ok(RoutedNativeAssembly {
        ready,
        client,
        read_owner,
    })
}
