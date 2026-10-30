//! Logical host ownership is distinct from a process/connection and from read tickets.
use super::{Failure, FailureStage, Fence, Registry};
use crate::application::api::{ApiError, ErrorCode};
use std::sync::{
    Arc, OnceLock, Weak,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

pub(in crate::infrastructure::mpv) struct HostScope {
    pub registry: Registry,
    pub word: Arc<Fence>,
    enabled: bool,
    trusted_standalone_export: bool,
    issuer_nonce:
        OnceLock<Result<crate::application::playback_session::v1::NativeIssuerScopeV1, ()>>,
}
impl HostScope {
    /// Only the original visible-child callback can construct this borrowed witness.
    #[cfg(feature = "native-proposal-window")]
    pub(in crate::infrastructure::mpv) fn for_verified_parent(
        _: &crate::infrastructure::mpv::TrustedMpvExecutable,
        _: &crate::infrastructure::mpv::CheckedMediaParent<'_>,
    ) -> Arc<Self> {
        Arc::new(Self {
            registry: Registry::default(),
            word: Fence::new(),
            enabled: true,
            trusted_standalone_export: false,
            issuer_nonce: OnceLock::new(),
        })
    }
    /// The verified executable is retained by the engine that owns this scope.
    pub(in crate::infrastructure::mpv) fn for_verified_standalone(
        _: &crate::infrastructure::mpv::TrustedMpvExecutable,
    ) -> Arc<Self> {
        Arc::new(Self {
            registry: Registry::default(),
            word: Fence::new(),
            enabled: true,
            trusted_standalone_export: true,
            issuer_nonce: OnceLock::new(),
        })
    }
    pub fn disabled() -> Arc<Self> {
        Arc::new(Self {
            registry: Registry::default(),
            word: Fence::new(),
            enabled: false,
            trusted_standalone_export: false,
            issuer_nonce: OnceLock::new(),
        })
    }
    #[cfg(test)]
    pub(in crate::infrastructure::mpv) fn testing() -> Arc<Self> {
        Arc::new(Self {
            registry: Registry::default(),
            word: Fence::new(),
            enabled: true,
            trusted_standalone_export: false,
            issuer_nonce: OnceLock::new(),
        })
    }
    pub fn require_enabled(&self) -> Result<(), ApiError> {
        if !self.enabled {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Native observation issuer is not enabled",
            ));
        }
        Ok(())
    }
    pub fn connect(self: &Arc<Self>, instance: u64) -> Result<Arc<ConnectionFence>, ApiError> {
        let generation = self.word.advance()?;
        Ok(Arc::new(ConnectionFence {
            host: Arc::downgrade(self),
            word: self.word.clone(),
            generation: AtomicU64::new(generation),
            retired: AtomicBool::new(false),
            instance,
        }))
    }
    pub(in crate::infrastructure::mpv) fn require_standalone_export(&self) -> Result<(), ApiError> {
        if !self.trusted_standalone_export {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Trusted standalone result export is unavailable",
            ));
        }
        Ok(())
    }
    pub(super) fn prepare_export_owner(
        self: &Arc<Self>,
    ) -> Result<StandaloneExportOwner, ApiError> {
        self.require_standalone_export()?;
        Ok(StandaloneExportOwner {
            host: Arc::downgrade(self),
        })
    }
    #[cfg(test)]
    pub(super) fn testing_export_owner() -> Arc<Self> {
        Arc::new(Self {
            registry: Registry::default(),
            word: Fence::new(),
            enabled: true,
            trusted_standalone_export: true,
            issuer_nonce: OnceLock::new(),
        })
    }
    pub fn close(&self, failure: Failure) -> Result<(), ApiError> {
        self.word.close();
        self.registry.close(failure)
    }
    fn issuer_scope_with(
        &self,
        generate: impl FnOnce() -> Result<
            crate::application::playback_session::v1::NativeIssuerScopeV1,
            (),
        >,
    ) -> Result<crate::application::playback_session::v1::NativeIssuerScopeV1, ApiError> {
        self.require_enabled()?;
        self.issuer_nonce
            .get_or_init(generate)
            .as_ref()
            .copied()
            .map_err(|_| {
                ApiError::new(
                    ErrorCode::EngineFailed,
                    "Original native issuer identity unavailable",
                )
            })
    }
    pub(super) fn issuer_scope(
        &self,
    ) -> Result<crate::application::playback_session::v1::NativeIssuerScopeV1, ApiError> {
        self.issuer_scope_with(|| {
            let bytes = crate::infrastructure::os_policy::fresh_guid().map_err(|_| ())?;
            crate::application::playback_session::v1::NativeIssuerScopeV1::from_original_nonce(
                bytes,
            )
            .map_err(|_| ())
        })
    }
    pub(in crate::infrastructure::mpv) fn export_issuer(
        self: &Arc<Self>,
    ) -> Result<crate::application::playback_session::v1::NativeExportIssuer, ApiError> {
        let id = self.issuer_scope()?;
        Ok(
            crate::application::playback_session::v1::NativeExportIssuer::from_original_source(
                Arc::new(IssuerReader {
                    owner: Arc::downgrade(self),
                    id,
                }),
            ),
        )
    }
    #[cfg(test)]
    pub(in crate::infrastructure::mpv) fn set_issuer_for_test(
        &self,
        value: Result<crate::application::playback_session::v1::NativeIssuerScopeV1, ()>,
    ) {
        assert!(self.issuer_nonce.set(value).is_ok());
    }
}
#[derive(Debug)]
struct IssuerReader {
    owner: Weak<HostScope>,
    id: crate::application::playback_session::v1::NativeIssuerScopeV1,
}
impl crate::application::playback_session::NativeExportIssuerSourceV1 for IssuerReader {
    fn matches_issuer(&self, issuer: &dyn std::any::Any) -> bool {
        issuer
            .downcast_ref::<Weak<HostScope>>()
            .is_some_and(|expected| Weak::ptr_eq(expected, &self.owner))
    }
    fn scope(&self) -> crate::application::playback_session::v1::NativeIssuerScopeV1 {
        self.id
    }
    fn list(
        &self,
        origin: &crate::application::playback_session::v1::NativeExportOriginV1,
    ) -> Result<crate::application::playback_session::v1::NativeExportListV1, ApiError> {
        let owner = self.owner.upgrade().ok_or_else(super::issuer::expired)?;
        owner.registry.list_wire_v1(self.id, origin)
    }
    fn read(
        &self,
        reference: crate::application::playback_session::v1::NativeResultRefV1,
        origin: &crate::application::playback_session::v1::NativeExportOriginV1,
        consume: bool,
    ) -> Result<crate::application::playback_session::v1::NativeExportResultV1, ApiError> {
        if reference.scope() != self.id {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Foreign original native issuer scope",
            ));
        }
        let owner = self.owner.upgrade().ok_or_else(super::issuer::expired)?;
        owner.registry.read_wire_v1(reference, origin, consume)
    }
}
pub(super) struct StandaloneExportOwner {
    pub(super) host: Weak<HostScope>,
}
impl Drop for HostScope {
    fn drop(&mut self) {
        let _ = self.close(Failure::text(
            FailureStage::Connection,
            "Original native issuer owner expired",
        ));
    }
}
pub(in crate::infrastructure::mpv) struct ConnectionFence {
    pub(super) host: Weak<HostScope>,
    word: Arc<Fence>,
    generation: AtomicU64,
    retired: AtomicBool,
    pub instance: u64,
}
impl ConnectionFence {
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }
    pub fn word(&self) -> &Arc<Fence> {
        &self.word
    }
    pub fn is_closed(&self) -> bool {
        self.retired.load(Ordering::Acquire) || self.word.validate(self.generation()).is_err()
    }
    pub fn validate(&self, expected: u64) -> Result<(), ApiError> {
        if self.retired.load(Ordering::Acquire) {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Original connection retired",
            ));
        }
        if expected != self.generation() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original connection generation changed",
            ));
        }
        self.word.validate(expected)
    }
    pub fn advance(&self) -> Result<u64, ApiError> {
        if self.retired.load(Ordering::Acquire) {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Original connection retired",
            ));
        }
        let next = self.word.advance_if(self.generation())?.ok_or_else(|| {
            ApiError::new(
                ErrorCode::RevisionConflict,
                "Another connection owns the current generation",
            )
        })?;
        self.generation.store(next, Ordering::Release);
        Ok(next)
    }
    pub fn retire(&self, failure: Failure) -> Result<(), ApiError> {
        if self.retired.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        // The hint above is not the authority boundary; this same-word CAS is.
        let advanced = self.word.advance_if(self.generation());
        if let Some(host) = self.host.upgrade() {
            host.registry.close_connection(self.instance, failure)?;
        }
        match advanced {
            Ok(_) => Ok(()),
            Err(error) if error.code == ErrorCode::Cancelled => Ok(()),
            Err(error) => Err(error),
        }
    }
    pub fn in_flight(&self) -> u64 {
        self.word.in_flight()
    }
}

#[cfg(test)]
mod issuer_nonce_tests {
    use super::*;
    use crate::application::playback_session::v1::NativeIssuerScopeV1;
    #[test]
    fn success_and_failure_are_each_generated_once() {
        let calls = std::cell::Cell::new(0);
        for succeeds in [true, false] {
            let scope = HostScope::testing();
            for _ in 0..2 {
                let value = scope.issuer_scope_with(|| {
                    calls.set(calls.get() + 1);
                    if succeeds {
                        Ok(NativeIssuerScopeV1::from_original_nonce([1; 16]).unwrap())
                    } else {
                        Err(())
                    }
                });
                assert_eq!(value.is_ok(), succeeds);
                if let Err(error) = value {
                    assert_eq!(error.code, ErrorCode::EngineFailed);
                }
            }
        }
        assert_eq!(calls.get(), 2);
    }
    #[test]
    fn identical_nonce_does_not_authorize_a_foreign_weak_owner() {
        let a = HostScope::testing_export_owner();
        let b = HostScope::testing_export_owner();
        let id = NativeIssuerScopeV1::from_original_nonce([2; 16]).unwrap();
        a.set_issuer_for_test(Ok(id));
        b.set_issuer_for_test(Ok(id));
        let issuer = a.export_issuer().unwrap();
        issuer.require_issuer(&Arc::downgrade(&a)).unwrap();
        assert_eq!(
            issuer.require_issuer(&Arc::downgrade(&b)).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        let weak = Arc::downgrade(&a);
        drop(a);
        assert!(weak.upgrade().is_none());
    }
}
