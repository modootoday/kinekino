//! Overlapping custody from the selected review session to an engine-openable object.
use super::{ReviewFiles, Root, io_error, paths};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_playback::{
            ReviewInputIdentity, ReviewInputWithRuntime, ReviewMediaInput, ReviewRuntimeLease,
            admit_review_runtime,
        },
    },
    infrastructure::content_digest,
};
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
};

/// Keep this owner, including its original non-Send session, until physical child exit.
/// Binding a path does not reserve a runtime or authorize playback.
pub struct GuardedReviewMedia {
    source: ReviewMediaInput,
    _root: Arc<Root>,
    file: File,
    path: PathBuf,
}
/// Source path guards and the concrete runtime grant have one owner.
pub struct GuardedAdmittedReviewMedia<L: ReviewRuntimeLease> {
    input: ReviewInputWithRuntime<L>,
    _root: Arc<Root>,
    file: File,
    path: PathBuf,
}
impl ReviewFiles {
    pub fn bind_playback_media(
        &self,
        source: ReviewMediaInput,
    ) -> Result<GuardedReviewMedia, ApiError> {
        source.check_current()?;
        let identity = source.identity();
        // Never accept a caller/media-supplied path. A different configured root must
        // independently contain exactly the same complete content.
        let path = self.root.path.join("objects").join(&identity.media.sha256);
        let mut file = paths::file(&path)?;
        let observed = content_digest::sha256(&mut file, identity.bytes)?;
        if observed != (identity.bytes, identity.media.sha256.clone()) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Review playback object differs from selected source",
            ));
        }
        source.check_current()?;
        let result = GuardedReviewMedia {
            source,
            _root: self.root.clone(),
            file,
            path,
        };
        result.check_current()?;
        Ok(result)
    }
}
impl GuardedReviewMedia {
    pub fn admit<L: ReviewRuntimeLease>(
        self,
        runtime: L,
    ) -> Result<GuardedAdmittedReviewMedia<L>, ApiError> {
        self.check_current()?;
        let input = admit_review_runtime(self.source, runtime)?;
        let result = GuardedAdmittedReviewMedia {
            input,
            _root: self._root,
            file: self.file,
            path: self.path,
        };
        result.check_current()?;
        Ok(result)
    }
    pub fn identity(&self) -> &ReviewInputIdentity {
        self.source.identity()
    }
    pub fn check_current(&self) -> Result<(), ApiError> {
        self.source.check_current()?;
        if self.file.metadata().map_err(io_error)?.len() != self.identity().bytes {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Review playback object length changed",
            ));
        }
        Ok(())
    }
    // Internal backend seam; no wire path, shell argument, or public path grant.
    #[allow(
        dead_code,
        reason = "Consumed by the pending owned local playback backend"
    )]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}
impl<L: ReviewRuntimeLease> GuardedAdmittedReviewMedia<L> {
    pub(crate) fn into_parts(self) -> (GuardedReviewMedia, L) {
        let (source, runtime) = self.input.into_parts();
        (
            GuardedReviewMedia {
                source,
                _root: self._root,
                file: self.file,
                path: self.path,
            },
            runtime,
        )
    }
    pub fn identity(&self) -> &ReviewInputIdentity {
        self.input.identity()
    }
    pub fn check_current(&self) -> Result<(), ApiError> {
        self.input.check_current()?;
        if self.file.metadata().map_err(io_error)?.len() != self.identity().bytes {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Admitted review playback object length changed",
            ));
        }
        Ok(())
    }
    #[allow(dead_code, reason = "Internal owned local backend seam")]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}
