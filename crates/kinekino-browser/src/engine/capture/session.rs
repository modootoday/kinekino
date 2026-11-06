//! Host-owned `BrowserCaptureSession`: open → observe/collect → close.

use super::admission::{CaptureAdmission, host_of};
use super::observe::NetworkObservationSource;
use super::types::{CaptureError, CaptureNavigateTarget, CaptureWaitHint, ObservedNetworkRequest};
use crate::engine::port::BrowserEngine;

/// Default cap on observed requests returned to the plugin.
pub const DEFAULT_MAX_OBSERVATIONS: usize = 64;

/// Host-owned capture session driving a [`BrowserEngine`] for plugin Resolve.
pub struct BrowserCaptureSession<E> {
    engine: E,
    admission: Box<dyn CaptureAdmission>,
    max_observations: usize,
    closed: bool,
}

impl<E> BrowserCaptureSession<E>
where
    E: BrowserEngine + NetworkObservationSource,
{
    pub fn open(engine: E, admission: Box<dyn CaptureAdmission>) -> Self {
        Self {
            engine,
            admission,
            max_observations: DEFAULT_MAX_OBSERVATIONS,
            closed: false,
        }
    }

    pub fn with_max_observations(mut self, max: usize) -> Self {
        self.max_observations = max;
        self
    }

    /// Navigate (if admitted) and collect capped, media-host-filtered observations.
    ///
    /// `wait` is accepted for the contract; the mock engine ignores timing. Real engines
    /// may honour network-idle / timeout in a later slice.
    pub fn collect(
        &mut self,
        target: &CaptureNavigateTarget,
        wait: CaptureWaitHint,
    ) -> Result<Vec<ObservedNetworkRequest>, CaptureError> {
        if self.closed {
            return Err(CaptureError::Closed);
        }
        let host = host_of(&target.url).ok_or(CaptureError::NavigateDenied)?;
        if !self.admission.admits_navigate_host(&host) {
            return Err(CaptureError::NavigateDenied);
        }
        let _ = wait; // reserved for real-engine idle/timeout (M3-5 mock ignores)
        self.engine.navigate(&target.url)?;
        let mut filtered: Vec<ObservedNetworkRequest> = self
            .engine
            .drain_observed_requests()
            .into_iter()
            .filter(|obs| {
                host_of(&obs.url)
                    .map(|h| self.admission.admits_media_host(&h))
                    .unwrap_or(false)
            })
            .collect();
        if filtered.len() > self.max_observations {
            filtered.truncate(self.max_observations);
        }
        Ok(filtered)
    }

    /// Close the session. Idempotent: a second close succeeds.
    pub fn close(&mut self) -> Result<(), CaptureError> {
        self.closed = true;
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Borrow the underlying engine (tests / interceptor checks).
    pub fn engine_mut(&mut self) -> &mut E {
        &mut self.engine
    }
}
