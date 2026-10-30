use crate::{
    application::api::{ApiError, ErrorCode},
    infrastructure::meari::product_styles::prepare_product_styles,
};
use meari::{
    application::assets::VerifiedAssets,
    domain::{
        assets::AssetDescriptor,
        build::{BuildPath, ContentHash, InputRole},
    },
};
use std::collections::BTreeMap;

#[derive(serde::Serialize)]
pub(super) struct FontMetadata {
    pub logical_id: &'static str,
    pub sha256: &'static str,
    pub release: &'static str,
    pub license_hint: &'static str,
    pub manifest_license: String,
}
pub(super) fn font_metadata(assets: &VerifiedAssets) -> Result<Vec<FontMetadata>, ApiError> {
    crate::assembly::card_export::product_template::png1_proposal()
        .common_fonts
        .into_iter()
        .map(|pin| {
            let path = BuildPath::try_from(pin.source_hint.to_owned()).map_err(error)?;
            let descriptor = assets
                .manifest()
                .files
                .get(&path)
                .ok_or_else(|| error("Required common font absent"))?;
            if descriptor.content.as_str() != pin.sha256 {
                return Err(error("Common font pin differs from the proposal"));
            }
            let license = descriptor
                .license
                .clone()
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| error("Common font license absent"))?;
            Ok(FontMetadata {
                logical_id: pin.logical_id,
                sha256: pin.sha256,
                release: pin.release,
                license_hint: pin.license_hint,
                manifest_license: license,
            })
        })
        .collect()
}
pub(super) const FRAME: &str = "card-review/frame.png";
pub(super) fn prepare(
    frame: Option<Vec<u8>>,
) -> Result<(VerifiedAssets, serde_json::Value), ApiError> {
    let common = prepare_product_styles().map_err(error)?;
    let mut manifest = common.assets.manifest().clone();
    let mut blobs = BTreeMap::new();
    for descriptor in manifest.files.values() {
        let path = descriptor.output_path().map_err(error)?;
        let (_, bytes) = common
            .assets
            .resource(&path)
            .ok_or_else(|| error("Common asset resource missing"))?;
        blobs.insert(descriptor.content.clone(), bytes.to_vec());
    }
    if let Some(frame) = frame {
        let hash = ContentHash::of(&frame);
        manifest.files.insert(
            BuildPath::try_from(FRAME.to_owned()).map_err(error)?,
            AssetDescriptor {
                role: InputRole::Image,
                content: hash.clone(),
                length: frame.len() as u64,
                media_type: "image/png".into(),
                license: None,
            },
        );
        blobs.insert(hash, frame);
    }
    let licenses = serde_json::json!(common.font_licenses);
    Ok((
        VerifiedAssets::new(manifest, blobs).map_err(error)?,
        licenses,
    ))
}
pub(super) fn error(message: impl std::fmt::Display) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, message.to_string())
}
