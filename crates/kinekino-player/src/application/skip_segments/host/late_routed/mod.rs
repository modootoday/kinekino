//! One prewired SDK port. Pending and live requests keep using the existing registry policy.
mod installation;
#[cfg(test)]
mod tests;
use super::{SkipExportCapability, SkipHostApi, SkipHostCapability, SkipHostRegistry};
use crate::application::{
    api::{ApiError, ErrorCode, Request},
    playback_session::EpisodePlaybackSessions,
};
pub(crate) use installation::LateRoutedInstaller;
pub(crate) use installation::LateRoutedRearm;
use serde_json::Value;
use std::sync::{Arc, Mutex};

enum State {
    Pending,
    Live(Arc<SkipHostRegistry>),
    Retired(Arc<SkipHostRegistry>),
    Closed,
}
pub(crate) struct LateRoutedPort {
    state: Mutex<State>,
    sessions: Arc<EpisodePlaybackSessions>,
}
impl LateRoutedPort {
    pub(crate) fn prepare(
        sessions: Arc<EpisodePlaybackSessions>,
    ) -> (Arc<Self>, LateRoutedInstaller) {
        let port = Arc::new(Self {
            state: Mutex::new(State::Pending),
            sessions,
        });
        let installer = LateRoutedInstaller { port: port.clone() };
        (port, installer)
    }
    fn registry(&self) -> Result<Arc<SkipHostRegistry>, ApiError> {
        match &*self.state.lock().map_err(|_| internal())? {
            State::Live(registry) => Ok(registry.clone()),
            State::Pending => Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Original native skip host awaits checked parent activation",
            )),
            State::Closed | State::Retired(_) => Err(ApiError::new(
                ErrorCode::Cancelled,
                "Original native skip activation is closed",
            )),
        }
    }
}
impl SkipHostApi for LateRoutedPort {
    fn capability(&self) -> SkipHostCapability {
        self.registry()
            .map(|registry| registry.capability())
            .unwrap_or(SkipHostCapability {
                sessions: false,
                seek_submission: false,
            })
    }
    fn export_capability(&self) -> SkipExportCapability {
        self.registry()
            .map(|registry| registry.export_capability())
            .unwrap_or(SkipExportCapability {
                available: false,
                issuer_scope: None,
            })
    }
    fn execute(&self, request: &Request) -> Result<Value, ApiError> {
        // Clone the original delegate, release this lock, forward the exact request unchanged.
        self.registry()?.execute(request)
    }
}
fn internal() -> ApiError {
    ApiError::new(
        ErrorCode::Internal,
        "Original native skip activation state is unavailable",
    )
}
