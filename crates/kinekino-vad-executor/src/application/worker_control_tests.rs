use super::*;
use crate::domain::CounterDomain;
use kinekino_media_contracts::api_values::DecimalU64;
use std::cell::Cell;

struct Clock;
impl CounterClock for Clock {
    fn sample(&self) -> Result<CounterReading, ApiError> {
        Ok(CounterReading {
            ticks: 104,
            domain: CounterDomain {
                kind: "windows-qpc/1".into(),
                frequency_hz: DecimalU64(1000),
                parent_pid: 42,
                parent_creation_filetime: DecimalU64(99),
            },
        })
    }
}
struct Signal {
    failed: Cell<bool>,
    requested: Cell<bool>,
}
impl WorkerCancellation for Signal {
    fn requested(&self) -> Result<bool, ApiError> {
        if self.failed.get() {
            let mut error =
                ApiError::new(ErrorCode::PermissionDenied, "Original signal read denied");
            error.details = serde_json::json!({"original_signal":"denied"});
            return Err(error);
        }
        Ok(self.requested.get())
    }
}
#[test]
fn signal_io_failure_stays_permission_error_after_initial_success() {
    let clock = Clock;
    let before = clock.sample().unwrap();
    let cutoff = WorkerOperationCutoff::correlate(
        before.clone(),
        before,
        Duration::from_secs(10),
        "a".repeat(64),
    )
    .unwrap();
    let signal = Signal {
        failed: Cell::new(false),
        requested: Cell::new(false),
    };
    let digest = "a".repeat(64);
    let control = WorkerControlView::new(&cutoff, &digest, &clock, &signal).unwrap();
    assert_eq!(control.checkpoint().unwrap(), WorkerControlState::Continue);
    signal.requested.set(true);
    assert_eq!(
        control.checkpoint().unwrap(),
        WorkerControlState::CancellationRequested
    );
    signal.failed.set(true);
    let error = control.checkpoint().unwrap_err();
    assert_eq!(error.code, ErrorCode::PermissionDenied);
    assert_eq!(error.message, "Original signal read denied");
    assert_eq!(
        error.details,
        serde_json::json!({"original_signal":"denied"})
    );
}
#[test]
fn clock_observation_failure_is_not_converted_to_cancellation() {
    struct Failed;
    impl CounterClock for Failed {
        fn sample(&self) -> Result<CounterReading, ApiError> {
            Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Original clock unavailable",
            ))
        }
    }
    let before = Clock.sample().unwrap();
    let cutoff = WorkerOperationCutoff::correlate(
        before.clone(),
        before,
        Duration::from_secs(10),
        "a".repeat(64),
    )
    .unwrap();
    let signal = Signal {
        failed: Cell::new(false),
        requested: Cell::new(true),
    };
    let digest = "a".repeat(64);
    let error = match WorkerControlView::new(&cutoff, &digest, &Failed, &signal) {
        Err(error) => error,
        Ok(_) => panic!("Failed clock accepted"),
    };
    assert_eq!(error.code, ErrorCode::EngineFailed);
    assert_eq!(error.message, "Original clock unavailable");
}

#[test]
fn signal_observation_cannot_hide_expiry_between_checkpoints() {
    struct Advancing(Cell<u64>);
    impl CounterClock for Advancing {
        fn sample(&self) -> Result<CounterReading, ApiError> {
            let mut reading = Clock.sample()?;
            reading.ticks = self.0.get();
            self.0.set(reading.ticks + 1);
            Ok(reading)
        }
    }
    let mut before = Clock.sample().unwrap();
    before.ticks = 100;
    let cutoff = WorkerOperationCutoff::correlate(
        before.clone(),
        before,
        Duration::from_millis(10),
        "a".repeat(64),
    )
    .unwrap();
    assert_eq!(cutoff.operation_cutoff.0, 107);
    let clock = Advancing(Cell::new(104));
    let signal = Signal {
        failed: Cell::new(false),
        requested: Cell::new(true),
    };
    let digest = "a".repeat(64);
    let control = WorkerControlView::new(&cutoff, &digest, &clock, &signal).unwrap();
    assert_eq!(control.checkpoint().unwrap_err().code, ErrorCode::Timeout);
}

#[test]
fn shared_control_decoder_is_strict_bounded_and_does_not_accept_a_new_timeout() {
    let before = Clock.sample().unwrap();
    let cutoff = WorkerOperationCutoff::correlate(
        before.clone(),
        before,
        Duration::from_secs(10),
        "a".repeat(64),
    )
    .unwrap();
    let bytes = serde_json::to_vec(&cutoff).unwrap();
    assert_eq!(
        decode_worker_control(&bytes).unwrap().operation_cutoff,
        cutoff.operation_cutoff
    );
    assert_eq!(
        decode_worker_control(&vec![b' '; MAX_WORKER_CONTROL_BYTES + 1])
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    let mut value = serde_json::to_value(cutoff).unwrap();
    value["timeout_ms"] = serde_json::json!(10000);
    assert_eq!(
        decode_worker_control(&serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}
