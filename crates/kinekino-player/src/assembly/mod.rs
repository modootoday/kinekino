//! Product composition adapters over the common SDK. No duplicate domain policy.
#[cfg(all(windows, feature = "card-review-export"))]
pub mod card_export;
pub mod meari;
pub mod native_playback;
pub mod review_playback;
