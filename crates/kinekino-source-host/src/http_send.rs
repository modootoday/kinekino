//! Transport lifecycle only. Caller must consume original browser replay grant
//! before polling; neither bootstrap bytes nor the guard issue that grant.
#[cfg(test)]
mod tests;
use crate::{
    http_response::{HttpResponseOwner, ResponseError},
    request::HttpBootstrap,
};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendError {
    Setup,
    Transport,
    Order,
    Invalidated,
    Cancelled,
    Response(ResponseError),
}
type SendFuture = Pin<Box<dyn Future<Output = Result<reqwest::Response, SendError>> + Send>>;

/// Original send future and response stay in this non-Clone/non-Debug owner.
/// This is not a physical-close receipt or a production admission constructor.
pub struct HttpSendOwner {
    pending: Option<SendFuture>,
    response: Option<HttpResponseOwner>,
    maximum: u64,
    expected_url: url::Url,
    ready: bool,
    failure: Option<SendError>,
}
impl HttpSendOwner {
    #[cfg(test)]
    pub(crate) fn response_fixture(
        response: reqwest::Response,
        maximum: u64,
        expected_url: url::Url,
    ) -> Self {
        Self::from_future(
            Box::pin(std::future::ready(Ok(response))),
            maximum,
            expected_url,
        )
    }
    /// Construction does not poll a request. Unsupported provider permissions
    /// must be rejected by the caller before this transport is ever polled.
    pub fn prepare(bootstrap: HttpBootstrap) -> Result<Self, SendError> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .no_proxy()
            .referer(false)
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .pool_max_idle_per_host(0)
            .http1_only()
            .https_only(true)
            .build()
            .map_err(|_| SendError::Setup)?;
        let mut request = client
            .get(bootstrap.target().clone())
            .header(reqwest::header::ACCEPT_ENCODING, "identity");
        if let Some(referer) = bootstrap.referer() {
            request = request.header(reqwest::header::REFERER, referer.as_str());
        }
        let request = request.build().map_err(|_| SendError::Setup)?;
        let maximum = bootstrap.maximum_bytes();
        let expected_url = bootstrap.target().clone();
        Ok(Self::from_future(
            Box::pin(async move {
                client
                    .execute(request)
                    .await
                    .map_err(|_| SendError::Transport)
            }),
            maximum,
            expected_url,
        ))
    }
    fn from_future(pending: SendFuture, maximum: u64, expected_url: url::Url) -> Self {
        Self {
            pending: Some(pending),
            response: None,
            maximum,
            expected_url,
            ready: false,
            failure: None,
        }
    }
    fn fail(&mut self, error: SendError) -> Poll<Result<(), SendError>> {
        self.failure = Some(error);
        Poll::Ready(Err(error))
    }
    /// Check original current/attempt/absolute cutoff before and after each poll.
    /// A guard failure retains the original future/response for same-owner close.
    pub fn poll_open(
        &mut self,
        cx: &mut Context<'_>,
        mut check: impl FnMut() -> Result<(), SendError>,
    ) -> Poll<Result<(), SendError>> {
        if let Some(error) = self.failure {
            return Poll::Ready(Err(error));
        }
        if self.ready {
            return Poll::Ready(Err(SendError::Order));
        }
        if let Err(error) = check() {
            return self.fail(error);
        }
        let Some(pending) = &mut self.pending else {
            return self.fail(SendError::Order);
        };
        let result = pending.as_mut().poll(cx);
        // Assign before post-poll guard or header validation can fail/unwind.
        if let Poll::Ready(Ok(response)) = result {
            self.response = Some(HttpResponseOwner::new(
                response,
                self.maximum,
                self.expected_url.clone(),
            ));
            self.pending = None;
        } else if let Poll::Ready(Err(error)) = result {
            self.pending = None;
            return self.fail(error);
        }
        if let Err(error) = check() {
            return self.fail(error);
        }
        let Some(response) = &mut self.response else {
            return Poll::Pending;
        };
        if let Err(error) = response.validate() {
            return self.fail(SendError::Response(error));
        }
        self.ready = true;
        Poll::Ready(Ok(()))
    }
    pub fn response_mut(&mut self) -> Result<&mut HttpResponseOwner, SendError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if !self.ready {
            return Err(SendError::Order);
        }
        self.response.as_mut().ok_or(SendError::Order)
    }
    /// Cancel the owned future. Existing Response remains owned until drop.
    /// No claim about OS DNS, remote request or process-family termination.
    pub fn abort(&mut self) {
        if self.failure.is_none() {
            self.failure = Some(SendError::Cancelled);
        }
        self.pending = None;
    }
}
