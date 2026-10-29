use super::{SubtitleError, SubtitleFormat};

pub fn timestamp(value: &str, format: SubtitleFormat) -> Result<u64, SubtitleError> {
    let separator = if format == SubtitleFormat::Srt {
        ','
    } else {
        '.'
    };
    let (clock, millis) = value
        .split_once(separator)
        .ok_or(SubtitleError::Invalid("Timestamp requires milliseconds"))?;
    let parts: Vec<_> = clock.split(':').collect();
    if millis.len() != 3
        || !millis.bytes().all(|b| b.is_ascii_digit())
        || !(parts.len() == 3 || (format == SubtitleFormat::Vtt && parts.len() == 2))
    {
        return Err(SubtitleError::Invalid("Invalid timestamp syntax"));
    }
    if parts
        .iter()
        .any(|p| p.len() < 2 || !p.bytes().all(|b| b.is_ascii_digit()))
        || parts[parts.len() - 2..].iter().any(|p| p.len() != 2)
    {
        return Err(SubtitleError::Invalid("Invalid clock fields"));
    }
    let numbers: Vec<u64> = parts
        .iter()
        .map(|p| {
            p.parse()
                .map_err(|_| SubtitleError::Invalid("Timestamp overflow"))
        })
        .collect::<Result<_, _>>()?;
    let seconds = numbers[numbers.len() - 1];
    let minutes = numbers[numbers.len() - 2];
    if seconds > 59 || minutes > 59 {
        return Err(SubtitleError::Invalid("Clock field out of range"));
    }
    let hours = if numbers.len() == 3 { numbers[0] } else { 0 };
    hours
        .checked_mul(3600)
        .and_then(|v| v.checked_add(minutes * 60 + seconds))
        .and_then(|v| v.checked_mul(1000))
        .and_then(|v| v.checked_add(millis.parse().ok()?))
        .and_then(|v| v.checked_mul(1000))
        .ok_or(SubtitleError::Invalid("Timestamp overflow"))
}

pub fn format_timestamp(micros: u64, format: SubtitleFormat) -> String {
    let milliseconds = micros / 1000;
    let separator = if format == SubtitleFormat::Srt {
        ','
    } else {
        '.'
    };
    format!(
        "{:02}:{:02}:{:02}{separator}{:03}",
        milliseconds / 3_600_000,
        milliseconds / 60_000 % 60,
        milliseconds / 1000 % 60,
        milliseconds % 1000
    )
}
