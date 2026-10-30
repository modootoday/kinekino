use super::{AssetReference, PixelPoint, ReadOrigin, ReadState};
use crate::domain::api_values::DecimalU64;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskReviewState {
    Unreviewed,
    NeedsReview,
    Confirmed,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubjectMask {
    pub track_id: String,
    pub polygon_px: Option<Vec<PixelPoint>>,
    /// Full source-canvas alpha, not a cropped subject image.
    pub alpha_asset: Option<AssetReference>,
    pub review_state: MaskReviewState,
}

#[derive(Debug, Clone, Serialize)]
pub struct MaskSnapshot {
    pub width_px: u32,
    pub height_px: u32,
    pub source_asset: AssetReference,
    pub source_pts_us: DecimalU64,
    pub subjects: Vec<SubjectMask>,
    pub origin: ReadOrigin,
    pub state: ReadState,
}
