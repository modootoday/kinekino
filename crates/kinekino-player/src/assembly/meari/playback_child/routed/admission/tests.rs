//! Original admission hook only; no OS window or native process is created.
#[path = "tests/route_retirement.rs"]
mod route_retirement;
use super::*;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_session::transition::TransitionSlot,
    },
    infrastructure::thread_completion::{DispatcherHost, DispatcherShutdown},
};
use std::{
    num::NonZeroUsize,
    rc::Rc,
    time::{Duration, Instant},
};

#[test]
fn diagnostic_reads_preserve_original_error_and_borrow_failure_does_not_revoke_or_close_slot() {
    let mut dispatcher = DispatcherHost::start(NonZeroUsize::new(2).unwrap()).unwrap();
    let slot = Arc::new(TransitionSlot::default());
    let router = Arc::new(ParentEngineRouter::new_with_transitions(
        dispatcher.handle(),
        slot.clone(),
    ));
    let admission = Rc::new(OriginalAdmission::new(router.clone()));
    let original = Arc::new(ApiError::new(ErrorCode::Internal, "original secondary"));
    *admission.secondary.borrow_mut() = Some(RoutedChildError::Api(original.clone()));
    let observed = admission.diagnostic_snapshot().unwrap();
    assert!(!observed.revoked);
    assert!(observed.route_generation.is_none());
    match observed.secondary_error {
        Some(RoutedChildError::Api(error)) => assert!(Arc::ptr_eq(&error, &original)),
        other => panic!("Original error not retained: {other:?}"),
    }
    let held = admission.secondary.borrow_mut();
    assert!(matches!(
        admission.diagnostic_snapshot(),
        Err(RoutedChildError::Borrowed)
    ));
    assert!(admission.require_live().is_ok());
    assert!(!slot.admission_closed());
    drop(held);
    let expired = super::super::ready::RoutedMediaCheckpoint {
        state: std::rc::Weak::new(),
        admission: admission.clone(),
    };
    assert!(matches!(
        expired.diagnostic_snapshot(),
        Err(RoutedChildError::Expired)
    ));
    assert!(admission.require_live().is_ok());
    assert!(!slot.admission_closed());
    drop(expired);
    drop(admission);
    drop(router);
    dispatcher.request_shutdown();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match dispatcher.poll_shutdown() {
            DispatcherShutdown::Joined {
                worker_panicked: false,
                faults: 0,
            } => break,
            DispatcherShutdown::Pending => {
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
            state => panic!("Original dispatcher did not join cleanly: {state:?}"),
        }
    }
}

#[test]
fn loss_before_route_install_closes_only_the_original_configured_transition_slot() {
    let mut dispatcher = DispatcherHost::start(NonZeroUsize::new(2).unwrap()).unwrap();
    let slot = Arc::new(TransitionSlot::default());
    let unrelated = Arc::new(TransitionSlot::default());
    let router = Arc::new(ParentEngineRouter::new_with_transitions(
        dispatcher.handle(),
        slot.clone(),
    ));
    assert_eq!(Arc::strong_count(&slot), 2);
    let admission = OriginalAdmission::new(router.clone());
    assert!(admission.require_live().is_ok());
    admission.revoke().unwrap();
    admission.revoke().unwrap();
    assert!(matches!(
        admission.require_live(),
        Err(RoutedChildError::NotReady)
    ));
    assert_eq!(
        slot.begin(|| panic!("parent loss must close the original slot before activation"))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    let called = Cell::new(false);
    assert_eq!(
        unrelated
            .begin(|| {
                called.set(true);
                Err(ApiError::new(
                    ErrorCode::NeedsInput,
                    "Unrelated slot remains open",
                ))
            })
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    assert!(called.get());
    assert!(slot.end().unwrap().is_none());
    assert!(slot.end().unwrap().is_none());
    drop(admission);
    drop(router);
    assert_eq!(Arc::strong_count(&slot), 1);
    dispatcher.request_shutdown();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match dispatcher.poll_shutdown() {
            DispatcherShutdown::Joined {
                worker_panicked: false,
                faults: 0,
            } => break,
            DispatcherShutdown::Pending => {
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
            state => panic!("Original dispatcher did not join cleanly: {state:?}"),
        }
    }
}
