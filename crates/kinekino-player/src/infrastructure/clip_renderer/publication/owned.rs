//! Original staged candidate. Only the Core worker decides when publish is called.
use super::*;
use kinekino_jobs::application::owned_attempt::{AttemptScope, OwnedOutputCandidate};
use std::sync::Arc;

pub(crate) struct ClipOwnedOutputCandidate {
    store: Arc<ClipArtifactStore>,
    id: String,
    staged: Option<PreparedClipPublication>,
    render: Option<RetainedClipRenderCandidate>,
    release: Option<crate::infrastructure::clip_renderer::retained::PendingClipRenderRelease>,
    deadline: Instant,
    published: bool,
    verification_attempted: bool,
    _host: crate::infrastructure::clip_render_jobs::ClipRenderJobExecutor,
}
struct ScopeCheck<'a, 'b>(&'a AttemptScope<'b>);
impl MutationControl for ScopeCheck<'_, '_> {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if self.0.cancellation_requested() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Clip staging cancelled before Core publication",
            ));
        }
        DeadlineCheck(self.0.deadline()).checkpoint()
    }
}
struct DeadlineCheck(Instant);
impl MutationControl for DeadlineCheck {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if Instant::now() >= self.0 {
            Err(ApiError::new(
                ErrorCode::Timeout,
                "Original clip candidate deadline elapsed",
            ))
        } else {
            Ok(())
        }
    }
}
impl ClipOwnedOutputCandidate {
    /// Return the parent-held candidate before staging may fail or unwind.
    pub(crate) fn new(
        store: Arc<ClipArtifactStore>,
        render: RetainedClipRenderCandidate,
        scope: &AttemptScope<'_>,
        host: crate::infrastructure::clip_render_jobs::ClipRenderJobExecutor,
    ) -> Self {
        Self {
            store,
            render: Some(render),
            staged: None,
            release: None,
            id: format!(
                "{}-attempt-{}",
                scope.identity().job_id().as_str(),
                scope.identity().attempt()
            ),
            deadline: scope.deadline(),
            published: false,
            verification_attempted: false,
            _host: host,
        }
    }
    pub(crate) fn stage(&mut self, scope: &AttemptScope<'_>) -> Result<(), ApiError> {
        if self.staged.is_some()
            || self.release.is_some()
            || self.render.is_none()
            || self.deadline != scope.deadline()
            || self.id
                != format!(
                    "{}-attempt-{}",
                    scope.identity().job_id().as_str(),
                    scope.identity().attempt()
                )
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original clip staging binding changed or was consumed",
            ));
        }
        self.staged = Some(self.store.stage(
            &self.id,
            &self.render.as_ref().expect("Original render").as_verified(),
            Some(scope.identity()),
            &ScopeCheck(scope),
        )?);
        Ok(())
    }
}
impl OwnedOutputCandidate for ClipOwnedOutputCandidate {
    fn poll_release(&mut self, _deadline: Instant) -> Result<bool, ApiError> {
        if self.release.is_none()
            && let Some(render) = self.render.take()
        {
            self.release = Some(render.into_release());
        }
        match &mut self.release {
            Some(release) => release.poll(),
            None => Ok(true),
        }
    }
    fn publish(&mut self) -> Result<(), ApiError> {
        if self.release.is_some() || self.published {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original clip publication already consumed or released",
            ));
        }
        DeadlineCheck(self.deadline).checkpoint()?;
        self.staged
            .as_mut()
            .ok_or_else(|| error("Original staged clip required"))?
            .commit()
            .map_err(|mut error| {
                if !error.details.is_object() {
                    error.details = json!({"original_details":error.details});
                }
                error.details["publication_may_exist"] = json!(true);
                error.details["artifact_id"] = json!(self.id);
                error.details["manifest_sha256"] =
                    json!(self.staged.as_ref().expect("Original stage").expected);
                error
            })?;
        self.published = true;
        Ok(())
    }
    fn verify(&mut self) -> Result<kinekino_jobs::domain::VerifiedJobOutput, ApiError> {
        if !self.published || self.verification_attempted || self.release.is_some() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original published clip required for one verification",
            ));
        }
        self.verification_attempted = true;
        let staged = self.staged.as_ref().expect("Published original stage");
        let result = (|| {
            let retained =
                self.store
                    .inspect(&self.id, &staged.expected, &DeadlineCheck(self.deadline))?;
            let receipt = retained.receipt();
            let output = kinekino_jobs::domain::VerifiedJobOutput {
                path: staged
                    .directory
                    .join("manifest.json")
                    .to_str()
                    .ok_or_else(|| error("Non-UTF8 clip output path"))?
                    .into(),
                sha256: receipt.manifest_sha256.clone(),
                bytes: receipt.manifest_bytes,
                receipt: json!({"schema_version":"clip-job-receipt/1","artifact":receipt,"color_verified":false,"quality_approved":false,"partial_cleanup_supported":false}),
            };
            output
                .validate()
                .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
            Ok(output)
        })();
        result.map_err(|mut error: ApiError| {
            if !error.details.is_object() {
                error.details = json!({"original_details":error.details});
            }
            error.details["publication_committed"] = json!(true);
            error.details["artifact_id"] = json!(self.id);
            error.details["manifest_sha256"] = json!(staged.expected);
            error
        })
    }
}
