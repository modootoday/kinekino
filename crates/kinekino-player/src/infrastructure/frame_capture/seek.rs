use crate::application::api::{ApiError, ErrorCode};

#[derive(Clone, Copy)]
pub struct TimeBase {
    base: crate::domain::media_timeline::TimeBase,
}
impl TimeBase {
    pub fn parse(value: &str) -> Result<Self, ApiError> {
        Ok(Self {
            base: crate::domain::media_timeline::TimeBase::parse(value).map_err(|_| invalid())?,
        })
    }

    pub fn threshold(self, micros: u64) -> Result<u128, ApiError> {
        let ticks = self.base.ticks_at_or_after(micros).map_err(|_| invalid())?;
        // FFmpeg expression operands are doubles; larger integers lose tick precision.
        if ticks > (1u128 << 53) {
            return Err(invalid());
        }
        Ok(ticks)
    }

    pub fn equivalent(self, value: &str) -> bool {
        Self::parse(value).is_ok_and(|other| self.base == other.base)
    }
}

pub fn input_timestamp(start: u64, requested: u64) -> Option<String> {
    let seek = requested.saturating_sub(2_000_000).max(start);
    (seek > start).then(|| format!("{}.{:06}", seek / 1_000_000, seek % 1_000_000))
}

fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Source time base cannot represent an exact capture threshold",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractional_frame_boundaries_round_toward_the_next_source_tick() {
        let base = TimeBase::parse("1/12288").unwrap();
        assert_eq!(base.threshold(583333).unwrap(), 7168);
        assert_eq!(base.threshold(583334).unwrap(), 7169);
        assert_eq!(base.threshold(500000).unwrap(), 6144);
        assert!(base.equivalent("2/24576"));
        assert!(!base.equivalent("1/1000"));
        assert!(TimeBase::parse("0/1000").is_err());
        assert!(TimeBase::parse("1/0").is_err());
        assert!(TimeBase::parse("-1/1000").is_err());
        assert!(
            TimeBase::parse("1/1000000")
                .unwrap()
                .threshold(u64::MAX)
                .is_err()
        );
    }

    #[test]
    fn preroll_preserves_start_and_absolute_timestamp() {
        assert_eq!(input_timestamp(0, 0), None);
        assert_eq!(input_timestamp(5_000_000, 6_000_000), None);
        assert_eq!(
            input_timestamp(5_000_000, 9_250_001).as_deref(),
            Some("7.250001")
        );
        assert_eq!(
            input_timestamp(0, 820_000_000).as_deref(),
            Some("818.000000")
        );
    }
}
