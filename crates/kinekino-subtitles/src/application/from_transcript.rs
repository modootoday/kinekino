//! `subtitles.from_transcript`: ASR transcript JSON (`{"chunks":[{start,end,text}]}`, seconds,
//! the `kinekino-inference` transcript shape) → SRT/WebVTT/ASS cues, with optional ASS style.
use super::{
    convert::domain_error,
    storage::{MAX_SUBTITLE_BYTES, SubtitleStorage},
};
use crate::domain::subtitles::{
    SubtitleFormat,
    ass::{self, AssStyle},
    render,
    timed_text::{CueLayout, TimedText, cues_from_timed_text},
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const OPERATION: &str = "subtitles.from_transcript";

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LayoutParams {
    pub max_chars: usize,
    pub max_secs: f64,
    pub max_gap_secs: f64,
    pub min_secs: f64,
}
impl Default for LayoutParams {
    fn default() -> Self {
        let l = CueLayout::default();
        Self {
            max_chars: l.max_chars,
            max_secs: l.max_secs,
            max_gap_secs: l.max_gap_secs,
            min_secs: l.min_secs,
        }
    }
}
impl LayoutParams {
    fn layout(self) -> Result<CueLayout, ApiError> {
        let ok = |v: f64, hi: f64| v.is_finite() && (0.0..=hi).contains(&v);
        if !(1..=200).contains(&self.max_chars)
            || !ok(self.max_secs, 60.0)
            || self.max_secs <= 0.0
            || !ok(self.max_gap_secs, 60.0)
            || !ok(self.min_secs, self.max_secs)
        {
            return Err(invalid(
                "layout needs max_chars 1..200, 0<max_secs<=60, gaps/min within range",
            ));
        }
        Ok(CueLayout {
            max_chars: self.max_chars,
            max_secs: self.max_secs,
            max_gap_secs: self.max_gap_secs,
            min_secs: self.min_secs,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CuesFromTranscript {
    pub input_path: String,
    pub output_path: String,
    pub output_format: SubtitleFormat,
    #[serde(default)]
    pub layout: LayoutParams,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ass_style: Option<AssStyle>,
    #[serde(default)]
    pub allow_loss: bool,
}

#[derive(Deserialize)]
struct TranscriptFile {
    chunks: Vec<TimedText>,
}

fn invalid(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

pub fn parse(payload: Value) -> Result<CuesFromTranscript, ApiError> {
    serde_json::from_value(payload).map_err(|_| {
        invalid("Expected transcript-to-subtitle payload with explicit paths and output_format")
    })
}

pub fn execute(storage: &impl SubtitleStorage, payload: &Value) -> Result<Value, ApiError> {
    execute_typed(storage, parse(payload.clone())?)
}

pub fn execute_typed(
    storage: &impl SubtitleStorage,
    request: CuesFromTranscript,
) -> Result<Value, ApiError> {
    if request.input_path.trim().is_empty()
        || request.output_path.trim().is_empty()
        || request.input_path == request.output_path
    {
        return Err(invalid("Input and output must be distinct nonempty paths"));
    }
    if request.ass_style.is_some() && request.output_format != SubtitleFormat::Ass {
        return Err(invalid("ass_style requires output_format ass"));
    }
    let style = request.ass_style.clone().unwrap_or_default();
    style.validate().map_err(domain_error)?;
    let layout = request.layout.layout()?;
    let input = storage.read_utf8(&request.input_path)?;
    if input.len() > MAX_SUBTITLE_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Transcript input exceeds 8 MiB",
        ));
    }
    let transcript: TranscriptFile = serde_json::from_str(&input)
        .map_err(|_| invalid("Transcript must be JSON with chunks[{start,end,text}] in seconds"))?;
    let document = cues_from_timed_text(&transcript.chunks, layout).map_err(domain_error)?;
    if !document.losses.is_empty() && !request.allow_loss {
        let mut error = ApiError::new(
            ErrorCode::NeedsInput,
            "Review conversion losses and explicitly set allow_loss to write",
        );
        error.details = json!({"losses":document.losses});
        error.remediation =
            "Resubmit with allow_loss=true only after reviewing the loss list".into();
        return Err(error);
    }
    let output = if request.output_format == SubtitleFormat::Ass {
        ass::render_styled(&document, &style)
    } else {
        render::render(&document, request.output_format)
    };
    storage.publish_new(&request.output_path, &output)?;
    Ok(json!({
        "output_path": request.output_path,
        "output_format": request.output_format,
        "cue_count": document.cues.len(),
        "source_chunks": transcript.chunks.len(),
        "output_bytes": output.len(),
        "losses": document.losses,
        "encoding": "utf-8",
        "overwrite": false
    }))
}

pub fn schema() -> Value {
    let colour = json!({"type":"string","pattern":"^&[Hh][0-9A-Fa-f]{8}$"});
    json!({"type":"object","additionalProperties":false,"required":["input_path","output_path","output_format"],"properties":{
        "input_path":{"type":"string","minLength":1,"description":"UTF-8 JSON transcript {\"chunks\":[{\"start\":s,\"end\":s,\"text\":...}]}"},
        "output_path":{"type":"string","minLength":1},
        "output_format":{"enum":["srt","vtt","ass"]},
        "allow_loss":{"type":"boolean","default":false},
        "layout":{"type":"object","additionalProperties":false,"properties":{"max_chars":{"type":"integer","minimum":1,"maximum":200,"default":42},"max_secs":{"type":"number","default":6.0},"max_gap_secs":{"type":"number","default":0.8},"min_secs":{"type":"number","default":0.7}}},
        "ass_style":{"type":"object","additionalProperties":false,"properties":{
            "font_name":{"type":"string","minLength":1,"maxLength":128},"font_size":{"type":"integer","minimum":1,"maximum":512},
            "bold":{"type":"boolean"},"italic":{"type":"boolean"},
            "primary_colour":colour,"outline_colour":colour,"back_colour":colour,
            "outline":{"type":"integer","minimum":0,"maximum":64},"shadow":{"type":"integer","minimum":0,"maximum":64},
            "alignment":{"type":"integer","minimum":1,"maximum":9},
            "margin_l":{"type":"integer","minimum":0,"maximum":1920},"margin_r":{"type":"integer","minimum":0,"maximum":1920},"margin_v":{"type":"integer","minimum":0,"maximum":1920}}}
    }})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::BTreeMap};

    #[derive(Default)]
    struct Mem(RefCell<BTreeMap<String, String>>);
    impl SubtitleStorage for Mem {
        fn read_utf8(&self, path: &str) -> Result<String, ApiError> {
            self.0
                .borrow()
                .get(path)
                .cloned()
                .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "missing"))
        }
        fn publish_new(&self, path: &str, contents: &str) -> Result<(), ApiError> {
            let mut files = self.0.borrow_mut();
            if files.contains_key(path) {
                return Err(ApiError::new(ErrorCode::RevisionConflict, "exists"));
            }
            files.insert(path.into(), contents.into());
            Ok(())
        }
    }
    const TRANSCRIPT: &str = r#"{"text":"x","chunks":[{"start":0.0,"end":0.5,"text":"안녕"},{"start":0.5,"end":1.0,"text":"하세요"},{"start":4.0,"end":4.2,"text":"네"}]}"#;

    #[test]
    fn transcript_becomes_styled_ass_and_srt() {
        let mem = Mem::default();
        mem.0
            .borrow_mut()
            .insert("t.json".into(), TRANSCRIPT.into());
        let out = execute(&mem, &json!({"input_path":"t.json","output_path":"a.ass","output_format":"ass","ass_style":{"font_name":"Noto Sans KR","alignment":8}})).unwrap();
        assert_eq!(
            (out["cue_count"].as_u64(), out["source_chunks"].as_u64()),
            (Some(2), Some(3))
        );
        let ass = mem.0.borrow()["a.ass"].clone();
        assert!(ass.contains("Style: Default,Noto Sans KR,64,"), "{ass}");
        assert!(ass.contains(",8,60,60,50,1\n"), "{ass}");
        assert!(ass.contains("Dialogue: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,안녕 하세요"));
        execute(&mem, &json!({"input_path":"t.json","output_path":"a.srt","output_format":"srt","layout":{"max_chars":3}})).unwrap();
        assert!(mem.0.borrow()["a.srt"].starts_with("1\n00:00:00,000 --> 00:00:00,500\n안녕\n"));
        let again = execute(
            &mem,
            &json!({"input_path":"t.json","output_path":"a.srt","output_format":"srt"}),
        );
        assert_eq!(again.unwrap_err().code, ErrorCode::RevisionConflict);
    }

    #[test]
    fn rejects_bad_requests_and_reports_loss() {
        let mem = Mem::default();
        mem.0
            .borrow_mut()
            .insert("t.json".into(), TRANSCRIPT.into());
        mem.0.borrow_mut().insert("bad.json".into(), "[]".into());
        mem.0.borrow_mut().insert(
            "ov.json".into(),
            r#"{"chunks":[{"start":0,"end":9,"text":"a"},{"start":1,"end":2,"text":"b"}]}"#.into(),
        );
        for (payload, code) in [
            (
                json!({"input_path":"t.json","output_path":"o","output_format":"srt","ass_style":{}}),
                ErrorCode::InvalidArgument,
            ),
            (
                json!({"input_path":"t.json","output_path":"o","output_format":"ass","ass_style":{"alignment":0}}),
                ErrorCode::InvalidArgument,
            ),
            (
                json!({"input_path":"t.json","output_path":"o","output_format":"srt","layout":{"max_chars":0}}),
                ErrorCode::InvalidArgument,
            ),
            (
                json!({"input_path":"t.json","output_path":"t.json","output_format":"srt"}),
                ErrorCode::InvalidArgument,
            ),
            (
                json!({"input_path":"bad.json","output_path":"o","output_format":"srt"}),
                ErrorCode::InvalidArgument,
            ),
            (
                json!({"input_path":"t.json","output_path":"o","output_format":"srt","extra":1}),
                ErrorCode::InvalidArgument,
            ),
            (
                json!({"input_path":"ov.json","output_path":"o","output_format":"srt","layout":{"max_chars":1}}),
                ErrorCode::NeedsInput,
            ),
        ] {
            assert_eq!(execute(&mem, &payload).unwrap_err().code, code, "{payload}");
        }
        assert!(!mem.0.borrow().contains_key("o"));
    }
}
