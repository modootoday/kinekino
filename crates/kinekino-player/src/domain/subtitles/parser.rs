use super::{Cue, CueDocument, SpeakerPolicy, SubtitleError, SubtitleFormat, format::timestamp};
use crate::domain::api_values::DecimalU64;

pub fn parse(
    input: &str,
    format: SubtitleFormat,
    speakers: SpeakerPolicy,
) -> Result<CueDocument, SubtitleError> {
    if input.contains('\0') || input.contains('\r') && input.replace("\r\n", "").contains('\r') {
        return Err(SubtitleError::Invalid("Invalid text control characters"));
    }
    let normalized = input.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    let grouped = cue_blocks(&normalized);
    let mut blocks = grouped.iter().map(String::as_str);
    let mut document = CueDocument {
        cues: Vec::new(),
        losses: Vec::new(),
    };
    if format == SubtitleFormat::Vtt {
        let header = blocks
            .next()
            .ok_or(SubtitleError::Invalid("Missing WEBVTT header"))?;
        let first = header.lines().next().unwrap_or_default();
        if !(first == "WEBVTT" || first.starts_with("WEBVTT ") || first.starts_with("WEBVTT\t"))
            || header.contains("-->")
        {
            return Err(SubtitleError::Invalid("Invalid WEBVTT header"));
        }
        if header != "WEBVTT" {
            document.add_loss("metadata_removed", "WebVTT header metadata is omitted");
        }
    }
    for block in blocks {
        parse_block(
            block.trim_end_matches('\n'),
            format,
            speakers,
            &mut document,
        )?;
    }
    if document.cues.is_empty() {
        return Err(SubtitleError::Invalid("No nonempty cues found"));
    }
    Ok(document)
}

fn cue_blocks(input: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut lines = Vec::new();
    for line in input.lines() {
        if line.trim().is_empty() {
            if !lines.is_empty() {
                blocks.push(lines.join("\n"));
                lines.clear();
            }
        } else {
            lines.push(line);
        }
    }
    if !lines.is_empty() {
        blocks.push(lines.join("\n"));
    }
    blocks
}

fn parse_block(
    block: &str,
    format: SubtitleFormat,
    speakers: SpeakerPolicy,
    document: &mut CueDocument,
) -> Result<(), SubtitleError> {
    let lines: Vec<_> = block.lines().collect();
    if format == SubtitleFormat::Vtt
        && lines.first().is_some_and(|line| {
            *line == "NOTE"
                || line.starts_with("NOTE ")
                || line.starts_with("NOTE\t")
                || *line == "STYLE"
                || *line == "REGION"
        })
    {
        document.add_loss(
            "metadata_removed",
            "WebVTT notes, styles or regions are omitted",
        );
        return Ok(());
    }
    let timing_index = if lines.first().is_some_and(|line| line.contains("-->")) {
        0
    } else {
        1
    };
    if format == SubtitleFormat::Srt
        && (timing_index != 1
            || lines[0].is_empty()
            || !lines[0].bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(SubtitleError::Invalid("SRT requires a numeric cue index"));
    }
    if format == SubtitleFormat::Vtt && timing_index == 1 {
        document.add_loss(
            "cue_identifier_removed",
            "Cue identifiers are regenerated or omitted",
        );
    }
    let timing = lines
        .get(timing_index)
        .ok_or(SubtitleError::Invalid("Missing cue timing"))?;
    let (start, end) = timing
        .split_once(" --> ")
        .ok_or(SubtitleError::Invalid("Invalid cue timing separator"))?;
    let mut end_fields = end.split_whitespace();
    let end = end_fields
        .next()
        .ok_or(SubtitleError::Invalid("Missing cue end"))?;
    if end_fields.next().is_some() {
        if format == SubtitleFormat::Srt {
            return Err(SubtitleError::Unsupported(
                "SRT positioning extensions are unsupported",
            ));
        }
        document.add_loss(
            "position_style_removed",
            "WebVTT cue positioning and region settings are omitted",
        );
    }
    let start = timestamp(start, format)?;
    let end = timestamp(end, format)?;
    if start >= end || document.cues.last().is_some_and(|cue| start < cue.in_us.0) {
        return Err(SubtitleError::Invalid(
            "Cue times must be positive ranges ordered by start time",
        ));
    }
    if document.cues.last().is_some_and(|cue| start < cue.out_us.0) {
        document.add_loss(
            "overlap_present",
            "Overlapping cues are preserved; target player display may differ",
        );
    }
    let text = lines[timing_index + 1..].join("\n");
    if text.trim().is_empty() {
        return Err(SubtitleError::Invalid("Empty cue text"));
    }
    let text = plain_text(&text, speakers, document)?;
    if text.trim().is_empty() {
        return Err(SubtitleError::Invalid("Cue has no visible text"));
    }
    document.cues.push(Cue {
        in_us: DecimalU64(start),
        out_us: DecimalU64(end),
        text,
    });
    Ok(())
}

fn plain_text(
    text: &str,
    speakers: SpeakerPolicy,
    document: &mut CueDocument,
) -> Result<String, SubtitleError> {
    let mut output = String::new();
    let mut rest = text;
    while let Some(index) = rest.find('<') {
        output.push_str(&rest[..index]);
        let (tag, tail) = rest[index + 1..]
            .split_once('>')
            .ok_or(SubtitleError::Invalid("Unclosed subtitle markup"))?;
        let voice_label = tag.strip_prefix("v ").or_else(|| {
            tag.strip_prefix("v.")
                .and_then(|value| value.split_once(' ').map(|(_, label)| label))
        });
        if let Some(label) = voice_label {
            document.add_loss("speaker_structure_removed", "Voice annotations lose structured speaker identity; prefix policy retains display labels only");
            if speakers == SpeakerPolicy::Prefix {
                output.push('[');
                output.push_str(label);
                output.push_str("] ");
            }
        } else if tag != "/v" {
            document.add_loss(
                "markup_removed",
                "Subtitle styling, language or inline timing markup is omitted",
            );
        }
        rest = tail;
    }
    output.push_str(rest);
    Ok(output
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", "\u{00a0}")
        .replace("&lrm;", "\u{200e}")
        .replace("&rlm;", "\u{200f}")
        .replace("&amp;", "&"))
}
