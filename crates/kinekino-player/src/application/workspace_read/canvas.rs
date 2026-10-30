use super::{AssetReference, ReadOrigin, ReadState};
use serde::Serialize;

/// Integer coordinates in the source canvas, independent of screen zoom or DPI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PixelPoint {
    pub x: i32,
    pub y: i32,
}

/// Source-canvas placement; width and height describe the placed asset size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PixelBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasViewMode {
    Original,
    Alpha,
    Composite,
}

#[derive(Debug, Clone, Serialize)]
pub struct LayerSnapshot {
    pub layer_id: String,
    pub label: String,
    pub asset: Option<AssetReference>,
    pub visible: bool,
    pub locked: bool,
    pub bounds_px: PixelBounds,
    /// Absolute source-canvas pixel coordinates, not relative to layer bounds.
    pub pivot_px: PixelPoint,
    pub state: ReadState,
}

/// Read-only composition. Layers are ordered back to front.
#[derive(Debug, Clone, Serialize)]
pub struct CanvasSnapshot {
    pub width_px: u32,
    pub height_px: u32,
    pub source_asset: Option<AssetReference>,
    pub layers: Vec<LayerSnapshot>,
    pub selected_layer_id: Option<String>,
    pub view_mode: CanvasViewMode,
    /// Initial presentation hint; changing local zoom does not mutate this data.
    pub zoom_percent: u16,
    pub origin: ReadOrigin,
    pub state: ReadState,
}
