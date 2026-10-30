use super::raw::RetainedRaw;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::review_source::timeline_evidence::{AudioFrame, MAX_OBSERVED_FRAMES},
};
use std::path::Path;

/// Trusted local assembly only. Retains the original ordinary file and ancestor guards.
/// This does not authenticate a decoder invocation or grant a Verified association.
pub struct RetainedAudioClock {
    frames: Vec<AudioFrame>,
    raw: RetainedRaw,
}
impl RetainedAudioClock {
    pub fn open(path: &Path, expected_sha256: &str) -> Result<Self, ApiError> {
        let mut raw = RetainedRaw::open(path, expected_sha256)?;
        let frames = parse(&raw.read()?)?;
        Ok(Self { frames, raw })
    }
    pub fn frames(&self) -> &[AudioFrame] {
        &self.frames
    }
    pub fn open_live(path: &Path, expected_sha256: &str, max_bytes: u64) -> Result<Self, ApiError> {
        let mut raw = RetainedRaw::open_bounded(path, expected_sha256, max_bytes)?;
        let frames = parse_live(&raw.read()?)?;
        Ok(Self { frames, raw })
    }
    pub fn sha256(&self) -> &str {
        self.raw.sha256()
    }
    pub fn sample_rate(&self) -> u32 {
        16000
    }
}
fn field<'a>(line: &'a str, prefix: &str) -> Result<&'a str, ApiError> {
    let mut found = line
        .split_whitespace()
        .filter_map(|token| token.strip_prefix(prefix));
    let value = found
        .next()
        .ok_or_else(|| invalid("Missing audio clock field"))?;
    if found.next().is_some() {
        return Err(invalid("Duplicate audio clock field"));
    }
    Ok(value)
}
fn unsigned(value: &str) -> Result<u64, ApiError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid("Invalid unsigned clock value"));
    }
    value.parse().map_err(|_| invalid("Audio clock overflow"))
}
fn signed(value: &str) -> Result<i64, ApiError> {
    let digits = value.strip_prefix('-').unwrap_or(value);
    unsigned(digits)?;
    value
        .parse()
        .map_err(|_| invalid("Invalid signed audio PTS"))
}
pub(super) fn parse(raw: &str) -> Result<Vec<AudioFrame>, ApiError> {
    parse_profile(raw, false)
}
pub(super) fn parse_live(raw: &str) -> Result<Vec<AudioFrame>, ApiError> {
    parse_profile(raw, true)
}
fn parse_profile(raw: &str, live: bool) -> Result<Vec<AudioFrame>, ApiError> {
    let mut frames = Vec::new();
    let mut previous_end = None;
    let mut diagnostic_kinds = 0u8;
    for raw_line in raw.lines() {
        // FFmpeg progress uses CR; an audio row can follow it on the same line.
        let Some(start) = raw_line.find("[Parsed_ashowinfo_") else {
            continue;
        };
        let line = &raw_line[start..];
        if line.len() > 4096 {
            return Err(invalid("Audio observation resource limit"));
        }
        if live {
            let tail = line
                .split_once(']')
                .map(|(_, tail)| tail.trim_start())
                .unwrap_or("");
            let kind = if tail.starts_with("side data - matrix encoding:") {
                1
            } else if tail.starts_with("side data - downmix:") {
                2
            } else {
                0
            };
            if kind != 0 {
                if frames.is_empty()
                    || diagnostic_kinds & kind != 0
                    || tail.split_whitespace().any(|token| {
                        ["n:", "pts:", "nb_samples:", "fmt:", "channels:", "rate:"]
                            .iter()
                            .any(|field| token.starts_with(field))
                    })
                {
                    return Err(invalid("Invalid or duplicate live audio diagnostic row"));
                }
                diagnostic_kinds |= kind;
                continue;
            }
        }
        if frames.len() >= MAX_OBSERVED_FRAMES {
            return Err(invalid("Audio observation frame limit"));
        }
        let sequence = unsigned(field(line, "n:")?)?;
        let pts = signed(field(line, "pts:")?)?;
        let samples = u32::try_from(unsigned(field(line, "nb_samples:")?)?)
            .map_err(|_| invalid("Audio sample count overflow"))?;
        if sequence != frames.len() as u64
            || samples == 0
            || field(line, "fmt:")? != "s16"
            || field(line, "channels:")? != "1"
            || field(line, "rate:")? != "16000"
            || previous_end.is_some_and(|end| end != i128::from(pts))
        {
            return Err(invalid("Audio frame sequence, format or PTS discontinuity"));
        }
        previous_end = Some(i128::from(pts) + i128::from(samples));
        diagnostic_kinds = 0;
        frames.push(AudioFrame {
            sequence,
            pts,
            samples,
        });
    }
    if frames.is_empty() {
        return Err(invalid("No audio frame observations"));
    }
    Ok(frames)
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests;
