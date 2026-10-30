//! One bounded command channel; observation grants and commands share one slot budget.
use super::{
    Failure, FailureStage, Identity, Registry, Ticket,
    barrier::Sample,
    issuer::{ObservationReply, ObservationWaiter},
    results::GrantKey,
    scope::{ConnectionFence, HostScope},
};
use crate::application::api::{ApiError, ErrorCode};
use std::{
    sync::{Arc, Mutex, Weak},
    time::Instant,
};
use tokio::sync::mpsc::{self, Receiver, Sender};

pub(in crate::infrastructure::mpv) struct ObserveInput {
    pub key: GrantKey,
    pub identity: Identity,
    pub generation: u64,
    pub deadline: Instant,
    pub reply: Arc<ObservationReply>,
}
pub(in crate::infrastructure::mpv) struct SubmitInput {
    pub ticket: Ticket,
    pub identity: Identity,
    pub generation: u64,
    pub position_us: u64,
    pub deadline: Instant,
}
pub(in crate::infrastructure::mpv) enum Envelope {
    Observe(ObserveInput),
    Submit(SubmitInput),
}
pub(in crate::infrastructure::mpv) struct GuardedQueue {
    sender: Sender<Envelope>,
    receiver: Mutex<Option<Receiver<Envelope>>>,
    scope: Weak<HostScope>,
    standalone_owner: Option<Arc<HostScope>>,
    pub registry: Registry,
    pub fence: Arc<ConnectionFence>,
}
impl Default for GuardedQueue {
    fn default() -> Self {
        Self::standalone(HostScope::disabled(), 0).expect("Fresh disabled native scope")
    }
}
impl GuardedQueue {
    pub fn standalone(scope: Arc<HostScope>, instance: u64) -> Result<Self, ApiError> {
        Self::assemble(scope, instance, true)
    }
    pub fn scoped(scope: Arc<HostScope>, instance: u64) -> Result<Self, ApiError> {
        Self::assemble(scope, instance, false)
    }
    fn assemble(scope: Arc<HostScope>, instance: u64, standalone: bool) -> Result<Self, ApiError> {
        let fence = scope.connect(instance)?;
        let (sender, receiver) = mpsc::channel(super::results::CAPACITY);
        Ok(Self {
            sender,
            receiver: Mutex::new(Some(receiver)),
            scope: Arc::downgrade(&scope),
            registry: scope.registry.clone(),
            fence,
            standalone_owner: standalone.then_some(scope),
        })
    }
    pub fn take_receiver(&self) -> Result<Receiver<Envelope>, ApiError> {
        self.receiver
            .lock()
            .map_err(|_| ApiError::new(ErrorCode::Internal, "Native guarded queue poisoned"))?
            .take()
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::NeedsInput,
                    "Original worker receiver already taken",
                )
            })
    }
    pub fn observe(
        &self,
        identity: Identity,
        has_error: bool,
        deadline: Instant,
    ) -> Result<ObservationWaiter, ApiError> {
        let scope = self.scope.upgrade().ok_or_else(super::issuer::expired)?;
        scope.require_enabled()?;
        let generation = self.fence.generation();
        self.fence.validate(generation)?;
        if has_error {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Native engine has an observed error",
            ));
        }
        let deadline = deadline.min(super::super::control::guarded_deadline(Instant::now())?);
        if Instant::now() >= deadline {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Native observation deadline elapsed",
            ));
        }
        let (key, reply) = self.registry.reserve_observation(
            &scope,
            &self.fence,
            identity.clone(),
            generation,
            deadline,
        )?;
        let packet = ObserveInput {
            key: key.clone(),
            identity,
            generation,
            deadline,
            reply: reply.clone(),
        };
        if let Err(error) = self.sender.try_send(Envelope::Observe(packet)) {
            self.registry.reject_observation(&key, send_error(error))?;
            return Err(ApiError::new(
                if self.sender.is_closed() {
                    ErrorCode::EngineFailed
                } else {
                    ErrorCode::ResourceExhausted
                },
                "Native observation packet was not accepted",
            ));
        }
        Ok(ObservationWaiter {
            key: Some(key),
            reply,
            deadline,
        })
    }
    pub fn finish_observation(&self, input: &ObserveInput, sample: Sample) -> Result<(), ApiError> {
        let scope = self.scope.upgrade().ok_or_else(super::issuer::expired)?;
        match super::issuer::issue(&scope, &self.fence, self.sender.clone(), &input.key, sample) {
            Ok(guard) => input.reply.finish(Ok(guard)),
            Err(error) => self.registry.reject_observation(&input.key, error),
        }
    }
    pub fn close(&self) -> Result<(), ApiError> {
        let failure = Failure::text(
            FailureStage::Connection,
            "Original native connection closed",
        );
        if let Some(owner) = &self.standalone_owner {
            owner.close(failure)
        } else {
            self.fence.retire(failure)
        }
    }
    pub fn fail_connection(&self, error: &std::io::Error) -> Result<(), ApiError> {
        let failure = Failure::io(
            if error.kind() == std::io::ErrorKind::TimedOut {
                FailureStage::Deadline
            } else {
                FailureStage::Connection
            },
            error,
        );
        if let Some(owner) = &self.standalone_owner {
            owner.close(failure)
        } else {
            self.fence.retire(failure)
        }
    }
}
pub(super) fn send_error(error: mpsc::error::TrySendError<Envelope>) -> ApiError {
    ApiError::new(
        match error {
            mpsc::error::TrySendError::Full(_) => ErrorCode::ResourceExhausted,
            mpsc::error::TrySendError::Closed(_) => ErrorCode::EngineFailed,
        },
        "Original native queue unavailable; packet not accepted",
    )
}
