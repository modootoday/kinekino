use super::{Cue, CueDocument, SubtitleError};
use kinekino_media_contracts::api_values::DecimalU64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SubtitleRange {
    pub in_us: DecimalU64,
    pub out_us: DecimalU64,
}

/// Select overlapping cues in [in, out), clip, and rebase to the selected start.
pub fn select(document: &CueDocument, range: SubtitleRange) -> Result<CueDocument, SubtitleError> {
    let start = range.in_us.0;
    let end = range.out_us.0;
    if start >= end {
        return Err(SubtitleError::Invalid(
            "Selected range must have positive duration",
        ));
    }
    let mut selected = CueDocument {
        cues: Vec::new(),
        losses: document.losses.clone(),
    };
    for cue in &document.cues {
        if cue.in_us.0 >= cue.out_us.0 {
            return Err(SubtitleError::Invalid("Cue must have positive duration"));
        }
        if cue.in_us.0 >= end || cue.out_us.0 <= start {
            continue;
        }
        let in_us = cue
            .in_us
            .0
            .max(start)
            .checked_sub(start)
            .ok_or(SubtitleError::Invalid("Range start underflow"))?;
        let out_us = cue
            .out_us
            .0
            .min(end)
            .checked_sub(start)
            .ok_or(SubtitleError::Invalid("Range end underflow"))?;
        if in_us / 1000 >= out_us / 1000 {
            return Err(SubtitleError::Invalid(
                "Selected cue collapses at SRT/VTT millisecond precision",
            ));
        }
        if in_us % 1000 != 0 || out_us % 1000 != 0 {
            selected.add_loss(
                "timestamp_precision_reduced",
                "SRT/VTT timestamps truncate microseconds to milliseconds",
            );
        }
        selected.cues.push(Cue {
            in_us: DecimalU64(in_us),
            out_us: DecimalU64(out_us),
            text: cue.text.clone(),
        });
    }
    if selected.cues.is_empty() {
        return Err(SubtitleError::Invalid(
            "Selected range contains no subtitle cues",
        ));
    }
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(times: &[(u64, u64)]) -> CueDocument {
        CueDocument {
            cues: times
                .iter()
                .map(|&(start, end)| Cue {
                    in_us: DecimalU64(start),
                    out_us: DecimalU64(end),
                    text: "한글 & <原文>".into(),
                })
                .collect(),
            losses: Vec::new(),
        }
    }

    #[test]
    fn clips_half_open_overlap_and_preserves_source() {
        let source = document(&[(0, 1000), (1000, 4000), (4000, 7000), (7000, 9000)]);
        let original = source.cues.clone();
        let selected = select(
            &source,
            SubtitleRange {
                in_us: DecimalU64(1000),
                out_us: DecimalU64(7000),
            },
        )
        .unwrap();
        assert_eq!(selected.cues.len(), 2);
        assert_eq!(
            (selected.cues[0].in_us.0, selected.cues[1].out_us.0),
            (0, 6000)
        );
        assert_eq!(source.cues, original);
        assert_eq!(selected.cues[0].text, original[0].text);
    }

    #[test]
    fn rejects_empty_invalid_and_millisecond_collapsed_selection() {
        let source = document(&[(1000, 2000)]);
        for (start, end) in [(2000, 3000), (1000, 1000), (2000, 1000), (1001, 1999)] {
            assert!(
                select(
                    &source,
                    SubtitleRange {
                        in_us: DecimalU64(start),
                        out_us: DecimalU64(end)
                    }
                )
                .is_err()
            );
        }
    }

    #[test]
    fn records_precision_loss_and_handles_u64_boundaries() {
        let source = document(&[(u64::MAX - 5000, u64::MAX)]);
        let selected = select(
            &source,
            SubtitleRange {
                in_us: DecimalU64(u64::MAX - 4999),
                out_us: DecimalU64(u64::MAX),
            },
        )
        .unwrap();
        assert_eq!(selected.cues[0].out_us.0, 4999);
        assert_eq!(selected.losses[0].code, "timestamp_precision_reduced");
    }
}
