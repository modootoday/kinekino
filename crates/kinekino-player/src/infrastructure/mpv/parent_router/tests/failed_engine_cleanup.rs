//! Actual existing dispatcher/task, synthetic enabled NoMedia engines; no production readiness.
use super::*;
use crate::infrastructure::mpv::{ParentCleanupObservation, ParentCleanupStatus};
#[path = "failed_engine_cleanup/fixture.rs"]
mod fixture;

#[test]
fn failed_engine_cleanup_closes_only_its_original_scope_and_joins_without_retargeting() {
    let mut dispatcher = host(8);
    let router = ParentEngineRouter::new(dispatcher.handle());
    let original = fixture::engine();
    let route = router.prepare(original.clone()).unwrap();
    router.activate(&route).unwrap();
    let original_scope = original.guard_scope().upgrade().unwrap();
    let failed = fixture::engine();
    let failed_scope = failed.guard_scope().upgrade().unwrap();
    let weak = Arc::downgrade(&failed);
    let refused = router.prepare(failed.clone()).unwrap_err();
    assert_eq!(refused.error.code, ErrorCode::RevisionConflict);
    assert!(Arc::ptr_eq(&refused.engine, &failed));
    let (notification, received) = wake();
    let mut task = router
        .prepare_failed_engine_cleanup(&refused.engine, notification)
        .unwrap();
    assert!(failed_scope.word.is_closed());
    assert!(!original_scope.word.is_closed());
    assert!(matches!(
        task.observe(),
        ParentCleanupObservation::NotStarted
    ));
    assert!(task.notification().is_none());
    drop(refused);
    drop(failed);
    assert!(weak.upgrade().is_some());
    task.start_attempt().unwrap();
    received.recv_timeout(LIMIT).unwrap();
    assert!(matches!(
        task.observe(),
        ParentCleanupObservation::Finished(Ok(ParentCleanupStatus::Complete(None)))
    ));
    assert!(
        matches!(router.status().unwrap(), RouterStatus::Active { generation } if generation == route.generation())
    );
    assert_eq!(
        router.current_route().unwrap().unwrap().generation(),
        route.generation()
    );
    assert!(!original_scope.word.is_closed());
    assert_eq!(
        task.start_attempt().unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    drop(task);
    assert!(weak.upgrade().is_none());
    complete(&router, &route);
    finish(&mut dispatcher);
}
