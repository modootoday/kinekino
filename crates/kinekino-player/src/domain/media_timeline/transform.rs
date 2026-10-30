use super::{TimeBase, TimelineError};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rounding {
    Down,
    Up,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conversion<T> {
    pub value: T,
    pub exact: bool,
}
/// Origin must be supplied explicitly. Construction never authenticates that origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimelineTransform {
    time_base: TimeBase,
    origin_pts: i64,
}
fn divide(value: u128, divisor: u128, rounding: Rounding) -> Conversion<u128> {
    Conversion {
        value: match rounding {
            Rounding::Down => value / divisor,
            Rounding::Up => value.div_ceil(divisor),
        },
        exact: value.is_multiple_of(divisor),
    }
}
impl TimelineTransform {
    pub fn new(time_base: TimeBase, origin_pts: i64) -> Self {
        Self {
            time_base,
            origin_pts,
        }
    }
    pub fn pts_to_us(self, pts: i64, rounding: Rounding) -> Result<Conversion<u64>, TimelineError> {
        let delta = i128::from(pts) - i128::from(self.origin_pts);
        if delta < 0 {
            return Err(TimelineError::BeforeOrigin);
        }
        let dividend = (delta as u128)
            .checked_mul(self.time_base.numerator)
            .and_then(|v| v.checked_mul(1_000_000))
            .ok_or(TimelineError::Overflow)?;
        let converted = divide(dividend, self.time_base.denominator, rounding);
        Ok(Conversion {
            value: converted
                .value
                .try_into()
                .map_err(|_| TimelineError::Overflow)?,
            exact: converted.exact,
        })
    }
    pub fn us_to_pts(
        self,
        micros: u64,
        rounding: Rounding,
    ) -> Result<Conversion<i64>, TimelineError> {
        let dividend = u128::from(micros)
            .checked_mul(self.time_base.denominator)
            .ok_or(TimelineError::Overflow)?;
        let divisor = self
            .time_base
            .numerator
            .checked_mul(1_000_000)
            .ok_or(TimelineError::Overflow)?;
        let converted = divide(dividend, divisor, rounding);
        let delta = i128::try_from(converted.value).map_err(|_| TimelineError::Overflow)?;
        let value = i128::from(self.origin_pts)
            .checked_add(delta)
            .and_then(|v| i64::try_from(v).ok())
            .ok_or(TimelineError::Overflow)?;
        Ok(Conversion {
            value,
            exact: converted.exact,
        })
    }
}
