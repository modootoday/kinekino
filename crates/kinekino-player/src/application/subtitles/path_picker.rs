use crate::{
    application::api::{ApiError, ErrorCode},
    domain::subtitles::SubtitleFormat,
};
use std::rc::Rc;

/// Call on the GUI thread. None means user cancellation, never a successful export.
pub trait SubtitlePathPicker {
    fn available(&self) -> bool {
        true
    }
    fn open_subtitle(&self) -> Result<Option<String>, ApiError>;
    fn save_subtitle(&self, format: SubtitleFormat) -> Result<Option<String>, ApiError>;
}

pub type SubtitlePathPickerHandle = Rc<dyn SubtitlePathPicker>;

pub struct TextPathOnly;
impl SubtitlePathPicker for TextPathOnly {
    fn available(&self) -> bool {
        false
    }
    fn open_subtitle(&self) -> Result<Option<String>, ApiError> {
        Err(unavailable())
    }
    fn save_subtitle(&self, _: SubtitleFormat) -> Result<Option<String>, ApiError> {
        Err(unavailable())
    }
}
fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Native path dialogs are unavailable; enter a path as text",
    )
}
