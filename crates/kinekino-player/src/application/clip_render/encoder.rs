use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_source::MutationControl,
    },
    domain::{
        clip_document::{
            AudioCodec, AudioSelection, ClipDocument, ClipOutputPreset, Container, CutMode,
            VideoCodec,
        },
        identifiers::valid_sha256,
    },
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// Inventory observation only; not a rendering, playback or redistribution approval.
#[derive(Debug, Clone, Serialize)]
pub struct ClipEncoderInventory {
    pub tool_sha256: String,
    pub inventory_sha256: String,
    pub version_line: String,
    pub build_configuration_sha256: String,
    pub license_sha256: String,
    pub encoders: BTreeSet<String>,
    pub filters: BTreeSet<String>,
    pub muxers: BTreeSet<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ClipEncoderRequirements {
    pub video_encoder: String,
    pub audio_encoder: Option<String>,
    pub muxer: String,
    pub filters: BTreeSet<String>,
}
pub trait ClipEncoderLease: Send + Sync {
    fn inventory(&self) -> &ClipEncoderInventory;
}
pub trait ClipEncoderAdmission {
    type Lease: ClipEncoderLease;
    fn open_encoder(&self, control: &dyn MutationControl) -> Result<Self::Lease, ApiError>;
}
/// Original tool lease and the exact document digest must survive until encoder exit.
pub struct PreparedClipEncoder<L: ClipEncoderLease> {
    lease: L,
    document_sha256: String,
    preset: ClipOutputPreset,
    requirements: ClipEncoderRequirements,
    preset_owner: super::presets::PreparedClipPreset,
}
impl<L: ClipEncoderLease> PreparedClipEncoder<L> {
    pub fn lease(&self) -> &L {
        &self.lease
    }
    pub fn document_sha256(&self) -> &str {
        &self.document_sha256
    }
    pub fn preset(&self) -> &ClipOutputPreset {
        &self.preset
    }
    pub fn requirements(&self) -> &ClipEncoderRequirements {
        &self.requirements
    }
    pub fn preset_evidence(&self) -> &super::presets::ClipPresetEvidence {
        self.preset_owner.evidence()
    }
}
pub fn encoder_requirements(document: &ClipDocument) -> Result<ClipEncoderRequirements, ApiError> {
    document
        .validate()
        .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
    if document.cut_mode != CutMode::ExactReencode {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Fast keyframe cutting requires a separate verified execution path",
        ));
    }
    // Additional preset families remain explicit future renderer work, not silent fallback.
    if document.preset.container != Container::Mp4
        || document.preset.video_codec != VideoCodec::H264
        || document
            .preset
            .audio_codec
            .is_some_and(|codec| codec != AudioCodec::Aac)
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Initial clip encoder profile requires MP4/H264 and optional AAC",
        ));
    }
    if !document.preset.width.is_multiple_of(2) || !document.preset.height.is_multiple_of(2) {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "H264 yuv420p output requires even dimensions",
        ));
    }
    let mut filters: BTreeSet<String> = [
        "trim",
        "setpts",
        "crop",
        "scale",
        "pad",
        "format",
        "fps",
        "setsar",
        "setparams",
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    if document
        .segments
        .iter()
        .any(|segment| segment.crop_keys.len() > 1)
    {
        filters.extend(["split", "blend"].into_iter().map(String::from));
    }
    let audio = document
        .segments
        .iter()
        .any(|segment| matches!(segment.audio, AudioSelection::Selected { .. }));
    if audio {
        filters.extend(["atrim", "asetpts", "volume"].into_iter().map(String::from));
    }
    if document
        .segments
        .iter()
        .any(|segment| segment.subtitles.is_some())
    {
        // Text is rasterized by the common Rust/Meari renderer, never libass/system fonts.
        filters.insert("overlay".into());
    }
    Ok(ClipEncoderRequirements {
        video_encoder: "libx264".into(),
        audio_encoder: audio.then(|| "aac".into()),
        muxer: "mp4".into(),
        filters,
    })
}
pub fn prepare_encoder<P: ClipEncoderAdmission, C: super::presets::ClipPresetAdmission + ?Sized>(
    document: &ClipDocument,
    presets: &C,
    admission: &P,
    control: &dyn MutationControl,
) -> Result<PreparedClipEncoder<P::Lease>, ApiError> {
    control.checkpoint()?;
    let requirements = encoder_requirements(document)?;
    let preset_owner = super::presets::prepare_clip_preset(document, presets, control)?;
    let lease = admission.open_encoder(control)?;
    control.checkpoint()?;
    let inventory = lease.inventory();
    if !valid_sha256(&inventory.tool_sha256)
        || !valid_sha256(&inventory.inventory_sha256)
        || !valid_sha256(&inventory.build_configuration_sha256)
        || !valid_sha256(&inventory.license_sha256)
        || !inventory.version_line.starts_with("ffmpeg version ")
    {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Clip encoder inventory identity is incomplete",
        ));
    }
    let missing: Vec<String> =
        std::iter::once((&inventory.encoders, requirements.video_encoder.as_str()))
            .chain(
                requirements
                    .audio_encoder
                    .as_deref()
                    .map(|name| (&inventory.encoders, name)),
            )
            .chain(std::iter::once((
                &inventory.muxers,
                requirements.muxer.as_str(),
            )))
            .chain(
                requirements
                    .filters
                    .iter()
                    .map(|name| (&inventory.filters, name.as_str())),
            )
            .filter(|(names, name)| !names.contains(*name))
            .map(|(_, name)| name.into())
            .collect();
    if !missing.is_empty() {
        let mut error = ApiError::new(
            ErrorCode::NeedsInput,
            "Pinned clip encoder is missing required features",
        );
        error.details = serde_json::json!({"missing_features":missing});
        return Err(error);
    }
    control.checkpoint()?;
    let document_sha256 = format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(document)
                .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid clip encoder document"))?
        )
    );
    Ok(PreparedClipEncoder {
        lease,
        document_sha256,
        preset: document.preset.clone(),
        requirements,
        preset_owner,
    })
}
#[cfg(test)]
mod tests;
