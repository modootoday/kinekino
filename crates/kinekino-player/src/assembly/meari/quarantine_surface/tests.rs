use super::{
    QuarantineCheckpoint,
    cleanup::progress,
    construction::{pending, request_redraw},
};
use crate::infrastructure::mpv::ProbeRelease;
use meari_native::application::window_lifecycle::CleanupProgress;

#[test]
fn pending_requests_only_one_redraw_for_original_owner() {
    let mut requested = false;
    assert!(request_redraw(&mut requested));
    assert!(!request_redraw(&mut requested));
    assert!(!request_redraw(&mut requested));
}
#[test]
fn missing_destroy_ack_never_completes() {
    assert_eq!(
        progress(ProbeRelease::Unobserved),
        CleanupProgress::ObservationUnavailable
    );
    assert_eq!(
        progress(ProbeRelease::NotCreated),
        CleanupProgress::Complete
    );
    assert_eq!(
        progress(ProbeRelease::Complete { generation: 7 }),
        CleanupProgress::Complete
    );
}

#[test]
fn expired_checkpoint_cannot_recreate_or_read_child() {
    let checkpoint = QuarantineCheckpoint {
        owner: std::rc::Weak::new(),
    };
    assert!(checkpoint.advance().is_err());
    assert!(
        checkpoint
            .with_disabled_child(|_| panic!("expired owner must not invoke consumer"))
            .is_err()
    );
    assert!(checkpoint.forward_cleanup_notice().is_err());
}

#[test]
fn only_transient_checked_access_can_request_a_new_frame() {
    use crate::infrastructure::mpv::{ChildInputError, ProbeError};
    use meari_native::{
        domain::surface_slot::SurfaceError,
        infrastructure::window::{NativeParentClientError, NativeSurfaceAccessError},
    };
    let access = |error| {
        ChildInputError::Parent(NativeParentClientError::Access(
            NativeSurfaceAccessError::Slot(error),
        ))
    };
    assert!(pending(&access(SurfaceError::NotReady)));
    assert!(pending(&access(SurfaceError::Busy)));
    assert!(!pending(&access(SurfaceError::Revoked)));
    assert!(!pending(&access(SurfaceError::Stale)));
    assert!(!pending(&ChildInputError::Parent(
        NativeParentClientError::PlacementMismatch
    )));
    assert!(!pending(&ChildInputError::Child(
        ProbeError::ObservationMismatch
    )));
}

#[test]
fn real_coordinator_retains_failed_partial_and_retries_class_finalization() {
    use super::{
        QuarantineError,
        cleanup::{release_failure, starts_new_attempt},
    };
    use meari_native::application::window_lifecycle::*;
    use std::{cell::RefCell, rc::Rc, task::Waker, time::Duration};
    struct Clock;
    impl MonotonicClock for Clock {
        fn now(&self) -> Duration {
            Duration::ZERO
        }
    }
    struct ClassFailureChild {
        _resource: Rc<()>,
        requests: Rc<RefCell<Vec<CleanupRequest>>>,
    }
    impl NativeChild for ClassFailureChild {
        type Error = QuarantineError;
        fn parent_lost(&mut self) {}
        fn begin_cleanup(
            &mut self,
            request: CleanupRequest,
            signal: CompletionSignal,
        ) -> Result<CleanupProgress, Self::Error> {
            assert_eq!(signal.stamp(), request.stamp);
            self.requests.borrow_mut().push(request);
            if self.requests.borrow().len() == 1 {
                release_failure(QuarantineError::Unavailable, true)
            } else {
                Ok(CleanupProgress::Complete)
            }
        }
        fn observe_cleanup(&mut self, _: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
            panic!("failed class finalization must await explicit retry, not restart in observe")
        }
    }
    let lifecycle = WindowLifecycle::new(
        LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
        Clock,
        Waker::noop().clone(),
    )
    .unwrap();
    let window = lifecycle.ready(ReadyKind::NewWindow).unwrap();
    let resource = Rc::new(());
    let weak = Rc::downgrade(&resource);
    let requests = Rc::new(RefCell::new(Vec::new()));
    let failure = lifecycle
        .register(window, |_| {
            Err(ChildCreationFailure {
                error: QuarantineError::Unavailable,
                partial: Some(ClassFailureChild {
                    _resource: resource,
                    requests: requests.clone(),
                }),
            })
        })
        .err()
        .unwrap();
    assert!(weak.upgrade().is_some());
    let lease = failure.cleanup.unwrap();
    let failed = lifecycle.release_child(&lease).unwrap();
    assert_eq!(failed.failed, 1);
    assert!(weak.upgrade().is_some());
    lifecycle.poll().unwrap();
    lifecycle.release_child(&lease).unwrap();
    assert_eq!(requests.borrow().len(), 1);
    assert_eq!(
        lifecycle.retry_failed().unwrap().status,
        CleanupStatus::Complete
    );
    assert!(weak.upgrade().is_none());
    let requests = requests.borrow();
    let (first, second) = (requests[0], requests[1]);
    assert_eq!(first.stamp.window(), second.stamp.window());
    assert_eq!(first.stamp.child(), second.stamp.child());
    assert_ne!(first.stamp, second.stamp);
    assert!(!starts_new_attempt(Some(first), first).unwrap());
    assert!(starts_new_attempt(Some(first), second).unwrap());
    assert!(
        starts_new_attempt(
            Some(first),
            CleanupRequest {
                reason: RevocationReason::ForcedLoss,
                ..first
            }
        )
        .is_err()
    );
    assert_eq!(
        release_failure(QuarantineError::Unavailable, false).unwrap(),
        CleanupProgress::ObservationUnavailable
    );
}

#[test]
fn uncertain_original_attempt_cannot_be_promoted_to_retry_by_access_failure() {
    use super::QuarantineError;
    use super::cleanup::{may_retry_release, release_failure};
    // First begin has installed no cleanup work; a new explicit attempt is allowed.
    assert!(may_retry_release(true, true, false, false));
    // Same-attempt observe or duplicate begin must retain uncertain original work.
    for begin in [false, true] {
        let retry = may_retry_release(begin, false, false, true);
        assert!(!retry);
        assert_eq!(
            release_failure(QuarantineError::Unavailable, retry).unwrap(),
            CleanupProgress::ObservationUnavailable
        );
    }
    // Confirmed original slot access and completed destruction allow class-only finalization.
    assert!(may_retry_release(false, false, true, true));
    assert!(!may_retry_release(false, false, true, false));
}

/// Allocated OS fixture only. Never selected by the ordinary Native unit gate.
mod own_window;
