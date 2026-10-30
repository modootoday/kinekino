//! Bounded raw ISO BMFF header classification; no codec/track/decode claim.
use crate::application::{
    api::{ApiError, ErrorCode},
    review_playback::{ReviewContentClass, ReviewContentClassifier},
};
mod matroska;
pub use matroska::MatroskaReviewClassifier;
pub struct LocalReviewContentClassifier;
impl ReviewContentClassifier for LocalReviewContentClassifier {
    fn classify(&self, prefix: &[u8]) -> Result<ReviewContentClass, ApiError> {
        if prefix.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]) {
            MatroskaReviewClassifier.classify(prefix)
        } else {
            IsoBmffReviewClassifier.classify(prefix)
        }
    }
}
pub struct IsoBmffReviewClassifier;
impl ReviewContentClassifier for IsoBmffReviewClassifier {
    fn classify(&self, prefix: &[u8]) -> Result<ReviewContentClass, ApiError> {
        if prefix.len() < 16 || &prefix[4..8] != b"ftyp" {
            return Err(unsupported());
        }
        let size = u32::from_be_bytes(prefix[..4].try_into().map_err(|_| unsupported())?) as usize;
        if size < 16 || size > prefix.len() || !(size - 16).is_multiple_of(4) {
            return Err(unsupported());
        }
        let known = |brand: &[u8]| {
            matches!(
                brand,
                b"isom"
                    | b"iso2"
                    | b"iso3"
                    | b"iso4"
                    | b"iso5"
                    | b"iso6"
                    | b"mp41"
                    | b"mp42"
                    | b"avc1"
            )
        };
        if !known(&prefix[8..12])
            || !prefix[16..size]
                .as_chunks::<4>()
                .0
                .iter()
                .any(|brand| known(brand))
        {
            return Err(unsupported());
        }
        Ok(ReviewContentClass::IsoBmffHeader)
    }
}
fn unsupported() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Raw source does not match the supported ISO BMFF review header contract",
    )
}
