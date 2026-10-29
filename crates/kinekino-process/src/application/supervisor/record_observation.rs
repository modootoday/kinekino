//! Fixed-size read observation only; never process or deadline authority.
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordKind {
    NotReached,
    Pending,
    Record,
    Error,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordWaitExit {
    NotReturned,
    PreCutoff,
    LimitError,
    PollError,
    PostCutoff,
    Record,
    ChildError,
    ChildExit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhaseObservation {
    pub calls: Option<u64>,
    pub total_us: Option<u64>,
    pub max_us: Option<u64>,
}
impl Default for PhaseObservation {
    fn default() -> Self {
        Self {
            calls: Some(0),
            total_us: Some(0),
            max_us: Some(0),
        }
    }
}
impl PhaseObservation {
    pub(super) fn record(&mut self, elapsed: Duration) {
        let micros = u64::try_from(elapsed.as_micros()).ok();
        self.calls = self.calls.and_then(|value| value.checked_add(1));
        self.total_us = self
            .total_us
            .zip(micros)
            .and_then(|(total, value)| total.checked_add(value));
        self.max_us = self.max_us.zip(micros).map(|(old, value)| old.max(value));
    }
    fn unknown(self) -> bool {
        self.calls.is_none() || self.total_us.is_none() || self.max_us.is_none()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordObservation {
    pub iterations: Option<u64>,
    pub limit_check: PhaseObservation,
    pub record_poll: PhaseObservation,
    pub child_poll: PhaseObservation,
    pub offset_before: u64,
    pub offset_after: u64,
    pub last_record: RecordKind,
    pub exit: RecordWaitExit,
    pub elapsed_us: Option<u64>,
}
impl Default for RecordObservation {
    fn default() -> Self {
        Self {
            iterations: Some(0),
            limit_check: PhaseObservation::default(),
            record_poll: PhaseObservation::default(),
            child_poll: PhaseObservation::default(),
            offset_before: 0,
            offset_after: 0,
            last_record: RecordKind::NotReached,
            exit: RecordWaitExit::NotReturned,
            elapsed_us: None,
        }
    }
}
impl RecordObservation {
    pub fn has_unknown(&self) -> bool {
        self.iterations.is_none()
            || self.limit_check.unknown()
            || self.record_poll.unknown()
            || self.child_poll.unknown()
            || self.elapsed_us.is_none()
    }
    pub(super) fn iteration(&mut self) {
        self.iterations = self.iterations.and_then(|value| value.checked_add(1));
    }
    pub(super) fn complete(&mut self, offset: u64, elapsed: Duration) {
        self.offset_after = offset;
        self.elapsed_us = u64::try_from(elapsed.as_micros()).ok();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aggregate_overflow_is_sticky_unknown_without_allocation() {
        let mut observed = RecordObservation {
            iterations: Some(u64::MAX),
            ..RecordObservation::default()
        };
        observed.iteration();
        observed.iteration();
        assert_eq!(observed.iterations, None);
        observed.record_poll.record(Duration::MAX);
        observed.record_poll.record(Duration::ZERO);
        assert_eq!(observed.record_poll.total_us, None);
        assert_eq!(observed.record_poll.max_us, None);
        observed.child_poll.calls = Some(u64::MAX);
        observed.child_poll.record(Duration::ZERO);
        assert_eq!(observed.child_poll.calls, None);
        assert!(observed.has_unknown());
        assert!(std::mem::size_of::<RecordObservation>() <= 256);
    }
}
