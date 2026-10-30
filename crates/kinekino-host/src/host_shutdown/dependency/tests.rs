use super::*;
use kinekino_player::infrastructure::local_host::{DispatchState, WorkerJoin};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct Owner {
    remaining: usize,
    polls: Arc<AtomicUsize>,
    drops: Arc<AtomicUsize>,
    errors: ShutdownErrors,
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
impl PendingDependency for Owner {
    fn errors(&self) -> &ShutdownErrors {
        &self.errors
    }
    fn poll(mut self: Box<Self>, _: Instant) -> DependencyPoll {
        self.polls.fetch_add(1, Ordering::SeqCst);
        if self.remaining != 0 {
            self.remaining -= 1;
            DependencyPoll::Pending(self)
        } else {
            DependencyPoll::Complete(completion(
                json!({"original_owner":7}),
                std::mem::take(&mut self.errors),
            ))
        }
    }
}
fn owner(remaining: usize) -> (Box<Owner>, Arc<AtomicUsize>, Arc<AtomicUsize>) {
    let polls = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    (
        Box::new(Owner {
            remaining,
            polls: Arc::clone(&polls),
            drops: Arc::clone(&drops),
            errors: ShutdownErrors::default(),
        }),
        polls,
        drops,
    )
}
fn local_status() -> LocalHostShutdownStatus {
    LocalHostShutdownStatus {
        pipe: WorkerJoin::Joined,
        sdk: WorkerJoin::Joined,
        dispatch: Some(DispatchState::Idle),
    }
}

#[test]
fn dependency_timeout_keeps_original_owner_after_transport_is_already_joined() {
    let (owner, polls, drops) = owner(2);
    let cutoff = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();
    let outcome = observe_dependency(owner, None, Some(local_status()), cutoff);
    let DependencyObservation::Incomplete(owner) = outcome else {
        panic!("No completion without dependency closure")
    };
    assert_eq!(polls.load(Ordering::SeqCst), 1);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    let diagnostic = owner.diagnostic("fixture");
    assert_eq!(diagnostic["cleanup_complete"], false);
    assert_eq!(diagnostic["sdk_cancellation_confirmed"], false);
    assert_eq!(diagnostic["local_host_shutdown"]["pipe"], "joined");
    assert_eq!(diagnostic["budget_expired"], true);
    drop(owner); // Explicit fixture disposal, not a product cleanup certificate.
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn explicit_second_poll_finishes_same_dependency_without_new_owner() {
    let (owner, polls, drops) = owner(1);
    let pending = DependencyIncomplete {
        owner,
        first_error: None,
        local_status: Some(local_status()),
        deadline: dependency_deadline(),
    };
    let DependencyObservation::Incomplete(owner) = advance_dependency(pending) else {
        panic!("First poll is pending")
    };
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    let DependencyObservation::Complete(result) = advance_dependency(owner) else {
        panic!("Original second poll closes")
    };
    assert_eq!(result.receipt["original_owner"], 7);
    assert!(result.errors.first().is_none());
    assert_eq!(polls.load(Ordering::SeqCst), 2);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn transport_first_error_and_dependency_secondary_survive_actual_driver() {
    let (mut owner, _, drops) = owner(0);
    owner.errors.record(ApiError::new(
        ErrorCode::SourceExpired,
        "Original dependency failure",
    ));
    let mut first = ApiError::new(ErrorCode::PermissionDenied, "Original transport error");
    first.request_id = "original-request".into();
    let pending = DependencyIncomplete {
        owner,
        first_error: Some(first),
        local_status: Some(local_status()),
        deadline: dependency_deadline(),
    };
    let DependencyObservation::Complete(result) = advance_dependency(pending) else {
        panic!("Expected closed original owner")
    };
    assert_eq!(
        result.errors.first().unwrap().request_id,
        "original-request"
    );
    assert_eq!(
        result.errors.first().unwrap().code,
        ErrorCode::PermissionDenied
    );
    assert_eq!(
        result.errors.secondary()[0].message(),
        "Original dependency failure"
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
