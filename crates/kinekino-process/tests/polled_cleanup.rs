use kinekino_process::{
    application::supervisor::{Clock, ProcessFamily, ReceiptSink, Supervisor},
    domain::{deadline::Deadline, identity::ProcessIdentity, observation::*},
    infrastructure::evidence_files::PreparedEvidence,
};
use std::{
    cell::Cell,
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    path: PathBuf,
    prepared: Option<PreparedEvidence>,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "owned-polled-cleanup-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&path).unwrap();
        let prepared = PreparedEvidence::create(&path, Some(b"request"), 32).unwrap();
        Self {
            path,
            prepared: Some(prepared),
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        drop(self.prepared.take());
        let _ = fs::remove_dir_all(&self.path);
    }
}
struct FakeClock {
    start: Instant,
    elapsed: Cell<Duration>,
}
impl Clock for FakeClock {
    fn now(&self) -> Instant {
        self.start + self.elapsed.get()
    }
    fn pause(&self, _: Duration) {
        panic!("Polled cleanup must not pause")
    }
}
#[derive(Clone, Copy)]
struct Facts {
    exit: Option<u32>,
    family: FamilyObservation,
    query_error: bool,
    kill_error: bool,
}
struct Family {
    deadline: Deadline,
    identity: ProcessIdentity,
    facts: Arc<Mutex<Facts>>,
    eof: Arc<AtomicUsize>,
    kills: Arc<AtomicUsize>,
    drops: Arc<AtomicUsize>,
}
impl Drop for Family {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
impl ProcessFamily for Family {
    fn deadline(&self) -> Deadline {
        self.deadline
    }
    fn identity(&self) -> &ProcessIdentity {
        &self.identity
    }
    fn close_stdin(&mut self) -> io::Result<()> {
        self.eof.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn poll_child(&mut self) -> io::Result<Option<u32>> {
        Ok(self.facts.lock().unwrap().exit)
    }
    fn poll_family(&mut self) -> io::Result<FamilyObservation> {
        let facts = self.facts.lock().unwrap();
        if facts.query_error {
            Err(io::Error::other("Original family query failed"))
        } else {
            Ok(facts.family)
        }
    }
    fn terminate_owned_family(&mut self) -> io::Result<()> {
        self.kills.fetch_add(1, Ordering::SeqCst);
        let mut facts = self.facts.lock().unwrap();
        if facts.kill_error {
            return Err(io::Error::other("Original termination failed"));
        }
        facts.exit.get_or_insert(124);
        facts.family = FamilyObservation::Empty;
        Ok(())
    }
}
fn family(clock: &FakeClock) -> Family {
    Family {
        deadline: Deadline::new(
            clock.start,
            Duration::from_millis(100),
            Duration::from_millis(30),
        )
        .unwrap(),
        identity: ProcessIdentity {
            pid: 7,
            creation_filetime: Some(88),
            executable: PathBuf::from("fixture-original"),
        },
        facts: Arc::new(Mutex::new(Facts {
            exit: None,
            family: FamilyObservation::Active(1),
            query_error: false,
            kill_error: false,
        })),
        eof: Arc::new(AtomicUsize::new(0)),
        kills: Arc::new(AtomicUsize::new(0)),
        drops: Arc::new(AtomicUsize::new(0)),
    }
}
fn clock() -> FakeClock {
    FakeClock {
        start: Instant::now(),
        elapsed: Cell::new(Duration::ZERO),
    }
}
#[derive(Default)]
struct Sink {
    fail: bool,
    receipts: Vec<ProcessReceipt>,
}
impl ReceiptSink for Sink {
    fn record(&mut self, receipt: &ProcessReceipt) -> io::Result<()> {
        if self.fail {
            return Err(io::Error::other("Original receipt write failed"));
        }
        self.receipts.push(receipt.clone());
        Ok(())
    }
}

#[test]
fn pending_poll_keeps_original_owner_and_requests_eof_once_without_wait() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let clock = clock();
    let family = family(&clock);
    let deadline = family.deadline;
    let drops = Arc::clone(&family.drops);
    let eof = Arc::clone(&family.eof);
    let mut owner = Supervisor::new(family, fixture.prepared.as_ref().unwrap().paths.clone(), 64);
    owner.begin_cleanup(deadline);
    for _ in 0..3 {
        let progress = owner.poll_cleanup(deadline, &clock);
        assert!(!progress.physical_closed());
        assert!(!progress.settled());
        assert_eq!(progress.receipt().identity.creation_filetime, Some(88));
    }
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(eof.load(Ordering::SeqCst), 1);
}

#[test]
fn renewed_request_never_renews_original_cutoff_and_force_needs_a_second_observation() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let clock = clock();
    let family = family(&clock);
    let kills = Arc::clone(&family.kills);
    let mut owner = Supervisor::new(family, fixture.prepared.as_ref().unwrap().paths.clone(), 64);
    let renewed = Deadline::new(
        clock.start + Duration::from_secs(1),
        Duration::from_secs(1),
        Duration::from_millis(100),
    )
    .unwrap();
    clock.elapsed.set(Duration::from_millis(75));
    let progress = owner.poll_cleanup(renewed, &clock);
    assert!(progress.operation_remaining(clock.now()).is_zero());
    assert_eq!(progress.remaining(clock.now()), Duration::from_millis(25));
    assert!(!progress.physical_closed());
    assert!(progress.receipt().forced);
    assert_eq!(kills.load(Ordering::SeqCst), 1);
    let progress = owner.poll_cleanup(renewed, &clock);
    assert!(progress.physical_closed());
    assert!(!progress.settled());
    let mut sink = Sink::default();
    owner.record_cleanup(&mut sink, &clock).unwrap();
    assert!(owner.cleanup_progress().unwrap().settled());
    assert!(sink.receipts[0].forced);
}

#[test]
fn failed_family_and_termination_queries_preserve_original_errors_and_owner() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let clock = clock();
    let family = family(&clock);
    let facts = Arc::clone(&family.facts);
    let drops = Arc::clone(&family.drops);
    let kills = Arc::clone(&family.kills);
    let deadline = family.deadline;
    {
        let mut facts = facts.lock().unwrap();
        facts.query_error = true;
        facts.kill_error = true;
    }
    let mut owner = Supervisor::new(family, fixture.prepared.as_ref().unwrap().paths.clone(), 64);
    let progress = owner.poll_cleanup(deadline, &clock);
    assert!(matches!(
        progress.first_error().unwrap().stage,
        FailureStage::FamilyObservation
    ));
    assert!(matches!(
        progress.secondary_errors()[0].stage,
        FailureStage::Termination
    ));
    assert_eq!(progress.receipt().family, FamilyObservation::Unknown);
    assert!(!progress.physical_closed());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    {
        let mut facts = facts.lock().unwrap();
        facts.query_error = false;
        facts.kill_error = false;
        facts.exit = Some(124);
        facts.family = FamilyObservation::Empty;
    }
    let progress = owner.poll_cleanup(deadline, &clock);
    assert!(progress.physical_closed());
    assert_eq!(kills.load(Ordering::SeqCst), 1);
    assert_eq!(
        progress.first_error().unwrap().error.to_string(),
        "Original family query failed"
    );
}

#[test]
fn partial_receipt_never_becomes_final_receipt_when_family_query_is_unknown() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let clock = clock();
    let family = family(&clock);
    let facts = Arc::clone(&family.facts);
    let deadline = family.deadline;
    {
        let mut facts = facts.lock().unwrap();
        facts.exit = Some(0);
        facts.family = FamilyObservation::Empty;
        facts.query_error = true;
    }
    let mut owner = Supervisor::new(family, fixture.prepared.as_ref().unwrap().paths.clone(), 64);
    owner.poll_cleanup(deadline, &clock);
    let mut sink = Sink::default();
    owner.record_cleanup(&mut sink, &clock).unwrap();
    assert!(owner.receipt_recorded());
    assert!(!owner.cleanup_progress().unwrap().final_receipt_recorded());
    assert!(!owner.cleanup_progress().unwrap().settled());
    facts.lock().unwrap().query_error = false;
    owner.poll_cleanup(deadline, &clock);
    owner.record_cleanup(&mut sink, &clock).unwrap();
    assert!(owner.cleanup_progress().unwrap().settled());
    assert_eq!(sink.receipts.len(), 2);
}

#[test]
fn final_receipt_failure_keeps_closed_owner_and_first_error_until_explicit_retry() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let clock = clock();
    let family = family(&clock);
    let drops = Arc::clone(&family.drops);
    let deadline = family.deadline;
    {
        let mut facts = family.facts.lock().unwrap();
        facts.exit = Some(0);
        facts.family = FamilyObservation::Empty;
    }
    let mut owner = Supervisor::new(family, fixture.prepared.as_ref().unwrap().paths.clone(), 64);
    owner.poll_cleanup(deadline, &clock);
    let mut sink = Sink {
        fail: true,
        ..Sink::default()
    };
    assert!(owner.record_cleanup(&mut sink, &clock).is_err());
    assert!(!owner.cleanup_progress().unwrap().settled());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    sink.fail = false;
    owner.record_cleanup(&mut sink, &clock).unwrap();
    let progress = owner.cleanup_progress().unwrap();
    assert!(progress.settled());
    assert!(matches!(
        progress.first_error().unwrap().stage,
        FailureStage::Receipt
    ));
}

#[test]
fn raw_overflow_causes_force_but_does_not_fake_process_closure() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let clock = clock();
    let family = family(&clock);
    let deadline = family.deadline;
    fs::OpenOptions::new()
        .append(true)
        .open(fixture.prepared.as_ref().unwrap().paths.stdout.path())
        .unwrap()
        .write_all(&[0; 65])
        .unwrap();
    let mut owner = Supervisor::new(family, fixture.prepared.as_ref().unwrap().paths.clone(), 64);
    let progress = owner.poll_cleanup(deadline, &clock);
    assert!(matches!(
        progress.first_error().unwrap().stage,
        FailureStage::RawOutput
    ));
    assert!(progress.receipt().forced);
    assert!(!progress.physical_closed());
    assert!(owner.poll_cleanup(deadline, &clock).physical_closed());
}

#[test]
fn contradictory_child_exit_never_releases_original_owner() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let clock = clock();
    let family = family(&clock);
    let deadline = family.deadline;
    let facts = Arc::clone(&family.facts);
    let drops = Arc::clone(&family.drops);
    facts.lock().unwrap().exit = Some(0);
    let mut owner = Supervisor::new(family, fixture.prepared.as_ref().unwrap().paths.clone(), 64);
    owner.poll_cleanup(deadline, &clock);
    {
        let mut facts = facts.lock().unwrap();
        facts.exit = Some(1);
        facts.family = FamilyObservation::Empty;
    }
    let progress = owner.poll_cleanup(deadline, &clock);
    assert!(!progress.physical_closed());
    assert_eq!(progress.receipt().child_exit, Some(0));
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert!(matches!(
        progress.first_error().unwrap().stage,
        FailureStage::ChildObservation
    ));
}

#[test]
fn late_closed_observation_is_closed_with_timeout_not_fresh_budget_success() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let clock = clock();
    let family = family(&clock);
    let deadline = family.deadline;
    {
        let mut facts = family.facts.lock().unwrap();
        facts.exit = Some(0);
        facts.family = FamilyObservation::Empty;
    }
    let mut owner = Supervisor::new(family, fixture.prepared.as_ref().unwrap().paths.clone(), 64);
    clock.elapsed.set(Duration::from_millis(110));
    let progress = owner.poll_cleanup(deadline, &clock);
    assert!(progress.physical_closed());
    assert!(progress.deadline_expired());
    assert!(!progress.receipt().forced);
    assert!(matches!(
        progress.first_error().unwrap().stage,
        FailureStage::Deadline
    ));
    owner.record_cleanup(&mut Sink::default(), &clock).unwrap();
    assert!(owner.cleanup_progress().unwrap().settled());
}

#[cfg(windows)]
#[test]
fn actual_windows_family_supervisor_is_send_without_unsafe_trait_implementations() {
    kinekino_platform::install();
    fn assert_send<T: Send>() {}
    assert_send::<Supervisor<kinekino_process::infrastructure::family::OwnedProcessFamily>>();
}
