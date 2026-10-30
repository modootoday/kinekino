//! One live owner and bounded replay history. The launcher owns platform/proof admission.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_control::ControlReceipt,
        review_source::{CaseLease, MutationControl},
    },
    domain::{
        api_values::DecimalU64,
        playback_control::PlaybackControl,
        review_source::{ReviewCase, valid_review_object_id},
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub case: ReviewCase,
    pub association_id: String,
    pub association_revision: DecimalU64,
}
impl Selection {
    fn validate(&self) -> Result<(), ApiError> {
        self.case.validate().map_err(|_| invalid())?;
        if !valid_review_object_id(&self.association_id) || self.association_revision.0 == 0 {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRef {
    pub host_scope: String,
    pub ordinal: DecimalU64,
    pub selection: Selection,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct OwnerObservation {
    pub process_created: bool,
    pub connected: bool,
    pub closing: bool,
    pub physical_closed: bool,
    pub family_empty: bool,
    pub ipc_exited: bool,
    pub owner_finished: bool,
    pub engine_instance_id: Option<DecimalU64>,
    pub has_error: bool,
    pub failure: Option<SessionFailure>,
    pub position_us: Option<DecimalU64>,
    pub duration_us: Option<DecimalU64>,
    pub paused: Option<bool>,
    pub volume_percent: Option<f64>,
    pub muted: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SessionFailure {
    pub code: ErrorCode,
    pub message: String,
}
impl OwnerObservation {
    fn quiescent(&self) -> bool {
        self.owner_finished
            && self.physical_closed
            && (!self.process_created || self.family_empty && self.ipc_exited)
    }
}
/// Dropping the public owner must request close while its original runtime
/// independently retains resources until observed physical completion.
pub trait SessionOwner: Send {
    fn selection(&self) -> &Selection;
    fn observe(&self) -> Result<OwnerObservation, ApiError>;
    fn request_close(&self) -> Result<(), ApiError>;
    fn submit(
        &self,
        _instance: DecimalU64,
        _intent: PlaybackControl,
    ) -> Result<ControlReceipt, ApiError> {
        Err(control_denied(
            ErrorCode::Unsupported,
            "Review controls are unavailable",
        ))
    }
}
pub trait SessionLauncher: Send + Sync {
    fn controls_available(&self) -> bool {
        false
    }
    /// Case authorization only, including replay/read/close after media disappears.
    fn retain_case(&self, case: &ReviewCase) -> Result<Box<dyn CaseLease + '_>, ApiError>;
    /// Resolve exact stored revision, require Verified metadata and freshly retained
    /// matching proof, then explicit runtime admission. Never trust caller summaries.
    /// Err is allowed only before an owner/process exists; partial starts return an owner.
    fn start(&self, selection: &Selection) -> Result<Box<dyn SessionOwner>, ApiError>;
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SessionView {
    pub session: SessionRef,
    pub close_requested: bool,
    pub observation: OwnerObservation,
}
struct Entry {
    key: String,
    session: SessionRef,
    owner: Box<dyn SessionOwner>,
    close_requested: bool,
    terminal: Option<OwnerObservation>,
}
impl Entry {
    fn retained_error(&self, mut error: ApiError) -> ApiError {
        error.details = serde_json::json!({"delivery":"execution_unknown","owner_retained":true,"session":self.session,"query_same_open_key":true,"automatic_retry":false});
        error
    }
    fn view(&mut self) -> Result<SessionView, ApiError> {
        if self.owner.selection() != &self.session.selection {
            self.close_requested = true;
            let _ = self.owner.request_close();
            return Err(
                self.retained_error(conflict("Runtime returned a foreign association owner"))
            );
        }
        let observation = if let Some(terminal) = &self.terminal {
            terminal.clone()
        } else {
            self.owner
                .observe()
                .map_err(|error| self.retained_error(error))?
        };
        if observation.quiescent() {
            self.terminal = Some(observation.clone());
        }
        Ok(SessionView {
            session: self.session.clone(),
            close_requested: self.close_requested,
            observation,
        })
    }
}
impl Drop for Entry {
    fn drop(&mut self) {
        if self.terminal.is_none() {
            let _ = self.owner.request_close();
        }
    }
}
pub struct ReviewPlaybackSessions {
    host_scope: String,
    max_history: usize,
    launcher: Box<dyn SessionLauncher>,
    entries: Mutex<Vec<Entry>>,
    closing: std::sync::atomic::AtomicBool,
}
mod host;
pub use host::ShutdownObservation;
impl ReviewPlaybackSessions {
    pub fn controls_available(&self) -> bool {
        self.launcher.controls_available()
    }
    pub fn control(
        &self,
        session: &SessionRef,
        instance: DecimalU64,
        intent: PlaybackControl,
    ) -> Result<ControlReceipt, ApiError> {
        if instance.0 == 0 {
            return Err(control_denied(
                ErrorCode::InvalidArgument,
                "Positive original engine instance required",
            ));
        }
        intent
            .validate()
            .map_err(|message| control_denied(ErrorCode::InvalidArgument, message))?;
        if !self.controls_available() {
            return Err(control_denied(
                ErrorCode::Unsupported,
                "Review controls are unavailable",
            ));
        }
        let attempted = std::cell::Cell::new(false);
        let result = self.with_entry(session, |entry| {
            if self.closing.load(std::sync::atomic::Ordering::Acquire) {
                return Err(control_denied(ErrorCode::NotFound, "Review host is closing"));
            }
            let view = entry.view()?;
            if view.close_requested || view.observation.closing || view.observation.physical_closed || view.observation.owner_finished {
                return Err(control_denied(ErrorCode::NotFound,"Original review session is closing or closed"));
            }
            if view.observation.has_error { return Err(control_denied(ErrorCode::EngineFailed,"Review session failed")); }
            let actual = view.observation.engine_instance_id.ok_or_else(|| control_denied(ErrorCode::NeedsInput,"Review instance is not ready"))?;
            if actual != instance { return Err(control_denied(ErrorCode::RevisionConflict,"Review instance differs")); }
            if !view.observation.connected { return Err(control_denied(ErrorCode::NeedsInput,"Review control is not connected")); }
            attempted.set(true);
            let receipt = entry.owner.submit(instance, intent)?;
            if receipt.engine_instance_id != instance || receipt.acceptance != crate::application::playback_control::ControlAcceptance::Accepted {
                let mut error = ApiError::new(ErrorCode::EngineFailed,"Review control receipt binding is uncertain");
                error.details = serde_json::json!({"delivery":"execution_unknown","automatic_retry":false,"session":session});
                return Err(error);
            }
            Ok(receipt)
        });
        result.map_err(|mut error| {
            if !attempted.get() { error.details = serde_json::json!({"delivery":"not_sent","automatic_retry":false,"cause":error.details}); }
            error
        })
    }
    pub fn max_history(&self) -> usize {
        self.max_history
    }
    /// Trusted host supplies a fresh scope for each host lifetime; never reuse it
    /// after restart. Capacity exhaustion preserves all committed replay identities.
    pub fn new(
        host_scope: String,
        max_history: usize,
        launcher: impl SessionLauncher + 'static,
    ) -> Result<Self, ApiError> {
        if !valid_review_object_id(&host_scope) || !(1..=64).contains(&max_history) {
            return Err(invalid());
        }
        Ok(Self {
            host_scope,
            max_history,
            launcher: Box::new(launcher),
            entries: Mutex::new(Vec::new()),
            closing: std::sync::atomic::AtomicBool::new(false),
        })
    }
    pub fn open(
        &self,
        selection: Selection,
        key: &str,
        control: &dyn MutationControl,
    ) -> Result<SessionView, ApiError> {
        selection.validate()?;
        if !valid_review_object_id(key) {
            return Err(invalid());
        }
        let case = self.launcher.retain_case(&selection.case)?;
        if case.case() != &selection.case {
            return Err(conflict("Foreign case authority"));
        }
        case.check_current()?;
        let mut entries = self.entries.lock().map_err(|_| poisoned())?;
        if let Some(entry) = entries
            .iter_mut()
            .find(|entry| entry.session.selection.case == selection.case && entry.key == key)
        {
            if entry.session.selection != selection {
                return Err(conflict("Open key belongs to a different selection"));
            }
            return entry.view();
        }
        if self.closing.load(std::sync::atomic::Ordering::Acquire) {
            return Err(ApiError::new(ErrorCode::NotFound, "Review host is closing"));
        }
        control.checkpoint()?;
        for entry in entries.iter_mut() {
            if !entry.view()?.observation.quiescent() {
                return Err(conflict(
                    "Original review owner has not physically finished",
                ));
            }
        }
        if entries.len() == self.max_history {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Review session history is full; existing keys remain queryable",
            ));
        }
        control.checkpoint()?;
        case.check_current()?;
        let owner = self.launcher.start(&selection)?;
        let session = SessionRef {
            host_scope: self.host_scope.clone(),
            ordinal: DecimalU64(entries.len() as u64 + 1),
            selection,
        };
        entries.push(Entry {
            key: key.into(),
            session,
            owner,
            close_requested: false,
            terminal: None,
        });
        // Keep the original owner even if observation/binding fails after launch.
        entries.last_mut().expect("just inserted owner").view()
    }
    pub fn status(&self, session: &SessionRef) -> Result<SessionView, ApiError> {
        self.access(session, false)
    }
    pub fn close(&self, session: &SessionRef) -> Result<SessionView, ApiError> {
        self.access(session, true)
    }
    fn access(&self, session: &SessionRef, close: bool) -> Result<SessionView, ApiError> {
        self.with_entry(session, |entry| {
            let before = entry.view()?;
            if close && !before.observation.quiescent() && !entry.close_requested {
                entry.owner.request_close()?;
                entry.close_requested = true;
            }
            entry.view()
        })
    }
    fn with_entry<T>(
        &self,
        session: &SessionRef,
        operation: impl FnOnce(&mut Entry) -> Result<T, ApiError>,
    ) -> Result<T, ApiError> {
        session.selection.validate()?;
        if session.host_scope != self.host_scope || session.ordinal.0 == 0 {
            return Err(conflict("Foreign review host/session"));
        }
        let case = self.launcher.retain_case(&session.selection.case)?;
        if case.case() != &session.selection.case {
            return Err(conflict("Foreign case authority"));
        }
        case.check_current()?;
        let mut entries = self.entries.lock().map_err(|_| poisoned())?;
        let entry = entries
            .iter_mut()
            .find(|entry| entry.session.ordinal == session.ordinal)
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Review session missing"))?;
        if &entry.session != session {
            return Err(conflict("Review session selection differs"));
        }
        operation(entry)
    }
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid explicit review session selection or scope",
    )
}
fn control_denied(code: ErrorCode, message: &str) -> ApiError {
    let mut error = ApiError::new(code, message);
    error.details = serde_json::json!({"delivery":"not_sent","automatic_retry":false});
    error
}
fn conflict(message: &str) -> ApiError {
    ApiError::new(ErrorCode::RevisionConflict, message)
}
fn poisoned() -> ApiError {
    ApiError::new(ErrorCode::Internal, "Review session reservation poisoned")
}

#[cfg(test)]
mod tests;
