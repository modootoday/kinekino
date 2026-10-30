use super::*;
use crate::application::{
    jobs::UnavailableJobs,
    playback_session::transition::{EpisodeDirection, TransitionSlot},
};

#[test]
fn same_router_slot_rearms_only_fresh_active_viewing_and_rejects_old_issuer_and_final_close() {
    let mut dispatcher = host(8);
    let first = Fixture::new();
    let second = Fixture::new();
    let slot = Arc::new(TransitionSlot::default());
    let router = Arc::new(
        ParentEngineRouter::new_with_transitions(dispatcher.handle(), slot.clone())
            .with_transition_jobs(Arc::new(UnavailableJobs)),
    );
    let old = router.prepare(first.engine.clone()).unwrap();
    router.activate(&old).unwrap();
    let issuer = router.capture_active_native_export_issuer(&old).unwrap();
    let paused = router.pause_route_transitions().unwrap();
    router.revoke_route_admission(&old).unwrap();
    assert_eq!(
        slot.begin(|| panic!("Studio cannot begin an adjacent transition"))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert!(router.poll_route_transitions(&paused).unwrap());
    assert!(router.resume_route_transitions(&paused, &old).is_err());
    complete(&router, &old);
    let fresh = router.prepare(second.engine.clone()).unwrap();
    assert!(router.resume_route_transitions(&paused, &fresh).is_err());
    router.activate(&fresh).unwrap();
    router.resume_route_transitions(&paused, &fresh).unwrap();
    assert!(router.resume_route_transitions(&paused, &fresh).is_err());
    assert!(router.observe_native_guard_for_export(&issuer).is_err());
    for direction in [EpisodeDirection::Previous, EpisodeDirection::Next] {
        let marker = format!("original {direction:?} preparation admitted");
        let result = slot
            .begin(|| Err(ApiError::new(ErrorCode::NeedsInput, marker.clone())))
            .unwrap_err();
        assert_eq!(result.message, marker);
    }
    let next_pause = router.pause_route_transitions().unwrap();
    router.close_transition_admission();
    assert_eq!(
        router
            .resume_route_transitions(&next_pause, &fresh)
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert_eq!(
        slot.begin(|| panic!("final close must be permanent"))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    complete(&router, &fresh);
    drop(issuer);
    drop(old);
    drop(fresh);
    drop(router);
    drop(first);
    drop(second);
    finish(&mut dispatcher);
}
