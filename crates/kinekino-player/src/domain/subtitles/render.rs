use super::{CueDocument, SubtitleFormat, format::format_timestamp};

pub fn render(document: &CueDocument, format: SubtitleFormat) -> String {
    let mut output = if format == SubtitleFormat::Vtt {
        "WEBVTT\n\n".into()
    } else {
        String::new()
    };
    for (index, cue) in document.cues.iter().enumerate() {
        if format == SubtitleFormat::Srt {
            output.push_str(&format!("{}\n", index + 1));
        }
        output.push_str(&format!(
            "{} --> {}\n{}\n\n",
            format_timestamp(cue.in_us.0, format),
            format_timestamp(cue.out_us.0, format),
            cue.text
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
        ));
    }
    output
}
