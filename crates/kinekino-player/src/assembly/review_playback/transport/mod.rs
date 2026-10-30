//! Keep original transport joins and review owners through explicit shutdown.
use super::*;
use crate::{
    application::{
        api::{ErrorCode, PlayerSdk},
        frame_capture::FrameCapturePort,
        model_review::ReviewStore,
        playback_control::PlaybackControlPort,
        playback_status::PlaybackStatusPort,
        review_playback::sessions::ShutdownObservation,
        subtitles::storage::SubtitleStorage,
    },
    infrastructure::local_host::{
        LocalHost, LocalHostShutdownStatus, PendingLocalHost, TransportOptions, WorkerJoin,
    },
};
use std::time::Instant;
#[cfg(test)]
pub(super) mod tests;

#[must_use = "Preserve the original session service and startup error for cleanup"]
pub struct ReviewTransportStartFailure {
    pub error: ApiError,
    pub sessions: Arc<ReviewPlaybackSessions>,
}
impl std::fmt::Debug for ReviewTransportStartFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReviewTransportStartFailure")
            .field("error", &self.error)
            .field("sessions_retained", &true)
            .finish()
    }
}

/// Binds the exact retained service into the supplied common SDK, preserving its
/// other capabilities. No separate request router or review policy is introduced.
pub fn start_transport<S, D, P, C, R>(
    base: PlayerSdk<S, D, P, C, R>,
    sessions: Arc<ReviewPlaybackSessions>,
    options: TransportOptions,
) -> Result<ReviewTransportHost, Box<ReviewTransportStartFailure>>
where
    S: SubtitleStorage + Send + Sync + 'static,
    D: FrameCapturePort + Send + Sync + 'static,
    P: PlaybackStatusPort + 'static,
    C: PlaybackControlPort + 'static,
    R: ReviewStore + 'static,
{
    match LocalHost::start(
        Arc::new(base.with_review_playback_sessions(sessions.clone())),
        options,
    ) {
        Ok(local) => Ok(ReviewTransportHost {
            local: Some(local),
            sessions,
        }),
        Err(error) => Err(Box::new(ReviewTransportStartFailure { error, sessions })),
    }
}

#[must_use = "Use begin_shutdown and retain the returned pending owner through physical completion"]
pub struct ReviewTransportHost {
    local: Option<LocalHost>,
    sessions: Arc<ReviewPlaybackSessions>,
}
impl ReviewTransportHost {
    pub fn endpoint(&self) -> &str {
        self.local.as_ref().expect("running transport").endpoint()
    }
    /// The supplied observation deadline is fixed; polling never extends it.
    pub fn begin_shutdown(mut self, deadline: Instant) -> PendingReviewTransport {
        let first_error = self.sessions.begin_close().err();
        let local = self
            .local
            .take()
            .expect("running transport")
            .begin_shutdown();
        PendingReviewTransport {
            local,
            sessions: self.sessions.clone(),
            deadline,
            first_error,
            finished: false,
        }
    }
}
impl Drop for ReviewTransportHost {
    fn drop(&mut self) {
        // Best effort only. Drop cannot certify joins or physical completion.
        let _ = self.sessions.begin_close();
    }
}

#[must_use = "Retain the original transport and session owners even after deadline or observation failure"]
pub struct PendingReviewTransport {
    local: PendingLocalHost,
    sessions: Arc<ReviewPlaybackSessions>,
    deadline: Instant,
    first_error: Option<ApiError>,
    finished: bool,
}
#[derive(Debug)]
pub struct ReviewTransportShutdownObservation<'a> {
    pub transport: LocalHostShutdownStatus,
    pub sessions: Option<ShutdownObservation>,
    pub complete: bool,
    pub deadline_expired: bool,
    pub first_error: Option<&'a ApiError>,
}
#[derive(Debug)]
pub struct ReviewTransportCompletion {
    pub transport: LocalHostShutdownStatus,
    pub sessions: ShutdownObservation,
    pub first_error: Option<ApiError>,
}
impl ReviewTransportShutdownObservation<'_> {
    pub fn succeeded(&self) -> bool {
        self.complete && self.first_error.is_none()
    }
}
impl PendingReviewTransport {
    /// Only observed physical completion releases these original owners. An
    /// earlier error remains in the receipt even if cleanup later succeeds.
    pub fn into_completion(mut self) -> Result<ReviewTransportCompletion, Box<Self>> {
        let completed = {
            let observation = self.poll();
            if observation.complete {
                Some((
                    observation.transport,
                    observation.sessions.expect("complete sessions"),
                ))
            } else {
                None
            }
        };
        match completed {
            Some((transport, sessions)) => Ok(ReviewTransportCompletion {
                transport,
                sessions,
                first_error: self.first_error,
            }),
            None => Err(Box::new(self)),
        }
    }
    /// Join polling itself is nonblocking; original session observation can wait
    /// for an in-flight reservation. This is not hard preemption or cancellation ACK.
    pub fn poll(&mut self) -> ReviewTransportShutdownObservation<'_> {
        let transport = self.local.poll();
        if transport.pipe == WorkerJoin::Panicked
            || transport.sdk == WorkerJoin::Panicked
            || transport.dispatch.is_none()
        {
            self.first_error.get_or_insert_with(|| {
                ApiError::new(
                    ErrorCode::EngineFailed,
                    "Review transport worker failed or dispatch state was lost",
                )
            });
        }
        let sessions = match self.sessions.observe_shutdown() {
            Ok(observation) => Some(observation),
            Err(error) => {
                self.first_error.get_or_insert(error);
                None
            }
        };
        let complete =
            transport.is_quiescent() && sessions.as_ref().is_some_and(|state| state.complete);
        let deadline_expired = Instant::now() >= self.deadline;
        if !self.finished && deadline_expired {
            self.first_error.get_or_insert_with(|| {
                ApiError::new(
                    ErrorCode::Timeout,
                    "Review host shutdown observation deadline expired",
                )
            });
        }
        self.finished |= complete;
        ReviewTransportShutdownObservation {
            transport,
            sessions,
            complete,
            deadline_expired,
            first_error: self.first_error.as_ref(),
        }
    }
}
