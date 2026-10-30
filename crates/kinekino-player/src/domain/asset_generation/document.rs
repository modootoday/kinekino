//! Editable asset plans. Saved values never stand in for source/review admission.
use super::*;
use crate::domain::{clip_document::SourceClock, identifiers::valid_id};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_ASSET_FRAMES: usize = 4096;
pub const MAX_ASSET_LAYERS: usize = 64;
pub const MAX_ASSET_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetVideoPosition {
    pub source_sha256: String,
    pub stream_index: u32,
    pub pts: String,
    pub clock: SourceClock,
    pub rotation_degrees: u16,
}
impl AssetVideoPosition {
    fn origin(&self) -> AssetFrameOrigin {
        AssetFrameOrigin::ObservedVideo {
            source_sha256: self.source_sha256.clone(),
            stream_index: self.stream_index,
            pts: self.pts.clone(),
            clock: self.clock.clone(),
            rotation_degrees: self.rotation_degrees,
        }
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        self.origin().validate()
    }
    fn matches(&self, origin: &AssetFrameOrigin) -> bool {
        self.origin() == *origin
    }
    fn same_track(&self, other: &Self) -> bool {
        self.source_sha256 == other.source_sha256
            && self.stream_index == other.stream_index
            && self.clock == other.clock
            && self.rotation_degrees == other.rotation_degrees
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AssetFramePosition {
    StaticPng,
    Video { expected: AssetVideoPosition },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetFrameSlot {
    pub position: AssetFramePosition,
    /// A selection value, not proof that bytes or observations are available.
    pub selected_image: Option<AssetImagePin>,
}
impl AssetFrameSlot {
    pub fn validate(&self) -> Result<(), &'static str> {
        if let AssetFramePosition::Video { expected } = &self.position {
            expected.validate()?;
        }
        if let Some(image) = &self.selected_image {
            image.validate()?;
            if !match &self.position {
                AssetFramePosition::StaticPng => {
                    matches!(image.origin, AssetFrameOrigin::StaticPng)
                }
                AssetFramePosition::Video { expected } => expected.matches(&image.origin),
            } {
                return Err("Selected asset image differs from its source slot");
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetFrameInput {
    pub image: AssetImagePin,
    pub mask: AssetMaskPin,
    pub review: AssetMaskReview,
}
impl AssetFrameInput {
    pub fn request(
        &self,
        layer: LayerKind,
        transform: AssetTransform,
    ) -> Result<AssetFrameRequest, &'static str> {
        let request = AssetFrameRequest {
            schema_version: "asset-frame-build/1".into(),
            image: self.image.clone(),
            mask: self.mask.clone(),
            review: self.review.clone(),
            layer,
            transform,
        };
        request.validate()?;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum AssetFrameEntry {
    Available {
        frame_id: String,
        input: Box<AssetFrameInput>,
    },
    Missing {
        frame_id: String,
        reason: String,
    },
}
impl AssetFrameEntry {
    pub fn frame_id(&self) -> &str {
        match self {
            Self::Available { frame_id, .. } | Self::Missing { frame_id, .. } => frame_id,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetOrderBasis {
    SingleFrame,
    DeclaredFrameOrder,
    SourceVideoClock,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetLayer {
    pub layer_id: String,
    pub kind: LayerKind,
    pub visible: bool,
    pub locked: bool,
    pub transform: AssetTransform,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetDocument {
    pub schema_version: String,
    pub document_id: String,
    pub revision: DecimalU64,
    /// Retained when a frame is removed, preventing later reuse with another source.
    pub sources: BTreeMap<String, AssetFrameSlot>,
    pub order_basis: AssetOrderBasis,
    pub frames: Vec<AssetFrameEntry>,
    pub layers: Vec<AssetLayer>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetFrameSelectionError {
    InvalidDocument,
    MissingFrame,
    MissingLayer,
    HiddenLayer,
}
impl AssetDocument {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != "asset-project/1"
            || !valid_id(&self.document_id)
            || self.revision.0 == 0
            || self.sources.len() > MAX_ASSET_FRAMES
            || self.frames.len() > MAX_ASSET_FRAMES
            || self.layers.is_empty()
            || self.layers.len() > MAX_ASSET_LAYERS
        {
            return Err("Invalid asset project identity, revision or counts");
        }
        for (id, slot) in &self.sources {
            if !valid_id(id) {
                return Err("Invalid asset source slot identity");
            }
            slot.validate()?;
        }
        let mut layers = BTreeSet::new();
        for layer in &self.layers {
            if !valid_id(&layer.layer_id) || !layers.insert(&layer.layer_id) {
                return Err("Invalid or duplicate asset layer identity");
            }
            layer.transform.validate()?;
        }
        if self.order_basis == AssetOrderBasis::SingleFrame && self.frames.len() > 1 {
            return Err("Single-frame asset project has multiple frames");
        }
        let mut frames = BTreeSet::new();
        let mut prior: Option<&AssetVideoPosition> = None;
        for frame in &self.frames {
            let id = frame.frame_id();
            if !valid_id(id) || !frames.insert(id) {
                return Err("Invalid or duplicate asset frame identity");
            }
            let slot = self
                .sources
                .get(id)
                .ok_or("Asset frame has no retained source slot")?;
            match frame {
                AssetFrameEntry::Available { input, .. } => {
                    if slot.selected_image.as_ref() != Some(&input.image) {
                        return Err("Available asset frame differs from selected source");
                    }
                    for layer in &self.layers {
                        input.request(layer.kind, layer.transform)?;
                    }
                }
                AssetFrameEntry::Missing { reason, .. } => {
                    if reason.trim().is_empty() || reason.len() > 2048 || reason.contains('\0') {
                        return Err("Missing asset frame requires bounded reason");
                    }
                }
            }
            if self.order_basis == AssetOrderBasis::SourceVideoClock {
                let AssetFramePosition::Video { expected } = &slot.position else {
                    return Err("Source-clock sequence requires explicit video positions");
                };
                if let Some(previous) = prior
                    && (!previous.same_track(expected)
                        || previous
                            .pts
                            .parse::<i64>()
                            .map_err(|_| "Invalid prior source PTS")?
                            >= expected
                                .pts
                                .parse::<i64>()
                                .map_err(|_| "Invalid source PTS")?)
                {
                    return Err(
                        "Asset source-clock frames must keep one track and increasing integer PTS",
                    );
                }
                prior = Some(expected);
            }
        }
        if serde_json::to_vec(self)
            .map_err(|_| "Invalid asset document encoding")?
            .len()
            > MAX_ASSET_DOCUMENT_BYTES
        {
            return Err("Asset document byte limit exceeded");
        }
        Ok(())
    }
    pub fn frame_request(
        &self,
        frame_id: &str,
        layer_id: &str,
    ) -> Result<AssetFrameRequest, AssetFrameSelectionError> {
        self.validate()
            .map_err(|_| AssetFrameSelectionError::InvalidDocument)?;
        let layer = self
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .ok_or(AssetFrameSelectionError::MissingLayer)?;
        if !layer.visible {
            return Err(AssetFrameSelectionError::HiddenLayer);
        }
        let frame = self
            .frames
            .iter()
            .find(|frame| frame.frame_id() == frame_id)
            .ok_or(AssetFrameSelectionError::MissingFrame)?;
        match frame {
            AssetFrameEntry::Available { input, .. } => input
                .request(layer.kind, layer.transform)
                .map_err(|_| AssetFrameSelectionError::InvalidDocument),
            AssetFrameEntry::Missing { .. } => Err(AssetFrameSelectionError::MissingFrame),
        }
    }
}
