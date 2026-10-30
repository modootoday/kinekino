//! Source-time boundary policy; constructing a value is not evidence authority.
use super::ValidationError;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaybackWindow {
    start_us: u64,
    end_us: u64,
}
impl PlaybackWindow {
    pub fn new(start_us: u64, end_us: u64) -> Result<Self, ValidationError> {
        if start_us >= end_us {
            return Err(ValidationError::InvalidRange);
        }
        Ok(Self { start_us, end_us })
    }
    pub fn contains_seek(self, position_us: u64) -> bool {
        self.start_us <= position_us && position_us <= self.end_us
    }
    pub fn contains_play(self, position_us: u64) -> bool {
        self.start_us <= position_us && position_us < self.end_us
    }
}
