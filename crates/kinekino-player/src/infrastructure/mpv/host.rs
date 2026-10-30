use super::{NativeEpisodeEngine, TrustedMpvExecutable};
use crate::application::{
    api::{ApiError, ErrorCode},
    catalog::CatalogArtifacts,
    playback_session::ClosedEpisodePlayback,
};
use std::{path::Path, sync::Arc};

/// Assembly owns this guard; SDK clones cannot extend an ended host's permission to open.
pub struct NativePlaybackHost {
    engine: NativeEpisodeEngine,
}
impl NativePlaybackHost {
    /// Explicit standalone assembly; no parent handle or renderer authority input.
    pub fn start_trusted_standalone(
        executable: &Path,
        sha256: &str,
        diagnostics: Arc<dyn CatalogArtifacts>,
    ) -> Result<Self, ApiError> {
        Self::from_verified_standalone(Self::verify_standalone(executable, sha256)?, diagnostics)
    }
    pub(crate) fn verify_standalone(
        executable: &Path,
        sha256: &str,
    ) -> Result<TrustedMpvExecutable, ApiError> {
        if !executable.is_absolute() {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Standalone native executable must be an absolute path",
            ));
        }
        TrustedMpvExecutable::open(executable, sha256)
    }
    pub(crate) fn from_verified_standalone(
        executable: TrustedMpvExecutable,
        diagnostics: Arc<dyn CatalogArtifacts>,
    ) -> Result<Self, ApiError> {
        Ok(Self {
            engine: NativeEpisodeEngine::new_trusted_standalone(executable, diagnostics)?,
        })
    }
    pub fn start(
        executable: &Path,
        sha256: &str,
        parent: Option<u32>,
        diagnostics: Arc<dyn CatalogArtifacts>,
    ) -> Result<Self, ApiError> {
        Ok(Self {
            engine: NativeEpisodeEngine::new(
                TrustedMpvExecutable::open(executable, sha256)?,
                parent,
                diagnostics,
            )?,
        })
    }
    pub fn engine(&self) -> NativeEpisodeEngine {
        self.engine.clone()
    }
    pub fn shutdown(&self) -> Result<Option<ClosedEpisodePlayback>, ApiError> {
        self.engine.shutdown()
    }
}
impl Drop for NativePlaybackHost {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
