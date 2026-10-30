use super::{PlaybackStatus, PlaybackStatusPort, project_native_status};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        native_playback::NativePlayback,
    },
    domain::api_values::DecimalU64,
};
use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

type Observation = Arc<RwLock<Option<(Instant, PlaybackStatus)>>>;
pub struct NativeStatusPublisher(Observation);
#[derive(Clone)]
pub struct NativeStatusReader(Observation);

/// Assembly gives the writer to the native lifecycle owner and the reader to SDK.
pub fn native_status_channel() -> (NativeStatusPublisher, NativeStatusReader) {
    let shared = Arc::new(RwLock::new(None));
    (
        NativeStatusPublisher(shared.clone()),
        NativeStatusReader(shared),
    )
}

impl NativeStatusPublisher {
    pub fn publish(&self, state: &NativePlayback) -> Result<(), ApiError> {
        let status = project_native_status(state);
        *self.0.write().map_err(|_| unavailable())? = Some((Instant::now(), status));
        Ok(())
    }

    /// Call when the native instance is replaced, closed, or the owning session ends.
    pub fn clear(&self) -> Result<(), ApiError> {
        *self.0.write().map_err(|_| unavailable())? = None;
        Ok(())
    }
}

impl PlaybackStatusPort for NativeStatusReader {
    fn status(&self) -> Result<PlaybackStatus, ApiError> {
        let observation = self.0.read().map_err(|_| unavailable())?;
        let (time, status) = observation.as_ref().ok_or_else(|| {
            ApiError::new(
                ErrorCode::NotFound,
                "No native playback observation published",
            )
        })?;
        let mut status = status.clone();
        status.published_age_ms =
            DecimalU64(time.elapsed().as_millis().try_into().unwrap_or(u64::MAX));
        Ok(status)
    }
}

impl Drop for NativeStatusPublisher {
    fn drop(&mut self) {
        let _ = self.clear();
    }
}

fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Internal,
        "Playback observation storage unavailable",
    )
}
