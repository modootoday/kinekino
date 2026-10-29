//! ASS (Advanced SubStation Alpha v4+) output: one editable `Default` style, one `Dialogue` per cue.
use super::{CueDocument, SubtitleError};
use serde::{Deserialize, Serialize};

const SCRIPT_INFO: &str = "[Script Info]\nScriptType: v4.00+\nPlayResX: 1920\nPlayResY: 1080\nWrapStyle: 0\nScaledBorderAndShadow: yes\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n";
const EVENTS: &str =
    "\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";

/// Editable fields of the `Default` style. Colours are ASS `&HAABBGGRR`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AssStyle {
    pub font_name: String,
    pub font_size: u32,
    pub bold: bool,
    pub italic: bool,
    pub primary_colour: String,
    pub outline_colour: String,
    pub back_colour: String,
    pub outline: u32,
    pub shadow: u32,
    /// Numpad layout, 1..=9 (2 = bottom centre).
    pub alignment: u8,
    pub margin_l: u32,
    pub margin_r: u32,
    pub margin_v: u32,
}

impl Default for AssStyle {
    fn default() -> Self {
        Self {
            font_name: "Sans".into(),
            font_size: 64,
            bold: false,
            italic: false,
            primary_colour: "&H00FFFFFF".into(),
            outline_colour: "&H00000000".into(),
            back_colour: "&H80000000".into(),
            outline: 3,
            shadow: 1,
            alignment: 2,
            margin_l: 60,
            margin_r: 60,
            margin_v: 50,
        }
    }
}

fn colour_ok(value: &str) -> bool {
    let b = value.as_bytes();
    b.len() == 10
        && b[0] == b'&'
        && b[1].eq_ignore_ascii_case(&b'h')
        && b[2..].iter().all(u8::is_ascii_hexdigit)
}

impl AssStyle {
    pub fn validate(&self) -> Result<(), SubtitleError> {
        if self.font_name.trim().is_empty()
            || self.font_name.len() > 128
            || self.font_name.contains([',', '\n', '\r'])
        {
            return Err(SubtitleError::Invalid(
                "ASS font_name must be 1..128 chars without commas or newlines",
            ));
        }
        if !(1..=512).contains(&self.font_size) || self.outline > 64 || self.shadow > 64 {
            return Err(SubtitleError::Invalid(
                "ASS font_size must be 1..512 and outline/shadow at most 64",
            ));
        }
        if ![
            &self.primary_colour,
            &self.outline_colour,
            &self.back_colour,
        ]
        .iter()
        .all(|c| colour_ok(c))
        {
            return Err(SubtitleError::Invalid("ASS colours must be &HAABBGGRR"));
        }
        if !(1..=9).contains(&self.alignment)
            || [self.margin_l, self.margin_r, self.margin_v]
                .iter()
                .any(|m| *m > 1920)
        {
            return Err(SubtitleError::Invalid(
                "ASS alignment must be 1..9 and margins at most 1920",
            ));
        }
        Ok(())
    }

    fn line(&self) -> String {
        let flag = |on: bool| if on { -1 } else { 0 };
        format!(
            "Style: Default,{},{},{},&H000000FF,{},{},{},{},0,0,100,100,0,0,1,{},{},{},{},{},{},1\n",
            self.font_name.trim(),
            self.font_size,
            self.primary_colour.to_ascii_uppercase(),
            self.outline_colour.to_ascii_uppercase(),
            self.back_colour.to_ascii_uppercase(),
            flag(self.bold),
            flag(self.italic),
            self.outline,
            self.shadow,
            self.alignment,
            self.margin_l,
            self.margin_r,
            self.margin_v
        )
    }
}

/// `H:MM:SS.cc` (centiseconds, truncated like libass expects).
pub fn ass_timestamp(micros: u64) -> String {
    let cs = micros / 10_000;
    format!(
        "{}:{:02}:{:02}.{:02}",
        cs / 360_000,
        cs / 6000 % 60,
        cs / 100 % 60,
        cs % 100
    )
}

/// Escapes override braces and backslashes; newlines become `\N`.
fn ass_text(text: &str) -> String {
    text.replace('\\', "\u{29F5}")
        .replace('{', "(")
        .replace('}', ")")
        .replace('\n', "\\N")
}

pub fn render(document: &CueDocument) -> String {
    render_styled(document, &AssStyle::default())
}

/// Caller validates `style` first (`AssStyle::validate`).
pub fn render_styled(document: &CueDocument, style: &AssStyle) -> String {
    let mut out = String::from(SCRIPT_INFO);
    out.push_str(&style.line());
    out.push_str(EVENTS);
    for cue in &document.cues {
        out.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            ass_timestamp(cue.in_us.0),
            ass_timestamp(cue.out_us.0),
            ass_text(&cue.text)
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_style_line_is_unchanged() {
        assert!(AssStyle::default().line().starts_with(
            "Style: Default,Sans,64,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,3,1,2,60,60,50,1"
        ));
    }

    #[test]
    fn edited_style_is_rendered_and_validated() {
        let style = AssStyle {
            font_name: "Noto Sans KR".into(),
            font_size: 48,
            bold: true,
            primary_colour: "&h0000ffff".into(),
            alignment: 8,
            margin_v: 20,
            ..AssStyle::default()
        };
        style.validate().unwrap();
        let doc = CueDocument {
            cues: vec![],
            losses: vec![],
        };
        let out = render_styled(&doc, &style);
        assert!(out.contains("Style: Default,Noto Sans KR,48,&H0000FFFF,&H000000FF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,3,1,8,60,60,20,1\n"), "{out}");
        for bad in [
            AssStyle {
                font_name: "a,b".into(),
                ..AssStyle::default()
            },
            AssStyle {
                font_size: 0,
                ..AssStyle::default()
            },
            AssStyle {
                back_colour: "red".into(),
                ..AssStyle::default()
            },
            AssStyle {
                alignment: 10,
                ..AssStyle::default()
            },
            AssStyle {
                margin_v: 5000,
                ..AssStyle::default()
            },
        ] {
            assert!(bad.validate().is_err(), "{bad:?}");
        }
    }
}
