//! A span of the current item the user marked to skip (ADR 20261105111300), in seconds.
//! On disk (ADR 20261105131646) the same span is stored as integer milliseconds.

/// `start ≤ position < end`; both finite and `0 ≤ start < end`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkipRange {
    start: f64,
    end: f64,
}

impl SkipRange {
    pub fn new(start: f64, end: f64) -> Option<Self> {
        let valid = start.is_finite() && end.is_finite() && 0.0 <= start && start < end;
        valid.then_some(Self { start, end })
    }
    pub fn start(&self) -> f64 {
        self.start
    }
    pub fn end(&self) -> f64 {
        self.end
    }
    pub fn contains(&self, position: f64) -> bool {
        self.start <= position && position < self.end
    }

    /// Integer milliseconds for the on-disk form (seconds floored to ms).
    pub fn to_ms(&self) -> (u64, u64) {
        (
            (self.start.max(0.0) * 1000.0).floor() as u64,
            (self.end.max(0.0) * 1000.0).floor() as u64,
        )
    }

    /// Rebuild from integer milliseconds; `None` when the pair is not a valid span.
    pub fn from_ms(start_ms: u64, end_ms: u64) -> Option<Self> {
        Self::new(start_ms as f64 / 1000.0, end_ms as f64 / 1000.0)
    }
}

#[cfg(test)]
mod tests;
