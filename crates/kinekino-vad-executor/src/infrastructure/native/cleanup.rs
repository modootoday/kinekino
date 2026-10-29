//! Maps original Supervisor facts without a second OS driver or consuming finish.
use super::*;
use kinekino_process::application::supervisor::{CleanupProgress, ProcessFamily, SystemClock};
use kinekino_process::domain::observation::{FailureCause, FailureStage};

#[derive(Default)]
pub(super) struct CleanupTracking {
    last_recorded: Option<Snapshot>,
    receipt_sequence: u64,
    reported_errors: usize,
    reported_secondary: usize,
    overflow_reported: bool,
    streams_eof: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Snapshot {
    exit: Option<u32>,
    family: FamilyObservation,
    forced: bool,
    eof: bool,
    physical_closed: bool,
    error_count: usize,
    expired: bool,
}
fn snapshot(p: &CleanupProgress<'_>) -> Snapshot {
    let r = p.receipt();
    Snapshot {
        exit: r.child_exit,
        family: r.family,
        forced: r.forced,
        eof: r.stdin_eof_requested,
        physical_closed: p.physical_closed(),
        error_count: usize::from(p.first_error().is_some()) + p.secondary_errors().len(),
        expired: p.deadline_expired(),
    }
}
fn cause(c: &FailureCause) -> ApiError {
    ApiError::new(
        if matches!(c.stage, FailureStage::Deadline) {
            ErrorCode::Timeout
        } else {
            ErrorCode::EngineFailed
        },
        format!("Original VAD cleanup {:?}: {}", c.stage, c.error),
    )
}
fn cap(end: Instant) -> Result<Deadline, ApiError> {
    let total = Duration::from_nanos(2);
    let start = end
        .checked_sub(total)
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "VAD cleanup cutoff underflow"))?;
    Deadline::new(start, total, Duration::from_nanos(1)).map_err(io)
}
pub(super) fn poll(t: &mut NativeTransport, end: Instant) -> Result<OwnerObservation, ApiError> {
    let Some(owner) = t.owner.as_mut() else {
        return Ok(if t.no_child_confirmed {
            OwnerObservation::NotStarted
        } else {
            OwnerObservation::Pending
        });
    };
    poll_owner(
        owner,
        &mut t.cleanup,
        Context {
            original: t.original,
            root: &t.root,
            secondary: &t.secondary,
            secondary_overflow: t.secondary_overflow,
        },
        end,
    )
}
struct Context<'a> {
    original: Deadline,
    root: &'a Path,
    secondary: &'a [AttemptFailure],
    secondary_overflow: bool,
}
fn poll_owner<P: ProcessFamily>(
    owner: &mut Supervisor<P>,
    tracking: &mut CleanupTracking,
    context: Context<'_>,
    end: Instant,
) -> Result<OwnerObservation, ApiError> {
    let requested = context.original.constrained_by(cap(end)?);
    let progress = owner.poll_cleanup(requested, &SystemClock);
    let current = snapshot(&progress);
    if !progress.final_receipt_recorded() && tracking.last_recorded != Some(current) {
        let index = tracking.receipt_sequence;
        tracking.receipt_sequence = index.checked_add(1).ok_or_else(|| {
            ApiError::new(
                ErrorCode::ResourceExhausted,
                "VAD receipt sequence exhausted",
            )
        })?;
        // Partial/final/retry writes always receive fresh immutable destinations.
        if owner
            .record_cleanup(
                &mut Sink(context.root.join(format!("cleanup-{index}.json"))),
                &SystemClock,
            )
            .is_ok()
        {
            tracking.last_recorded = Some(current);
        }
    }
    let progress = owner
        .cleanup_progress()
        .expect("Original cleanup initialized");
    // Each original cause reaches core once; errors never stop future polls of
    // this same owner, and never turn partial receipt persistence into ACK.
    if let Some(error) = context.secondary.get(tracking.reported_secondary) {
        tracking.reported_secondary += 1;
        return Err(ApiError::new(error.code(), error.message()));
    }
    let errors: Vec<_> = progress
        .first_error()
        .into_iter()
        .chain(progress.secondary_errors().iter())
        .collect();
    if let Some(error) = errors.get(tracking.reported_errors) {
        tracking.reported_errors += 1;
        return Err(cause(error));
    }
    if (progress.secondary_overflow() || context.secondary_overflow) && !tracking.overflow_reported
    {
        tracking.overflow_reported = true;
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Original VAD secondary diagnostics overflow preserved",
        ));
    }
    if !progress.physical_closed() || !progress.final_receipt_recorded() {
        return Ok(OwnerObservation::Pending);
    };
    let receipt = progress.receipt().clone();
    if !tracking.streams_eof {
        observe_eof(owner)?;
        tracking.streams_eof = true;
    }
    Ok(OwnerObservation::Exited {
        exit_code: receipt
            .child_exit
            .expect("Physical closure requires original exit") as i32,
        family_empty: receipt.family == FamilyObservation::Empty,
        streams_eof: tracking.streams_eof,
        forced: receipt.forced,
    })
}
fn observe_eof<P: ProcessFamily>(owner: &Supervisor<P>) -> Result<(), ApiError> {
    for raw in [&owner.evidence().stdout, &owner.evidence().stderr] {
        let mut reader = raw.open_read().map_err(io)?.take(MAX_OUTCOME_BYTES + 1);
        if std::io::copy(&mut reader, &mut std::io::sink()).map_err(io)? > MAX_OUTCOME_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Original VAD raw stream exceeded bound",
            ));
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, atomic::AtomicUsize};
    struct FakeFamily {
        deadline: Deadline,
        identity: kinekino_process::domain::identity::ProcessIdentity,
        closed: Arc<Mutex<bool>>,
        eof: Arc<AtomicUsize>,
        forced: Arc<AtomicUsize>,
    }
    impl ProcessFamily for FakeFamily {
        fn deadline(&self) -> Deadline {
            self.deadline
        }
        fn identity(&self) -> &kinekino_process::domain::identity::ProcessIdentity {
            &self.identity
        }
        fn close_stdin(&mut self) -> std::io::Result<()> {
            self.eof.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn poll_child(&mut self) -> std::io::Result<Option<u32>> {
            Ok((*self.closed.lock().unwrap()).then_some(0))
        }
        fn poll_family(&mut self) -> std::io::Result<FamilyObservation> {
            Ok(if *self.closed.lock().unwrap() {
                FamilyObservation::Empty
            } else {
                FamilyObservation::Active(1)
            })
        }
        fn terminate_owned_family(&mut self) -> std::io::Result<()> {
            self.forced.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    struct Fixture {
        root: PathBuf,
        owner: Supervisor<FakeFamily>,
        closed: Arc<Mutex<bool>>,
        eof: Arc<AtomicUsize>,
        forced: Arc<AtomicUsize>,
        original: Deadline,
    }
    impl Fixture {
        fn new(operation_expired: bool, closed: bool) -> Self {
            kinekino_platform::install();
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let root = std::env::temp_dir().join(format!(
                "vad-polled-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            fs::create_dir(&root).unwrap();
            let prepared = PreparedEvidence::create(&root, None, 16).unwrap();
            let paths = prepared.paths.clone();
            drop(prepared);
            let now = Instant::now();
            let original = if operation_expired {
                Deadline::new(
                    now - Duration::from_secs(3),
                    Duration::from_secs(10),
                    Duration::from_secs(8),
                )
                .unwrap()
            } else {
                Deadline::new(now, Duration::from_secs(10), Duration::from_secs(2)).unwrap()
            };
            let closed = Arc::new(Mutex::new(closed));
            let eof = Arc::new(AtomicUsize::new(0));
            let forced = Arc::new(AtomicUsize::new(0));
            let family = FakeFamily {
                deadline: original,
                identity: kinekino_process::domain::identity::ProcessIdentity {
                    pid: 7,
                    creation_filetime: None,
                    executable: PathBuf::from("fake-only"),
                },
                closed: closed.clone(),
                eof: eof.clone(),
                forced: forced.clone(),
            };
            Self {
                root,
                owner: Supervisor::new(family, paths, MAX_OUTCOME_BYTES),
                closed,
                eof,
                forced,
                original,
            }
        }
        fn poll(&mut self, tracking: &mut CleanupTracking) -> Result<OwnerObservation, ApiError> {
            poll_owner(
                &mut self.owner,
                tracking,
                Context {
                    original: self.original,
                    root: &self.root,
                    secondary: &[],
                    secondary_overflow: false,
                },
                Instant::now() + Duration::from_secs(5),
            )
        }
        fn remove(self) {
            let root = self.root.clone();
            drop(self);
            fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn requested_cleanup_cap_preserves_past_end_without_renewal() {
        let end = Instant::now() - Duration::from_secs(1);
        assert!(cap(end).unwrap().remaining(Instant::now()).is_zero());
    }
    #[test]
    fn partial_receipt_snapshot_does_not_imply_physical_closure_or_final_persistence() {
        let pending = Snapshot {
            exit: None,
            family: FamilyObservation::Active(1),
            forced: false,
            eof: true,
            physical_closed: false,
            error_count: 0,
            expired: false,
        };
        let closed = Snapshot {
            exit: Some(0),
            family: FamilyObservation::Empty,
            physical_closed: true,
            ..pending
        };
        assert!(pending != closed);
        let tracking = CleanupTracking {
            last_recorded: Some(pending),
            ..Default::default()
        };
        assert!(tracking.last_recorded != Some(closed));
        assert!(!tracking.streams_eof);
    }
    #[test]
    fn adapter_poll_retains_owner_until_later_original_exit_and_empty_after_force() {
        let mut fixture = Fixture::new(true, false);
        let mut tracking = CleanupTracking::default();
        assert_eq!(
            fixture.poll(&mut tracking).unwrap(),
            OwnerObservation::Pending
        );
        assert_eq!(fixture.forced.load(Ordering::SeqCst), 1);
        assert!(
            !fixture
                .owner
                .cleanup_progress()
                .unwrap()
                .final_receipt_recorded()
        );
        *fixture.closed.lock().unwrap() = true;
        assert!(matches!(
            fixture.poll(&mut tracking).unwrap(),
            OwnerObservation::Exited {
                forced: true,
                streams_eof: true,
                family_empty: true,
                ..
            }
        ));
        assert_eq!(fixture.forced.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.eof.load(Ordering::SeqCst), 1);
        assert!(fixture.owner.cleanup_progress().unwrap().settled());
        fixture.remove();
    }
    #[test]
    fn receipt_write_failure_reaches_core_before_fresh_final_receipt_and_eof() {
        let mut fixture = Fixture::new(false, true);
        fs::write(fixture.root.join("cleanup-0.json"), b"original").unwrap();
        let mut tracking = CleanupTracking::default();
        assert!(fixture.poll(&mut tracking).is_err());
        assert!(
            !fixture
                .owner
                .cleanup_progress()
                .unwrap()
                .final_receipt_recorded()
        );
        assert!(matches!(
            fixture.poll(&mut tracking).unwrap(),
            OwnerObservation::Exited {
                forced: false,
                streams_eof: true,
                ..
            }
        ));
        assert_eq!(
            fs::read(fixture.root.join("cleanup-0.json")).unwrap(),
            b"original"
        );
        assert!(fixture.root.join("cleanup-1.json").is_file());
        assert_eq!(tracking.reported_errors, 1);
        assert_eq!(fixture.eof.load(Ordering::SeqCst), 1);
        fixture.remove();
    }
}
