mod capture;
mod frame_completion;
mod frame_events;
mod frame_runtime;
mod navigation_events;
mod network_worker;
mod protocol_observer;
mod runtime_worker;
#[cfg(test)]
pub(crate) mod test_host;
#[cfg(test)]
mod tests;
mod ui_policy;
mod window;
#[cfg(test)]
pub(crate) use window::{Apartment, HiddenWindow};
mod worker;

use crate::application::{
    api::{ApiError, ErrorCode},
    browser_provider::*,
};
use std::{
    path::Path,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
    },
};

pub(super) enum Command {
    Start(BrowserTicket),
    Navigate(BrowserTicket, String),
    Cancel(BrowserTicket),
    ActivateEpisode(BrowserTicket, u64),
}
pub(super) struct Shared {
    works: Vec<BrowserWork>,
    host: BrowserHostSnapshot,
    next_generation: u64,
}
pub(super) struct Handle {
    shared: Arc<Mutex<Shared>>,
    commands: SyncSender<Command>,
    shutdown: Arc<AtomicBool>,
    session_id: String,
}

/// Owns only the shutdown signal; COM/window ownership stays on the worker thread.
pub struct BrowserHost {
    handle: Arc<Handle>,
}
pub struct BrowserSystemClock;
impl BrowserEvidenceClock for BrowserSystemClock {
    fn now_ms(&self) -> crate::domain::api_values::DecimalU64 {
        capture::now_ms()
    }
}
impl BrowserHost {
    pub fn port(&self) -> Arc<dyn BrowserProviderPort> {
        self.handle.clone()
    }
    pub fn owner(&self) -> Arc<dyn BrowserProviderOwner> {
        self.handle.clone()
    }
}
impl Drop for BrowserHost {
    fn drop(&mut self) {
        self.handle.shutdown.store(true, Ordering::Release);
    }
}

pub fn start(profile_root: &Path) -> Result<BrowserHost, ApiError> {
    if !profile_root.is_absolute() || !profile_root.is_dir() {
        return Err(error(
            ErrorCode::InvalidArgument,
            "Browser profile must be an existing absolute directory",
        ));
    }
    static SESSIONS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let session_id = format!(
        "browser-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        SESSIONS.fetch_add(1, Ordering::Relaxed)
    );
    let shared = Arc::new(Mutex::new(Shared {
        works: vec![],
        next_generation: 1,
        host: BrowserHostSnapshot {
            state: BrowserHostState::Starting,
            worker_exited: false,
            failure: None,
        },
    }));
    let shutdown = Arc::new(AtomicBool::new(false));
    let (commands, receiver) = mpsc::sync_channel(MAX_COMMANDS);
    let handle = Arc::new(Handle {
        shared: shared.clone(),
        commands,
        shutdown: shutdown.clone(),
        session_id,
    });
    let profile = profile_root.to_owned();
    std::thread::Builder::new()
        .name("browser-provider-sta".into())
        .spawn(move || worker::run(profile, shared, shutdown, receiver))
        .map_err(|_| error(ErrorCode::EngineFailed, "Browser worker could not start"))?;
    Ok(BrowserHost { handle })
}

pub(super) fn error(code: ErrorCode, message: &str) -> ApiError {
    ApiError::new(code, message)
}
impl Handle {
    fn state(&self) -> Result<MutexGuard<'_, Shared>, ApiError> {
        self.shared.try_lock().map_err(|failure| match failure {
            std::sync::TryLockError::WouldBlock => {
                error(ErrorCode::ResourceExhausted, "Browser state is busy")
            }
            std::sync::TryLockError::Poisoned(_) => {
                error(ErrorCode::Internal, "Browser state lock is poisoned")
            }
        })
    }
    fn send(&self, command: Command) -> Result<(), ApiError> {
        self.commands
            .try_send(command)
            .map_err(|failure| match failure {
                mpsc::TrySendError::Full(_) => error(
                    ErrorCode::ResourceExhausted,
                    "Browser command queue is full",
                ),
                mpsc::TrySendError::Disconnected(_) => error(
                    ErrorCode::EngineFailed,
                    "Browser worker command channel is closed",
                ),
            })
    }
}
fn work<'a>(
    state: &'a mut Shared,
    ticket: &BrowserTicket,
) -> Result<&'a mut BrowserWork, ApiError> {
    state
        .works
        .iter_mut()
        .find(|work| work.snapshot().ticket == *ticket)
        .ok_or_else(|| error(ErrorCode::NotFound, "Browser ticket not found"))
}

impl BrowserProviderPort for Handle {
    fn issue_media_replay(
        &self,
        stream: OwnedMediaStream,
        policy: &dyn crate::application::browser_provider::MediaReplayPolicy,
    ) -> Result<crate::application::browser_provider::OwnedMediaReplay, ApiError> {
        let mut state = self.state()?;
        if self.shutdown.load(Ordering::Acquire) || state.host.state != BrowserHostState::Ready {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original browser host stopped",
            ));
        }
        let original = work(&mut state, &stream.ticket)?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            original.issue_media_replay(stream, policy)
        }));
        // Retain the original work and unlock before resuming the ORIGINAL
        // payload. A policy unwind must not poison shared browser cleanup.
        drop(state);
        match result {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }
    fn consume_media_replay(
        &self,
        replay: &crate::application::browser_provider::OwnedMediaReplay,
    ) -> Result<(), ApiError> {
        let mut state = self.state()?;
        if self.shutdown.load(Ordering::Acquire) || state.host.state != BrowserHostState::Ready {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original browser host stopped",
            ));
        }
        let ticket = replay.ticket();
        work(&mut state, ticket)?.consume_media_replay(replay)
    }
    fn check_media_replay(
        &self,
        replay: &crate::application::browser_provider::OwnedMediaReplay,
    ) -> Result<(), ApiError> {
        let mut state = self.state()?;
        if self.shutdown.load(Ordering::Acquire) || state.host.state != BrowserHostState::Ready {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original browser host stopped",
            ));
        }
        work(&mut state, replay.ticket())?.check_media_replay(replay)
    }
    fn admit_media_stream(
        &self,
        ticket: &BrowserTicket,
        generation: u64,
        session: &crate::application::browser_network::ProtocolSession,
        request_id: &str,
    ) -> Result<OwnedMediaStream, ApiError> {
        let mut state = self.state()?;
        if self.shutdown.load(Ordering::Acquire) || state.host.state != BrowserHostState::Ready {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original browser host is not ready",
            ));
        }
        work(&mut state, ticket)?.admit_media_stream(generation, session, request_id)
    }
    fn check_media_stream(&self, stream: &OwnedMediaStream) -> Result<(), ApiError> {
        let mut state = self.state()?;
        if self.shutdown.load(Ordering::Acquire) || state.host.state != BrowserHostState::Ready {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original browser host stopped",
            ));
        }
        work(&mut state, &stream.ticket)?.check_media_stream(stream)
    }
    fn activate_episode(&self, ticket: &BrowserTicket, generation: u64) -> Result<(), ApiError> {
        let mut state = self.state()?;
        let work = work(&mut state, ticket)?;
        let mut candidate = work.clone();
        candidate.reserve_episode_activation(generation)?;
        self.send(Command::ActivateEpisode(ticket.clone(), generation))?;
        *work = candidate;
        Ok(())
    }
    fn try_start(&self, request: ValidatedBrowserRequest) -> Result<BrowserTicket, ApiError> {
        let mut state = self.state()?;
        if self.shutdown.load(Ordering::Acquire) || state.host.state != BrowserHostState::Ready {
            return Err(error(ErrorCode::NeedsInput, "Browser host is not ready"));
        }
        if state.works.len() >= MAX_RETAINED_TICKETS
            || state.works.iter().any(|w| !w.snapshot().state.terminal())
        {
            return Err(error(
                ErrorCode::ResourceExhausted,
                "Browser ticket or active-work limit reached",
            ));
        }
        let ticket = BrowserTicket::new(
            self.session_id.clone(),
            request.request().request_id.clone(),
            state.next_generation,
        )
        .map_err(|_| error(ErrorCode::Internal, "Browser ticket identity failed"))?;
        let next = state
            .next_generation
            .checked_add(1)
            .ok_or_else(|| error(ErrorCode::ResourceExhausted, "Browser generation exhausted"))?;
        let admitted = BrowserWork::new(ticket.clone(), request)?;
        self.send(Command::Start(ticket.clone()))?;
        state.works.push(admitted);
        state.next_generation = next;
        Ok(ticket)
    }
    fn poll(&self, ticket: &BrowserTicket) -> Result<BrowserSnapshot, ApiError> {
        let mut state = self.state()?;
        Ok(work(&mut state, ticket)?.snapshot())
    }
    fn cancel(&self, ticket: &BrowserTicket) -> Result<CancelDisposition, ApiError> {
        let mut state = self.state()?;
        let work = work(&mut state, ticket)?;
        let snapshot = work.snapshot();
        if snapshot.state.terminal() {
            return Ok(CancelDisposition::AlreadyTerminal);
        }
        if snapshot.state == BrowserWorkState::Cancelling {
            return Ok(CancelDisposition::AlreadyRequested);
        }
        self.send(Command::Cancel(ticket.clone()))?;
        Ok(work.cancel())
    }
    fn navigate(&self, ticket: &BrowserTicket, generation: u64, url: &str) -> Result<(), ApiError> {
        let mut state = self.state()?;
        let work = work(&mut state, ticket)?;
        // Queue space is reserved while the worker cannot inspect this locked work.
        // Validate on a snapshot clone first so failed queue admission never leaves pending navigation.
        let mut candidate = work.clone();
        candidate.reserve_navigation(generation, url)?;
        self.send(Command::Navigate(ticket.clone(), url.to_owned()))?;
        *work = candidate;
        Ok(())
    }
    fn release(&self, ticket: &BrowserTicket) -> Result<(), ApiError> {
        let mut state = self.state()?;
        if !work(&mut state, ticket)?.snapshot().state.terminal() {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Active browser work cannot be released",
            ));
        }
        if !work(&mut state, ticket)?.native_capture_released() {
            return Err(error(
                ErrorCode::ResourceExhausted,
                "Original browser capture release is pending",
            ));
        }
        state.works.retain(|work| work.snapshot().ticket != *ticket);
        Ok(())
    }
}
impl BrowserProviderOwner for Handle {
    fn request_shutdown(&self) -> Result<(), ApiError> {
        self.shutdown.store(true, Ordering::Release);
        Ok(())
    }
    fn shutdown_snapshot(&self) -> Result<BrowserHostSnapshot, ApiError> {
        Ok(self.state()?.host.clone())
    }
}
