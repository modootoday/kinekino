//! VAD score → speech region bridge and WAV cutting.
use super::PcmTrack;
use crate::port::InferenceError;

/// Track-relative seconds, `start < end`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeechRegion {
    pub start: f64,
    pub end: f64,
}

/// Merges frames with `score >= threshold` into regions. Gaps shorter than `merge_gap_secs`
/// join neighbours, regions longer than `max_secs` are split (ASR request limit), and each
/// region is padded by `pad_secs` within `[0, duration]`.
pub fn speech_regions(
    scores: &[f32],
    frame_secs: f64,
    threshold: f32,
    merge_gap_secs: f64,
    pad_secs: f64,
    max_secs: f64,
    duration_secs: f64,
) -> Vec<SpeechRegion> {
    let mut raw: Vec<SpeechRegion> = Vec::new();
    for (i, score) in scores.iter().enumerate() {
        if *score < threshold {
            continue;
        }
        let (start, end) = (i as f64 * frame_secs, (i + 1) as f64 * frame_secs);
        match raw.last_mut() {
            Some(last) if start - last.end <= merge_gap_secs + 1e-9 => last.end = end,
            _ => raw.push(SpeechRegion { start, end }),
        }
    }
    let limit = if max_secs > 0.0 {
        max_secs
    } else {
        f64::INFINITY
    };
    let mut out = Vec::new();
    for r in raw {
        let (start, end) = (
            (r.start - pad_secs).max(0.0),
            (r.end + pad_secs).min(duration_secs),
        );
        let mut at = start;
        while end - at > 1e-9 {
            let next = (at + limit).min(end);
            out.push(SpeechRegion {
                start: at,
                end: next,
            });
            at = next;
        }
    }
    out
}

/// RIFF/WAVE PCM16 mono bytes for `region` of `track`.
pub fn wav_segment(track: &PcmTrack, region: SpeechRegion) -> Result<Vec<u8>, InferenceError> {
    let rate = f64::from(track.sample_rate);
    if track.sample_rate == 0 || !(region.start >= 0.0 && region.end > region.start) {
        return Err(InferenceError::InvalidInput(format!(
            "bad region {:.3}..{:.3} at {} Hz",
            region.start, region.end, track.sample_rate
        )));
    }
    let len = track.samples.len();
    let first = ((region.start * rate).floor() as usize).min(len);
    let last = ((region.end * rate).ceil() as usize).min(len);
    if first >= last {
        return Err(InferenceError::InvalidInput(format!(
            "region {:.3}..{:.3} outside track",
            region.start, region.end
        )));
    }
    let data = &track.samples[first..last];
    let bytes = u32::try_from(data.len() * 2)
        .map_err(|_| InferenceError::InvalidInput("segment too long".into()))?;
    let mut wav = Vec::with_capacity(44 + data.len() * 2);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + bytes).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&track.sample_rate.to_le_bytes());
    wav.extend_from_slice(&(track.sample_rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&bytes.to_le_bytes());
    for s in data {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    Ok(wav)
}
