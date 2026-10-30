//! Fixed administrative files, raw SHA identity, retained ordinary-Disk ancestor/file guards.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        clip_render::presets::*,
        review_source::MutationControl,
    },
    domain::{
        api_values::DecimalU64,
        clip_document::ArtifactPin,
        identifiers::{valid_id, valid_sha256},
    },
    infrastructure::immutable_storage_paths as paths,
};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs::File, io::Read, path::PathBuf};
pub struct FixedClipPresets {
    files: BTreeMap<String, PathBuf>,
}
struct FileLease {
    definition: ClipPresetDefinition,
    pin: ArtifactPin,
    bytes: u64,
    _guards: Vec<File>,
}
impl ClipPresetLease for FileLease {
    fn definition(&self) -> &ClipPresetDefinition {
        &self.definition
    }
    fn pin(&self) -> &ArtifactPin {
        &self.pin
    }
    fn bytes(&self) -> u64 {
        self.bytes
    }
}
fn error(code: ErrorCode, message: &str) -> ApiError {
    ApiError::new(code, message)
}
impl FixedClipPresets {
    pub fn open(files: BTreeMap<String, PathBuf>) -> Result<Self, ApiError> {
        let builtin = BuiltinClipPresets::entries()?;
        if files.len() > 64
            || files.iter().any(|(id, path)| {
                !valid_id(id)
                    || !path.is_absolute()
                    || builtin
                        .iter()
                        .any(|entry| entry.preset.catalog.artifact_id == *id)
            })
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Bounded fixed local preset paths required; application IDs cannot be shadowed",
            ));
        }
        Ok(Self { files })
    }
    pub fn catalog_binding_sha256(&self) -> Result<String, ApiError> {
        let bytes = serde_json::to_vec(&self.files)
            .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid preset catalog binding"))?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }
    fn read(
        &self,
        id: &str,
        expected: Option<&ArtifactPin>,
        control: &dyn MutationControl,
    ) -> Result<FileLease, ApiError> {
        control.checkpoint()?;
        let path = self.files.get(id).ok_or_else(|| {
            error(
                ErrorCode::NeedsInput,
                "Preset ID is absent from the fixed host catalog",
            )
        })?;
        let (parent, mut guards) = paths::root(
            path.parent()
                .ok_or_else(|| error(ErrorCode::InvalidArgument, "Preset parent required"))?,
        )?;
        let file = paths::file(
            &parent
                .join(path.file_name().ok_or_else(|| {
                    error(ErrorCode::InvalidArgument, "Preset filename required")
                })?),
        )?;
        if file
            .metadata()
            .map_err(kinekino_core::file_publication::io_error)?
            .len()
            > MAX_PRESET_BYTES as u64
        {
            return Err(error(
                ErrorCode::ResourceExhausted,
                "Preset definition exceeds byte limit",
            ));
        }
        let mut bytes = Vec::new();
        (&file)
            .take(MAX_PRESET_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(kinekino_core::file_publication::io_error)?;
        control.checkpoint()?;
        if bytes.len() > MAX_PRESET_BYTES {
            return Err(error(
                ErrorCode::ResourceExhausted,
                "Preset definition exceeds byte limit",
            ));
        }
        let sha256 = format!("{:x}", Sha256::digest(&bytes));
        if expected.is_some_and(|pin| pin.sha256 != sha256) {
            return Err(error(
                ErrorCode::SourceExpired,
                "Preset raw bytes differ from selected publication",
            ));
        }
        let definition: ClipPresetDefinition = serde_json::from_slice(&bytes).map_err(|_| {
            error(
                ErrorCode::InvalidArgument,
                "Strict clip-render-preset/1 JSON required",
            )
        })?;
        definition.validate()?;
        if definition.preset_id != id
            || expected
                .is_some_and(|pin| pin.artifact_id != id || pin.revision != definition.revision)
        {
            return Err(error(
                ErrorCode::SourceExpired,
                "Preset definition identity differs from selected publication",
            ));
        }
        if definition.origin != ClipPresetOrigin::HostManual {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Local preset files must declare host_manual; application origin belongs to built-in definitions",
            ));
        }
        guards.push(file);
        let pin = ArtifactPin {
            artifact_id: id.into(),
            revision: definition.revision,
            sha256,
        };
        Ok(FileLease {
            definition,
            pin,
            bytes: bytes.len() as u64,
            _guards: guards,
        })
    }
}
impl ClipPresetAdmission for FixedClipPresets {
    fn open_preset(
        &self,
        pin: &ArtifactPin,
        control: &dyn MutationControl,
    ) -> Result<Box<dyn ClipPresetLease>, ApiError> {
        control.checkpoint()?;
        if !valid_id(&pin.artifact_id) || pin.revision.0 == 0 || !valid_sha256(&pin.sha256) {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Immutable preset pin required",
            ));
        }
        if self.files.contains_key(&pin.artifact_id) {
            Ok(Box::new(self.read(&pin.artifact_id, Some(pin), control)?))
        } else {
            BuiltinClipPresets.open_preset(pin, control)
        }
    }
}
struct Inspect;
impl MutationControl for Inspect {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
impl ClipPresetCatalogPort for FixedClipPresets {
    fn inspect_presets(&self) -> Result<ClipPresetCatalog, ApiError> {
        let mut catalog = BuiltinClipPresets.inspect_presets()?;
        for id in self.files.keys() {
            let lease = self.read(id, None, &Inspect)?;
            catalog.presets.push(ClipPresetEntry {
                preset: lease.definition.output_preset(lease.pin.clone()),
                definition: lease.definition.clone(),
                definition_bytes: DecimalU64(lease.bytes),
                platform_requirements_verified: false,
            });
        }
        Ok(catalog)
    }
}
#[cfg(test)]
mod tests;
