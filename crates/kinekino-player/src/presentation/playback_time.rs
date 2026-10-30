//! Presentation timestamps only. SDK inputs remain exact unsigned microseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimeInputError {
    Format,
    Range,
    Overflow,
}

/// Floor to milliseconds so a displayed endpoint never advances past its observation.
pub(crate) fn display_time(microseconds: Option<u64>) -> String {
    let Some(us) = microseconds else {
        return "미확인".into();
    };
    let milliseconds = us / 1_000;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        milliseconds / 3_600_000,
        milliseconds / 60_000 % 60,
        milliseconds / 1_000 % 60,
        milliseconds % 1_000
    )
}

/// H:MM:SS[.ffffff], ASCII only; fractional digits are exact, never floating point.
pub(crate) fn parse_seek_time(value: &str) -> Result<u64, TimeInputError> {
    let value = value.trim();
    if value.is_empty() || value.len() > 64 {
        return Err(TimeInputError::Format);
    }
    let parts: Vec<_> = value.split(':').collect();
    let [hour, minute, seconds] = parts.as_slice() else {
        return Err(TimeInputError::Format);
    };
    let (second, fraction) = match seconds.split_once('.') {
        Some(parts) => parts,
        None => (*seconds, ""),
    };
    fn integer(text: &str) -> Result<u64, TimeInputError> {
        if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(TimeInputError::Format);
        }
        text.parse().map_err(|_| TimeInputError::Overflow)
    }
    if minute.len() != 2
        || second.len() != 2
        || fraction.len() > 6
        || (seconds.contains('.') && fraction.is_empty())
    {
        return Err(TimeInputError::Format);
    }
    let hour = integer(hour)?;
    let minute = integer(minute)?;
    let second = integer(second)?;
    if minute >= 60 || second >= 60 {
        return Err(TimeInputError::Range);
    }
    let fraction = if fraction.is_empty() {
        0
    } else {
        integer(fraction)?
            .checked_mul(10_u64.pow(6 - fraction.len() as u32))
            .ok_or(TimeInputError::Overflow)?
    };
    hour.checked_mul(3_600)
        .and_then(|value| value.checked_add(minute * 60))
        .and_then(|value| value.checked_add(second))
        .and_then(|value| value.checked_mul(1_000_000))
        .and_then(|value| value.checked_add(fraction))
        .ok_or(TimeInputError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observed_time_has_hours_milliseconds_and_unknown_without_invented_zero() {
        assert_eq!(display_time(None), "미확인");
        assert_eq!(display_time(Some(0)), "00:00:00.000");
        assert_eq!(display_time(Some(3_723_456_789)), "01:02:03.456");
        assert_eq!(display_time(Some(59_999_999)), "00:00:59.999");
        assert_eq!(display_time(Some(360_000_000_000)), "100:00:00.000");
        let maximum = display_time(Some(u64::MAX));
        let represented = parse_seek_time(&maximum).unwrap();
        assert!(u64::MAX - represented < 1_000);
    }
    #[test]
    fn human_input_preserves_sdk_microseconds_and_rejects_ambiguous_or_overflow() {
        assert_eq!(parse_seek_time("1:02:03.456789"), Ok(3_723_456_789));
        assert_eq!(parse_seek_time("00:00:00.000001"), Ok(1));
        assert_eq!(parse_seek_time(" 00:01:00.1 "), Ok(60_100_000));
        assert_eq!(parse_seek_time("5124095576:01:49.551615"), Ok(u64::MAX));
        for invalid in [
            "",
            "1000000",
            "01:60:00",
            "00:00:60",
            "-1:00:00",
            "00:00:00.",
            "00:00:00.0000001",
            "00:00:00.nan",
            "0:0:0",
            "１:00:00",
            "5124095576:01:49.551616",
            "99999999999999999:00:00",
        ] {
            assert!(parse_seek_time(invalid).is_err(), "{invalid}");
        }
    }
}
