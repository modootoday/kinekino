use super::*;
use crate::{
    application::{ExecutionControl, JobAttemptIdentityV1},
    domain::{JobRecord, PreparedJobInput, SourceBlobRef},
};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

struct Control {
    running: JobRecord,
    identity: JobAttemptIdentityV1,
    deadline: Instant,
    cancel: AtomicBool,
    cas_fail: AtomicBool,
    reservations: AtomicUsize,
    clock: Arc<Clock>,
    delay_cas: AtomicBool,
}

struct Clock(Mutex<Instant>);
impl AttemptClock for Clock {
    fn now(&self) -> Instant {
        *self.0.lock().unwrap()
    }
}
impl Clock {
    fn advance(&self, duration: Duration) {
        let mut now = self.0.lock().unwrap();
        *now = now.checked_add(duration).unwrap();
    }
}

impl Control {
    fn new() -> Self {
        let mut record = JobRecord::queued(
            "job-00000000000000000000000000000001"
                .to_owned()
                .try_into()
                .unwrap(),
            PreparedJobInput {
                operation: "test.owned".into(),
                parameters: serde_json::json!({}),
                source: SourceBlobRef {
                    sha256: "a".repeat(64),
                    bytes: DecimalU64(1),
                },
                execution_timeout_ms: DecimalU64(30000),
            },
        )
        .unwrap();
        record.start().unwrap();
        Self {
            identity: JobAttemptIdentityV1::from_running_record(&record),
            deadline: Instant::now() + Duration::from_secs(30),
            cancel: AtomicBool::new(false),
            cas_fail: AtomicBool::new(false),
            reservations: AtomicUsize::new(0),
            clock: Arc::new(Clock(Mutex::new(Instant::now()))),
            delay_cas: AtomicBool::new(false),
            running: record,
        }
    }
}

impl AttemptControl for Control {
    fn cancellation_requested(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
}

impl ExecutionControl for Control {
    fn execution_identity(&self) -> Option<&JobAttemptIdentityV1> {
        Some(&self.identity)
    }
    fn cancellation_requested(&self) -> bool {
        AttemptControl::cancellation_requested(self)
    }
    fn deadline(&self) -> Instant {
        self.deadline
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        self.reservations.fetch_add(1, Ordering::SeqCst);
        if self.cas_fail.load(Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original CAS failed",
            ));
        }
        if AttemptControl::cancellation_requested(self) {
            return Err(ApiError::new(ErrorCode::Cancelled, "Cancellation won"));
        }
        if self.delay_cas.load(Ordering::SeqCst) {
            self.clock.advance(Duration::from_secs(60));
        }
        Ok(())
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        panic!("Not engine authority")
    }
}

// Test-only mint. Production P1b must bind original start-CAS/control/session.
fn fixture(control: &Control) -> (AttemptScope<'_>, SettlementPublicationGate<'_>) {
    let binding = Arc::new(AttemptBinding {
        identity: control.identity.clone(),
        deadline: control.deadline,
        clock: control.clock.clone(),
    });
    (
        AttemptScope {
            binding: Arc::clone(&binding),
            control: control::ControlSource::Restricted(control),
        },
        SettlementPublicationGate { binding, control },
    )
}

struct Owner {
    polls: VecDeque<Result<OwnerObservation, ApiError>>,
    deadlines: Arc<Mutex<Vec<Instant>>>,
    drops: Arc<AtomicUsize>,
    panic: bool,
    panic_payload_drops: Option<Arc<AtomicUsize>>,
}
struct PanicPayload(Arc<AtomicUsize>);
impl Drop for PanicPayload {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
impl RetainedAttemptOwner for Owner {
    fn poll_close(&mut self, deadline: Instant) -> Result<OwnerObservation, ApiError> {
        self.deadlines.lock().unwrap().push(deadline);
        if let Some(drops) = self.panic_payload_drops.take() {
            std::panic::panic_any(PanicPayload(drops));
        }
        assert!(!self.panic, "Synthetic owner panic");
        self.polls
            .pop_front()
            .expect("Original owner polled too often")
    }
}
type OwnerFixture = (Box<Owner>, Arc<AtomicUsize>, Arc<Mutex<Vec<Instant>>>);
fn owner(polls: Vec<Result<OwnerObservation, ApiError>>) -> OwnerFixture {
    let drops = Arc::new(AtomicUsize::new(0));
    let deadlines = Arc::new(Mutex::new(Vec::new()));
    (
        Box::new(Owner {
            polls: polls.into(),
            deadlines: Arc::clone(&deadlines),
            drops: Arc::clone(&drops),
            panic: false,
            panic_payload_drops: None,
        }),
        drops,
        deadlines,
    )
}
fn exited(forced: bool, exit_code: i32) -> OwnerObservation {
    OwnerObservation::Exited {
        exit_code,
        family_empty: true,
        streams_eof: true,
        forced,
    }
}
fn retained<T>(outcome: OwnedAttemptOutcome<T>) -> RetainedAttempt<T> {
    match outcome {
        OwnedAttemptOutcome::Retained(value) => value,
        _ => panic!("Expected original owner retained"),
    }
}
fn settled<T>(outcome: OwnedAttemptOutcome<T>) -> SettledAttempt<T> {
    match outcome {
        OwnedAttemptOutcome::Settled(value) => value,
        _ => panic!("Expected observed settlement"),
    }
}

#[test]
fn pending_and_eof_unknown_keep_same_owner_and_shortened_deadline() {
    let control = Control::new();
    let (scope, _) = fixture(&control);
    let (owner, drops, deadlines) = owner(vec![
        Ok(OwnerObservation::Pending),
        Ok(OwnerObservation::Exited {
            exit_code: 0,
            family_empty: true,
            streams_eof: false,
            forced: false,
        }),
        Ok(exited(false, 0)),
    ]);
    let short = scope.deadline() - Duration::from_secs(1);
    let pending = retained(scope.retain(owner, Ok(17)).poll_close(short));
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    let pending = retained(pending.poll_close(control.deadline + Duration::from_secs(90)));
    assert_eq!(pending.identity(), &control.identity);
    assert_eq!(pending.close_deadline(), short);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    let result = settled(pending.poll_close(control.deadline));
    assert_eq!(result.kind(), SettlementKind::Graceful);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(*deadlines.lock().unwrap(), vec![short, short, short]);
}

#[test]
fn forced_and_nonzero_settlement_never_grant_success_publication() {
    for (forced, exit_code) in [(true, 0), (false, 1)] {
        let control = Control::new();
        let (scope, gate) = fixture(&control);
        let (owner, drops, _) = owner(vec![Ok(exited(forced, exit_code))]);
        let mut result = settled(scope.retain(owner, Ok(1)).poll_close(control.deadline));
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert_eq!(
            result.diagnostic().first().unwrap().code(),
            ErrorCode::EngineFailed
        );
        assert!(gate.reserve(&mut result).is_err());
        assert_eq!(control.reservations.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn original_failure_survives_cleanup_and_diagnostic_persistence_errors() {
    let control = Control::new();
    let (scope, _) = fixture(&control);
    let (owner, drops, _) = owner(vec![Err(ApiError::new(
        ErrorCode::Timeout,
        "Cleanup observation failed",
    ))]);
    let original = ApiError::new(ErrorCode::EngineFailed, "Original engine failure");
    let mut pending = retained(
        scope
            .retain::<()>(owner, Err(original))
            .poll_close(control.deadline),
    );
    for _ in 0..20 {
        pending.record_failure(&ApiError::new(
            ErrorCode::Internal,
            "Diagnostic write failed",
        ));
    }
    assert_eq!(
        pending.diagnostic().first().unwrap().message(),
        "Original engine failure"
    );
    assert_eq!(pending.diagnostic().secondary().len(), 8);
    assert!(pending.diagnostic().secondary_overflow());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
}

#[test]
fn successful_settlement_requires_original_cas_and_permit_is_single_use() {
    let control = Control::new();
    let (scope, gate) = fixture(&control);
    let (owner, _, _) = owner(vec![Ok(exited(false, 0))]);
    let mut result = settled(scope.retain(owner, Ok(41)).poll_close(control.deadline));
    let permit = gate.reserve(&mut result).unwrap();
    assert!(gate.reserve(&mut result).is_err());
    assert_eq!(control.reservations.load(Ordering::SeqCst), 1);
    assert_eq!(result.into_candidate(permit).ok(), Some(41));
}

#[test]
fn same_identity_different_core_binding_cannot_issue_permit() {
    let control = Control::new();
    let (scope, _) = fixture(&control);
    let (_, other_gate) = fixture(&control);
    let (owner, _, _) = owner(vec![Ok(exited(false, 0))]);
    let mut result = settled(scope.retain(owner, Ok(1)).poll_close(control.deadline));
    assert!(other_gate.reserve(&mut result).is_err());
    assert_eq!(control.reservations.load(Ordering::SeqCst), 0);
}

#[test]
fn cancel_and_cas_failure_keep_unpublished_candidate_without_permit() {
    for (cancel, expected) in [
        (true, ErrorCode::Cancelled),
        (false, ErrorCode::RevisionConflict),
    ] {
        let control = Control::new();
        let (scope, gate) = fixture(&control);
        let (owner, _, _) = owner(vec![Ok(exited(false, 0))]);
        let mut result = settled(scope.retain(owner, Ok(1)).poll_close(control.deadline));
        control.cancel.store(cancel, Ordering::SeqCst);
        control.cas_fail.store(!cancel, Ordering::SeqCst);
        assert_eq!(gate.reserve(&mut result).err().unwrap().code, expected);
        assert!(!result.permit_issued);
        assert!(result.candidate.is_some());
        assert_eq!(result.diagnostic().first().unwrap().code(), expected);
        assert!(gate.reserve(&mut result).is_err());
        assert_eq!(control.reservations.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn prelaunch_failure_cannot_grant_permit_or_become_cancel_ack() {
    let control = Control::new();
    let (scope, gate) = fixture(&control);
    let mut result = scope.not_started::<()>(ApiError::new(ErrorCode::Cancelled, "Launch skipped"));
    assert_eq!(result.kind(), SettlementKind::NotStarted);
    assert!(gate.reserve(&mut result).is_err());
    assert_eq!(control.reservations.load(Ordering::SeqCst), 0);
}

#[test]
fn diagnostic_utf8_limits_and_first_error_are_stable() {
    let error = ApiError::new(ErrorCode::EngineFailed, "가".repeat(3000));
    let failure = AttemptFailure::capture(&error);
    assert!(failure.message().len() <= 4096);
    assert_eq!(failure.message(), "가".repeat(1365));
    assert!(failure.remediation().is_empty());
}

#[test]
fn panic_retains_original_owner_and_poison_is_sticky() {
    let control = Control::new();
    let (scope, _) = fixture(&control);
    let (mut owner, drops, deadlines) = owner(vec![]);
    owner.panic = true;
    let pending = retained(scope.retain(owner, Ok(1)).poll_close(control.deadline));
    assert!(pending.poll_poisoned());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    let pending = retained(pending.poll_close(control.deadline));
    assert_eq!(deadlines.lock().unwrap().len(), 1);
    assert_eq!(
        pending.diagnostic().first().unwrap().code(),
        ErrorCode::Internal
    );
    assert_eq!(drops.load(Ordering::SeqCst), 0);
}

#[test]
fn candidate_carriers_are_send_without_unsafe_implementations() {
    fn assert_send<T: Send>() {}
    assert_send::<RetainedAttempt<u64>>();
    assert_send::<SettledAttempt<u64>>();
    assert_send::<PublicationPermit>();
}

#[test]
fn incomplete_forced_observation_cannot_later_become_graceful() {
    let control = Control::new();
    let (scope, gate) = fixture(&control);
    let (owner, drops, _) = owner(vec![
        Ok(OwnerObservation::Exited {
            exit_code: 0,
            family_empty: false,
            streams_eof: false,
            forced: true,
        }),
        Ok(exited(false, 0)),
    ]);
    let pending = retained(scope.retain(owner, Ok(1)).poll_close(control.deadline));
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    let mut result = settled(pending.poll_close(control.deadline));
    assert_eq!(result.kind(), SettlementKind::ForcedObserved);
    assert!(gate.reserve(&mut result).is_err());
}

#[test]
fn contradictory_original_exit_observation_blocks_publication() {
    let control = Control::new();
    let (scope, gate) = fixture(&control);
    let (owner, _, _) = owner(vec![
        Ok(OwnerObservation::Exited {
            exit_code: 7,
            family_empty: false,
            streams_eof: true,
            forced: false,
        }),
        Ok(exited(false, 0)),
    ]);
    let pending = retained(scope.retain(owner, Ok(1)).poll_close(control.deadline));
    let mut result = settled(pending.poll_close(control.deadline));
    assert_eq!(
        result.diagnostic().first().unwrap().code(),
        ErrorCode::Internal
    );
    assert!(gate.reserve(&mut result).is_err());
}

#[test]
fn expired_original_deadline_blocks_cas_even_after_clean_settlement() {
    let mut control = Control::new();
    control.deadline = Instant::now() - Duration::from_secs(1);
    let (scope, gate) = fixture(&control);
    let (owner, _, _) = owner(vec![Ok(exited(false, 0))]);
    let mut result = settled(scope.retain(owner, Ok(1)).poll_close(control.deadline));
    assert_eq!(
        gate.reserve(&mut result).err().unwrap().code,
        ErrorCode::Timeout
    );
    assert_eq!(control.reservations.load(Ordering::SeqCst), 0);
}

#[test]
fn permit_from_other_binding_does_not_release_candidate() {
    let control = Control::new();
    let (scope, gate) = fixture(&control);
    let (other_scope, other_gate) = fixture(&control);
    let (original_owner, _, _) = owner(vec![Ok(exited(false, 0))]);
    let (other_owner, _, _) = owner(vec![Ok(exited(false, 0))]);
    let mut original = settled(
        scope
            .retain(original_owner, Ok(1))
            .poll_close(control.deadline),
    );
    let mut other = settled(
        other_scope
            .retain(other_owner, Ok(2))
            .poll_close(control.deadline),
    );
    let original_permit = gate.reserve(&mut original).unwrap();
    let other_permit = other_gate.reserve(&mut other).unwrap();
    let rejected = original.into_candidate(other_permit).err().unwrap();
    let (original, other_permit) = rejected.into_parts();
    assert_eq!(original.into_candidate(original_permit).ok(), Some(1));
    assert_eq!(other.into_candidate(other_permit).ok(), Some(2));
}

#[test]
fn post_cas_deadline_keeps_reservation_and_candidate_without_repeating_cas() {
    let control = Control::new();
    let (scope, gate) = fixture(&control);
    let (owner, _, _) = owner(vec![Ok(exited(false, 0))]);
    let mut result = settled(scope.retain(owner, Ok(3)).poll_close(control.deadline));
    control.delay_cas.store(true, Ordering::SeqCst);
    assert_eq!(
        gate.reserve(&mut result).err().unwrap().code,
        ErrorCode::Timeout
    );
    assert!(result.publication_reserved());
    assert!(!result.permit_issued);
    assert!(result.candidate.is_some());
    assert_eq!(
        result.diagnostic().first().unwrap().code(),
        ErrorCode::Timeout
    );
    assert!(gate.reserve(&mut result).is_err());
    assert_eq!(control.reservations.load(Ordering::SeqCst), 1);
}

#[test]
fn expired_permit_keeps_candidate_permit_and_original_timeout() {
    let control = Control::new();
    let (scope, gate) = fixture(&control);
    let (owner, _, _) = owner(vec![Ok(exited(false, 0))]);
    let mut result = settled(scope.retain(owner, Ok(4)).poll_close(control.deadline));
    let permit = gate.reserve(&mut result).unwrap();
    control.clock.advance(Duration::from_secs(60));
    let rejected = result.into_candidate(permit).err().unwrap();
    let (mut result, permit) = rejected.into_parts();
    assert!(result.publication_reserved());
    assert!(result.candidate.is_some());
    assert_eq!(
        result.diagnostic().first().unwrap().code(),
        ErrorCode::Timeout
    );
    assert!(gate.reserve(&mut result).is_err());
    assert!(result.into_candidate(permit).is_err());
    assert_eq!(control.reservations.load(Ordering::SeqCst), 1);
}

#[test]
fn panic_payload_and_original_owner_are_retained_together() {
    let control = Control::new();
    let (scope, _) = fixture(&control);
    let (mut owner, drops, _) = owner(vec![]);
    let payload_drops = Arc::new(AtomicUsize::new(0));
    owner.panic_payload_drops = Some(Arc::clone(&payload_drops));
    let pending = retained(scope.retain(owner, Ok(5)).poll_close(control.deadline));
    assert!(pending.poll_poisoned());
    assert!(pending.panic_payload_retained());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(payload_drops.load(Ordering::SeqCst), 0);
    let pending = retained(pending.poll_close(control.deadline));
    assert_eq!(payload_drops.load(Ordering::SeqCst), 0);
    drop(pending);
    // Explicit fixture disposal only; this is not a product cleanup receipt.
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(payload_drops.load(Ordering::SeqCst), 1);
}

#[test]
fn core_binding_rejects_non_running_record_and_changed_original_deadline() {
    let mut control = Control::new();
    let binding = CoreAttemptBinding::from_running(&control.running, &control).unwrap();
    let scope = binding.scope(&control).unwrap();
    assert_eq!(scope.identity(), &control.identity);
    assert_eq!(scope.deadline(), control.deadline);
    assert!(scope.observe_cooperative_cancellation().is_none());
    let mut queued = control.running.clone();
    queued.state = crate::domain::JobState::Queued;
    assert!(CoreAttemptBinding::from_running(&queued, &control).is_err());
    control.deadline += Duration::from_secs(1);
    assert!(binding.scope(&control).is_err());
    assert!(binding.gate(&control).is_err());
}

#[test]
fn cooperative_marker_remains_bound_to_one_core_admission() {
    let control = Control::new();
    control.cancel.store(true, Ordering::SeqCst);
    let binding = CoreAttemptBinding::from_running(&control.running, &control).unwrap();
    let other = CoreAttemptBinding::from_running(&control.running, &control).unwrap();
    let marker = binding
        .scope(&control)
        .unwrap()
        .observe_cooperative_cancellation()
        .unwrap();
    assert!(binding.cancellation_matches(&marker));
    assert!(!other.cancellation_matches(&marker));
}

struct Session {
    drops: Arc<AtomicUsize>,
    panic_run: bool,
    fail_cancelled: bool,
}
impl Drop for Session {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
impl OwnedAttemptSession for Session {
    type Candidate = u64;
    fn run(
        &mut self,
        _: &PreparedJobInput,
        _: &dyn crate::application::SourceBlobStore,
        _: &AttemptScope<'_>,
    ) -> SessionRun<u64> {
        assert!(!self.panic_run, "Synthetic original session run panic");
        if self.fail_cancelled {
            let mut error = ApiError::new(
                ErrorCode::Cancelled,
                "Original engine error, not cooperative marker",
            );
            error.request_id = "original-diagnostic-request".into();
            SessionRun::Failed(error)
        } else {
            SessionRun::Candidate(9)
        }
    }
    fn poll_close(&mut self, _: Instant) -> Result<OwnerObservation, ApiError> {
        Ok(exited(false, 0))
    }
}
struct Blobs;
impl crate::application::SourceBlobStore for Blobs {
    fn put_utf8(&self, _: &str) -> Result<SourceBlobRef, ApiError> {
        unreachable!()
    }
    fn read_utf8(&self, _: &SourceBlobRef) -> Result<String, ApiError> {
        unreachable!()
    }
}

#[test]
fn session_run_unwind_keeps_original_parent_owned_box_until_observed_close() {
    let control = Control::new();
    let binding = CoreAttemptBinding::from_running(&control.running, &control).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let execution = run_prepared(
        Box::new(Session {
            drops: Arc::clone(&drops),
            panic_run: true,
            fail_cancelled: false,
        }),
        &control.running.input,
        &Blobs,
        binding.scope(&control).unwrap(),
    );
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(
        execution.first_error.as_ref().unwrap().code,
        ErrorCode::Internal
    );
    assert!(execution.panic_payload.is_some());
    let mut result = settled(execution.retained.poll_close(control.deadline));
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(
        binding
            .gate(&control)
            .unwrap()
            .reserve(&mut result)
            .is_err()
    );
}

#[test]
fn cancelled_error_remains_failure_in_parent_owned_session_bridge() {
    let control = Control::new();
    control.cancel.store(true, Ordering::SeqCst);
    let binding = CoreAttemptBinding::from_running(&control.running, &control).unwrap();
    let execution = run_prepared(
        Box::new(Session {
            drops: Arc::new(AtomicUsize::new(0)),
            panic_run: false,
            fail_cancelled: true,
        }),
        &control.running.input,
        &Blobs,
        binding.scope(&control).unwrap(),
    );
    assert_eq!(
        execution.first_error.as_ref().unwrap().request_id,
        "original-diagnostic-request"
    );
    let mut result = settled(execution.retained.poll_close(control.deadline));
    assert!(result.candidate_ref().is_none());
    assert_eq!(
        result.diagnostic().first().unwrap().code(),
        ErrorCode::Cancelled
    );
    assert!(
        binding
            .gate(&control)
            .unwrap()
            .reserve(&mut result)
            .is_err()
    );
}

#[test]
fn parent_no_child_settlement_preserves_absent_exit_and_forbids_publication() {
    let control = Control::new();
    let (scope, gate) = fixture(&control);
    let (owner, drops, _) = owner(vec![Ok(OwnerObservation::NotStarted)]);
    let mut result = settled(scope.retain(owner, Ok(1)).poll_close(control.deadline));
    assert_eq!(result.kind(), SettlementKind::NotStarted);
    assert_eq!(result.observed_exit(), None);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(gate.reserve(&mut result).is_err());
    assert_eq!(control.reservations.load(Ordering::SeqCst), 0);
}

#[test]
fn contradictory_no_child_report_retains_original_owner_until_actual_closure() {
    let control = Control::new();
    let (scope, gate) = fixture(&control);
    let (owner, drops, _) = owner(vec![
        Ok(OwnerObservation::Exited {
            exit_code: 0,
            family_empty: false,
            streams_eof: false,
            forced: false,
        }),
        Ok(OwnerObservation::NotStarted),
        Ok(exited(false, 0)),
    ]);
    let owner = retained(scope.retain(owner, Ok(1)).poll_close(control.deadline));
    let owner = retained(owner.poll_close(control.deadline));
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    let mut result = settled(owner.poll_close(control.deadline));
    assert_eq!(
        result.diagnostic().first().unwrap().code(),
        ErrorCode::Internal
    );
    assert!(gate.reserve(&mut result).is_err());
}
