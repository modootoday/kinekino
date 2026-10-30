use super::*;
use crate::application::api::ErrorCode;
#[test]
fn revoke_before_commit_never_obtains_a_writer_permit() {
    let fence = Fence::new();
    let generation = fence.generation();
    fence.close();
    assert!(matches!(fence.commit(generation),Err(error) if error.code==ErrorCode::Cancelled));
    assert_eq!(fence.in_flight(), 0);
}
#[test]
fn old_permit_drop_preserves_new_generation_and_drain() {
    let fence = Fence::new();
    let old = fence.generation();
    let permit = fence.commit(old).unwrap();
    let new = fence.advance().unwrap();
    assert_ne!(old, new);
    assert_eq!(fence.in_flight(), 1);
    assert!(matches!(fence.commit(old),Err(error) if error.code==ErrorCode::RevisionConflict));
    let second = fence.commit(new).unwrap();
    fence.close();
    drop(permit);
    assert_eq!(fence.in_flight(), 1);
    drop(second);
    fence.close_and_drain().unwrap();
    assert_eq!(fence.generation(), new);
}
#[test]
fn committed_permit_keeps_cleanup_waiting_until_original_drop() {
    let fence = Fence::new();
    let permit = fence.commit(fence.generation()).unwrap();
    fence.close();
    let (sender, receiver) = std::sync::mpsc::channel();
    let other = fence.clone();
    let cleanup = std::thread::spawn(move || {
        other.close_and_drain().unwrap();
        sender.send(()).unwrap();
    });
    assert!(receiver.try_recv().is_err());
    drop(permit);
    receiver
        .recv_timeout(std::time::Duration::from_secs(2))
        .unwrap();
    cleanup.join().unwrap();
}
#[test]
fn commit_revoke_race_has_only_original_committed_or_rejected_outcomes() {
    for _ in 0..64 {
        let fence = Fence::new();
        let generation = fence.generation();
        let rendezvous = std::sync::Arc::new(std::sync::Barrier::new(2));
        let other = fence.clone();
        let gate = rendezvous.clone();
        let writer = std::thread::spawn(move || {
            gate.wait();
            other.commit(generation)
        });
        rendezvous.wait();
        fence.close();
        match writer.join().unwrap() {
            Ok(permit) => {
                assert_eq!(fence.in_flight(), 1);
                drop(permit);
            }
            Err(error) => assert_eq!(error.code, ErrorCode::Cancelled),
        }
        fence.close_and_drain().unwrap();
        assert_eq!(fence.in_flight(), 0);
    }
}
