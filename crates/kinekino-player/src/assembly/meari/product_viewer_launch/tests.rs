use super::*;

#[derive(Default)]
struct BrokenPipe {
    attempts: usize,
}
impl Write for BrokenPipe {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        self.attempts += 1;
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "injected output closure",
        ))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[derive(Default)]
struct FlushBrokenPipe {
    bytes: Vec<u8>,
    flushes: usize,
}
impl Write for FlushBrokenPipe {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.flushes += 1;
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "injected flush closure",
        ))
    }
}
#[test]
fn completed_writes_with_both_flush_failures_stay_secondary() {
    let report = json!({"success":false,"owners_retained":true});
    let before = report.clone();
    let mut stdout = FlushBrokenPipe::default();
    let mut stderr = FlushBrokenPipe::default();
    let failures = write_viewer_report(&report, false, true, 11, &mut stdout, &mut stderr);
    assert_eq!(failures.stdout.unwrap().kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(failures.stderr.unwrap().kind(), io::ErrorKind::BrokenPipe);
    assert_eq!((stdout.flushes, stderr.flushes), (1, 1));
    assert!(!stdout.bytes.is_empty() && !stderr.bytes.is_empty());
    assert_eq!(report, before);
}
#[test]
fn initialization_event_borrows_metadata_and_retains_its_kind_on_flush_failure() {
    let event = json!({"endpoint":"owned-test-endpoint","is_running":true});
    let before = event.clone();
    let mut stdout = FlushBrokenPipe::default();
    let failure = write_viewer_event(
        "local-host-ready",
        &event,
        true,
        12,
        "native-viewer-initialization",
        &mut stdout,
    )
    .unwrap();
    assert_eq!(failure.kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(stdout.flushes, 1);
    let recorded: Value = serde_json::from_slice(&stdout.bytes).unwrap();
    assert_eq!(recorded["kind"], "local-host-ready");
    assert_eq!(recorded["data"], before);
    assert_eq!(event, before);
    let mut disabled = BrokenPipe::default();
    assert!(
        write_viewer_event(
            "local-host-ready",
            &event,
            false,
            13,
            "native-viewer-initialization",
            &mut disabled
        )
        .is_none()
    );
    assert_eq!(disabled.attempts, 0);
}
#[test]
fn both_output_failures_are_secondary_and_preserve_the_primary_failure_report() {
    let report = json!({"success":false,"stage":"native_return","owners_retained":true});
    let before = report.clone();
    let mut stdout = BrokenPipe::default();
    let mut stderr = BrokenPipe::default();
    let failures = write_viewer_report(&report, false, true, 7, &mut stdout, &mut stderr);
    assert_eq!(failures.stdout.unwrap().kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(failures.stderr.unwrap().kind(), io::ErrorKind::BrokenPipe);
    assert!(stdout.attempts > 0 && stderr.attempts > 0);
    assert_eq!(report, before);
}
#[test]
fn broken_recording_output_does_not_print_failure_or_change_a_success_report() {
    let report = json!({"success":true,"cleanup":"WorkersQuiescent(Closed)"});
    let mut stdout = BrokenPipe::default();
    let mut stderr = BrokenPipe::default();
    let failures = write_viewer_report(&report, true, true, 8, &mut stdout, &mut stderr);
    assert_eq!(failures.stdout.unwrap().kind(), io::ErrorKind::BrokenPipe);
    assert!(failures.stderr.is_none());
    assert_eq!(stderr.attempts, 0);
    assert_eq!(report["success"], true);
}
#[test]
fn disabled_recording_skips_stdout_but_still_observes_failure_output_errors() {
    let mut stdout = BrokenPipe::default();
    let mut stderr = BrokenPipe::default();
    let failures = write_viewer_report(
        &json!({"success":false}),
        false,
        false,
        9,
        &mut stdout,
        &mut stderr,
    );
    assert_eq!(stdout.attempts, 0);
    assert!(failures.stdout.is_none());
    assert_eq!(failures.stderr.unwrap().kind(), io::ErrorKind::BrokenPipe);
}
#[test]
fn writable_outputs_keep_the_original_report_in_both_channels() {
    let report = json!({"success":false,"error":"pending ACK"});
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let failures = write_viewer_report(&report, false, true, 10, &mut stdout, &mut stderr);
    assert!(failures.stdout.is_none() && failures.stderr.is_none());
    let recorded: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(recorded["kind"], "native-viewer-outcome");
    assert_eq!(recorded["elapsed_scope"], "native-viewer");
    assert_eq!(recorded["data"], report);
    assert_eq!(serde_json::from_slice::<Value>(&stderr).unwrap(), report);
}

#[test]
fn closed_workers_require_the_original_returned_native_close() {
    let closed = ViewerCleanupStatus::WorkersQuiescent(ViewerNativeState::Closed);
    assert!(success_gate(&closed, true, false, false));
    assert!(!success_gate(&closed, false, false, false));
}
#[test]
fn unconfirmed_or_unrun_workers_never_certify_native_success() {
    for native in [
        ViewerNativeState::NotRun,
        ViewerNativeState::Unconfirmed,
        ViewerNativeState::ForcedIncomplete,
    ] {
        assert!(!success_gate(
            &ViewerCleanupStatus::WorkersQuiescent(native),
            true,
            false,
            false
        ));
    }
}
#[test]
fn ack_fault_and_budget_remain_incomplete_after_window_close() {
    for status in [
        ViewerCleanupStatus::NeedsTerminalAck,
        ViewerCleanupStatus::Faulted,
        ViewerCleanupStatus::BudgetExhausted,
    ] {
        assert!(!success_gate(&status, true, false, false));
    }
}
#[test]
fn original_cleanup_and_acceptance_errors_are_not_overwritten_by_closed_state() {
    let closed = ViewerCleanupStatus::WorkersQuiescent(ViewerNativeState::Closed);
    assert!(!success_gate(&closed, true, true, false));
    assert!(!success_gate(&closed, true, false, true));
}
