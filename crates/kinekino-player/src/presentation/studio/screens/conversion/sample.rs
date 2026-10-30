//! Explicit design fixture snippets, never a live subtitle converter.
use crate::{
    application::workspace_read::{CellValue, ReadOrigin, WorkspaceSnapshot},
    presentation::components::cue_comparison::Cue,
};

pub(super) fn cues(snapshot: &WorkspaceSnapshot, output: bool) -> Vec<Cue> {
    if snapshot.origin != ReadOrigin::DesignSample {
        return Vec::new();
    }
    let times = [
        (
            0,
            15_000_000,
            "00:00:00.000",
            "00:00:15.000",
            "00:00:00,000",
            "00:00:15,000",
        ),
        (
            25_000_000,
            40_000_000,
            "00:00:25.000",
            "00:00:40.000",
            "00:00:25,000",
            "00:00:40,000",
        ),
        (
            50_000_000,
            65_000_000,
            "00:00:50.000",
            "00:01:05.000",
            "00:00:50,000",
            "00:01:05,000",
        ),
    ];
    snapshot
        .table
        .rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            let &(start_us, end_us, vtt_start, vtt_end, srt_start, srt_end) = times.get(index)?;
            let cell = |id: &str| {
                row.cells
                    .iter()
                    .find(|cell| cell.column_id == id)
                    .map(|cell| &cell.value)
            };
            if !matches!(cell("start"), Some(CellValue::TimeUs(time)) if time.0 == start_us)
                || !matches!(cell("end"), Some(CellValue::TimeUs(time)) if time.0 == end_us)
            {
                return None;
            }
            let Some(CellValue::Text(text)) = cell("original") else {
                return None;
            };
            let (start, end) = if output {
                (srt_start, srt_end)
            } else {
                (vtt_start, vtt_end)
            };
            Some(Cue {
                id: row.row_id.clone(),
                number: index + 1,
                timing: format!("{start} → {end}"),
                text: text.clone(),
                syntax: format!("{}\n{start} --> {end}\n{text}", index + 1),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::workspace_read::{
            WorkspaceId, WorkspaceReadPort,
            preview::{PreviewScenario, PreviewWorkspaceProvider},
        },
        domain::subtitles::{SpeakerPolicy, SubtitleFormat, parser},
    };

    #[test]
    fn fixture_pairs_preserve_cue_text_and_times_without_using_correction() {
        for scenario in [
            PreviewScenario::Populated,
            PreviewScenario::LongLabels,
            PreviewScenario::NeedsInput,
        ] {
            let snapshot = PreviewWorkspaceProvider::new(scenario).snapshot(WorkspaceId::Sub);
            let source = cues(&snapshot, false);
            let target = cues(&snapshot, true);
            assert_eq!(source.len(), 3);
            for (source, target) in source.iter().zip(&target) {
                let original = parser::parse(
                    &format!("WEBVTT\n\n{}\n\n", source.syntax),
                    SubtitleFormat::Vtt,
                    SpeakerPolicy::Omit,
                )
                .unwrap();
                let output = parser::parse(
                    &format!("{}\n\n", target.syntax),
                    SubtitleFormat::Srt,
                    SpeakerPolicy::Omit,
                )
                .unwrap();
                assert_eq!(original.cues, output.cues);
                assert_eq!(source.id, target.id);
                assert_eq!(source.text, target.text);
                assert!(!target.text.contains("교정 제안"));
            }
        }
        let mut snapshot =
            PreviewWorkspaceProvider::new(PreviewScenario::Populated).snapshot(WorkspaceId::Sub);
        snapshot.origin = ReadOrigin::Live;
        assert!(cues(&snapshot, true).is_empty());
    }
}
