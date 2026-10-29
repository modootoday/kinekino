//! ASS/SSA v4+ input: `[Events]` `Dialogue` lines → plain cues. Styles, positioning and
//! override tags are dropped and reported as losses; `Name` follows the speaker policy.
use super::{Cue, CueDocument, SpeakerPolicy, SubtitleError};
use kinekino_media_contracts::api_values::DecimalU64;

/// `H:MM:SS.cc` → microseconds.
pub fn ass_time(value: &str) -> Result<u64, SubtitleError> {
    let bad = SubtitleError::Invalid("Invalid ASS timestamp");
    let (clock, centis) = value.trim().split_once('.').ok_or(bad.clone())?;
    let parts: Vec<&str> = clock.split(':').collect();
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if parts.len() != 3
        || !parts.iter().all(|p| digits(p))
        || parts[1].len() != 2
        || parts[2].len() != 2
        || centis.len() != 2
        || !digits(centis)
    {
        return Err(bad);
    }
    let n = |s: &str| s.parse::<u64>().map_err(|_| bad.clone());
    let (h, m, s, c) = (n(parts[0])?, n(parts[1])?, n(parts[2])?, n(centis)?);
    if m > 59 || s > 59 {
        return Err(SubtitleError::Invalid("ASS clock field out of range"));
    }
    h.checked_mul(3600)
        .and_then(|v| v.checked_add(m * 60 + s))
        .and_then(|v| v.checked_mul(100))
        .and_then(|v| v.checked_add(c))
        .and_then(|v| v.checked_mul(10_000))
        .ok_or(SubtitleError::Invalid("Timestamp overflow"))
}

fn plain(text: &str, document: &mut CueDocument) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        match rest[i..].find('}') {
            Some(j) => {
                document.add_loss(
                    "markup_removed",
                    "Subtitle styling, language or inline timing markup is omitted",
                );
                rest = &rest[i + j + 1..];
            }
            None => {
                out.push_str(&rest[i..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out.replace("\\N", "\n")
        .replace("\\n", "\n")
        .replace("\\h", "\u{00a0}")
}

pub fn parse_ass(input: &str, speakers: SpeakerPolicy) -> Result<CueDocument, SubtitleError> {
    if input.contains('\0') {
        return Err(SubtitleError::Invalid("Invalid text control characters"));
    }
    let text = input.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    let mut document = CueDocument {
        cues: Vec::new(),
        losses: Vec::new(),
    };
    let mut section = String::new();
    let mut format: Option<Vec<String>> = None;
    let mut saw_script_info = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') && line.ends_with(']') {
            section = line.to_ascii_lowercase();
            saw_script_info |= section == "[script info]";
            if section.contains("styles") {
                document.add_loss(
                    "position_style_removed",
                    "ASS styles and positioning are omitted",
                );
            }
            continue;
        }
        if section != "[events]" {
            continue;
        }
        let Some((kind, value)) = line.split_once(':') else {
            continue;
        };
        match kind.trim() {
            "Format" => {
                format = Some(
                    value
                        .split(',')
                        .map(|f| f.trim().to_ascii_lowercase())
                        .collect(),
                );
            }
            "Dialogue" => {
                let fields = format
                    .as_ref()
                    .ok_or(SubtitleError::Invalid("ASS Dialogue before Format"))?;
                let text_at = fields
                    .iter()
                    .position(|f| f == "text")
                    .filter(|i| *i + 1 == fields.len());
                let text_at =
                    text_at.ok_or(SubtitleError::Invalid("ASS Format must end with Text"))?;
                let values: Vec<&str> = value.trim_start().splitn(fields.len(), ',').collect();
                if values.len() != fields.len() {
                    return Err(SubtitleError::Invalid("ASS Dialogue field count mismatch"));
                }
                let field = |name: &str| {
                    fields
                        .iter()
                        .position(|f| f == name)
                        .map(|i| values[i].trim())
                };
                let start = ass_time(
                    field("start").ok_or(SubtitleError::Invalid("ASS Format lacks Start"))?,
                )?;
                let end =
                    ass_time(field("end").ok_or(SubtitleError::Invalid("ASS Format lacks End"))?)?;
                let mut body = plain(values[text_at], &mut document);
                if body.trim().is_empty() {
                    continue;
                }
                if let Some(name) = field("name").filter(|n| !n.is_empty()) {
                    document.add_loss("speaker_structure_removed", "Voice annotations lose structured speaker identity; prefix policy retains display labels only");
                    if speakers == SpeakerPolicy::Prefix {
                        body = format!("[{name}] {body}");
                    }
                }
                if start >= end {
                    return Err(SubtitleError::Invalid(
                        "Cue times must be positive ranges ordered by start time",
                    ));
                }
                document.cues.push(Cue {
                    in_us: DecimalU64(start),
                    out_us: DecimalU64(end),
                    text: body,
                });
            }
            "Comment" => document.add_loss("metadata_removed", "ASS comments are omitted"),
            _ => {}
        }
    }
    if !saw_script_info {
        return Err(SubtitleError::Invalid("Missing [Script Info] section"));
    }
    if document
        .cues
        .windows(2)
        .any(|w| w[1].in_us.0 < w[0].in_us.0)
    {
        document.cues.sort_by_key(|c| c.in_us.0);
        document.add_loss("cues_reordered", "ASS events were reordered by start time");
    }
    if document
        .cues
        .windows(2)
        .any(|w| w[1].in_us.0 < w[0].out_us.0)
    {
        document.add_loss(
            "overlap_present",
            "Overlapping cues are preserved; target player display may differ",
        );
    }
    if document.cues.is_empty() {
        return Err(SubtitleError::Invalid("No nonempty cues found"));
    }
    Ok(document)
}
