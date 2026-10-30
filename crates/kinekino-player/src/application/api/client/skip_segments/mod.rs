#[path = "ports.rs"]
mod ports;
use super::{ApplicationClient, review_call};
use crate::application::{
    api::{ApiError, RequestContext},
    skip_segments::{SkipSnapshot, host::*},
};
pub use ports::SkipSegmentsClient;
impl<T: ApplicationClient + ?Sized> SkipSegmentsClient for T {}
