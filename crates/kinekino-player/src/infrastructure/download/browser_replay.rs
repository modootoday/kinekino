//! Retains an original browser grant and the original coordinator attempt.
mod binding;
mod issuance;
use super::process_source::{ReplayAdmission, control_check, copy};
use crate::application::{
    api::{ApiError, ErrorCode},
    browser_provider::{
        BrowserProviderPort, MediaReplayPolicy, OwnedMediaReplay, OwnedMediaStream,
    },
    jobs::{ExecutionControl, JobAttemptIdentityV1, owned_attempt::AttemptScope},
};
use binding::BoundRequest;
use std::{sync::Arc, time::Instant};

/// Retains the original document/lease while its same owner is closing.
pub trait ReplayOriginOwner: Send {
    fn check_stream(
        &self,
        _provider: &Arc<dyn BrowserProviderPort>,
        _stream: &OwnedMediaStream,
    ) -> Result<(), ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Original metadata owner association is unavailable",
        ))
    }
    fn check_origin(
        &self,
        provider: &Arc<dyn BrowserProviderPort>,
        grant: &OwnedMediaReplay,
    ) -> Result<(), ApiError>;
    fn poll_release(&mut self, deadline: Instant) -> Result<bool, ApiError>;
}
impl ReplayOriginOwner for crate::application::browser_provider::BrowserDocumentSourceOwner {
    fn check_stream(
        &self,
        provider: &Arc<dyn BrowserProviderPort>,
        stream: &OwnedMediaStream,
    ) -> Result<(), ApiError> {
        self.check_stream(provider, stream)
    }
    fn check_origin(
        &self,
        provider: &Arc<dyn BrowserProviderPort>,
        grant: &OwnedMediaReplay,
    ) -> Result<(), ApiError> {
        self.check_origin(provider, grant)
    }
    fn poll_release(&mut self, deadline: Instant) -> Result<bool, ApiError> {
        self.poll_release(deadline)
    }
}

pub struct BrowserReplayAdmission<G: Send> {
    provider: Arc<dyn BrowserProviderPort>,
    grant: issuance::Grant,
    identity: JobAttemptIdentityV1,
    deadline: Instant,
    attempted: bool,
    consumed: bool,
    releasing: bool,
    request: Option<BoundRequest>,
    request_error: Option<ApiError>,
    // The original live document/source owner remains held through publication.
    owner: G,
}
impl<G: Send> BrowserReplayAdmission<G> {
    /// Launch-free, infallible custody before any admission/check can fail.
    /// `owner` must retain the same original live document, never a URL snapshot.
    pub fn new(
        provider: Arc<dyn BrowserProviderPort>,
        grant: OwnedMediaReplay,
        owner: G,
        scope: &AttemptScope<'_>,
        input: &kinekino_jobs::domain::PreparedJobInput,
    ) -> Self {
        Self::with_grant(provider, issuance::Grant::Ready(grant), owner, scope, input)
    }
    /// No callback or replay issue before the original core owns this admission.
    pub fn pending(
        provider: Arc<dyn BrowserProviderPort>,
        stream: OwnedMediaStream,
        owner: G,
        policy: Arc<dyn MediaReplayPolicy>,
        scope: &AttemptScope<'_>,
        input: &kinekino_jobs::domain::PreparedJobInput,
    ) -> Self {
        Self::with_grant(
            provider,
            issuance::Grant::Pending(Some((stream, policy))),
            owner,
            scope,
            input,
        )
    }
    fn with_grant(
        provider: Arc<dyn BrowserProviderPort>,
        grant: issuance::Grant,
        owner: G,
        scope: &AttemptScope<'_>,
        input: &kinekino_jobs::domain::PreparedJobInput,
    ) -> Self {
        let mut owned = Self {
            provider,
            grant,
            identity: scope.identity().clone(),
            deadline: scope.deadline(),
            attempted: false,
            consumed: false,
            releasing: false,
            request: None,
            request_error: None,
            owner,
        };
        match BoundRequest::new(scope.identity(), input) {
            Ok(request) => owned.request = Some(request),
            Err(error) => owned.request_error = Some(error),
        }
        owned
    }
    fn bound_request(&self) -> Result<&BoundRequest, ApiError> {
        if let Some(error) = &self.request_error {
            return Err(copy(error));
        }
        self.request.as_ref().ok_or_else(|| {
            ApiError::new(ErrorCode::SourceExpired, "Original wire request is absent")
        })
    }
    /// Correlation bytes only; this value cannot construct a replay grant.
    pub fn wire_binding(&self) -> Result<[u8; 32], ApiError> {
        Ok(self.bound_request()?.digest)
    }
    fn current(&self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        if self.releasing {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original replay owner is releasing",
            ));
        }
        control_check(control)?;
        if Instant::now() >= self.deadline {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Original replay attempt expired",
            ));
        }
        self.provider.check_media_replay(self.grant.ready()?)
    }
}
impl<G: ReplayOriginOwner> ReplayAdmission for BrowserReplayAdmission<G> {
    fn prepare(&mut self) -> Result<(), ApiError> {
        self.bound_request()?;
        issuance::check_preparation(self.releasing, self.deadline)?;
        self.grant.prepare(&self.provider, &self.owner)
    }
    fn poll_release(&mut self, deadline: Instant) -> Result<bool, ApiError> {
        self.releasing = true;
        self.owner.poll_release(self.deadline.min(deadline))
    }
    fn check_request(
        &self,
        request: &kinekino_source_host::request::HttpBootstrap,
    ) -> Result<(), ApiError> {
        let original = self.bound_request()?;
        if request.binding() != &original.digest {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Wire binding differs from original attempt",
            ));
        }
        if request.maximum_bytes() > original.maximum_bytes {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Wire request exceeds original byte budget",
            ));
        }
        self.grant.ready()?.check_request(
            request.target().as_str(),
            request.referer().map(url::Url::as_str),
        )
    }
    fn check(&self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        self.bound_request()?;
        if control.execution_identity() != Some(&self.identity)
            || control.deadline() != self.deadline
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original replay coordinator binding differs",
            ));
        }
        self.owner
            .check_origin(&self.provider, self.grant.ready()?)?;
        self.current(control)
    }
    fn consume(&mut self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        if self.attempted {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original replay consumption was already attempted",
            ));
        }
        self.attempted = true;
        self.check(control)?;
        self.provider.consume_media_replay(self.grant.ready()?)?;
        self.consumed = true;
        self.current(control)
    }
    fn check_origin(&self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        if !self.consumed {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original replay was not consumed",
            ));
        }
        // Publication controls need not expose the active execution identity.
        // The retained grant/owner and original deadline remain mandatory.
        self.owner
            .check_origin(&self.provider, self.grant.ready()?)?;
        self.current(control)
    }
}
