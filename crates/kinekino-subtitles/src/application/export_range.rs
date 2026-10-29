use super::{
    convert::{self, ConvertSubtitles},
    storage::SubtitleStorage,
};
use crate::domain::subtitles::{
    SpeakerPolicy, SubtitleFormat,
    range::{self, SubtitleRange},
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Separate request contract: existing conversion callers remain unchanged.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportSubtitleRange {
    pub input_path: String,
    pub output_path: String,
    pub input_format: SubtitleFormat,
    pub output_format: SubtitleFormat,
    pub range: SubtitleRange,
    #[serde(default)]
    pub allow_loss: bool,
    #[serde(default)]
    pub speaker_policy: SpeakerPolicy,
}

pub fn execute(storage: &impl SubtitleStorage, payload: &Value) -> Result<Value, ApiError> {
    convert::validate_formats(payload)?;
    let request: ExportSubtitleRange = serde_json::from_value(payload.clone()).map_err(|_| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Expected SRT/VTT range export with decimal-string microsecond bounds",
        )
    })?;
    execute_typed(storage, request)
}

pub fn execute_typed(
    storage: &impl SubtitleStorage,
    request: ExportSubtitleRange,
) -> Result<Value, ApiError> {
    let range = request.range;
    convert::execute_transform(
        storage,
        ConvertSubtitles {
            input_path: request.input_path,
            output_path: request.output_path,
            input_format: request.input_format,
            output_format: request.output_format,
            allow_loss: request.allow_loss,
            speaker_policy: request.speaker_policy,
        },
        |document| range::select(&document, range),
    )
}

pub fn schema() -> Value {
    let mut schema = convert::schema();
    schema["required"]
        .as_array_mut()
        .expect("conversion required array")
        .push(json!("range"));
    schema["properties"]["range"] = json!({"type":"object","additionalProperties":false,"required":["in_us","out_us"],"properties":{"in_us":{"type":"string","pattern":"^[0-9]+$"},"out_us":{"type":"string","pattern":"^[0-9]+$"}}});
    schema
}
