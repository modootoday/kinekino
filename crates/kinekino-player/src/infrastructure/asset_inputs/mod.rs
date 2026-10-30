//! Fixed host catalogs only. Requests select immutable pins, never arbitrary paths.
mod png_samples;
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        asset_generation::*,
        review_source::MutationControl,
    },
    domain::{asset_generation::*, frame_capture::MAX_FRAME_PNG_BYTES},
    infrastructure::{immutable_storage_paths as paths, snapshot_export},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedAssetImage {
    pub pin: AssetImagePin,
    pub path: PathBuf,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedAssetMask {
    pub pin: AssetMaskPin,
    pub path: PathBuf,
}
pub struct FixedAssetInputs {
    images: BTreeMap<String, FixedAssetImage>,
    masks: BTreeMap<String, FixedAssetMask>,
}
pub struct NativeAssetImage {
    original_png: Vec<u8>,
    pin: AssetImagePin,
    rgba: Vec<u8>,
    _guards: Vec<File>,
}
pub struct NativeAssetMask {
    original_png: Vec<u8>,
    pin: AssetMaskPin,
    coverage: Vec<u8>,
    _guards: Vec<File>,
}
pub struct UnavailableAssetReview {
    record: AssetBlobPin,
    mask: AssetMaskPin,
}
impl AssetImageLease for NativeAssetImage {
    fn original_png(&self) -> Option<&[u8]> {
        Some(&self.original_png)
    }
    fn pin(&self) -> &AssetImagePin {
        &self.pin
    }
    fn rgba(&self) -> &[u8] {
        &self.rgba
    }
}
impl AssetMaskLease for NativeAssetMask {
    fn original_png(&self) -> Option<&[u8]> {
        Some(&self.original_png)
    }
    fn pin(&self) -> &AssetMaskPin {
        &self.pin
    }
    fn coverage(&self) -> &[u8] {
        &self.coverage
    }
}
impl AssetReviewLease for UnavailableAssetReview {
    fn record(&self) -> &AssetBlobPin {
        &self.record
    }
    fn mask(&self) -> &AssetMaskPin {
        &self.mask
    }
    fn usable(&self) -> bool {
        false
    }
}
fn changed() -> ApiError {
    ApiError::new(
        ErrorCode::SourceExpired,
        "Fixed asset input differs from its original pin",
    )
}
fn unsupported(message: &str) -> ApiError {
    ApiError::new(ErrorCode::Unsupported, message)
}
fn read_original(
    path: &Path,
    pin: &AssetBlobPin,
    control: &dyn MutationControl,
) -> Result<(Vec<u8>, Vec<File>), ApiError> {
    control.checkpoint()?;
    let parent = path.parent().ok_or_else(changed)?;
    let (_, mut guards) = paths::root(parent)?;
    let mut file = paths::file(path)?;
    if file
        .metadata()
        .map_err(kinekino_core::file_publication::io_error)?
        .len()
        > MAX_FRAME_PNG_BYTES
    {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Asset PNG byte limit exceeded",
        ));
    }
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 65536];
    loop {
        control.checkpoint()?;
        let count = file
            .read(&mut chunk)
            .map_err(kinekino_core::file_publication::io_error)?;
        if count == 0 {
            break;
        }
        if bytes.len() as u64 + count as u64 > MAX_FRAME_PNG_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Asset PNG byte limit exceeded",
            ));
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    if format!("{:x}", Sha256::digest(&bytes)) != pin.sha256 {
        return Err(changed());
    }
    control.checkpoint()?;
    guards.push(file);
    Ok((bytes, guards))
}
impl FixedAssetInputs {
    pub fn new(
        images: BTreeMap<String, FixedAssetImage>,
        masks: BTreeMap<String, FixedAssetMask>,
    ) -> Result<Self, ApiError> {
        if images.is_empty() || images.len() > 4096 || masks.len() > 4096 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Asset catalog counts must be bounded and images nonempty",
            ));
        }
        for (id, image) in &images {
            image
                .pin
                .validate()
                .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
            if id != &image.pin.blob.artifact_id || !image.path.is_absolute() {
                return Err(changed());
            }
        }
        for (id, mask) in &masks {
            mask.pin
                .validate()
                .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
            if id != &mask.pin.blob.artifact_id
                || !mask.path.is_absolute()
                || images
                    .get(&mask.pin.source.blob.artifact_id)
                    .is_none_or(|image| image.pin != mask.pin.source)
            {
                return Err(changed());
            }
        }
        Ok(Self { images, masks })
    }
}
impl AssetInputAdmission for FixedAssetInputs {
    type Image = NativeAssetImage;
    type Mask = NativeAssetMask;
    type Review = UnavailableAssetReview;
    fn open_image(
        &self,
        pin: &AssetImagePin,
        control: &dyn MutationControl,
    ) -> Result<Self::Image, ApiError> {
        if !matches!(pin.origin, AssetFrameOrigin::StaticPng) {
            return Err(unsupported(
                "Observed video assets require an original frame-observation resolver",
            ));
        }
        let entry = self
            .images
            .get(&pin.blob.artifact_id)
            .filter(|entry| entry.pin == *pin)
            .ok_or_else(changed)?;
        let (bytes, guards) = read_original(&entry.path, &pin.blob, control)?;
        let rgba = png_samples::image(&bytes, pin.width, pin.height, control)?;
        Ok(NativeAssetImage {
            original_png: bytes,
            pin: entry.pin.clone(),
            rgba,
            _guards: guards,
        })
    }
    fn open_mask(
        &self,
        pin: &AssetMaskPin,
        control: &dyn MutationControl,
    ) -> Result<Self::Mask, ApiError> {
        let entry = self
            .masks
            .get(&pin.blob.artifact_id)
            .filter(|entry| entry.pin == *pin)
            .ok_or_else(changed)?;
        let (bytes, guards) = read_original(&entry.path, &pin.blob, control)?;
        let coverage = png_samples::mask(
            &bytes,
            pin.source.width,
            pin.source.height,
            pin.encoding,
            control,
        )?;
        Ok(NativeAssetMask {
            original_png: bytes,
            pin: entry.pin.clone(),
            coverage,
            _guards: guards,
        })
    }
    fn open_review(
        &self,
        _record: &AssetBlobPin,
        _mask: &AssetMaskPin,
        control: &dyn MutationControl,
    ) -> Result<Self::Review, ApiError> {
        control.checkpoint()?;
        Err(unsupported(
            "Authenticated mask review resolver is not configured; reviewed JSON is not admission",
        ))
    }
}
