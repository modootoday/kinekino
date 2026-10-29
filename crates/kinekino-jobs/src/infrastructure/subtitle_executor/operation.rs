use super::invalid;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_subtitles::application::{
    convert::{self, ConvertSubtitles},
    export_range::{self, ExportSubtitleRange},
    from_transcript::{self, CuesFromTranscript},
    storage::SubtitleStorage,
};
use serde_json::Value;

pub(super) const OPERATIONS: [&str; 3] = [
    "subtitles.convert",
    "subtitles.export_range",
    from_transcript::OPERATION,
];
pub(super) enum Request {
    Convert(ConvertSubtitles),
    ExportRange(ExportSubtitleRange),
    FromTranscript(CuesFromTranscript),
}
impl Request {
    pub(super) fn parse(operation: &str, parameters: Value) -> Result<Self, ApiError> {
        match operation {
            "subtitles.convert" => serde_json::from_value(parameters)
                .map(Self::Convert)
                .map_err(|_| invalid("Expected typed SRT/VTT conversion parameters")),
            "subtitles.export_range" => serde_json::from_value(parameters)
                .map(Self::ExportRange)
                .map_err(|_| invalid("Expected typed SRT/VTT range export parameters")),
            from_transcript::OPERATION => {
                from_transcript::parse(parameters).map(Self::FromTranscript)
            }
            _ => Err(ApiError::new(
                ErrorCode::Unsupported,
                "Job operation is unsupported",
            )),
        }
    }
    pub(super) fn input_path(&self) -> &str {
        match self {
            Self::Convert(value) => &value.input_path,
            Self::ExportRange(value) => &value.input_path,
            Self::FromTranscript(value) => &value.input_path,
        }
    }
    pub(super) fn output_path(&self) -> &str {
        match self {
            Self::Convert(value) => &value.output_path,
            Self::ExportRange(value) => &value.output_path,
            Self::FromTranscript(value) => &value.output_path,
        }
    }
    pub(super) fn parameters(&self) -> Result<Value, ApiError> {
        match self {
            Self::Convert(value) => {
                serde_json::to_value(value).map_err(|_| invalid("Invalid conversion parameters"))
            }
            Self::ExportRange(value) => {
                serde_json::to_value(value).map_err(|_| invalid("Invalid range export parameters"))
            }
            Self::FromTranscript(value) => serde_json::to_value(value)
                .map_err(|_| invalid("Invalid transcript subtitle parameters")),
        }
    }
    pub(super) fn execute(&self, storage: &impl SubtitleStorage) -> Result<Value, ApiError> {
        match self {
            Self::Convert(_) => convert::execute(storage, &self.parameters()?),
            Self::ExportRange(value) => export_range::execute_typed(storage, value.clone()),
            Self::FromTranscript(value) => from_transcript::execute_typed(storage, value.clone()),
        }
    }
}
