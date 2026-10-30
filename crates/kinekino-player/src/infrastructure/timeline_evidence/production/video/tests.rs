use super::*;
#[test]
fn typed_window_bounds_reject_overflow_and_do_not_round_subseconds() {
    for (start, end) in [
        (1, 1),
        (2, 1),
        (0, 60_000_001),
        (u64::MAX, 0),
        (86_400_000_000, 86_400_000_001),
    ] {
        assert!(VideoProbeWindow::new(start, end).is_err());
    }
    assert_eq!(
        VideoProbeWindow::new(16_000_001, 26_000_007)
            .unwrap()
            .interval(),
        "16.000001%26.000007"
    );
    assert!(VideoProbeWindow::new(0, 60_000_000).is_ok());
}
