use super::*;

#[test]
fn pause_waiter_reasserts_admission_after_original_resume_and_final_close_never_reopens() {
    let slot = Arc::new(TransitionSlot::default());
    let original = slot.pause_route().unwrap();
    let (marked_tx, marked_rx) = std::sync::mpsc::channel();
    let (continue_tx, continue_rx) = std::sync::mpsc::channel();
    let waiter_slot = slot.clone();
    let waiter = std::thread::spawn(move || {
        waiter_slot
            .pause_route_after_mark(|| {
                marked_tx.send(()).unwrap();
                continue_rx.recv().unwrap();
            })
            .unwrap()
    });
    marked_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    slot.resume_route(&original, || Ok(())).unwrap();
    continue_tx.send(()).unwrap();
    let next = waiter.join().unwrap();
    assert!(slot.paused.load(Ordering::Acquire));
    assert_eq!(
        slot.begin(|| panic!("paused route must not submit"))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert_eq!(
        slot.resume_route(&original, || panic!("stale ticket cannot verify a route"))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let foreign = TransitionSlot::default().pause_route().unwrap();
    assert_eq!(
        slot.resume_route(&foreign, || panic!("foreign token cannot verify a route"))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    slot.close_admission();
    assert_eq!(
        slot.resume_route(&next, || panic!("final close cannot verify a route"))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert_eq!(
        slot.begin(|| panic!("final close is permanent"))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
}

#[test]
fn fresh_route_verification_failure_retains_original_pause_and_blocks_new_preparation() {
    let slot = TransitionSlot::default();
    let ticket = slot.pause_route().unwrap();
    assert_eq!(
        slot.resume_route(&ticket, || Err(ApiError::new(
            ErrorCode::EngineFailed,
            "original fresh route failure"
        )))
        .unwrap_err()
        .code,
        ErrorCode::EngineFailed
    );
    assert!(slot.paused.load(Ordering::Acquire));
    assert_eq!(
        slot.begin(|| panic!("failed fresh activation cannot submit"))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    slot.resume_route(&ticket, || Ok(())).unwrap();
    let error = slot
        .begin(|| {
            Err(ApiError::new(
                ErrorCode::NeedsInput,
                "shared previous/next preparation reached",
            ))
        })
        .unwrap_err();
    assert_eq!(error.message, "shared previous/next preparation reached");
    assert_eq!(
        slot.resume_route(&ticket, || Ok(())).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
}
