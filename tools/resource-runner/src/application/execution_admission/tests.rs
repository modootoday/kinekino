use super::*;
use crate::execution_profile::MIN_FREE_BYTES;
use std::collections::VecDeque;

struct Probe {
    snapshots: VecDeque<Result<Snapshot, QueryFailure>>,
    waits: usize,
}
impl Resources for Probe {
    fn snapshot(&mut self) -> Result<Snapshot, QueryFailure> {
        self.snapshots.pop_front().expect("unexpected query")
    }
    fn wait_interval(&mut self) -> u128 {
        self.waits += 1;
        1000
    }
}
fn sample(n: u64) -> Snapshot {
    Snapshot {
        idle: n * 60,
        kernel: n * 70,
        user: n * 30,
        free_bytes: MIN_FREE_BYTES,
    }
}
fn probe() -> Probe {
    Probe {
        snapshots: [Ok(sample(0)), Ok(sample(1)), Ok(sample(2))].into(),
        waits: 0,
    }
}
#[test]
fn conservative_and_insufficient_masks_never_sample_or_wait() {
    let mut p = probe();
    let old = admit(Profile::Conservative2, 0b10101, &mut p);
    assert_eq!(old.selected_mask, Some(0b101));
    assert!(old.rejection.is_none());
    assert_eq!(
        admit(Profile::Build4, 0b10101, &mut p).rejection,
        Some("insufficient_permitted_cpus")
    );
    assert_eq!(p.waits, 0);
    assert_eq!(p.snapshots.len(), 3);
}
#[test]
fn build4_requires_two_successful_intervals_and_stays_inside_permitted_mask() {
    let mut p = probe();
    let result = admit(Profile::Build4, 0b101010101, &mut p);
    assert_eq!(result.selected_mask, Some(0b1010101));
    assert_eq!(result.intervals.len(), 2);
    assert_eq!(p.waits, 2);
    assert!(result.rejection.is_none());
}
#[test]
fn second_interval_failure_retains_both_observations_without_admission() {
    let mut p = probe();
    p.snapshots[2] = Ok(Snapshot {
        idle: 70,
        kernel: 140,
        user: 60,
        free_bytes: MIN_FREE_BYTES,
    });
    let result = admit(Profile::Build4, usize::MAX, &mut p);
    assert_eq!(result.rejection, Some("cpu_not_below_50_percent"));
    assert_eq!(result.intervals.len(), 2);
}
#[test]
fn query_and_ram_failures_are_closed_without_retry_or_fallback() {
    for failure_at in 0..3 {
        let mut p = probe();
        p.snapshots[failure_at] = Err(QueryFailure {
            stage: "injected",
            os_code: Some(5),
        });
        let result = admit(Profile::Build4, usize::MAX, &mut p);
        assert_eq!(result.rejection, Some("resource_query_failed"));
        assert_eq!(p.waits, failure_at);
        assert_eq!(result.query_failure.unwrap().os_code, Some(5));
    }
    let mut p = probe();
    p.snapshots[0] = Ok(Snapshot {
        free_bytes: MIN_FREE_BYTES - 1,
        ..sample(0)
    });
    assert_eq!(
        admit(Profile::Build4, usize::MAX, &mut p).rejection,
        Some("free_ram_below_16_gib")
    );
    assert_eq!(p.waits, 1);
}
