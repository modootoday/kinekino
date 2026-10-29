use super::*;

fn reading(ticks: u64) -> CounterReading {
    CounterReading {
        ticks,
        domain: CounterDomain {
            kind: "windows-qpc/1".into(),
            frequency_hz: DecimalU64(1000),
            parent_pid: 42,
            parent_creation_filetime: DecimalU64(99),
        },
    }
}
fn cutoff() -> WorkerOperationCutoff {
    WorkerOperationCutoff::correlate(
        reading(100),
        reading(104),
        Duration::from_secs(10),
        "a".repeat(64),
    )
    .unwrap()
}

#[test]
fn early_counter_projection_and_cancellation_keep_the_absolute_cutoff() {
    let cutoff = cutoff();
    assert_eq!(cutoff.operation_cutoff.0, 10097);
    assert_eq!(cutoff.uncertainty_ticks.0, 3);
    assert_eq!(
        cutoff
            .checkpoint(&"a".repeat(64), &reading(104), false)
            .unwrap(),
        WorkerControlState::Continue
    );
    assert_eq!(
        cutoff
            .checkpoint(&"a".repeat(64), &reading(10096), true)
            .unwrap(),
        WorkerControlState::CancellationRequested
    );
    assert_eq!(
        cutoff
            .checkpoint(&"a".repeat(64), &reading(10097), true)
            .unwrap_err()
            .code,
        ErrorCode::Timeout
    );
    // Receiving/rechecking late never renews a duration.
    assert_eq!(
        cutoff
            .checkpoint(&"a".repeat(64), &reading(20000), false)
            .unwrap_err()
            .code,
        ErrorCode::Timeout
    );
}
#[test]
fn mismatched_request_live_parent_frequency_and_counter_regression_fail_closed() {
    let cutoff = cutoff();
    assert_eq!(
        cutoff
            .checkpoint(&"b".repeat(64), &reading(104), false)
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
    for changed in 0..3 {
        let mut now = reading(104);
        match changed {
            0 => now.domain.parent_creation_filetime.0 += 1,
            1 => now.domain.frequency_hz.0 += 1,
            _ => now.domain.parent_pid += 1,
        }
        assert_eq!(
            cutoff
                .checkpoint(&"a".repeat(64), &now, false)
                .unwrap_err()
                .code,
            ErrorCode::SourceExpired
        );
    }
    assert_eq!(
        cutoff
            .checkpoint(&"a".repeat(64), &reading(99), false)
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
}
#[test]
fn expired_calibration_zero_frequency_and_overflow_are_not_new_timeouts() {
    assert_eq!(
        WorkerOperationCutoff::correlate(
            reading(100),
            reading(104),
            Duration::from_millis(7),
            "a".repeat(64)
        )
        .unwrap_err()
        .code,
        ErrorCode::Timeout
    );
    let mut zero = reading(100);
    zero.domain.frequency_hz.0 = 0;
    assert_eq!(
        WorkerOperationCutoff::correlate(
            zero.clone(),
            zero,
            Duration::from_secs(1),
            "a".repeat(64)
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert!(
        WorkerOperationCutoff::correlate(
            reading(104),
            reading(100),
            Duration::from_secs(10),
            "a".repeat(64)
        )
        .is_err()
    );
    assert!(
        WorkerOperationCutoff::correlate(reading(100), reading(104), Duration::MAX, "a".repeat(64))
            .is_err()
    );
}
#[test]
fn strict_metadata_rejects_unknown_fields_and_weakened_uncertainty() {
    let mut value = serde_json::to_value(cutoff()).unwrap();
    value["timeout_ms"] = serde_json::json!(10000);
    assert!(serde_json::from_value::<WorkerOperationCutoff>(value).is_err());
    let mut weak = cutoff();
    weak.uncertainty_ticks.0 = 1;
    assert!(weak.validate().is_err());
}
