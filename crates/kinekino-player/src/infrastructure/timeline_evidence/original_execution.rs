//! Explicit original-invocation contracts; no Default and no inferred stream selection.
use crate::application::api::{ApiError, ErrorCode};
use serde::Deserialize;
use std::collections::BTreeMap;
#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum ObservedCompletion {
    ObserverAccepted,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum ExecutionContract {
    AudioCopyTsMono16kS16FirstStream,
    VideoFirstStreamFrameWindow { interval: String },
}
fn args(command: &str) -> Result<Vec<String>, ApiError> {
    let line = command.lines().next().ok_or_else(invalid)?;
    serde_json::Deserializer::from_str(line)
        .into_iter::<String>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| invalid())
}
fn fields(raw: &str) -> Result<BTreeMap<&str, &str>, ApiError> {
    let mut result = BTreeMap::new();
    for line in raw.lines() {
        let (key, value) = line.split_once('=').ok_or_else(invalid)?;
        if result.insert(key, value).is_some() {
            return Err(invalid());
        }
    }
    Ok(result)
}
fn value<'a>(fields: &BTreeMap<&'a str, &'a str>, key: &str) -> Result<&'a str, ApiError> {
    fields.get(key).copied().ok_or_else(invalid)
}
fn terminal(process: &str) -> Result<ObservedCompletion, ApiError> {
    let values = fields(process)?;
    // ObserverAccepted is the original wrapper result, not an invented numeric exit.
    if values.len() != 4
        || value(&values, "terminal")? != "true"
        || value(&values, "result")? != "Ok(())"
        || value(&values, "pid")?
            .parse::<u32>()
            .ok()
            .is_none_or(|pid| pid == 0)
        || value(&values, "elapsed_ms")?.parse::<u64>().is_err()
    {
        return Err(invalid());
    }
    Ok(ObservedCompletion::ObserverAccepted)
}
pub(super) fn audio(
    contract: &ExecutionContract,
    command: &str,
    inputs: &str,
    process: &str,
    source_sha: &str,
    engine_sha: &str,
    full_wav_sha: &str,
) -> Result<ObservedCompletion, ApiError> {
    if !matches!(
        contract,
        ExecutionContract::AudioCopyTsMono16kS16FirstStream
    ) {
        return Err(invalid());
    }
    let lines: Vec<_> = inputs.lines().collect();
    let mut pins = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if let Some(path) = line.strip_prefix("path=") {
            let sha = lines
                .get(index + 1)
                .and_then(|line| line.strip_prefix("sha256="))
                .ok_or_else(invalid)?;
            pins.push((path, sha));
        }
    }
    if pins.len() != 3
        || !pins[0].1.eq_ignore_ascii_case(source_sha)
        || !pins[1].1.eq_ignore_ascii_case(full_wav_sha)
        || !pins[2].1.eq_ignore_ascii_case(engine_sha)
    {
        return Err(invalid());
    }
    let expected = [
        pins[2].0,
        "-hide_banner",
        "-nostdin",
        "-nostats",
        "-loglevel",
        "info",
        "-threads",
        "2",
        "-filter_threads",
        "1",
        "-protocol_whitelist",
        "file",
        "-f",
        "mov",
        "-enable_drefs",
        "0",
        "-copyts",
        "-i",
        pins[0].0,
        "-map",
        "0:a:0",
        "-vn",
        "-sn",
        "-dn",
        "-af",
        "aresample=16000:osf=s16:ochl=mono:async=0,asettb=expr=1/16000,ashowinfo",
        "-c:a",
        "pcm_s16le",
        "-f",
        "s16le",
        "pipe:1",
    ];
    if args(command)?.iter().map(String::as_str).ne(expected) {
        return Err(invalid());
    }
    terminal(process)
}
pub(super) fn video(
    contract: &ExecutionContract,
    command: &str,
    inputs: &str,
    process: &str,
    source_sha: &str,
    engine_sha: &str,
) -> Result<ObservedCompletion, ApiError> {
    let ExecutionContract::VideoFirstStreamFrameWindow { interval } = contract else {
        return Err(invalid());
    };
    let (start, end) = interval.split_once('%').ok_or_else(invalid)?;
    let start = start.parse::<u64>().map_err(|_| invalid())?;
    let end = end.parse::<u64>().map_err(|_| invalid())?;
    if start >= end {
        return Err(invalid());
    }
    let values = fields(inputs)?;
    let source = value(&values, "source")?;
    let engine = value(&values, "ffprobe")?;
    if !value(&values, "source_sha256")?.eq_ignore_ascii_case(source_sha)
        || !value(&values, "ffprobe_sha256")?.eq_ignore_ascii_case(engine_sha)
    {
        return Err(invalid());
    }
    let expected = [
        engine,
        "-v",
        "error",
        "-protocol_whitelist",
        "file",
        "-f",
        "mov",
        "-enable_drefs",
        "0",
        "-probesize",
        "1048576",
        "-analyzeduration",
        "1000000",
        "-threads",
        "1",
        "-select_streams",
        "v:0",
        "-read_intervals",
        interval,
        "-show_frames",
        "-show_streams",
        "-show_entries",
        "frame=pts,best_effort_timestamp,duration,key_frame:stream=index,time_base",
        "-of",
        "json",
        source,
    ];
    if args(command)?.iter().map(String::as_str).ne(expected) {
        return Err(invalid());
    }
    terminal(process)
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::RevisionConflict,
        "Original invocation, selected contract or terminal observation differs",
    )
}
#[cfg(test)]
mod tests;
