//! ASR timed text → cues: joins consecutive chunks into readable cues (line length, duration
//! and gap limits). Inputs are seconds (ASR chunks); cues are microseconds.
use super::{Cue, CueDocument, SubtitleError};
use kinekino_media_contracts::api_values::DecimalU64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TimedText {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CueLayout {
    pub max_chars: usize,
    pub max_secs: f64,
    /// A silence longer than this always starts a new cue.
    pub max_gap_secs: f64,
    /// Short cues are extended (without overlapping the next) to this length.
    pub min_secs: f64,
}

impl Default for CueLayout {
    fn default() -> Self {
        Self {
            max_chars: 42,
            max_secs: 6.0,
            max_gap_secs: 0.8,
            min_secs: 0.7,
        }
    }
}

fn micros(secs: f64) -> u64 {
    (secs * 1_000_000.0).round() as u64
}

pub fn cues_from_timed_text(
    items: &[TimedText],
    layout: CueLayout,
) -> Result<CueDocument, SubtitleError> {
    let mut spans: Vec<(f64, f64, String)> = Vec::new();
    let mut last_start = 0.0;
    for item in items {
        if !(item.start.is_finite()
            && item.end.is_finite()
            && item.start >= 0.0
            && item.end > item.start)
        {
            return Err(SubtitleError::Invalid(
                "Timed text needs positive finite ranges",
            ));
        }
        if item.start < last_start {
            return Err(SubtitleError::Invalid(
                "Timed text must be ordered by start time",
            ));
        }
        last_start = item.start;
        let text = item.text.split_whitespace().collect::<Vec<_>>().join(" ");
        if text.is_empty() {
            continue;
        }
        match spans.last_mut() {
            Some((start, end, joined))
                if item.start - *end <= layout.max_gap_secs
                    && item.end - *start <= layout.max_secs
                    && joined.chars().count() + 1 + text.chars().count() <= layout.max_chars =>
            {
                *end = end.max(item.end);
                joined.push(' ');
                joined.push_str(&text);
            }
            _ => spans.push((item.start, item.end, text)),
        }
    }
    if spans.is_empty() {
        return Err(SubtitleError::Invalid("No nonempty cues found"));
    }
    let mut document = CueDocument {
        cues: Vec::with_capacity(spans.len()),
        losses: Vec::new(),
    };
    for i in 0..spans.len() {
        let (start, end, _) = &spans[i];
        let next = spans.get(i + 1).map_or(f64::INFINITY, |s| s.0);
        let mut end = end.max(start + layout.min_secs).min(next.max(*end));
        if end <= *start {
            end = spans[i].1;
        }
        if spans[i].1 > next {
            document.add_loss(
                "overlap_present",
                "Overlapping cues are preserved; target player display may differ",
            );
        }
        document.cues.push(Cue {
            in_us: DecimalU64(micros(*start)),
            out_us: DecimalU64(micros(end)),
            text: std::mem::take(&mut spans[i].2),
        });
    }
    Ok(document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::subtitles::{SubtitleFormat, render};

    fn t(start: f64, end: f64, text: &str) -> TimedText {
        TimedText {
            start,
            end,
            text: text.into(),
        }
    }

    #[test]
    fn joins_until_gap_length_or_duration_limit() {
        let items = [
            t(0.0, 0.5, " 안녕하세요 "),
            t(0.5, 1.0, "반갑습니다"),
            t(3.0, 3.2, "네"),
            t(3.2, 9.5, "긴 문장"),
            t(9.5, 9.6, "   "),
        ];
        let doc = cues_from_timed_text(&items, CueLayout::default()).unwrap();
        let texts: Vec<_> = doc.cues.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, ["안녕하세요 반갑습니다", "네", "긴 문장"]);
        assert_eq!(
            (doc.cues[1].in_us.0, doc.cues[1].out_us.0),
            (3_000_000, 3_200_000)
        );
        assert_eq!(doc.cues[2].out_us.0, 9_500_000);
        let narrow = CueLayout {
            max_chars: 6,
            ..CueLayout::default()
        };
        assert_eq!(cues_from_timed_text(&items, narrow).unwrap().cues.len(), 4);
        let srt = render::render(&doc, SubtitleFormat::Srt);
        assert!(
            srt.starts_with("1\n00:00:00,000 --> 00:00:01,000\n안녕하세요 반갑습니다\n"),
            "{srt}"
        );
        let ass = render::render(&doc, SubtitleFormat::Ass);
        assert!(
            ass.contains("Dialogue: 0,0:00:03.00,0:00:03.20,Default,,0,0,0,,네"),
            "{ass}"
        );
    }

    #[test]
    fn short_cue_extends_to_min_without_overlap() {
        let doc = cues_from_timed_text(&[t(1.0, 1.1, "a"), t(5.0, 5.2, "b")], CueLayout::default())
            .unwrap();
        assert_eq!(doc.cues[0].out_us.0, 1_700_000);
        assert_eq!(doc.cues[1].out_us.0, 5_700_000);
        assert!(doc.losses.is_empty());
    }

    #[test]
    fn rejects_bad_ranges_order_and_empty() {
        let l = CueLayout::default();
        assert!(cues_from_timed_text(&[t(2.0, 1.0, "a")], l).is_err());
        assert!(cues_from_timed_text(&[t(f64::NAN, 1.0, "a")], l).is_err());
        assert!(cues_from_timed_text(&[t(2.0, 3.0, "a"), t(1.0, 3.0, "b")], l).is_err());
        assert!(cues_from_timed_text(&[t(0.0, 1.0, " ")], l).is_err());
        assert!(cues_from_timed_text(&[], l).is_err());
    }
}
