//! Bounded uncommitted MP4 candidate. Output validation/publication and Jobs are separate.
mod command;
mod retained;
mod verification;
pub(crate) use retained::RetainedClipRenderCandidate;
pub use verification::{
    ClipOutputMetadata, ClipOutputVerification, ClipOutputVerifier, OwnedVerifiedClipCandidate,
    VerifiedClipCandidate,
};
mod publication;
pub mod subtitles;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        clip_render::{
            ClipEncoderLease, ManualClipPlan, PreparedClipEncoder, PreparedClipMedia,
            SavedClipProjectPin, manual_clip_plan,
        },
        review_source::MutationControl,
    },
    infrastructure::{
        clip_encoder::PinnedClipEncoder,
        clip_sources::{NativeClipBoundaryLease, NativeClipSourceLease},
        frame_capture::process,
    },
};
pub(crate) use publication::ClipOwnedOutputCandidate;
pub use publication::{ClipArtifactStore, PublishedClipArtifact, RetainedClipArtifact};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    process::Command,
    time::{Duration, Instant},
};
/// Explicit host interpretation, never inferred from missing source metadata or caller JSON.
/// It does not assert verified color accuracy and remains visible in candidate evidence.
pub struct DeclaredBt709Limited {
    pub source_sha256: String,
}
#[derive(Serialize)]
pub struct ClipCandidateFacts {
    pub recipe: String,
    pub preset_definition: crate::application::clip_render::presets::ClipPresetEvidence,
    pub project: SavedClipProjectPin,
    pub encoder_tool_sha256: String,
    pub encoder_inventory_sha256: String,
    pub boundary_window_sha256: String,
    pub plan: ManualClipPlan,
    pub output_sha256: String,
    pub output_bytes: usize,
    pub source_color_mode: String,
    pub color_verified: bool,
    pub quality_approved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitles: Option<subtitles::ClipOverlayFacts>,
}
pub struct EncodedClipCandidate {
    facts: ClipCandidateFacts,
    bytes: Vec<u8>,
    stderr: Vec<u8>,
}
impl EncodedClipCandidate {
    pub fn facts(&self) -> &ClipCandidateFacts {
        &self.facts
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }
}
pub fn encode_candidate(
    media: &PreparedClipMedia<NativeClipSourceLease>,
    encoder: &PreparedClipEncoder<PinnedClipEncoder>,
    boundaries: &NativeClipBoundaryLease<'_>,
    color: &DeclaredBt709Limited,
    timeout: Duration,
    max_output_bytes: usize,
    control: &dyn MutationControl,
) -> Result<EncodedClipCandidate, ApiError> {
    control.checkpoint()?;
    let plan = manual_clip_plan(media, encoder, &boundaries.observation().boundaries)?;
    encode_plan(CandidateEncoding {
        media,
        encoder,
        boundaries,
        color,
        plan,
        overlay: None,
        timeout,
        max_output_bytes,
        control,
    })
}
struct CandidateEncoding<'a, 'source> {
    media: &'a PreparedClipMedia<NativeClipSourceLease>,
    encoder: &'a PreparedClipEncoder<PinnedClipEncoder>,
    boundaries: &'a NativeClipBoundaryLease<'source>,
    color: &'a DeclaredBt709Limited,
    plan: ManualClipPlan,
    overlay: Option<&'a subtitles::PreparedClipOverlay<'a, 'source>>,
    timeout: Duration,
    max_output_bytes: usize,
    control: &'a dyn MutationControl,
}
fn encode_plan(request: CandidateEncoding<'_, '_>) -> Result<EncodedClipCandidate, ApiError> {
    let CandidateEncoding {
        media,
        encoder,
        boundaries,
        color,
        plan,
        overlay,
        timeout,
        max_output_bytes,
        control,
    } = request;
    control.checkpoint()?;
    if timeout.is_zero()
        || timeout > Duration::from_secs(120)
        || max_output_bytes == 0
        || max_output_bytes > 128 * 1024 * 1024
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Bounded clip candidate deadline and output bytes required",
        ));
    }
    let source = media
        .sources()
        .iter()
        .find(|source| {
            source.facts().source.media_id == plan.source_media_id
                && source.facts().source.sha256 == plan.source_sha256
                && source.facts().source.clock.observation_sha256 == plan.source_observation_sha256
        })
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::SourceExpired,
                "Manual clip source lease is unavailable",
            )
        })?;
    if !std::ptr::eq(source, boundaries.source()) || color.source_sha256 != plan.source_sha256 {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Boundary or explicit color declaration belongs to a different source owner",
        ));
    }
    if source
        .facts()
        .color_transfer
        .as_deref()
        .is_some_and(|transfer| matches!(transfer, "smpte2084" | "arib-std-b67"))
        || source
            .facts()
            .rotation_degrees
            .is_some_and(|rotation| rotation != 0)
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "HDR tone mapping or source rotation requires a verified adapter",
        ));
    }
    let deadline = Instant::now() + timeout;
    let mut command = Command::new(encoder.lease().path());
    command.args(if overlay.is_some() {
        command::arguments_with_overlay(source.path(), source.demuxer(), &plan, overlay)?
    } else {
        command::arguments(source.path(), source.demuxer(), &plan)?
    });
    let run = |input| match input {
        Some(input) => process::run_with_input(
            &mut command,
            deadline,
            max_output_bytes,
            Some(control),
            input,
        ),
        None => process::run_with_recovery(
            &mut command,
            deadline,
            max_output_bytes,
            Some(control),
            None,
        ),
    };
    let output = match overlay {
        Some(overlay) => overlay.with_input(run)?,
        None => {
            let mut run = run;
            run(None)?
        }
    };
    control.checkpoint()?;
    if output.stdout.len() < 12 || &output.stdout[4..8] != b"ftyp" {
        return Err(ApiError::new(
            ErrorCode::EngineFailed,
            "Clip encoder produced no MP4 candidate",
        ));
    }
    let facts = ClipCandidateFacts {
        preset_definition: encoder.preset_evidence().clone(),
        recipe: if overlay.is_some() {
            "manual-mp4-seekable-stdout/3"
        } else {
            "manual-mp4-seekable-stdout/2"
        }
        .into(),
        project: media.project().clone(),
        encoder_tool_sha256: encoder.lease().inventory().tool_sha256.clone(),
        encoder_inventory_sha256: encoder.lease().inventory().inventory_sha256.clone(),
        boundary_window_sha256: boundaries.observation().raw_frame_window_sha256.clone(),
        output_sha256: format!("{:x}", Sha256::digest(&output.stdout)),
        output_bytes: output.stdout.len(),
        plan,
        source_color_mode: "host_declared_bt709_limited_unverified".into(),
        color_verified: false,
        quality_approved: false,
        subtitles: overlay.map(|overlay| overlay.facts().clone()),
    };
    Ok(EncodedClipCandidate {
        facts,
        bytes: output.stdout,
        stderr: output.stderr,
    })
}
#[cfg(test)]
mod tests;
