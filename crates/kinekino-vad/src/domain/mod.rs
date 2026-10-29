mod input;
mod profile;
mod result;
mod runtime_notices;
mod worker_observation;

pub use input::*;
pub use profile::*;
pub use result::*;
pub use runtime_notices::*;
pub use worker_observation::*;

use kinekino_media_contracts::errors::{ApiError, ErrorCode};
pub(crate) fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
