//! Bounded UI coordinates. SDK time remains exact integer microseconds.
pub(crate) const SEEK_MAX: u64 = 1_000_000;
pub(crate) const VOLUME_MAX: u64 = 1_000;

pub(crate) fn coordinate(text: &str, max: u64) -> Result<u64, &'static str> {
    if text.is_empty() || text.len() > 7 || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("Invalid range coordinate");
    }
    let value = text
        .parse::<u64>()
        .map_err(|_| "Invalid range coordinate")?;
    if value > max {
        return Err("Range coordinate exceeds bounds");
    }
    Ok(value)
}
pub(crate) fn seek_us(value: u64, duration: u64) -> Result<u64, &'static str> {
    if value > SEEK_MAX || duration == 0 {
        return Err("Seek range unavailable");
    }
    u64::try_from(u128::from(value) * u128::from(duration) / u128::from(SEEK_MAX))
        .map_err(|_| "Seek coordinate overflow")
}
pub(crate) fn seek_coordinate(position: u64, duration: u64) -> u64 {
    if duration == 0 {
        return 0;
    }
    (u128::from(position.min(duration)) * u128::from(SEEK_MAX) / u128::from(duration)) as u64
}
pub(crate) fn volume_coordinate(volume: f64) -> Option<u64> {
    (volume.is_finite() && (0.0..=100.0).contains(&volume)).then(|| (volume * 10.0).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seek_coordinates_preserve_full_u64_endpoints_without_float_time() {
        assert_eq!(seek_us(SEEK_MAX, u64::MAX), Ok(u64::MAX));
        assert_eq!(seek_us(0, u64::MAX), Ok(0));
        assert_eq!(seek_us(500_000, u64::MAX), Ok(u64::MAX / 2));
        assert_eq!(seek_coordinate(u64::MAX, u64::MAX), SEEK_MAX);
        assert_eq!(seek_coordinate(99, 10), SEEK_MAX);
        assert!(seek_us(1, 0).is_err());
        assert!(seek_us(SEEK_MAX + 1, 10).is_err());
    }
    #[test]
    fn coordinates_reject_nonfinite_fractional_signed_and_out_of_bounds_input() {
        for value in [
            "", "-1", "+1", "1.5", "NaN", "inf", " 1", "1000001", "99999999",
        ] {
            assert!(coordinate(value, SEEK_MAX).is_err(), "{value}");
        }
        assert_eq!(coordinate("1000", VOLUME_MAX), Ok(1000));
        assert!(coordinate("1001", VOLUME_MAX).is_err());
        assert_eq!(volume_coordinate(65.5), Some(655));
        assert_eq!(volume_coordinate(f64::NAN), None);
    }
}
