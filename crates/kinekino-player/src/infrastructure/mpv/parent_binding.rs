//! Trusted one-shot engine binding, without OS handles or renderer leases.
//! Admission is not proof that an OS parent remains alive.
#[path = "parent_binding/ports.rs"]
mod ports;
use crate::application::{
    api::{ApiError, ErrorCode},
    playback_session::{ClosedEpisodePlayback, EpisodePlaybackEngine},
};
pub use ports::ParentOwnedEngine;
use std::{
    num::NonZeroU64,
    sync::{
        Arc, Condvar, Mutex, TryLockError, Weak,
        atomic::{AtomicBool, Ordering},
    },
};

mod cleanup_task;
pub use cleanup_task::{ParentCleanupObservation, ParentCleanupTask};

#[derive(Clone, Debug)]
pub enum ParentCleanupStatus {
    Pending,
    Incomplete(ClosedEpisodePlayback),
    Complete(Option<ClosedEpisodePlayback>),
}

/// A refused bind returns the proposed owner; it is never silently replaced.
pub struct ParentBindFailure {
    pub error: ApiError,
    pub engine: Arc<dyn ParentOwnedEngine>,
}
impl std::fmt::Debug for ParentBindFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ParentBindFailure")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

struct Slot {
    engine: Option<Arc<dyn ParentOwnedEngine>>,
    in_flight: usize,
    completed: Option<Option<ClosedEpisodePlayback>>,
}
struct Shared {
    scope: Option<ScopedWord>,
    epoch: NonZeroU64,
    revoked: AtomicBool,
    slot: Mutex<Slot>,
    cleanup: Mutex<()>,
    drained: Condvar,
}
struct ScopedWord {
    owner: Weak<super::guarded_command::HostScope>,
    word: Arc<super::guarded_command::Fence>,
}
struct ReadOwner<R> {
    binding: Weak<Shared>,
    router: Option<Weak<R>>,
    epoch: NonZeroU64,
    scope: Option<Weak<super::guarded_command::HostScope>>,
}
struct BoundExportIssuer<R> {
    identity: ReadOwner<R>,
    original: crate::application::playback_session::v1::NativeExportIssuer,
}
struct ExportLookupOwners<R> {
    _binding: Arc<Shared>,
    _scope: Arc<super::guarded_command::HostScope>,
    _router: Option<Arc<R>>,
}
impl<R> std::fmt::Debug for BoundExportIssuer<R> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("OriginalBoundExportIssuer")
    }
}
impl<R: Send + Sync + 'static> ReadOwner<R> {
    fn same_owner(&self, other: &Self) -> bool {
        self.epoch == other.epoch
            && Weak::ptr_eq(&self.binding, &other.binding)
            && match (&self.router, &other.router) {
                (None, None) => true,
                (Some(a), Some(b)) => Weak::ptr_eq(a, b),
                _ => false,
            }
            && match (&self.scope, &other.scope) {
                (Some(a), Some(b)) => Weak::ptr_eq(a, b),
                _ => false,
            }
    }
    fn hold_for_lookup(&self) -> Result<ExportLookupOwners<R>, ApiError> {
        let binding = self.binding.upgrade().ok_or_else(expired_read_owner)?;
        if binding.epoch != self.epoch {
            return Err(foreign_read_owner());
        }
        let original = self.scope.as_ref().ok_or_else(foreign_read_owner)?;
        if !binding
            .scope
            .as_ref()
            .is_some_and(|scope| Weak::ptr_eq(&scope.owner, original))
        {
            return Err(foreign_read_owner());
        }
        let scope = original.upgrade().ok_or_else(expired_read_owner)?;
        let router = self
            .router
            .as_ref()
            .map(|owner| owner.upgrade().ok_or_else(expired_read_owner))
            .transpose()?;
        // Revocation stops admission; it does not revoke original terminal reads.
        Ok(ExportLookupOwners {
            _binding: binding,
            _scope: scope,
            _router: router,
        })
    }
}
impl<R: Send + Sync + 'static> crate::application::playback_session::NativeExportIssuerSourceV1
    for BoundExportIssuer<R>
{
    fn matches_issuer(&self, issuer: &dyn std::any::Any) -> bool {
        if let Some(identity) = issuer.downcast_ref::<ReadOwner<R>>() {
            self.identity.same_owner(identity)
        } else {
            self.original.require_issuer(issuer).is_ok()
        }
    }
    fn scope(&self) -> crate::application::playback_session::v1::NativeIssuerScopeV1 {
        self.original.scope()
    }
    fn list(
        &self,
        origin: &crate::application::playback_session::v1::NativeExportOriginV1,
    ) -> Result<crate::application::playback_session::v1::NativeExportListV1, ApiError> {
        let _owners = self.identity.hold_for_lookup()?;
        self.original.list(origin)
    }
    fn read(
        &self,
        reference: crate::application::playback_session::v1::NativeResultRefV1,
        origin: &crate::application::playback_session::v1::NativeExportOriginV1,
        consume: bool,
    ) -> Result<crate::application::playback_session::v1::NativeExportResultV1, ApiError> {
        if reference.scope() != self.original.scope() {
            return Err(foreign_read_owner());
        }
        let _owners = self.identity.hold_for_lookup()?;
        if consume {
            self.original.consume(reference, origin)
        } else {
            self.original.peek(reference, origin)
        }
    }
}
impl<R> std::fmt::Debug for ReadOwner<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OriginalParentExportReadOwner")
    }
}
impl<R: Send + Sync + 'static> crate::application::playback_session::NativeExportReadOwner
    for ReadOwner<R>
{
    fn issuer(&self) -> &dyn std::any::Any {
        match &self.scope {
            Some(scope) => scope,
            None => &(),
        }
    }
    fn matches_context(&self, context: &(dyn std::any::Any + Send + Sync)) -> bool {
        context
            .downcast_ref::<Self>()
            .is_some_and(|original| std::ptr::eq(self, original))
    }
    fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()
    }
    fn hold_for_admission(&self) -> Result<Box<dyn std::any::Any + Send + Sync>, ApiError> {
        let binding = self.binding.upgrade().ok_or_else(expired_read_owner)?;
        if binding.epoch != self.epoch {
            return Err(foreign_read_owner());
        }
        if binding.revoked.load(Ordering::Acquire) {
            return Err(ApiError::new(ErrorCode::Cancelled, "Parent was revoked"));
        }
        let original = self.scope.as_ref().ok_or_else(foreign_read_owner)?;
        if !binding
            .scope
            .as_ref()
            .is_some_and(|scope| Weak::ptr_eq(&scope.owner, original))
        {
            return Err(foreign_read_owner());
        }
        let scope = original.upgrade().ok_or_else(expired_read_owner)?;
        if scope.word.is_closed() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Original native scope is closed",
            ));
        }
        let router = self
            .router
            .as_ref()
            .map(|router| router.upgrade().ok_or_else(expired_read_owner))
            .transpose()?;
        Ok(Box::new((binding, router, scope)))
    }
    fn read(
        &self,
        ticket: &crate::application::playback_session::NativeSeekTicket,
        consume: bool,
    ) -> Result<crate::application::playback_session::GuardedSeekResult, ApiError> {
        let context = ticket.read_context().ok_or_else(foreign_read_owner)?;
        if !self.matches_context(context) {
            return Err(foreign_read_owner());
        }
        let binding = self.binding.upgrade().ok_or_else(expired_read_owner)?;
        if binding.epoch != self.epoch {
            return Err(foreign_read_owner());
        }
        let original = self.scope.as_ref().ok_or_else(foreign_read_owner)?;
        if !binding
            .scope
            .as_ref()
            .is_some_and(|scope| Weak::ptr_eq(&scope.owner, original))
        {
            return Err(foreign_read_owner());
        }
        let _scope = original.upgrade().ok_or_else(expired_read_owner)?;
        let _router = self
            .router
            .as_ref()
            .map(|router| router.upgrade().ok_or_else(expired_read_owner))
            .transpose()?;
        ticket.require_issuer(original)?;
        ticket.read_in_context(context, consume)
    }
}

#[derive(Clone)]
pub struct ParentBinding(Arc<Shared>);

fn internal() -> ApiError {
    ApiError::new(ErrorCode::Internal, "Parent binding ownership lock failed")
}

impl ParentBinding {
    pub fn new(epoch: NonZeroU64) -> Self {
        Self::with_scope(epoch, None)
    }
    fn with_scope(epoch: NonZeroU64, scope: Option<ScopedWord>) -> Self {
        Self(Arc::new(Shared {
            scope,
            epoch,
            revoked: AtomicBool::new(false),
            slot: Mutex::new(Slot {
                engine: None,
                in_flight: 0,
                completed: None,
            }),
            cleanup: Mutex::new(()),
            drained: Condvar::new(),
        }))
    }
    pub(super) fn new_scoped(
        epoch: NonZeroU64,
        owner: Weak<super::guarded_command::HostScope>,
    ) -> Result<Self, ApiError> {
        let scope = owner.upgrade().ok_or_else(expired_read_owner)?;
        if scope.word.is_closed() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Original native scope is closed",
            ));
        }
        let word = scope.word.clone();
        Ok(Self::with_scope(epoch, Some(ScopedWord { owner, word })))
    }

    fn require_epoch(&self, epoch: NonZeroU64) -> Result<(), ApiError> {
        if self.0.epoch != epoch {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Stale parent binding epoch",
            ));
        }
        Ok(())
    }

    pub fn bind(
        &self,
        epoch: NonZeroU64,
        engine: Arc<dyn ParentOwnedEngine>,
    ) -> Result<(), Box<ParentBindFailure>> {
        let result = (|| {
            self.require_epoch(epoch)?;
            let mut slot = self.0.slot.lock().map_err(|_| internal())?;
            if self.0.revoked.load(Ordering::Acquire) {
                return Err(ApiError::new(ErrorCode::Cancelled, "Parent was revoked"));
            }
            if slot.engine.is_some() {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Parent engine is already bound",
                ));
            }
            slot.engine = Some(engine.clone());
            Ok(())
        })();
        result.map_err(|error| Box::new(ParentBindFailure { error, engine }))
    }

    /// UI-thread safe signal only. Does not wait, call SDK, or certify cleanup.
    pub fn revoke_admission(&self) {
        if let Some(scope) = &self.0.scope {
            scope.word.close();
        }
        self.0.revoked.store(true, Ordering::Release);
    }

    pub(super) fn admit(&self, epoch: NonZeroU64) -> Result<EngineAdmission, ApiError> {
        self.require_epoch(epoch)?;
        let mut slot = self.0.slot.lock().map_err(|_| internal())?;
        if self.0.revoked.load(Ordering::Acquire) {
            return Err(ApiError::new(ErrorCode::Cancelled, "Parent was revoked"));
        }
        if let Some(scope) = &self.0.scope {
            if scope.owner.upgrade().is_none() {
                return Err(expired_read_owner());
            }
            if scope.word.is_closed() {
                return Err(ApiError::new(
                    ErrorCode::Cancelled,
                    "Original native scope is closed",
                ));
            }
        }
        let engine = slot.engine.clone().ok_or_else(|| {
            ApiError::new(ErrorCode::NeedsInput, "Native parent engine is not ready")
        })?;
        slot.in_flight = slot.in_flight.checked_add(1).ok_or_else(|| {
            ApiError::new(ErrorCode::ResourceExhausted, "Parent admission overflow")
        })?;
        Ok(EngineAdmission {
            shared: self.0.clone(),
            engine,
        })
    }
    pub(super) fn read_bound_result(
        &self,
        epoch: NonZeroU64,
        input: crate::application::playback_session::GuardedSeekResultQuery,
    ) -> Result<crate::application::playback_session::GuardedSeekResult, ApiError> {
        self.require_epoch(epoch)?;
        let opaque = input.ticket.read_context().ok_or_else(foreign_read_owner)?;
        let context = opaque
            .downcast_ref::<ReadOwner<()>>()
            .ok_or_else(foreign_read_owner)?;
        if context.router.is_some() || !Weak::ptr_eq(&context.binding, &Arc::downgrade(&self.0)) {
            return Err(foreign_read_owner());
        }
        let _binding = context.binding.upgrade().ok_or_else(expired_read_owner)?;
        input.ticket.read_in_context(opaque, input.consume_terminal)
    }
    pub(super) fn read_routed_result<R: Send + Sync + 'static>(
        router: &Weak<R>,
        input: crate::application::playback_session::GuardedSeekResultQuery,
    ) -> Result<crate::application::playback_session::GuardedSeekResult, ApiError> {
        let opaque = input.ticket.read_context().ok_or_else(foreign_read_owner)?;
        let context = opaque
            .downcast_ref::<ReadOwner<R>>()
            .ok_or_else(foreign_read_owner)?;
        let original = context.router.as_ref().ok_or_else(foreign_read_owner)?;
        if !Weak::ptr_eq(original, router) {
            return Err(foreign_read_owner());
        }
        let _router = original.upgrade().ok_or_else(expired_read_owner)?;
        let _binding = context.binding.upgrade().ok_or_else(expired_read_owner)?;
        input.ticket.read_in_context(opaque, input.consume_terminal)
    }

    pub(super) fn read_bound_export(
        &self,
        epoch: NonZeroU64,
        input: crate::application::playback_session::NativeExportResultQuery,
    ) -> Result<crate::application::playback_session::GuardedSeekResult, ApiError> {
        self.require_epoch(epoch)?;
        let context = input
            .reference
            .ticket
            .read_context()
            .and_then(|context| context.downcast_ref::<ReadOwner<()>>())
            .ok_or_else(foreign_read_owner)?;
        if context.router.is_some()
            || context.epoch != epoch
            || !Weak::ptr_eq(&context.binding, &Arc::downgrade(&self.0))
        {
            return Err(foreign_read_owner());
        }
        let scope = context
            .scope
            .as_ref()
            .ok_or_else(foreign_read_owner)?
            .upgrade()
            .ok_or_else(expired_read_owner)?;
        input
            .reference
            .ticket
            .require_issuer(context.scope.as_ref().ok_or_else(foreign_read_owner)?)?;
        scope
            .registry
            .read_export(&input.reference, input.consume_terminal)
    }
    pub(super) fn list_bound_exports(
        &self,
        epoch: NonZeroU64,
        provenance: crate::application::playback_session::NativeResultProvenance,
    ) -> Result<Vec<crate::application::playback_session::NativeResultReference>, ApiError> {
        self.require_epoch(epoch)?;
        let scope = self
            .0
            .scope
            .as_ref()
            .ok_or_else(|| {
                ApiError::new(ErrorCode::Unsupported, "Original export scope unavailable")
            })?
            .owner
            .upgrade()
            .ok_or_else(expired_read_owner)?;
        Ok(scope
            .registry
            .exported_references(provenance)?
            .into_iter()
            .filter(|reference| {
                reference
                    .ticket
                    .read_context()
                    .and_then(|context| context.downcast_ref::<ReadOwner<()>>())
                    .is_some_and(|context| {
                        context.router.is_none()
                            && context.epoch == epoch
                            && Weak::ptr_eq(&context.binding, &Arc::downgrade(&self.0))
                    })
            })
            .collect())
    }
    pub(super) fn read_routed_export<R: Send + Sync + 'static>(
        router: &Weak<R>,
        input: crate::application::playback_session::NativeExportResultQuery,
    ) -> Result<crate::application::playback_session::GuardedSeekResult, ApiError> {
        let context = input
            .reference
            .ticket
            .read_context()
            .and_then(|context| context.downcast_ref::<ReadOwner<R>>())
            .ok_or_else(foreign_read_owner)?;
        let original = context.router.as_ref().ok_or_else(foreign_read_owner)?;
        if !Weak::ptr_eq(original, router) {
            return Err(foreign_read_owner());
        }
        let _router = original.upgrade().ok_or_else(expired_read_owner)?;
        let _binding = context.binding.upgrade().ok_or_else(expired_read_owner)?;
        let scope = context
            .scope
            .as_ref()
            .ok_or_else(foreign_read_owner)?
            .upgrade()
            .ok_or_else(expired_read_owner)?;
        input
            .reference
            .ticket
            .require_issuer(context.scope.as_ref().ok_or_else(foreign_read_owner)?)?;
        scope
            .registry
            .read_export(&input.reference, input.consume_terminal)
    }

    /// Worker-only operation: the underlying shutdown may block. The UI must only signal.
    /// Previously admitted work may finish after revoke; cleanup waits for its owner.
    pub fn cleanup(&self, epoch: NonZeroU64) -> Result<ParentCleanupStatus, ApiError> {
        self.cleanup_impl(epoch, false)
    }

    pub(super) fn cleanup_wait(&self, epoch: NonZeroU64) -> Result<ParentCleanupStatus, ApiError> {
        self.cleanup_impl(epoch, true)
    }

    fn cleanup_impl(&self, epoch: NonZeroU64, wait: bool) -> Result<ParentCleanupStatus, ApiError> {
        self.require_epoch(epoch)?;
        if !self.0.revoked.load(Ordering::Acquire) {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Revoke admission first",
            ));
        }
        let _cleanup = if wait {
            self.0.cleanup.lock().map_err(|_| internal())?
        } else {
            match self.0.cleanup.try_lock() {
                Ok(guard) => guard,
                Err(TryLockError::WouldBlock) => return Ok(ParentCleanupStatus::Pending),
                Err(TryLockError::Poisoned(_)) => return Err(internal()),
            }
        };
        let engine = {
            let mut slot = self.0.slot.lock().map_err(|_| internal())?;
            if let Some(receipt) = &slot.completed {
                return Ok(ParentCleanupStatus::Complete(receipt.clone()));
            }
            while slot.in_flight != 0 {
                if !wait {
                    return Ok(ParentCleanupStatus::Pending);
                }
                slot = self.0.drained.wait(slot).map_err(|_| internal())?;
            }
            match &slot.engine {
                Some(engine) => engine.clone(),
                None => {
                    slot.completed = Some(None);
                    return Ok(ParentCleanupStatus::Complete(None));
                }
            }
        };
        let closed = engine.shutdown_for_parent()?;
        if let Some(receipt) = &closed
            && !(receipt.engine_exited
                && receipt.ipc_worker_exited
                && receipt.stderr_worker_exited
                && receipt.stdout_worker_exited)
        {
            return Ok(ParentCleanupStatus::Incomplete(receipt.clone()));
        }
        self.0.slot.lock().map_err(|_| internal())?.completed = Some(closed.clone());
        Ok(ParentCleanupStatus::Complete(closed))
    }
}

pub(super) struct EngineAdmission {
    shared: Arc<Shared>,
    pub engine: Arc<dyn ParentOwnedEngine>,
}
impl EngineAdmission {
    fn export_identity<R: Send + Sync + 'static>(
        &self,
        router: Option<Weak<R>>,
    ) -> Result<ReadOwner<R>, ApiError> {
        let scope = self.shared.scope.as_ref().ok_or_else(|| {
            ApiError::new(
                ErrorCode::Unsupported,
                "Original parent scope is not bound for export",
            )
        })?;
        Ok(ReadOwner {
            binding: Arc::downgrade(&self.shared),
            router,
            epoch: self.shared.epoch,
            scope: Some(scope.owner.clone()),
        })
    }
    pub(super) fn capture_export_issuer<R: Send + Sync + 'static>(
        &self,
        router: Option<Weak<R>>,
    ) -> Result<crate::application::playback_session::v1::NativeExportIssuer, ApiError> {
        let identity = self.export_identity(router)?;
        let scope = identity
            .scope
            .as_ref()
            .ok_or_else(foreign_read_owner)?
            .upgrade()
            .ok_or_else(expired_read_owner)?;
        scope.require_enabled()?;
        let original = scope.export_issuer()?;
        Ok(
            crate::application::playback_session::v1::NativeExportIssuer::from_original_source(
                Arc::new(BoundExportIssuer { identity, original }),
            ),
        )
    }
    pub(super) fn require_export_issuer<R: Send + Sync + 'static>(
        &self,
        issuer: &crate::application::playback_session::v1::NativeExportIssuer,
        router: Option<Weak<R>>,
    ) -> Result<(), ApiError> {
        issuer.require_issuer(&self.export_identity(router)?)
    }
    pub(super) fn prepare_export_read_owner<R: Send + Sync + 'static>(
        &self,
        router: Option<Weak<R>>,
    ) -> Result<crate::application::playback_session::PreparedNativeExportReadOwner, ApiError> {
        let owner = Arc::new(self.export_identity(router)?);
        crate::application::playback_session::PreparedNativeExportReadOwner::new(
            owner.clone(),
            owner,
        )
    }
    pub(super) fn bind_result<R: Send + Sync + 'static>(
        &self,
        submission: crate::application::playback_session::GuardedSeekSubmission,
        router: Option<Weak<R>>,
    ) -> Result<crate::application::playback_session::GuardedSeekSubmission, ApiError> {
        submission.ticket.bind_read_context(Arc::new(ReadOwner {
            binding: Arc::downgrade(&self.shared),
            router,
            epoch: self.shared.epoch,
            scope: self.shared.scope.as_ref().map(|scope| scope.owner.clone()),
        }))?;
        Ok(submission)
    }
}
fn foreign_read_owner() -> ApiError {
    ApiError::new(
        ErrorCode::RevisionConflict,
        "Read ticket belongs to another original route owner",
    )
}
fn expired_read_owner() -> ApiError {
    ApiError::new(
        ErrorCode::SourceExpired,
        "Original read route owner expired",
    )
}
impl Drop for EngineAdmission {
    fn drop(&mut self) {
        if let Ok(mut slot) = self.shared.slot.lock() {
            slot.in_flight -= 1;
            if slot.in_flight == 0 {
                self.shared.drained.notify_all();
            }
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod export_owner_tests {
    use super::*;
    use crate::application::{
        playback_control::ControlReceipt,
        playback_session::{
            NativeExportReadOwner, NativeSeekResultParts, NativeSeekResultSource,
            PreparedNativeExportReadOwner, native_seek_submission,
        },
    };
    use crate::domain::api_values::DecimalU64;
    #[derive(Debug)]
    struct Source(Weak<super::super::guarded_command::HostScope>);
    impl NativeSeekResultSource for Source {
        fn matches_issuer(&self, issuer: &dyn std::any::Any) -> bool {
            issuer
                .downcast_ref::<Weak<super::super::guarded_command::HostScope>>()
                .is_some_and(|issuer| Weak::ptr_eq(issuer, &self.0))
        }
        fn identity(&self) -> (DecimalU64, String) {
            (DecimalU64(7), "03".repeat(32))
        }
        fn read(&self, _: bool) -> Result<NativeSeekResultParts, ApiError> {
            panic!("preparation must not read native source")
        }
    }
    #[test]
    fn prepared_owner_uses_fixed_metadata_and_preserves_generic_single_bind() {
        let scope = super::super::guarded_command::HostScope::testing();
        let epoch = NonZeroU64::new(1).unwrap();
        let binding = ParentBinding::new_scoped(epoch, Arc::downgrade(&scope)).unwrap();
        let owner = Arc::new(ReadOwner::<()> {
            binding: Arc::downgrade(&binding.0),
            router: None,
            epoch,
            scope: Some(Arc::downgrade(&scope)),
        });
        assert!(owner.retained_bytes() <= PreparedNativeExportReadOwner::MAX_METADATA_BYTES);
        let strong = Arc::strong_count(&scope);
        let prepared = PreparedNativeExportReadOwner::new(owner.clone(), owner.clone()).unwrap();
        assert_eq!(Arc::strong_count(&scope), strong);
        let lease = prepared.hold_for_admission().unwrap();
        assert_eq!(Arc::strong_count(&scope), strong + 1);
        drop(lease);
        assert_eq!(Arc::strong_count(&scope), strong);
        let submission = native_seek_submission(
            ControlReceipt::enqueued(7),
            Arc::new(Source(Arc::downgrade(&scope))),
        )
        .unwrap();
        let clone = submission.ticket.clone();
        prepared.bind(&submission.ticket).unwrap();
        assert!(clone.read_context().is_some());
        assert_eq!(
            PreparedNativeExportReadOwner::new(owner.clone(), owner)
                .unwrap()
                .bind(&clone)
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(
            clone.bind_read_context(Arc::new(7_u64)).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
    }
    #[test]
    fn prepared_owner_rejects_unrelated_context_before_binding_any_ticket() {
        let scope = super::super::guarded_command::HostScope::testing();
        let epoch = NonZeroU64::new(1).unwrap();
        let binding = ParentBinding::new_scoped(epoch, Arc::downgrade(&scope)).unwrap();
        let owner = Arc::new(ReadOwner::<()> {
            binding: Arc::downgrade(&binding.0),
            router: None,
            epoch,
            scope: Some(Arc::downgrade(&scope)),
        });
        assert_eq!(
            PreparedNativeExportReadOwner::new(owner, Arc::new(()))
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
    }
}
