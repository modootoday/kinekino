use super::*;
use std::{cell::Cell, rc::Rc};

#[derive(Clone)]
struct Backend {
    muted: Rc<Cell<bool>>,
    generation: Rc<Cell<u64>>,
    fail_set_after_mutation: Rc<Cell<bool>>,
    fail_read: Rc<Cell<bool>>,
    sets: Rc<Cell<u32>>,
}
impl Backend {
    fn new(muted: bool) -> Self {
        Self {
            muted: Rc::new(Cell::new(muted)),
            generation: Rc::new(Cell::new(11)),
            fail_set_after_mutation: Rc::new(Cell::new(false)),
            fail_read: Rc::new(Cell::new(false)),
            sets: Rc::new(Cell::new(0)),
        }
    }
}
impl WebAudioBackend for Backend {
    fn document_generation(&self) -> u64 {
        self.generation.get()
    }
    fn is_muted(&self) -> Result<bool, ApiError> {
        if self.fail_read.get() {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Synthetic native property read failure",
            ));
        }
        Ok(self.muted.get())
    }
    fn set_muted(&self, muted: bool) -> Result<(), ApiError> {
        self.sets.set(self.sets.get() + 1);
        self.muted.set(muted);
        if self.fail_set_after_mutation.get() {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Synthetic setter failure after mutation",
            ));
        }
        Ok(())
    }
}
fn stopped(engine: u64) -> StoppedNativeEngine {
    StoppedNativeEngine::observed(DecimalU64(engine), true, true, true, true).unwrap()
}

#[test]
fn whole_view_silence_survives_navigation_and_restores_after_bound_stop() {
    let backend = Backend::new(false);
    let mut owner = WebAudioSilenceOwner::new(backend.clone());
    let receipt = owner.silence_view(DecimalU64(7), DecimalU64(3)).unwrap();
    backend.generation.set(12);
    let observed = owner.observe(receipt.lease_id, DecimalU64(4)).unwrap();
    assert_eq!(observed.document_generation, DecimalU64(12));
    assert_eq!(observed.view_muted, Some(true));
    owner.restore(receipt.lease_id, &stopped(7)).unwrap();
    assert!(!backend.muted.get());
}

#[test]
fn silence_reads_native_property_and_restores_original_value_only_after_bound_native_shutdown() {
    let backend = Backend::new(false);
    let mut owner = WebAudioSilenceOwner::new(backend.clone());
    let receipt = owner
        .silence(DecimalU64(11), DecimalU64(7), DecimalU64(40))
        .unwrap();
    assert!(!receipt.previous_muted);
    assert_eq!(receipt.observation.view_muted, Some(true));
    assert_eq!(
        receipt.observation.confirmed_for_engine_instance_id,
        Some(DecimalU64(7))
    );
    assert_eq!(receipt.observation.observation_sequence.0, 41);
    assert_eq!(receipt.observation.paused, None);
    assert!(backend.muted.get());
    assert_eq!(
        owner
            .restore(receipt.lease_id, &stopped(6))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert!(backend.muted.get());
    let restored = owner.restore(receipt.lease_id, &stopped(7)).unwrap();
    assert!(!restored.restored_muted);
    assert!(!backend.muted.get());
    assert!(owner.active_lease_id().is_none());
}

#[test]
fn uncertain_setter_failure_keeps_original_value_and_owned_recovery_handle() {
    let backend = Backend::new(false);
    backend.fail_set_after_mutation.set(true);
    let mut owner = WebAudioSilenceOwner::new(backend.clone());
    let error = owner
        .silence(DecimalU64(11), DecimalU64(7), DecimalU64(1))
        .err()
        .expect("Injected setter must fail");
    assert_eq!(error.code, ErrorCode::EngineFailed);
    assert_eq!(error.details["web_audio_state_may_have_changed"], true);
    assert!(backend.muted.get());
    let lease = owner.active_lease_id().unwrap();
    backend.fail_set_after_mutation.set(false);
    assert!(!owner.restore(lease, &stopped(7)).unwrap().restored_muted);
}

#[test]
fn navigation_invalidates_confirmation_but_does_not_discard_view_level_cleanup() {
    let backend = Backend::new(false);
    let mut owner = WebAudioSilenceOwner::new(backend.clone());
    let lease = owner
        .silence(DecimalU64(11), DecimalU64(7), DecimalU64(1))
        .unwrap()
        .lease_id;
    backend.generation.set(12);
    assert_eq!(
        owner
            .observe(lease, DecimalU64(2))
            .err()
            .expect("Changed document must reject confirmation")
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(owner.active_lease_id(), Some(lease));
    let restored = owner.restore(lease, &stopped(7)).unwrap();
    assert_eq!(restored.current_document_generation.0, 12);
    assert!(!restored.restored_muted);
}

#[test]
fn prior_explicit_webview_mute_is_preserved_and_stale_leases_cannot_restore_new_owners() {
    let backend = Backend::new(true);
    let mut owner = WebAudioSilenceOwner::new(backend.clone());
    let first = owner
        .silence(DecimalU64(11), DecimalU64(7), DecimalU64(1))
        .unwrap();
    assert!(first.previous_muted);
    assert!(
        owner
            .restore(first.lease_id, &stopped(7))
            .unwrap()
            .restored_muted
    );
    let second = owner
        .silence(DecimalU64(11), DecimalU64(8), DecimalU64(2))
        .unwrap();
    assert_ne!(second.lease_id, first.lease_id);
    assert_eq!(
        owner.restore(first.lease_id, &stopped(7)).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(owner.active_lease_id(), Some(second.lease_id));
    assert!(
        owner
            .restore(second.lease_id, &stopped(8))
            .unwrap()
            .restored_muted
    );
}

#[test]
fn failed_restoration_retains_lease_for_explicit_retry() {
    let backend = Backend::new(false);
    let mut owner = WebAudioSilenceOwner::new(backend.clone());
    let lease = owner
        .silence(DecimalU64(11), DecimalU64(7), DecimalU64(1))
        .unwrap()
        .lease_id;
    backend.fail_read.set(true);
    assert_eq!(
        owner.restore(lease, &stopped(7)).unwrap_err().code,
        ErrorCode::EngineFailed
    );
    assert_eq!(owner.active_lease_id(), Some(lease));
    backend.fail_read.set(false);
    assert!(!owner.restore(lease, &stopped(7)).unwrap().restored_muted);
}

#[test]
fn exhaustion_and_stale_preflight_reject_before_mutating_the_webview() {
    let backend = Backend::new(false);
    let mut owner = WebAudioSilenceOwner::new(backend.clone());
    assert_eq!(
        owner
            .silence(DecimalU64(10), DecimalU64(7), DecimalU64(1))
            .err()
            .expect("Stale generation must reject")
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        owner
            .silence(DecimalU64(11), DecimalU64(7), DecimalU64(u64::MAX))
            .err()
            .expect("Exhausted sequence must reject")
            .code,
        ErrorCode::ResourceExhausted
    );
    owner.last_lease_id = u64::MAX;
    assert_eq!(
        owner
            .silence(DecimalU64(11), DecimalU64(7), DecimalU64(1))
            .err()
            .expect("Exhausted lease must reject")
            .code,
        ErrorCode::ResourceExhausted
    );
    assert_eq!(backend.sets.get(), 0);
    assert!(!backend.muted.get());
    assert!(StoppedNativeEngine::observed(DecimalU64(7), true, false, true, true).is_err());
}

#[test]
fn user_mute_changes_are_deferred_while_leased_and_override_only_the_restore_intent() {
    let backend = Backend::new(true);
    let mut owner = WebAudioSilenceOwner::new(backend.clone());
    let lease = owner
        .silence(DecimalU64(11), DecimalU64(7), DecimalU64(1))
        .unwrap()
        .lease_id;
    let request = owner.set_user_muted(DecimalU64(11), false).unwrap();
    assert!(!request.applied && request.deferred_until_native_shutdown);
    assert!(backend.muted.get());
    let restored = owner.restore(lease, &stopped(7)).unwrap();
    assert!(!restored.restored_muted);
    assert!(!backend.muted.get());
}

#[test]
fn newer_user_mute_is_not_overwritten_by_the_captured_initial_unmuted_value() {
    let backend = Backend::new(false);
    let mut owner = WebAudioSilenceOwner::new(backend.clone());
    let lease = owner
        .silence(DecimalU64(11), DecimalU64(7), DecimalU64(1))
        .unwrap()
        .lease_id;
    owner.set_user_muted(DecimalU64(11), true).unwrap();
    assert!(owner.restore(lease, &stopped(7)).unwrap().restored_muted);
    assert!(backend.muted.get());
}

#[test]
fn untracked_external_unmute_does_not_get_overwritten_during_restore() {
    let backend = Backend::new(true);
    let mut owner = WebAudioSilenceOwner::new(backend.clone());
    let lease = owner
        .silence(DecimalU64(11), DecimalU64(7), DecimalU64(1))
        .unwrap()
        .lease_id;
    backend.muted.set(false);
    let sets_before = backend.sets.get();
    assert_eq!(
        owner.restore(lease, &stopped(7)).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(backend.sets.get(), sets_before);
    assert!(!backend.muted.get());
    assert_eq!(owner.active_lease_id(), Some(lease));
    owner.set_user_muted(DecimalU64(11), false).unwrap();
    assert!(!owner.restore(lease, &stopped(7)).unwrap().restored_muted);
}

#[test]
fn stale_or_exhausted_user_intent_never_changes_the_webview() {
    let backend = Backend::new(false);
    let mut owner = WebAudioSilenceOwner::new(backend.clone());
    assert_eq!(
        owner.set_user_muted(DecimalU64(10), true).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    owner.user_intent_revision = u64::MAX;
    assert_eq!(
        owner.set_user_muted(DecimalU64(11), true).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    assert_eq!(backend.sets.get(), 0);
}
