//! Versioned output definitions. Admission proves a selected definition, never platform approval.
#[path = "presets/ports.rs"]
mod ports;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_source::MutationControl,
    },
    domain::{
        api_values::DecimalU64,
        clip_document::{
            ArtifactPin, AudioCodec, ClipDocument, ClipOutputPreset, Container, FrameRate,
            OutputColor, VideoCodec,
        },
        identifiers::{valid_id, valid_sha256},
    },
};
pub use ports::ClipPresetCatalogPort;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub const MAX_PRESET_BYTES: usize = 65536;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipPresetOrigin {
    ApplicationProposal,
    HostManual,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipPresetProfile {
    pub width: u32,
    pub height: u32,
    pub frame_rate: FrameRate,
    pub container: Container,
    pub video_codec: VideoCodec,
    pub audio_codec: Option<AudioCodec>,
    pub color: OutputColor,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipPresetDefinition {
    pub schema_version: String,
    pub preset_id: String,
    pub revision: DecimalU64,
    pub label: String,
    pub origin: ClipPresetOrigin,
    pub profile: ClipPresetProfile,
    pub max_duration_us: DecimalU64,
}
impl ClipPresetDefinition {
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.schema_version != "clip-render-preset/1"
            || !valid_id(&self.preset_id)
            || self.revision.0 == 0
            || self.label.trim().is_empty()
            || self.label.len() > 256
            || self.label.chars().any(char::is_control)
            || self.profile.width == 0
            || self.profile.height == 0
            || self.profile.width > 8192
            || self.profile.height > 8192
            || u64::from(self.profile.width) * u64::from(self.profile.height)
                > crate::domain::frame_capture::MAX_FRAME_PIXELS
            || self.max_duration_us.0 == 0
            || self.max_duration_us.0 > 180_000_000
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Invalid bounded clip preset definition",
            ));
        }
        self.profile
            .frame_rate
            .validate()
            .map_err(|m| error(ErrorCode::InvalidArgument, m))
    }
    /// Data for a project; a caller cannot reconstruct an admitted owner from these values.
    pub fn output_preset(&self, pin: ArtifactPin) -> ClipOutputPreset {
        ClipOutputPreset {
            catalog: pin,
            width: self.profile.width,
            height: self.profile.height,
            frame_rate: self.profile.frame_rate,
            container: self.profile.container,
            video_codec: self.profile.video_codec,
            audio_codec: self.profile.audio_codec,
            color: self.profile.color,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipPresetEntry {
    pub preset: ClipOutputPreset,
    pub definition: ClipPresetDefinition,
    pub definition_bytes: DecimalU64,
    pub platform_requirements_verified: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipPresetCatalog {
    pub schema_version: String,
    pub definitions_available: bool,
    pub engine_availability_verified: bool,
    pub presets: Vec<ClipPresetEntry>,
}
pub trait ClipPresetLease: Send + Sync {
    fn definition(&self) -> &ClipPresetDefinition;
    fn pin(&self) -> &ArtifactPin;
    fn bytes(&self) -> u64;
}
pub trait ClipPresetAdmission: Send + Sync {
    fn open_preset(
        &self,
        pin: &ArtifactPin,
        control: &dyn MutationControl,
    ) -> Result<Box<dyn ClipPresetLease>, ApiError>;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipPresetEvidence {
    pub pin: ArtifactPin,
    pub definition: ClipPresetDefinition,
    pub definition_bytes: DecimalU64,
    pub platform_requirements_verified: bool,
    pub quality_approved: bool,
}
/// Non-Serialize owner retains the original definition bytes/OS guards until encoding exits.
pub struct PreparedClipPreset {
    evidence: ClipPresetEvidence,
    _lease: Box<dyn ClipPresetLease>,
}
impl PreparedClipPreset {
    pub fn evidence(&self) -> &ClipPresetEvidence {
        &self.evidence
    }
}
pub fn prepare_clip_preset<P: ClipPresetAdmission + ?Sized>(
    document: &ClipDocument,
    port: &P,
    control: &dyn MutationControl,
) -> Result<PreparedClipPreset, ApiError> {
    control.checkpoint()?;
    document
        .validate()
        .map_err(|m| error(ErrorCode::InvalidArgument, m))?;
    let selected = &document.preset;
    if !valid_sha256(&selected.catalog.sha256) {
        return Err(error(
            ErrorCode::InvalidArgument,
            "Clip preset SHA256 required",
        ));
    }
    let lease = port.open_preset(&selected.catalog, control)?;
    control.checkpoint()?;
    let definition = lease.definition();
    definition.validate()?;
    if lease.pin() != &selected.catalog
        || definition.preset_id != selected.catalog.artifact_id
        || definition.revision != selected.catalog.revision
        || definition.output_preset(lease.pin().clone()) != *selected
        || lease.bytes() == 0
        || lease.bytes() > MAX_PRESET_BYTES as u64
    {
        return Err(error(
            ErrorCode::SourceExpired,
            "Selected clip preset pin or settings differ from the original definition",
        ));
    }
    let duration = document
        .segments
        .iter()
        .try_fold(0u64, |total, segment| {
            total.checked_add(segment.range.end_us.0 - segment.range.start_us.0)
        })
        .ok_or_else(|| {
            error(
                ErrorCode::ResourceExhausted,
                "Clip output duration overflow",
            )
        })?;
    if duration > definition.max_duration_us.0 {
        return Err(error(
            ErrorCode::NeedsInput,
            "Clip duration exceeds the selected preset definition",
        ));
    }
    control.checkpoint()?;
    let evidence = ClipPresetEvidence {
        pin: lease.pin().clone(),
        definition: definition.clone(),
        definition_bytes: DecimalU64(lease.bytes()),
        platform_requirements_verified: false,
        quality_approved: false,
    };
    Ok(PreparedClipPreset {
        evidence,
        _lease: lease,
    })
}
fn error(code: ErrorCode, message: &str) -> ApiError {
    ApiError::new(code, message)
}
#[derive(Default)]
pub struct BuiltinClipPresets;
struct BuiltinLease {
    definition: ClipPresetDefinition,
    pin: ArtifactPin,
    bytes: u64,
}
impl ClipPresetLease for BuiltinLease {
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
impl BuiltinClipPresets {
    pub fn entries() -> Result<Vec<ClipPresetEntry>, ApiError> {
        [true, false]
            .into_iter()
            .map(|audio| {
                let definition = ClipPresetDefinition {
                    schema_version: "clip-render-preset/1".into(),
                    preset_id: if audio {
                        "clip.portrait-1080p30-aac"
                    } else {
                        "clip.portrait-1080p30-silent"
                    }
                    .into(),
                    revision: DecimalU64(1),
                    label: if audio {
                        "세로 1080p · 30fps · AAC"
                    } else {
                        "세로 1080p · 30fps · 음성 없음"
                    }
                    .into(),
                    origin: ClipPresetOrigin::ApplicationProposal,
                    profile: ClipPresetProfile {
                        width: 1080,
                        height: 1920,
                        frame_rate: FrameRate {
                            numerator: 30,
                            denominator: 1,
                        },
                        container: Container::Mp4,
                        video_codec: VideoCodec::H264,
                        audio_codec: audio.then_some(AudioCodec::Aac),
                        color: OutputColor::SdrBt709,
                    },
                    max_duration_us: DecimalU64(60_000_000),
                };
                definition.validate()?;
                let bytes = serde_json::to_vec(&definition).map_err(|_| {
                    error(
                        ErrorCode::Internal,
                        "Cannot encode application preset definition",
                    )
                })?;
                let pin = ArtifactPin {
                    artifact_id: definition.preset_id.clone(),
                    revision: definition.revision,
                    sha256: format!("{:x}", Sha256::digest(&bytes)),
                };
                Ok(ClipPresetEntry {
                    preset: definition.output_preset(pin),
                    definition,
                    definition_bytes: DecimalU64(bytes.len() as u64),
                    platform_requirements_verified: false,
                })
            })
            .collect()
    }
}
impl ClipPresetAdmission for BuiltinClipPresets {
    fn open_preset(
        &self,
        pin: &ArtifactPin,
        control: &dyn MutationControl,
    ) -> Result<Box<dyn ClipPresetLease>, ApiError> {
        control.checkpoint()?;
        let entry = Self::entries()?
            .into_iter()
            .find(|entry| entry.preset.catalog.artifact_id == pin.artifact_id)
            .ok_or_else(|| {
                error(
                    ErrorCode::NeedsInput,
                    "Clip preset is absent from the configured catalog",
                )
            })?;
        if entry.preset.catalog != *pin {
            return Err(error(
                ErrorCode::SourceExpired,
                "Application preset publication differs from selected pin",
            ));
        }
        Ok(Box::new(BuiltinLease {
            definition: entry.definition,
            pin: entry.preset.catalog,
            bytes: entry.definition_bytes.0,
        }))
    }
}
impl ClipPresetCatalogPort for BuiltinClipPresets {
    fn inspect_presets(&self) -> Result<ClipPresetCatalog, ApiError> {
        Ok(ClipPresetCatalog {
            schema_version: "1".into(),
            definitions_available: true,
            engine_availability_verified: false,
            presets: Self::entries()?,
        })
    }
}
#[cfg(test)]
pub(crate) mod tests;
