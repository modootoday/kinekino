use super::storage::{MAX_SUBTITLE_BYTES, SubtitleStorage};
use crate::domain::subtitles::{
    CueDocument, Loss, SpeakerPolicy, SubtitleError, SubtitleFormat, parser, render,
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConvertSubtitles {
    pub input_path: String,
    pub output_path: String,
    pub input_format: SubtitleFormat,
    pub output_format: SubtitleFormat,
    #[serde(default)]
    pub allow_loss: bool,
    #[serde(default)]
    pub speaker_policy: SpeakerPolicy,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConversionResult {
    pub output_path: String,
    pub output_format: SubtitleFormat,
    pub cue_count: usize,
    pub output_bytes: usize,
    pub losses: Vec<Loss>,
    pub encoding: String,
    pub overwrite: bool,
}

pub fn execute(storage: &impl SubtitleStorage, payload: &Value) -> Result<Value, ApiError> {
    validate_formats(payload)?;
    let request: ConvertSubtitles = serde_json::from_value(payload.clone()).map_err(|_| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Expected SRT/VTT conversion payload with explicit formats and paths",
        )
    })?;
    execute_transform(storage, request, Ok)
}

pub(crate) fn validate_formats(payload: &Value) -> Result<(), ApiError> {
    for (field, rejected) in [
        ("input_format", &["ssa", "ocr", "pgs", "vobsub"][..]),
        ("output_format", &["ssa", "ocr", "pgs", "vobsub"][..]),
    ] {
        if payload
            .get(field)
            .and_then(Value::as_str)
            .is_some_and(|format| rejected.contains(&format))
        {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Input must be UTF-8 SRT, WebVTT or ASS v4+; output SRT, WebVTT or ASS (image/OCR unavailable)",
            ));
        }
    }
    Ok(())
}

pub(crate) fn execute_transform(
    storage: &impl SubtitleStorage,
    request: ConvertSubtitles,
    transform: impl FnOnce(CueDocument) -> Result<CueDocument, SubtitleError>,
) -> Result<Value, ApiError> {
    if request.input_path.trim().is_empty()
        || request.output_path.trim().is_empty()
        || request.input_path == request.output_path
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Input and output must be distinct nonempty paths",
        ));
    }
    let input = storage.read_utf8(&request.input_path)?;
    if input.len() > MAX_SUBTITLE_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Subtitle input exceeds 8 MiB",
        ));
    }
    let document = parser::parse(&input, request.input_format, request.speaker_policy)
        .map_err(domain_error)?;
    let document = transform(document).map_err(domain_error)?;
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
    let output = render::render(&document, request.output_format);
    storage.publish_new(&request.output_path, &output)?;
    Ok(json!(ConversionResult {
        output_path: request.output_path,
        output_format: request.output_format,
        cue_count: document.cues.len(),
        output_bytes: output.len(),
        losses: document.losses,
        encoding: "utf-8".into(),
        overwrite: false
    }))
}

pub(crate) fn domain_error(error: SubtitleError) -> ApiError {
    match error {
        SubtitleError::Invalid(message) => ApiError::new(ErrorCode::InvalidArgument, message),
        SubtitleError::Unsupported(message) => ApiError::new(ErrorCode::Unsupported, message),
    }
}

pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["input_path","output_path","input_format","output_format"],"properties":{"input_path":{"type":"string","minLength":1},"output_path":{"type":"string","minLength":1},"input_format":{"enum":["srt","vtt","ass"]},"output_format":{"enum":["srt","vtt","ass"]},"allow_loss":{"type":"boolean","default":false},"speaker_policy":{"enum":["omit","prefix"],"default":"omit"}}})
}
