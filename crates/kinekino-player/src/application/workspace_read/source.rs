use crate::domain::api_values::DecimalU64;
use serde::Serialize;
#[derive(Debug, Clone, Serialize)]
pub struct SourceSnapshot {
    pub source_id: String,
    pub title: String,
    pub duration_us: Option<DecimalU64>,
}
#[derive(Debug, Clone, Serialize)]
pub struct AssetReference {
    pub asset_id: String,
    pub path: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct PreviewSnapshot {
    pub position_us: DecimalU64,
    pub duration_us: Option<DecimalU64>,
    pub frame: Option<AssetReference>,
}
impl Default for PreviewSnapshot {
    fn default() -> Self {
        Self {
            position_us: DecimalU64(0),
            duration_us: None,
            frame: None,
        }
    }
}
