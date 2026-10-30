//! Concrete original-child engine boundary. Never accepts a raw handle or a ready flag.
mod frame_refresh;
#[cfg(test)]
mod permit_contract;
use super::*;
use crate::{
    application::{api::ApiError, catalog::CatalogArtifacts},
    infrastructure::mpv::{NativeEpisodeEngine, TrustedMpvExecutable, guarded_command::HostScope},
};
use meari_native::application::window_lifecycle::WindowStamp;
use std::{fmt, num::NonZeroU32};

/// Callback-borrowed witness; fields and construction stay in the actual child adapter.
pub(crate) struct CheckedMediaParent<'a> {
    _proof: &'a VisibleRenderOnlyProof<'a>,
    child: NonZeroU32,
}

#[derive(Clone, Debug)]
enum Cause {
    Visible(Arc<VisibleError>),
    Engine(Arc<ApiError>),
    AlreadyBound,
    ChildHandleRange,
    Unbound,
}
#[derive(Clone, Debug)]
pub(crate) struct ParentMediaSurfaceError(Cause);
impl fmt::Display for ParentMediaSurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Cause::Visible(error) => write!(f, "original parent media: {error}"),
            Cause::Engine(error) => write!(f, "original parent engine: {}", error.message),
            Cause::AlreadyBound => f.write_str("Original parent already has an engine"),
            Cause::ChildHandleRange => {
                f.write_str("Original child cannot fit the existing engine parent type")
            }
            Cause::Unbound => f.write_str("Original parent engine is not bound"),
        }
    }
}
impl std::error::Error for ParentMediaSurfaceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            Cause::Visible(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}
fn visible(error: Arc<VisibleError>) -> ParentMediaSurfaceError {
    ParentMediaSurfaceError(Cause::Visible(error))
}

#[must_use = "Return the original input to its existing registration cleanup carrier"]
pub(crate) struct ParentMediaFailure {
    pub error: ParentMediaSurfaceError,
    pub input: NativeSurfaceInput,
}

/// The only original lease and OS child live here, retained before construction.
pub(crate) struct ParentMediaSurface {
    surface: VisibleRenderOnlyChild,
    engine: Option<Arc<NativeEpisodeEngine>>,
}
impl ParentMediaSurface {
    /// Explicit current-frame transaction. It does not make an old proof valid again.
    pub fn refresh_original_frame(&mut self) -> std::result::Result<(), ParentMediaSurfaceError> {
        frame_refresh::refresh(&mut self.surface).map_err(visible)
    }
    pub fn prepare(
        input: NativeSurfaceInput,
    ) -> std::result::Result<Self, Box<ParentMediaFailure>> {
        match VisibleRenderOnlyChild::prepare(input) {
            Ok(surface) => Ok(Self {
                surface,
                engine: None,
            }),
            Err(failure) => {
                let PrepareFailure { error, input } = *failure;
                Err(Box::new(ParentMediaFailure {
                    error: visible(Arc::new(error)),
                    input,
                }))
            }
        }
    }
    pub fn identity(&self) -> (WindowStamp, u64) {
        (self.surface.input.window(), self.surface.input.child())
    }
    pub fn construct_visible(&mut self) -> std::result::Result<(), ParentMediaSurfaceError> {
        self.surface.construct_retained().map_err(visible)?;
        self.surface.transition_visible().map_err(visible)
    }
    /// No process opens here. Engine and enabled non-standalone scope come from this actual child.
    pub fn bind_original_engine(
        &mut self,
        executable: TrustedMpvExecutable,
        artifacts: Arc<dyn CatalogArtifacts>,
    ) -> std::result::Result<Arc<NativeEpisodeEngine>, ParentMediaSurfaceError> {
        if self.engine.is_some() {
            return Err(ParentMediaSurfaceError(Cause::AlreadyBound));
        }
        let engine = self
            .surface
            .with_visible_render_only(|proof| {
                proof.verify().map_err(visible)?;
                let child = self
                    .surface
                    .child()
                    .map_err(|error| visible(Arc::new(error)))?;
                let child = u32::try_from(child.0 as usize)
                    .ok()
                    .and_then(NonZeroU32::new)
                    .ok_or(ParentMediaSurfaceError(Cause::ChildHandleRange))?;
                let permit = CheckedMediaParent {
                    _proof: proof,
                    child,
                };
                let scope = HostScope::for_verified_parent(&executable, &permit);
                let engine = NativeEpisodeEngine::new_with_scope(
                    executable,
                    Some(permit.child.get()),
                    artifacts,
                    scope,
                )
                .map_err(|error| ParentMediaSurfaceError(Cause::Engine(Arc::new(error))))?;
                proof.verify().map_err(visible)?;
                Ok(Arc::new(engine))
            })
            .map_err(visible)??;
        self.engine = Some(engine.clone());
        Ok(engine)
    }
    /// The engine is borrowed only while the same child is freshly checked.
    pub fn with_original_engine<T>(
        &self,
        visit: impl FnOnce(&Arc<NativeEpisodeEngine>) -> T,
    ) -> std::result::Result<T, ParentMediaSurfaceError> {
        let engine = self
            .engine
            .as_ref()
            .ok_or(ParentMediaSurfaceError(Cause::Unbound))?;
        self.surface
            .with_visible_render_only(|proof| {
                proof.verify().map_err(visible)?;
                let value = visit(engine);
                proof.verify().map_err(visible)?;
                Ok(value)
            })
            .map_err(visible)?
    }
    pub fn revoke_admission(&mut self) {
        self.surface.revoked = true;
        self.surface.input.revoke();
    }
    pub fn begin_release(
        &mut self,
        request: CleanupRequest,
    ) -> std::result::Result<ProbeRelease, ParentMediaSurfaceError> {
        // A hide error is retained by the original surface; release still uses its cleanup authority.
        let _ = self.surface.revoke(request);
        self.surface.release(request).map_err(visible)
    }
    pub fn observe_release(
        &mut self,
        request: CleanupRequest,
    ) -> std::result::Result<ProbeRelease, ParentMediaSurfaceError> {
        self.surface
            .input
            .with_cleanup_parent(request, |parent| {
                require_parent(&self.surface.native, &self.surface.input, parent)?;
                self.surface
                    .native
                    .observe_quarantine_release()
                    .map_err(VisibleError::Child)
            })
            .map_err(VisibleError::Input)
            .and_then(|value| value)
            .map_err(|error| visible(Arc::new(error)))
    }
    pub fn first_error(&self) -> Option<ParentMediaSurfaceError> {
        self.surface.first_error().map(visible)
    }
    pub fn class_retry_ready(&self) -> bool {
        self.surface.native.quarantine_class_retry_ready()
    }
}
