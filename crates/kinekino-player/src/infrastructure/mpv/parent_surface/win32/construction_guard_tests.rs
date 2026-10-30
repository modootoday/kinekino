use super::construction_guard::{ConstructionGuard, Phase};
#[test]
fn interrupted_os_calls_cannot_authorize_release_or_not_created() {
    for phase in [Phase::Registering, Phase::Creating] {
        let guard = ConstructionGuard {
            phase,
            pending: None,
        };
        assert!(!guard.release_observable());
    }
    for phase in [Phase::Prepared, Phase::ClassOwned, Phase::Returned] {
        let guard = ConstructionGuard {
            phase,
            pending: None,
        };
        assert!(guard.release_observable());
    }
}
#[test]
fn interrupted_creator_reference_remains_recorded_without_reclaim() {
    use super::window_events::Events;
    use std::sync::Arc;
    let events = Arc::new(Events::new(17));
    let pending = Arc::into_raw(Arc::clone(&events));
    let guard = ConstructionGuard {
        phase: Phase::Creating,
        pending: Some(pending),
    };
    assert!(!guard.release_observable());
    assert_eq!(guard.pending, Some(pending));
    assert_eq!(Arc::strong_count(&events), 2);
    // Only this fake test knows there was no OS call/userdata installation.
    unsafe {
        drop(Arc::from_raw(pending));
    }
}
