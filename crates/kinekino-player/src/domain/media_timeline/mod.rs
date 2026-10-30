//! Integer timestamp arithmetic only. These values carry no custody or Verified proof.
mod time_base;
mod transform;
pub use time_base::TimeBase;
pub use transform::{Conversion, Rounding, TimelineTransform};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimelineError {
    InvalidTimeBase,
    BeforeOrigin,
    Overflow,
}
#[cfg(test)]
mod tests;
