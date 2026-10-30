//! Fixed local subtitle/style catalog plus the existing verified Meari font assets.
#[cfg(feature = "native-proposal-window")]
use crate::domain::api_values::DecimalU64;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        clip_render::subtitles::*,
        review_source::MutationControl,
    },
    domain::{
        clip_document::{ArtifactPin, SourceBoundArtifact},
        identifiers::{valid_id, valid_sha256},
    },
    infrastructure::{content_digest, immutable_storage_paths as paths},
};
use meari::application::assets::VerifiedAssets;
#[cfg(feature = "native-proposal-window")]
use meari::domain::build::{BuildPath, ContentHash, InputRole};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};
pub struct SubtitleAssetFiles {
    documents: BTreeMap<String, PathBuf>,
    styles: BTreeMap<String, PathBuf>,
}
pub struct NativeSubtitleDocument {
    value: ClipSubtitleDocument,
    _guards: Vec<File>,
}
pub struct NativeSubtitleStyle {
    value: ClipSubtitleStyle,
    fonts: Vec<ClipSubtitleFontPin>,
    assets: VerifiedAssets,
    _guards: Vec<File>,
}
impl ClipSubtitleDocumentLease for NativeSubtitleDocument {
    fn document(&self) -> &ClipSubtitleDocument {
        &self.value
    }
}
impl ClipSubtitleStyleLease for NativeSubtitleStyle {
    fn style(&self) -> &ClipSubtitleStyle {
        &self.value
    }
    fn fonts(&self) -> &[ClipSubtitleFontPin] {
        &self.fonts
    }
}
impl NativeSubtitleStyle {
    pub fn assets(&self) -> &VerifiedAssets {
        &self.assets
    }
}
fn error(code: ErrorCode, message: &str) -> ApiError {
    ApiError::new(code, message)
}
impl SubtitleAssetFiles {
    pub fn catalog_binding_sha256(&self) -> Result<String, ApiError> {
        use sha2::{Digest, Sha256};
        let bytes = serde_json::to_vec(&(&self.documents, &self.styles))
            .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid fixed subtitle catalog"))?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }
    pub fn require_selection_available(
        &self,
        selection: &crate::domain::clip_document::SubtitleSelection,
    ) -> Result<(), ApiError> {
        if !self
            .documents
            .contains_key(&selection.document.artifact.artifact_id)
            || !self.styles.contains_key(&selection.style.artifact_id)
        {
            return Err(error(
                ErrorCode::NeedsInput,
                "Selected subtitle document/style is absent from the fixed host catalog",
            ));
        }
        Ok(())
    }
    pub fn open(
        documents: BTreeMap<String, PathBuf>,
        styles: BTreeMap<String, PathBuf>,
    ) -> Result<Self, ApiError> {
        if documents.len() > 128
            || styles.len() > 128
            || documents
                .iter()
                .chain(&styles)
                .any(|(id, path)| !valid_id(id) || !path.is_absolute())
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Bounded fixed subtitle/style catalogs require valid IDs and absolute local paths",
            ));
        }
        Ok(Self { documents, styles })
    }
}
fn read<T: serde::de::DeserializeOwned>(
    catalog: &BTreeMap<String, PathBuf>,
    pin: &ArtifactPin,
    control: &dyn MutationControl,
) -> Result<(T, Vec<File>), ApiError> {
    control.checkpoint()?;
    if !valid_id(&pin.artifact_id) || pin.revision.0 == 0 || !valid_sha256(&pin.sha256) {
        return Err(error(
            ErrorCode::InvalidArgument,
            "Immutable subtitle asset pin required",
        ));
    }
    let path = catalog.get(&pin.artifact_id).ok_or_else(|| {
        error(
            ErrorCode::NotFound,
            "Selected subtitle asset is absent from fixed catalog",
        )
    })?;
    let (parent, mut guards) = paths::root(
        path.parent()
            .ok_or_else(|| error(ErrorCode::InvalidArgument, "Subtitle asset parent required"))?,
    )?;
    let mut file = paths::file(&parent.join(path.file_name().ok_or_else(|| {
        error(
            ErrorCode::InvalidArgument,
            "Subtitle asset filename required",
        )
    })?))?;
    let (_, sha) =
        content_digest::sha256_with_checkpoint(&mut file, 1024 * 1024, || control.checkpoint())
            .map_err(|error| error.into_api_error())?;
    if sha != pin.sha256 {
        return Err(error(
            ErrorCode::SourceExpired,
            "Subtitle asset bytes differ from selected immutable pin",
        ));
    }
    file.seek(SeekFrom::Start(0))
        .map_err(kinekino_core::file_publication::io_error)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(kinekino_core::file_publication::io_error)?;
    control.checkpoint()?;
    if bytes.len() > 1024 * 1024 {
        return Err(error(
            ErrorCode::ResourceExhausted,
            "Subtitle asset byte bound exceeded",
        ));
    }
    let value = serde_json::from_slice(&bytes).map_err(|_| {
        error(
            ErrorCode::InvalidArgument,
            "Expected strict subtitle document/style JSON",
        )
    })?;
    guards.push(file);
    Ok((value, guards))
}
impl ClipSubtitleAdmission for SubtitleAssetFiles {
    type Document = NativeSubtitleDocument;
    type Style = NativeSubtitleStyle;
    fn open_document(
        &self,
        expected: &SourceBoundArtifact,
        control: &dyn MutationControl,
    ) -> Result<Self::Document, ApiError> {
        let (value, guards) = read(&self.documents, &expected.artifact, control)?;
        Ok(NativeSubtitleDocument {
            value,
            _guards: guards,
        })
    }
    #[cfg(not(feature = "native-proposal-window"))]
    fn open_style(
        &self,
        _expected: &ArtifactPin,
        control: &dyn MutationControl,
    ) -> Result<Self::Style, ApiError> {
        control.checkpoint()?;
        Err(error(
            ErrorCode::Unsupported,
            "Common-font subtitle rendering requires native-proposal-window",
        ))
    }
    #[cfg(feature = "native-proposal-window")]
    fn open_style(
        &self,
        expected: &ArtifactPin,
        control: &dyn MutationControl,
    ) -> Result<Self::Style, ApiError> {
        let (value, guards): (ClipSubtitleStyle, _) = read(&self.styles, expected, control)?;
        validate_subtitle_style(&value)?;
        let common = crate::infrastructure::meari::product_styles::prepare_product_styles()
            .map_err(|_| {
                error(
                    ErrorCode::EngineFailed,
                    "Verified common font assets unavailable",
                )
            })?;
        // Release identity of the pinned Noto download set (fonts are never compiled in).
        let release = ContentHash::of(meari::domain::design::fonts::REVISION.as_bytes())
            .as_str()
            .to_owned();
        let mut fonts = Vec::new();
        for selected in &value.fonts {
            control.checkpoint()?;
            let key = BuildPath::try_from(selected.asset_path.clone())
                .map_err(|_| error(ErrorCode::InvalidArgument, "Expected common font asset ID"))?;
            let descriptor = common.assets.manifest().files.get(&key).ok_or_else(|| {
                error(
                    ErrorCode::NotFound,
                    "Selected font is not in the product common asset contract",
                )
            })?;
            if descriptor.role != InputRole::Font
                || descriptor.content.as_str() != selected.sha256
                || descriptor
                    .license
                    .as_ref()
                    .is_none_or(|license| license.trim().is_empty())
            {
                return Err(error(
                    ErrorCode::SourceExpired,
                    "Selected common font role, SHA or license differs",
                ));
            }
            let output = descriptor
                .output_path()
                .map_err(|_| error(ErrorCode::EngineFailed, "Invalid common font output path"))?;
            let (_, bytes) = common.assets.resource(&output).ok_or_else(|| {
                error(
                    ErrorCode::SourceExpired,
                    "Pinned common font bytes unavailable",
                )
            })?;
            if bytes.len() as u64 != descriptor.length
                || ContentHash::of(bytes).as_str() != selected.sha256
            {
                return Err(error(
                    ErrorCode::SourceExpired,
                    "Common font bytes differ from manifest",
                ));
            }
            fonts.push(ClipSubtitleFontPin {
                asset_path: selected.asset_path.clone(),
                sha256: selected.sha256.clone(),
                bytes: DecimalU64(bytes.len() as u64),
                license: descriptor.license.clone().unwrap(),
                release_manifest_sha256: release.clone(),
            });
        }
        Ok(NativeSubtitleStyle {
            value,
            fonts,
            assets: common.assets,
            _guards: guards,
        })
    }
}
#[cfg(all(test, feature = "native-proposal-window"))]
mod tests;
#[cfg(all(test, not(feature = "native-proposal-window")))]
mod unavailable_tests {
    use super::*;
    struct Control;
    impl MutationControl for Control {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    #[test]
    fn missing_cpu_feature_rejects_before_style_io() {
        let files = SubtitleAssetFiles::open(BTreeMap::new(), BTreeMap::new()).unwrap();
        let pin = ArtifactPin {
            artifact_id: "absent".into(),
            revision: crate::domain::api_values::DecimalU64(1),
            sha256: "0".repeat(64),
        };
        assert_eq!(
            files.open_style(&pin, &Control).err().unwrap().code,
            ErrorCode::Unsupported
        );
    }
}
