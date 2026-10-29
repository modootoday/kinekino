use kinekino_process::{
    application::{
        raw_records::{RecordCursor, RecordPoll},
        supervisor::*,
    },
    domain::{deadline::Deadline, identity::ProcessIdentity, observation::*},
    infrastructure::evidence_files::*,
};
use std::{
    cell::Cell,
    fs,
    io::{self, Write},
    path::PathBuf,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
fn evidence() -> (PathBuf, PreparedEvidence) {
    let path = std::env::temp_dir().join(format!(
        "owned-process-pure-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let evidence = PreparedEvidence::create(&path, Some(b"request"), 64).unwrap();
    (path, evidence)
}
struct FakeClock {
    start: Instant,
    elapsed: Cell<Duration>,
}
impl Clock for FakeClock {
    fn now(&self) -> Instant {
        self.start + self.elapsed.get()
    }
    fn pause(&self, duration: Duration) {
        self.elapsed.set(self.elapsed.get() + duration);
    }
}
struct Fake {
    deadline: Deadline,
    identity: ProcessIdentity,
    exit: Option<u32>,
    active: bool,
    query_error: bool,
    kill_error: bool,
    killed: Rc<Cell<u32>>,
    dropped: Rc<Cell<u32>>,
}
impl Drop for Fake {
    fn drop(&mut self) {
        self.dropped.set(self.dropped.get() + 1);
    }
}
impl ProcessFamily for Fake {
    fn deadline(&self) -> Deadline {
        self.deadline
    }
    fn identity(&self) -> &ProcessIdentity {
        &self.identity
    }
    fn close_stdin(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn poll_child(&mut self) -> io::Result<Option<u32>> {
        Ok(self.exit)
    }
    fn poll_family(&mut self) -> io::Result<FamilyObservation> {
        if self.query_error {
            Err(io::Error::other("query original"))
        } else {
            Ok(if self.active {
                FamilyObservation::Active(1)
            } else {
                FamilyObservation::Empty
            })
        }
    }
    fn terminate_owned_family(&mut self) -> io::Result<()> {
        self.killed.set(self.killed.get() + 1);
        if self.kill_error {
            Err(io::Error::other("kill secondary"))
        } else {
            self.active = false;
            self.exit.get_or_insert(124);
            Ok(())
        }
    }
}
fn fake() -> Fake {
    Fake {
        deadline: Deadline::new(
            Instant::now(),
            Duration::from_secs(1),
            Duration::from_millis(100),
        )
        .unwrap(),
        identity: ProcessIdentity {
            pid: 1,
            creation_filetime: Some(2),
            executable: PathBuf::from("fixture"),
        },
        exit: None,
        active: true,
        query_error: false,
        kill_error: false,
        killed: Rc::new(Cell::new(0)),
        dropped: Rc::new(Cell::new(0)),
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
            Err(io::Error::other("receipt secondary"))
        } else {
            self.receipts.push(receipt.clone());
            Ok(())
        }
    }
}

#[test]
fn deadline_does_not_reset_and_request_budget_reserves_cleanup() {
    kinekino_platform::install();
    let start = Instant::now();
    let deadline =
        Deadline::new(start, Duration::from_millis(100), Duration::from_millis(30)).unwrap();
    assert_eq!(
        deadline.request_budget(start + Duration::from_millis(60), Duration::from_secs(1)),
        Duration::from_millis(10)
    );
    assert!(
        deadline
            .operation_remaining(start + Duration::from_millis(70))
            .is_zero()
    );
    assert_eq!(
        deadline.remaining(start + Duration::from_millis(90)),
        Duration::from_millis(10)
    );
    assert!(Deadline::new(start, Duration::ZERO, Duration::ZERO).is_err());
    assert!(Deadline::new(start, Duration::from_secs(1), Duration::from_secs(1)).is_err());
    assert!(Deadline::new(start, Duration::MAX, Duration::from_secs(1)).is_err());
}
#[test]
fn parent_exit_is_not_family_empty_and_forced_is_not_graceful() {
    kinekino_platform::install();
    let (root, prepared) = evidence();
    let mut process = fake();
    process.exit = Some(0);
    let killed = process.killed.clone();
    let clock = FakeClock {
        start: Instant::now(),
        elapsed: Cell::new(Duration::ZERO),
    };
    let deadline = Deadline::new(
        clock.start,
        Duration::from_millis(100),
        Duration::from_millis(30),
    )
    .unwrap();
    match Supervisor::new(process, prepared.paths.clone(), 1024).finish(
        deadline,
        &clock,
        &mut Sink::default(),
    ) {
        FinishOutcome::Complete(done) => {
            assert!(done.receipt().forced);
            assert_eq!(done.receipt().family, FamilyObservation::Empty);
            assert_eq!(done.receipt().child_exit, Some(0));
        }
        _ => panic!("expected observed forced cleanup"),
    }
    assert_eq!(killed.get(), 1);
    assert_eq!(clock.elapsed.get(), Duration::from_millis(80));
    drop(prepared);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn original_query_error_and_secondary_errors_keep_same_owner() {
    kinekino_platform::install();
    let (root, prepared) = evidence();
    let mut process = fake();
    process.query_error = true;
    process.kill_error = true;
    let dropped = process.dropped.clone();
    let identity = process.identity.clone();
    let now = Instant::now();
    let outcome = Supervisor::new(process, prepared.paths.clone(), 1024).finish(
        Deadline::new(now, Duration::from_secs(1), Duration::from_millis(100)).unwrap(),
        &SystemClock,
        &mut Sink {
            fail: true,
            receipts: vec![],
        },
    );
    match outcome {
        FinishOutcome::Incomplete(failure) => {
            assert_eq!(failure.owner.process().identity(), &identity);
            assert!(matches!(
                failure.cause.stage,
                FailureStage::FamilyObservation
            ));
            assert_eq!(failure.cause.error.to_string(), "query original");
            assert!(failure.termination_error.is_some());
            assert!(failure.receipt_error.is_some());
            assert_eq!(dropped.get(), 0);
            drop(failure);
        }
        _ => panic!("unknown query cannot complete"),
    }
    assert_eq!(dropped.get(), 1);
    drop(prepared);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn raw_records_have_independent_offsets_bounded_prefix_and_file_identity() {
    kinekino_platform::install();
    let (root, prepared) = evidence();
    let raw = prepared.paths.stdout.clone();
    let mut writer = fs::OpenOptions::new()
        .append(true)
        .open(raw.path())
        .unwrap();
    writer.write_all(b"ready\nterminal").unwrap();
    let mut first = RecordCursor::default();
    let mut second = RecordCursor::default();
    assert_eq!(
        first.poll(&raw, 6, false).unwrap(),
        RecordPoll::Record {
            offset: 0,
            bytes: b"ready\n".to_vec()
        }
    );
    assert_eq!(
        second.poll(&raw, 6, false).unwrap(),
        RecordPoll::Record {
            offset: 0,
            bytes: b"ready\n".to_vec()
        }
    );
    assert_eq!(first.poll(&raw, 16, false).unwrap(), RecordPoll::Pending);
    assert!(first.poll(&raw, 3, false).is_err());
    assert_eq!(
        first.poll(&raw, 16, true).unwrap(),
        RecordPoll::Record {
            offset: 6,
            bytes: b"terminal".to_vec()
        }
    );
    drop(writer);
    drop(prepared);
    fs::rename(raw.path(), root.join("original.stdout")).unwrap();
    fs::write(raw.path(), b"replacement\n").unwrap();
    assert!(RecordCursor::default().poll(&raw, 64, false).is_err());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn exhausted_record_budget_preserves_owner_without_starting_a_new_deadline() {
    kinekino_platform::install();
    let (root, prepared) = evidence();
    let mut process = fake();
    let dropped = process.dropped.clone();
    let clock = FakeClock {
        start: Instant::now(),
        elapsed: Cell::new(Duration::from_millis(80)),
    };
    let deadline = Deadline::new(
        clock.start,
        Duration::from_millis(100),
        Duration::from_millis(30),
    )
    .unwrap();
    process.deadline = deadline;
    let renewed = Deadline::new(
        clock.now(),
        Duration::from_secs(1),
        Duration::from_millis(100),
    )
    .unwrap();
    let mut owner = Supervisor::new(process, prepared.paths.clone(), 1024);
    let error = owner
        .wait_stdout_record(&mut RecordCursor::default(), 64, renewed, &clock)
        .unwrap_err();
    assert!(matches!(error.stage, FailureStage::Deadline));
    assert_eq!(dropped.get(), 0);
    assert_eq!(clock.elapsed.get(), Duration::from_millis(80));
    drop(owner);
    drop(prepared);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn observed_wait_preserves_clock_calls_cutoffs_offsets_and_original_owner() {
    kinekino_platform::install();
    struct ScriptClock {
        start: Instant,
        calls: Cell<usize>,
        cutoff_at: usize,
    }
    impl Clock for ScriptClock {
        fn now(&self) -> Instant {
            let call = self.calls.get();
            self.calls.set(call + 1);
            self.start
                + if call >= self.cutoff_at {
                    Duration::from_secs(2)
                } else {
                    Duration::ZERO
                }
        }
        fn pause(&self, _: Duration) {}
    }
    struct Counted {
        original: Fake,
        polls: Rc<Cell<u32>>,
    }
    impl ProcessFamily for Counted {
        fn deadline(&self) -> Deadline {
            self.original.deadline()
        }
        fn identity(&self) -> &ProcessIdentity {
            self.original.identity()
        }
        fn close_stdin(&mut self) -> io::Result<()> {
            self.original.close_stdin()
        }
        fn poll_child(&mut self) -> io::Result<Option<u32>> {
            self.polls.set(self.polls.get() + 1);
            self.original.poll_child()
        }
        fn poll_family(&mut self) -> io::Result<FamilyObservation> {
            self.original.poll_family()
        }
        fn terminate_owned_family(&mut self) -> io::Result<()> {
            self.original.terminate_owned_family()
        }
    }
    for case in ["pre", "post", "record", "exit", "pending"] {
        let mut outcomes = Vec::new();
        for observed in [false, true] {
            let (root, prepared) = evidence();
            if matches!(case, "post" | "record") {
                fs::write(prepared.paths.stdout.path(), b"ready\n").unwrap();
            }
            let clock = ScriptClock {
                start: Instant::now(),
                calls: Cell::new(0),
                cutoff_at: match case {
                    "pre" => 0,
                    "post" => 1,
                    "pending" => 3,
                    _ => usize::MAX,
                },
            };
            let deadline = Deadline::new(
                clock.start,
                Duration::from_secs(1),
                Duration::from_millis(100),
            )
            .unwrap();
            let mut original = fake();
            original.deadline = deadline;
            if case == "exit" {
                original.exit = Some(9);
            }
            let dropped = original.dropped.clone();
            let killed = original.killed.clone();
            let identity = original.identity.clone();
            let polls = Rc::new(Cell::new(0));
            let mut owner = Supervisor::new(
                Counted {
                    original,
                    polls: polls.clone(),
                },
                prepared.paths.clone(),
                1024,
            );
            let mut cursor = RecordCursor::default();
            let mut metrics = RecordObservation::default();
            let result = if observed {
                owner.wait_stdout_record_observed(&mut cursor, 64, deadline, &clock, &mut metrics)
            } else {
                owner.wait_stdout_record(&mut cursor, 64, deadline, &clock)
            };
            let signature = match result {
                Ok(record) => format!("{record:?}"),
                Err(error) => format!("{:?}:{:?}:{}", error.stage, error.error.kind(), error.error),
            };
            assert_eq!(owner.process().identity(), &identity);
            assert_eq!(dropped.get(), 0);
            assert_eq!(killed.get(), 0);
            assert_eq!(
                cursor.offset(),
                if matches!(case, "post" | "record") {
                    6
                } else {
                    0
                }
            );
            assert_eq!(
                polls.get(),
                if matches!(case, "exit" | "pending") {
                    1
                } else {
                    0
                }
            );
            assert_eq!(
                clock.calls.get(),
                match case {
                    "pre" => 1,
                    "post" | "record" | "exit" => 2,
                    "pending" => 4,
                    _ => unreachable!(),
                }
            );
            if observed {
                assert_eq!(
                    metrics.exit,
                    match case {
                        "pre" | "pending" => RecordWaitExit::PreCutoff,
                        "post" => RecordWaitExit::PostCutoff,
                        "record" => RecordWaitExit::Record,
                        "exit" => RecordWaitExit::ChildExit,
                        _ => unreachable!(),
                    }
                );
                assert_eq!(metrics.child_poll.calls, Some(u64::from(polls.get())));
                assert_eq!(metrics.offset_after, cursor.offset());
                assert_eq!(
                    metrics.last_record,
                    match case {
                        "pre" => RecordKind::NotReached,
                        "post" | "record" => RecordKind::Record,
                        _ => RecordKind::Pending,
                    }
                );
                assert!(!metrics.has_unknown());
            }
            outcomes.push((signature, cursor.offset(), polls.get(), clock.calls.get()));
            drop(owner);
            assert_eq!(dropped.get(), 1);
            drop(prepared);
            fs::remove_dir_all(root).unwrap();
        }
        assert_eq!(outcomes[0], outcomes[1], "{case}");
    }
}

#[test]
fn exhausted_cleanup_reserve_never_fabricates_a_post_termination_exit() {
    kinekino_platform::install();
    let (root, prepared) = evidence();
    let process = fake();
    let killed = process.killed.clone();
    let dropped = process.dropped.clone();
    let clock = FakeClock {
        start: Instant::now(),
        elapsed: Cell::new(Duration::from_millis(100)),
    };
    let deadline = Deadline::new(
        clock.start,
        Duration::from_millis(100),
        Duration::from_millis(30),
    )
    .unwrap();
    match Supervisor::new(process, prepared.paths.clone(), 1024).finish(
        deadline,
        &clock,
        &mut Sink::default(),
    ) {
        FinishOutcome::Incomplete(failure) => {
            assert!(matches!(failure.cause.stage, FailureStage::Deadline));
            assert_eq!(failure.owner.receipt().child_exit, None);
            assert_eq!(failure.owner.receipt().family, FamilyObservation::Active(1));
            assert!(failure.owner.receipt().forced);
            assert_eq!(killed.get(), 1);
            assert_eq!(dropped.get(), 0);
            drop(failure);
        }
        _ => panic!("termination request is not an exit observation"),
    }
    assert_eq!(clock.elapsed.get(), Duration::from_millis(100));
    drop(prepared);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn terminal_receipt_failure_retains_owner_and_original_nonzero_exit() {
    kinekino_platform::install();
    let (root, prepared) = evidence();
    let mut process = fake();
    process.exit = Some(7);
    process.active = false;
    let killed = process.killed.clone();
    let dropped = process.dropped.clone();
    let now = Instant::now();
    match Supervisor::new(process, prepared.paths.clone(), 1024).finish(
        Deadline::new(now, Duration::from_secs(1), Duration::from_millis(100)).unwrap(),
        &SystemClock,
        &mut Sink {
            fail: true,
            receipts: vec![],
        },
    ) {
        FinishOutcome::Incomplete(failure) => {
            assert!(matches!(failure.cause.stage, FailureStage::Receipt));
            assert_eq!(failure.owner.receipt().child_exit, Some(7));
            assert_eq!(failure.owner.receipt().family, FamilyObservation::Empty);
            assert!(failure.receipt_error.is_some());
            assert_eq!(killed.get(), 0);
            assert_eq!(dropped.get(), 0);
            drop(failure);
        }
        _ => panic!("failed receipt persistence cannot report completed evidence"),
    }
    drop(prepared);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sink_ok_after_deadline_keeps_owner_terminal_facts_and_successful_write() {
    kinekino_platform::install();
    struct LateSink<'a> {
        clock: &'a FakeClock,
        records: Vec<ProcessReceipt>,
    }
    impl ReceiptSink for LateSink<'_> {
        fn record(&mut self, receipt: &ProcessReceipt) -> io::Result<()> {
            self.records.push(receipt.clone());
            self.clock.elapsed.set(Duration::from_millis(200));
            Ok(())
        }
    }
    let (root, prepared) = evidence();
    let mut process = fake();
    process.exit = Some(0);
    process.active = false;
    let dropped = process.dropped.clone();
    let killed = process.killed.clone();
    let clock = FakeClock {
        start: Instant::now(),
        elapsed: Cell::new(Duration::ZERO),
    };
    let deadline = Deadline::new(
        clock.start,
        Duration::from_millis(100),
        Duration::from_millis(30),
    )
    .unwrap();
    process.deadline = deadline;
    let identity = process.identity.clone();
    let mut sink = LateSink {
        clock: &clock,
        records: vec![],
    };
    match Supervisor::new(process, prepared.paths.clone(), 1024).finish(deadline, &clock, &mut sink)
    {
        FinishOutcome::Incomplete(failure) => {
            assert!(matches!(failure.cause.stage, FailureStage::Deadline));
            assert_eq!(failure.cause.error.kind(), io::ErrorKind::TimedOut);
            assert_eq!(failure.owner.process().identity(), &identity);
            assert_eq!(failure.owner.receipt().child_exit, Some(0));
            assert_eq!(failure.owner.receipt().family, FamilyObservation::Empty);
            assert!(failure.owner.receipt_recorded());
            assert!(!failure.owner.receipt().forced);
            assert!(failure.termination_error.is_none());
            assert!(failure.receipt_error.is_none());
            assert_eq!(sink.records.len(), 1);
            assert_eq!(killed.get(), 0);
            assert_eq!(dropped.get(), 0);
            drop(failure);
        }
        FinishOutcome::Complete(_) => {
            panic!("late sink Ok must not complete within the original deadline")
        }
    }
    assert_eq!(dropped.get(), 1);
    drop(prepared);
    fs::remove_dir_all(root).unwrap();
}
