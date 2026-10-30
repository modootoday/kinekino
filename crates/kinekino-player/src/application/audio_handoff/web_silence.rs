//! Intended application/audio_handoff/web_silence.rs. Exclusive view-level mute lease.
#[path = "web_silence/ports.rs"]
mod ports;
use super::WebSilenceObservation;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_control::StoppedNativeEngine,
    },
    domain::api_values::DecimalU64,
};
pub use ports::WebAudioBackend;

#[derive(Clone, Copy)]
struct Lease {
    id: DecimalU64,
    generation: DecimalU64,
    engine: DecimalU64,
    previous_muted: bool,
    user_revision_at_acquire: u64,
    document_bound: bool,
}
pub struct WebAudioSilenceOwner<B> {
    backend: B,
    active: Option<Lease>,
    last_lease_id: u64,
    last_observation_sequence: u64,
    user_intent_revision: u64,
    last_user_intent: Option<bool>,
}
pub struct WebAudioSilenceReceipt {
    pub lease_id: DecimalU64,
    pub previous_muted: bool,
    pub observation: WebSilenceObservation,
}
#[derive(Debug, serde::Serialize)]
pub struct RestoredWebAudio {
    pub lease_id: DecimalU64,
    pub restored_muted: bool,
    pub current_document_generation: DecimalU64,
}
#[derive(Debug, serde::Serialize)]
pub struct WebUserMuteReceipt {
    pub requested_muted: bool,
    pub revision: DecimalU64,
    pub applied: bool,
    pub deferred_until_native_shutdown: bool,
}
impl<B: WebAudioBackend> WebAudioSilenceOwner<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            active: None,
            last_lease_id: 0,
            last_observation_sequence: 0,
            user_intent_revision: 0,
            last_user_intent: None,
        }
    }
    pub fn active_lease_id(&self) -> Option<DecimalU64> {
        self.active.map(|lease| lease.id)
    }
    fn lease(&self, id: DecimalU64) -> Result<Lease, ApiError> {
        self.active.filter(|lease| lease.id == id).ok_or_else(|| {
            ApiError::new(
                ErrorCode::RevisionConflict,
                "Web audio silence lease changed",
            )
        })
    }
    fn owned_error(&self, mut error: ApiError) -> ApiError {
        error.details = serde_json::json!({"cause":error.details,"web_audio_lease_id":self.active_lease_id(),"web_audio_state_may_have_changed":self.active.is_some(),"cleanup_requires_native_close":true});
        error
    }
    fn next_sequence(&self, after: DecimalU64) -> Result<u64, ApiError> {
        self.last_observation_sequence
            .max(after.0)
            .checked_add(1)
            .ok_or_else(|| {
                self.owned_error(ApiError::new(
                    ErrorCode::ResourceExhausted,
                    "Web audio observation identities exhausted",
                ))
            })
    }
    pub fn silence(
        &mut self,
        generation: DecimalU64,
        engine: DecimalU64,
        after: DecimalU64,
    ) -> Result<WebAudioSilenceReceipt, ApiError> {
        self.acquire(generation, engine, after, true)
    }
    pub fn silence_view(
        &mut self,
        engine: DecimalU64,
        after: DecimalU64,
    ) -> Result<WebAudioSilenceReceipt, ApiError> {
        self.acquire(
            DecimalU64(self.backend.document_generation()),
            engine,
            after,
            false,
        )
    }
    fn acquire(
        &mut self,
        generation: DecimalU64,
        engine: DecimalU64,
        after: DecimalU64,
        document_bound: bool,
    ) -> Result<WebAudioSilenceReceipt, ApiError> {
        if generation.0 == 0 || engine.0 == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid web audio silence identity",
            ));
        }
        if self.active.is_some() {
            return Err(self.owned_error(ApiError::new(
                ErrorCode::RevisionConflict,
                "A web audio silence lease is already owned",
            )));
        }
        if self.backend.document_generation() != generation.0 {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Web document changed before native audio handoff",
            ));
        }
        let id = self.last_lease_id.checked_add(1).ok_or_else(|| {
            ApiError::new(
                ErrorCode::ResourceExhausted,
                "Web audio lease identities exhausted",
            )
        })?;
        self.next_sequence(after)?;
        let previous_muted = self.backend.is_muted()?;
        if self.backend.document_generation() != generation.0 {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Web document changed during audio preflight",
            ));
        }
        self.last_lease_id = id;
        // Retain the original value before a fallible setter: failure may have side effects.
        self.active = Some(Lease {
            id: DecimalU64(id),
            generation,
            engine,
            previous_muted,
            user_revision_at_acquire: self.user_intent_revision,
            document_bound,
        });
        self.backend
            .set_muted(true)
            .map_err(|error| self.owned_error(error))?;
        let observation = self.observe(DecimalU64(id), after)?;
        if observation.view_muted != Some(true) {
            return Err(self.owned_error(ApiError::new(
                ErrorCode::EngineFailed,
                "WebView audio silence was not observed",
            )));
        }
        Ok(WebAudioSilenceReceipt {
            lease_id: DecimalU64(id),
            previous_muted,
            observation,
        })
    }
    /// All host-level user mute changes use this owner. While leased, a request
    /// changes the eventual restore intent without unmuting a hidden web player.
    pub fn set_user_muted(
        &mut self,
        generation: DecimalU64,
        muted: bool,
    ) -> Result<WebUserMuteReceipt, ApiError> {
        if generation.0 == 0 || self.backend.document_generation() != generation.0 {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Web user mute intent belongs to another document",
            ));
        }
        let revision = self.user_intent_revision.checked_add(1).ok_or_else(|| {
            ApiError::new(
                ErrorCode::ResourceExhausted,
                "Web audio user intent identities exhausted",
            )
        })?;
        self.user_intent_revision = revision;
        self.last_user_intent = Some(muted);
        if self.active.is_some() {
            return Ok(WebUserMuteReceipt {
                requested_muted: muted,
                revision: DecimalU64(revision),
                applied: false,
                deferred_until_native_shutdown: true,
            });
        }
        self.backend.set_muted(muted)?;
        let observed = self.backend.is_muted()?;
        if self.backend.document_generation() != generation.0 || observed != muted {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Web user mute intent was not observed",
            ));
        }
        Ok(WebUserMuteReceipt {
            requested_muted: muted,
            revision: DecimalU64(revision),
            applied: true,
            deferred_until_native_shutdown: false,
        })
    }
    pub fn observe(
        &mut self,
        id: DecimalU64,
        after: DecimalU64,
    ) -> Result<WebSilenceObservation, ApiError> {
        let lease = self.lease(id)?;
        let sequence = self.next_sequence(after)?;
        if lease.document_bound && self.backend.document_generation() != lease.generation.0 {
            return Err(self.owned_error(ApiError::new(
                ErrorCode::RevisionConflict,
                "Web document changed during audio silence observation",
            )));
        }
        let muted = self
            .backend
            .is_muted()
            .map_err(|error| self.owned_error(error))?;
        if lease.document_bound && self.backend.document_generation() != lease.generation.0 {
            return Err(self.owned_error(ApiError::new(
                ErrorCode::RevisionConflict,
                "Web document changed during audio silence observation",
            )));
        }
        self.last_observation_sequence = sequence;
        Ok(WebSilenceObservation {
            document_generation: DecimalU64(self.backend.document_generation()),
            observation_sequence: DecimalU64(sequence),
            paused: None,
            muted: None,
            view_muted: Some(muted),
            confirmed_for_engine_instance_id: Some(lease.engine),
        })
    }
    /// Restore only after the bound native process and every worker have stopped.
    /// The lease belongs to the WebView instance, so navigation does not discard cleanup.
    pub fn restore(
        &mut self,
        id: DecimalU64,
        native_closed: &StoppedNativeEngine,
    ) -> Result<RestoredWebAudio, ApiError> {
        let lease = self.lease(id)?;
        if native_closed.engine_instance_id() != lease.engine {
            return Err(self.owned_error(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native shutdown must be confirmed before restoring web audio",
            )));
        }
        let target = if self.user_intent_revision > lease.user_revision_at_acquire {
            self.last_user_intent.unwrap_or(lease.previous_muted)
        } else {
            lease.previous_muted
        };
        let current = self
            .backend
            .is_muted()
            .map_err(|error| self.owned_error(error))?;
        if !current && current != target {
            return Err(self.owned_error(ApiError::new(
                ErrorCode::RevisionConflict,
                "WebView mute changed outside the owned silence lease",
            )));
        }
        if current != target {
            self.backend
                .set_muted(target)
                .map_err(|error| self.owned_error(error))?;
        }
        let observed = self
            .backend
            .is_muted()
            .map_err(|error| self.owned_error(error))?;
        if observed != target {
            return Err(self.owned_error(ApiError::new(
                ErrorCode::EngineFailed,
                "Web audio restoration was not observed",
            )));
        }
        self.active = None;
        Ok(RestoredWebAudio {
            lease_id: id,
            restored_muted: observed,
            current_document_generation: DecimalU64(self.backend.document_generation()),
        })
    }
}

#[cfg(test)]
mod tests;
