//! Immutable input values; validation alone never authenticates a source or review.
use super::*;
use crate::domain::{
    clip_document::SourceClock,
    identifiers::{valid_id, valid_sha256},
};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetBlobPin {
    pub artifact_id: String,
    pub revision: DecimalU64,
    pub sha256: String,
}
impl AssetBlobPin {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !valid_id(&self.artifact_id) || self.revision.0 == 0 || !valid_sha256(&self.sha256) {
            Err("Invalid asset blob pin")
        } else {
            Ok(())
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AssetFrameOrigin {
    StaticPng,
    ObservedVideo {
        source_sha256: String,
        stream_index: u32,
        pts: String,
        clock: SourceClock,
        rotation_degrees: u16,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetImagePin {
    pub blob: AssetBlobPin,
    pub width: u32,
    pub height: u32,
    pub origin: AssetFrameOrigin,
    pub declared_color: AssetColor,
}
impl AssetFrameOrigin {
    pub fn validate(&self) -> Result<(), &'static str> {
        if let AssetFrameOrigin::ObservedVideo {
            source_sha256,
            pts,
            clock,
            rotation_degrees,
            ..
        } = self
        {
            let value = pts.parse::<i64>().map_err(|_| "Invalid asset source PTS")?;
            clock.transform()?;
            if value.to_string() != *pts
                || !valid_sha256(source_sha256)
                || !valid_sha256(&clock.observation_sha256)
                || !matches!(rotation_degrees, 0 | 90 | 180 | 270)
            {
                return Err("Invalid asset observed frame binding");
            }
        }
        Ok(())
    }
}
impl AssetImagePin {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.blob.validate()?;
        pixel_count(self.width, self.height)?;
        self.origin.validate()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetMaskPin {
    pub blob: AssetBlobPin,
    pub source: AssetImagePin,
    pub encoding: AssetMaskEncoding,
    pub origin: AssetMaskOrigin,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetMaskEncoding {
    Gray8Coverage,
    Rgba8Alpha,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetMaskOrigin {
    ImportedUnverified,
}
impl AssetMaskPin {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.blob.validate()?;
        self.source.validate()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AssetMaskReview {
    /// Must resolve an authenticated usable user record for the exact mask/source.
    Reference { record: AssetBlobPin },
    /// This is an explicit warning acknowledgement, never a quality approval.
    UnreviewedAcknowledged { mask: AssetBlobPin },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetFrameRequest {
    pub schema_version: String,
    pub image: AssetImagePin,
    pub mask: AssetMaskPin,
    pub review: AssetMaskReview,
    pub layer: LayerKind,
    pub transform: AssetTransform,
}
impl AssetFrameRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != "asset-frame-build/1" {
            return Err("Unsupported asset frame schema");
        }
        self.image.validate()?;
        self.mask.validate()?;
        self.transform.validate()?;
        if self.mask.source != self.image || self.image.declared_color != self.transform.color {
            return Err("Mask or color differs from original image binding");
        }
        match &self.review {
            AssetMaskReview::Reference { record } => record.validate()?,
            AssetMaskReview::UnreviewedAcknowledged { mask } => {
                mask.validate()?;
                if mask != &self.mask.blob {
                    return Err("Unreviewed acknowledgement differs from exact mask revision");
                }
            }
        }
        Ok(())
    }
}
