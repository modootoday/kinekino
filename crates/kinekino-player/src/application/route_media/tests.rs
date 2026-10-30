use super::*;

#[test]
fn incomplete_retirement_blocks_viewing_and_duplicate_begin_keeps_original_ticket() {
    let mut state = RouteMediaState::new(7);
    let ticket = state.begin_retire(&7).unwrap();
    let duplicate = state.begin_retire(&7).unwrap();
    assert!(Arc::ptr_eq(&ticket.identity, &duplicate.identity));
    assert_eq!(ticket.generation(), duplicate.generation());
    assert_eq!(
        state.poll_retire(&ticket).unwrap(),
        RouteMediaProgress::Pending
    );
    assert_eq!(
        state.begin_viewing(&7, &ticket).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        state.poll_retire(&ticket).unwrap(),
        RouteMediaProgress::Pending
    );
}
#[test]
fn same_owner_foreign_port_and_foreign_scope_tickets_never_authorize_work() {
    let mut original = RouteMediaState::new(7);
    let mut foreign = RouteMediaState::new(7);
    let ticket = original.begin_retire(&7).unwrap();
    let other = foreign.begin_retire(&7).unwrap();
    assert_eq!(
        original.begin_retire(&9).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        original.poll_retire(&other).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        foreign.complete_retire(&ticket).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        original.poll_retire(&ticket).unwrap(),
        RouteMediaProgress::Pending
    );
}
#[test]
fn viewing_roundtrips_use_fresh_generation_and_reject_all_old_tickets() {
    let mut state = RouteMediaState::new(7);
    let first = state.begin_retire(&7).unwrap();
    state.complete_retire(&first).unwrap();
    let viewing = state.begin_viewing(&7, &first).unwrap();
    let duplicate = state.begin_viewing(&7, &first).unwrap();
    assert_eq!(duplicate.generation(), viewing.generation());
    assert_eq!(
        state.poll_viewing(&viewing).unwrap(),
        RouteMediaProgress::Pending
    );
    assert!(state.poll_retire(&first).is_err());
    state.complete_viewing(&viewing).unwrap();
    assert_eq!(
        state.poll_viewing(&viewing).unwrap(),
        RouteMediaProgress::Complete
    );
    let second = state.begin_retire(&7).unwrap();
    assert!(second.generation() > first.generation());
    assert!(state.poll_viewing(&viewing).is_err());
    assert!(state.poll_retire(&first).is_err());
    state.complete_retire(&second).unwrap();
    let second_viewing = state.begin_viewing(&7, &second).unwrap();
    assert!(second_viewing.generation() > viewing.generation());
    assert!(state.complete_viewing(&viewing).is_err());
    state.complete_viewing(&second_viewing).unwrap();
}
#[test]
fn retirement_failure_and_cancellation_preserve_first_error_and_block_new_mount() {
    let mut state = RouteMediaState::new(7);
    let ticket = state.begin_retire(&7).unwrap();
    let first = Arc::new(ApiError::new(
        ErrorCode::EngineFailed,
        "original joined failure",
    ));
    state.fail_retire(&ticket, first.clone()).unwrap();
    state.cancel_retire(&ticket).unwrap();
    assert!(Arc::ptr_eq(
        &state.poll_retire(&ticket).unwrap_err(),
        &first
    ));
    assert!(Arc::ptr_eq(
        &state.begin_viewing(&7, &ticket).unwrap_err(),
        &first
    ));
    assert!(state.complete_retire(&ticket).is_err());
}
#[test]
fn cancel_never_rolls_back_cleanup_or_grants_fresh_viewing() {
    let mut state = RouteMediaState::new(7);
    let ticket = state.begin_retire(&7).unwrap();
    state.cancel_retire(&ticket).unwrap();
    assert_eq!(
        state.poll_retire(&ticket).unwrap_err().code,
        ErrorCode::Cancelled
    );
    assert!(state.begin_viewing(&7, &ticket).is_err());
    assert!(state.begin_retire(&7).is_err());
}
#[test]
fn fresh_registration_failure_preserves_owner_and_first_error() {
    let mut state = RouteMediaState::new(7);
    let retired = state.begin_retire(&7).unwrap();
    state.complete_retire(&retired).unwrap();
    let viewing = state.begin_viewing(&7, &retired).unwrap();
    let first = Arc::new(ApiError::new(
        ErrorCode::EngineFailed,
        "new registration failed",
    ));
    state.fail_viewing(&viewing, first.clone()).unwrap();
    assert!(Arc::ptr_eq(
        &state.poll_viewing(&viewing).unwrap_err(),
        &first
    ));
    assert!(state.complete_viewing(&viewing).is_err());
    assert!(state.begin_retire(&7).is_err());
}
