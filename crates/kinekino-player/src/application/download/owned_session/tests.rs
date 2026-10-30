use super::*;
#[test]
fn parallel_cleanup_errors_preserve_run_failure_and_original_error_fields() {
    let mut run = ApiError::new(ErrorCode::NeedsInput, "original run failure");
    run.request_id = "run-id".into();
    let mut first = Some(run);
    let mut pending = None;
    let mut source = ApiError::new(ErrorCode::EngineFailed, "source cleanup failure");
    source.request_id = "source-id".into();
    source.details = serde_json::json!({"source_detail":7});
    retain_close_error(&mut first, &mut pending, source);
    let mut probe = ApiError::new(ErrorCode::Timeout, "probe cleanup failure");
    probe.request_id = "probe-id".into();
    retain_close_error(&mut first, &mut pending, probe);
    assert_eq!(first.as_ref().unwrap().request_id, "run-id");
    let report = pending.unwrap();
    assert_eq!(report.request_id, "source-id");
    assert_eq!(report.details["primary_details"]["source_detail"], 7);
    assert_eq!(
        report.details["parallel_close_failure"]["request_id"],
        "probe-id"
    );
}
fn exited(exit_code: i32, forced: bool) -> OwnerObservation {
    OwnerObservation::Exited {
        exit_code,
        family_empty: true,
        streams_eof: true,
        forced,
    }
}
#[test]
fn no_child_does_not_manufacture_a_graceful_process_exit() {
    assert_eq!(
        merge_observations(OwnerObservation::NotStarted, OwnerObservation::NotStarted),
        OwnerObservation::NotStarted
    );
    assert_eq!(
        merge_observations(OwnerObservation::NotStarted, exited(0, false)),
        exited(0, false)
    );
}
#[test]
fn source_or_probe_failure_and_force_cannot_be_erased() {
    assert_eq!(
        merge_observations(exited(0, false), exited(23, true)),
        exited(23, true)
    );
    assert_eq!(
        merge_observations(exited(17, true), exited(0, false)),
        exited(17, true)
    );
}
#[test]
fn pending_and_incomplete_streams_never_become_closed() {
    assert_eq!(
        merge_observations(exited(0, false), OwnerObservation::Pending),
        OwnerObservation::Pending
    );
    let incomplete = OwnerObservation::Exited {
        exit_code: 0,
        family_empty: true,
        streams_eof: false,
        forced: false,
    };
    assert!(!closed(merge_observations(exited(0, false), incomplete)));
}
