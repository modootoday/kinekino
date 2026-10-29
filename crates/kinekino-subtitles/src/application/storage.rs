use kinekino_media_contracts::errors::ApiError;

pub const MAX_SUBTITLE_BYTES: usize = 8 * 1024 * 1024;

pub trait SubtitleStorage {
    fn available(&self) -> bool {
        true
    }
    fn read_utf8(&self, path: &str) -> Result<String, ApiError>;
    fn publish_new(&self, path: &str, contents: &str) -> Result<(), ApiError>;
}

#[derive(Default)]
pub struct UnavailableStorage;
impl SubtitleStorage for UnavailableStorage {
    fn available(&self) -> bool {
        false
    }
    fn read_utf8(&self, _: &str) -> Result<String, ApiError> {
        Err(unavailable())
    }
    fn publish_new(&self, _: &str, _: &str) -> Result<(), ApiError> {
        Err(unavailable())
    }
}
fn unavailable() -> ApiError {
    ApiError::new(
        kinekino_media_contracts::errors::ErrorCode::Unsupported,
        "Subtitle file storage adapter is unavailable",
    )
}
