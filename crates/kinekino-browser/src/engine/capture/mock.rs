//! In-memory [`BrowserEngine`] that yields scripted network observations (no real browser).

use std::collections::HashMap;

use super::observe::NetworkObservationSource;
use super::types::ObservedNetworkRequest;
use crate::engine::port::{
    BrowserEngine, BrowserEvent, EngineError, EngineKind, EngineResult, NetworkInterceptor,
    RequestDecision,
};

/// Mock engine: scripted observations after navigate; interceptor stays Allow/Block only.
pub struct MockBrowserEngine {
    events: Vec<BrowserEvent>,
    interceptor: Option<Box<dyn NetworkInterceptor>>,
    /// Observations to emit the next time `url` is navigated.
    scripted: HashMap<String, Vec<ObservedNetworkRequest>>,
    pending: Vec<ObservedNetworkRequest>,
    navigated: Vec<String>,
}

impl MockBrowserEngine {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            interceptor: None,
            scripted: HashMap::new(),
            pending: Vec::new(),
            navigated: Vec::new(),
        }
    }

    /// Queue observations returned (after interceptor Allow filter) on navigate to `url`.
    pub fn script_observations(
        &mut self,
        url: impl Into<String>,
        observations: Vec<ObservedNetworkRequest>,
    ) {
        self.scripted.insert(url.into(), observations);
    }

    pub fn navigated_urls(&self) -> &[String] {
        &self.navigated
    }
}

impl Default for MockBrowserEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserEngine for MockBrowserEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::ChromeCdp
    }

    fn navigate(&mut self, url: &str) -> EngineResult<()> {
        self.events
            .push(BrowserEvent::NavigationStarted { url: url.into() });
        self.navigated.push(url.to_string());
        let raw = self.scripted.get(url).cloned().unwrap_or_default();
        let mut pending = Vec::new();
        let mut blocked = Vec::new();
        for obs in raw {
            let allow = match self.interceptor.as_ref() {
                None => true,
                Some(interceptor) => {
                    match interceptor.decide(&crate::engine::port::InterceptedRequest {
                        url: obs.url.clone(),
                        method: obs.method.clone(),
                        resource_type: obs.resource_type.clone(),
                    }) {
                        RequestDecision::Allow => true,
                        RequestDecision::Block => false,
                    }
                }
            };
            if allow {
                pending.push(obs);
            } else {
                blocked.push(obs.url);
            }
        }
        for blocked_url in blocked {
            self.events
                .push(BrowserEvent::RequestBlocked { url: blocked_url });
        }
        self.pending = pending;
        self.events.push(BrowserEvent::Loaded { url: url.into() });
        Ok(())
    }

    fn eval(&mut self, _: &str) -> EngineResult<serde_json::Value> {
        Err(EngineError::Unsupported("mock-capture"))
    }

    fn inject_script(&mut self, _: &str) -> EngineResult<()> {
        Err(EngineError::Unsupported("mock-capture"))
    }

    fn set_interceptor(&mut self, interceptor: Box<dyn NetworkInterceptor>) -> EngineResult<()> {
        self.interceptor = Some(interceptor);
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<BrowserEvent> {
        std::mem::take(&mut self.events)
    }
}

impl NetworkObservationSource for MockBrowserEngine {
    fn drain_observed_requests(&mut self) -> Vec<ObservedNetworkRequest> {
        std::mem::take(&mut self.pending)
    }
}
