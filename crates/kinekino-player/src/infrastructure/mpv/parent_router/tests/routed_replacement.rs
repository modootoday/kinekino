//! Pure original admission/retirement tests; no window, browser, decoder or native process.
use super::*;
use crate::application::playback_session::transition::{EpisodeReplacementOwner, ReplacedEpisode};
#[path = "routed_replacement/fixture.rs"]
mod fixture;
use fixture::ReplacementEngine;

#[test]
fn replacement_preserves_expected_identities_and_rejects_unready_or_non_replacing_owner() {
    let mut dispatcher = host(8);
    let router = ParentEngineRouter::new(dispatcher.handle());
    assert_eq!(
        router
            .replace_verified(DecimalU64(10), &"a".repeat(64), source(), true, 17.0)
            .err()
            .expect("original replacement must reject this invalid admission")
            .code,
        ErrorCode::NeedsInput
    );
    let fake = FakeEngine::new(10);
    let route = ready(&router, &fake);
    assert_eq!(
        router
            .replace_verified(DecimalU64(10), &"a".repeat(64), source(), true, 17.0)
            .err()
            .expect("original replacement must reject this invalid admission")
            .code,
        ErrorCode::Unsupported
    );
    assert_eq!(fake.applied.load(Ordering::SeqCst), 0);
    complete(&router, &route);

    let engine = ReplacementEngine::new(20, &router);
    let owner: Arc<dyn ParentOwnedEngine> = engine.clone();
    let route = router.prepare_engine(&owner).unwrap();
    router.activate(&route).unwrap();
    router.open(source(), true, 17.0).unwrap();
    assert_eq!(
        router
            .replace_verified(DecimalU64(99), &"a".repeat(64), source(), true, 17.0)
            .err()
            .expect("original replacement must reject this invalid admission")
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        router
            .replace_verified(DecimalU64(20), &"b".repeat(64), source(), true, 17.0)
            .err()
            .expect("original replacement must reject this invalid admission")
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(engine.replacements.load(Ordering::SeqCst), 0);
    let result = router
        .replace_verified(DecimalU64(20), &"a".repeat(64), source(), false, 23.0)
        .unwrap();
    assert_eq!(result.previous.engine_instance_id, DecimalU64(20));
    assert_eq!(result.current.engine_instance_id, DecimalU64(21));
    assert_eq!(
        result.current.resolution_artifact_id,
        source().artifact_id()
    );
    assert_eq!(*engine.options.lock().unwrap(), Some((false, 23.0)));
    assert_eq!(engine.replacements.load(Ordering::SeqCst), 1);
    router.revoke_route_admission(&route).unwrap();
    assert_eq!(
        router
            .replace_verified(DecimalU64(20), &"a".repeat(64), source(), true, 17.0)
            .err()
            .expect("original replacement must reject this invalid admission")
            .code,
        ErrorCode::Cancelled
    );
    assert_eq!(engine.replacements.load(Ordering::SeqCst), 1);
    complete(&router, &route);
    drop(route);
    drop(router);
    drop(owner);
    drop(engine);
    finish(&mut dispatcher);
}

#[test]
fn in_flight_replacement_keeps_original_engine_and_retirement_waits_before_retargeting() {
    let mut dispatcher = host(8);
    let router = ParentEngineRouter::new(dispatcher.handle());
    let engine = ReplacementEngine::new(30, &router);
    let owner: Arc<dyn ParentOwnedEngine> = engine.clone();
    let route = router.prepare_engine(&owner).unwrap();
    router.activate(&route).unwrap();
    router.open(source(), true, 17.0).unwrap();
    let (gate, entered, resume) = gate();
    *engine.base.gates[0].lock().unwrap() = Some(gate);
    let worker_router = router.clone();
    let worker = std::thread::spawn(move || {
        worker_router.replace_verified(DecimalU64(30), &"a".repeat(64), source(), true, 17.0)
    });
    entered.recv_timeout(LIMIT).unwrap();
    let retirement = router.retire(&route, quiet()).unwrap();
    assert!(retirement.dispatch_error.is_none());
    assert!(!matches!(
        router
            .observe_retirement(&retirement.handle)
            .unwrap()
            .progress,
        RetirementProgress::Complete(_)
    ));
    let other = ReplacementEngine::new(40, &router);
    let other_owner: Arc<dyn ParentOwnedEngine> = other.clone();
    assert_eq!(
        router.prepare_engine(&other_owner).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(other.replacements.load(Ordering::SeqCst), 0);
    assert_eq!(
        router
            .replace_verified(DecimalU64(40), &"a".repeat(64), source(), true, 17.0)
            .err()
            .expect("original replacement must reject this invalid admission")
            .code,
        ErrorCode::Cancelled
    );
    resume.send(()).unwrap();
    let replaced = worker.join().unwrap().unwrap();
    assert_eq!(replaced.previous.engine_instance_id, DecimalU64(30));
    assert_eq!(replaced.current.engine_instance_id, DecimalU64(31));
    let deadline = Instant::now() + LIMIT;
    loop {
        let observed = router.observe_retirement(&retirement.handle).unwrap();
        assert!(observed.retained_failure.is_none());
        assert!(observed.dispatch_error.is_none());
        match observed.progress {
            RetirementProgress::Complete(_) => break,
            RetirementProgress::Pending => {
                assert!(
                    Instant::now() < deadline,
                    "Original retirement did not join"
                );
                std::thread::yield_now();
            }
            state => panic!("Original retirement failed: {state:?}"),
        }
    }
    assert_eq!(engine.replacements.load(Ordering::SeqCst), 1);
    assert_eq!(other.replacements.load(Ordering::SeqCst), 0);
    let next_route = router.prepare_engine(&other_owner).unwrap();
    assert!(next_route.generation() > route.generation());
    router.activate(&next_route).unwrap();
    assert_eq!(
        router
            .replace_verified(DecimalU64(30), &"a".repeat(64), source(), true, 17.0)
            .err()
            .expect("Late original instance cannot target the new route")
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(other.replacements.load(Ordering::SeqCst), 0);
    complete(&router, &next_route);
    drop(next_route);
    drop(retirement);
    drop(route);
    drop(router);
    drop(owner);
    drop(engine);
    drop(other_owner);
    drop(other);
    finish(&mut dispatcher);
}
