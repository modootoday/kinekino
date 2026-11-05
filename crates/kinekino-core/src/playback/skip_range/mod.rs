//! A span of the current item the user marked to skip (ADR 20261105111300), in seconds.

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
}

#[cfg(test)]
mod tests;
