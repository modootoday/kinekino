use super::TimelineError;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeBase {
    pub(super) numerator: u128,
    pub(super) denominator: u128,
}
impl TimeBase {
    pub fn new(numerator: u128, denominator: u128) -> Result<Self, TimelineError> {
        if numerator == 0 || denominator == 0 {
            return Err(TimelineError::InvalidTimeBase);
        }
        let mut left = numerator;
        let mut right = denominator;
        while right != 0 {
            (left, right) = (right, left % right);
        }
        Ok(Self {
            numerator: numerator / left,
            denominator: denominator / left,
        })
    }
    pub fn parse(value: &str) -> Result<Self, TimelineError> {
        let (n, d) = value
            .split_once('/')
            .ok_or(TimelineError::InvalidTimeBase)?;
        if n.is_empty() || d.is_empty() || !n.bytes().chain(d.bytes()).all(|b| b.is_ascii_digit()) {
            return Err(TimelineError::InvalidTimeBase);
        }
        Self::new(
            n.parse().map_err(|_| TimelineError::InvalidTimeBase)?,
            d.parse().map_err(|_| TimelineError::InvalidTimeBase)?,
        )
    }
    pub fn ratio(self) -> (u128, u128) {
        (self.numerator, self.denominator)
    }
    pub fn ticks_at_or_after(self, micros: u64) -> Result<u128, TimelineError> {
        let dividend = u128::from(micros)
            .checked_mul(self.denominator)
            .ok_or(TimelineError::Overflow)?;
        let divisor = self
            .numerator
            .checked_mul(1_000_000)
            .ok_or(TimelineError::Overflow)?;
        Ok(dividend.div_ceil(divisor))
    }
}
