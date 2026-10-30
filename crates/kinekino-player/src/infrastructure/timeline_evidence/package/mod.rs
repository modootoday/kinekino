//! Only a producer bundle explicitly nominated by trusted Rust assembly is admitted.
mod manifest;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        model_review::ReviewStore,
        review_source::{
            RetainedReviewEvidence, ReviewCaseEvidence,
            timeline_evidence::{TimelineEvidenceLease, TimelineEvidenceResolver},
        },
    },
    domain::{
        identifiers::valid_sha256,
        media_timeline::TimeBase,
        review_source::{
            EvidenceReference, MediaReference, ReviewCase, timeline_evidence::*,
            valid_review_object_id,
        },
    },
};
use manifest::*;
use serde::de::DeserializeOwned;
/// A trusted host nominates an already reviewed producer bundle and tool pins.
/// No Deserialize implementation, SDK command, file path, or automatic discovery.
/// Nomination is the provenance authority; hashing arbitrary caller JSON is not.
#[derive(Clone)]
pub struct TrustedTimelineAdmission {
    pub case: ReviewCase,
    pub media: MediaReference,
    pub evidence: EvidenceReference,
    pub audio_engine_sha256: String,
    pub video_engine_sha256: String,
    pub capture_engine_sha256: String,
    pub capture_cli_sha256: String,
}
pub struct ReviewTimelineEvidence<S> {
    reader: ReviewCaseEvidence<S>,
    admission: TrustedTimelineAdmission,
}
impl<S: ReviewStore> ReviewTimelineEvidence<S> {
    pub fn new(store: S, admission: TrustedTimelineAdmission) -> Result<Self, ApiError> {
        admission.case.validate().map_err(|_| invalid())?;
        admission.media.validate().map_err(|_| invalid())?;
        if !valid_review_object_id(&admission.evidence.asset_id)
            || [
                &admission.evidence.sha256,
                &admission.audio_engine_sha256,
                &admission.video_engine_sha256,
                &admission.capture_engine_sha256,
                &admission.capture_cli_sha256,
            ]
            .iter()
            .any(|sha| !valid_sha256(sha))
        {
            return Err(invalid());
        }
        Ok(Self {
            reader: ReviewCaseEvidence::new(store),
            admission,
        })
    }
}
struct Lease {
    input: RetainedReviewEvidence,
    facts: TimelineObservations,
}
impl TimelineEvidenceLease for Lease {
    fn observations(&self) -> &TimelineObservations {
        &self.facts
    }
    fn check_current(&self) -> Result<(), ApiError> {
        self.input.check_current()
    }
}
impl<S: ReviewStore> TimelineEvidenceResolver for ReviewTimelineEvidence<S> {
    fn retain(
        &self,
        case: &ReviewCase,
        media: &MediaReference,
        reference: &EvidenceReference,
    ) -> Result<Box<dyn TimelineEvidenceLease + '_>, ApiError> {
        let admitted = &self.admission;
        if case != &admitted.case || media != &admitted.media || reference != &admitted.evidence {
            return Err(conflict());
        }
        let input = self.reader.retain(case)?;
        let manifest: Manifest = json(&input, reference)?;
        if manifest.schema_version != "1"
            || manifest.source_request_sha256 != case.source_request_sha256
            || manifest.media != *media
        {
            return Err(conflict());
        }
        input.inspect(&EvidenceReference {
            asset_id: media.asset_id.clone(),
            sha256: media.sha256.clone(),
        })?;
        engine(
            &input,
            &manifest.audio.engine,
            &admitted.audio_engine_sha256,
        )?;
        engine(
            &input,
            &manifest.video.engine,
            &admitted.video_engine_sha256,
        )?;
        let audio: AudioExecution = json(&input, &manifest.audio.execution)?;
        if audio.schema_version != "1"
            || audio.source_sha256 != media.sha256
            || audio.engine_sha256 != manifest.audio.engine.sha256
            || audio.log_sha256 != manifest.audio.log.sha256
            || audio.replay_sha256 != manifest.audio.replay_pcm.sha256
            || !audio.copy_timestamps
            || audio.sample_rate != 16000
            || audio.channels != 1
            || audio.sample_format != "s16"
            || audio.time_base != "1/16000"
            || audio.async_correction
        {
            return Err(conflict());
        }
        let frames = super::audio::parse(&text(&input, &manifest.audio.log)?)?;
        let audio_completion = super::original_execution::audio(
            &audio.trace.contract,
            &text(&input, &audio.trace.command)?,
            &text(&input, &audio.trace.inputs)?,
            &text(&input, &audio.trace.process)?,
            &media.sha256,
            &manifest.audio.engine.sha256,
            &manifest.audio.full_wav.sha256,
        )?;
        if audio.completion != audio_completion {
            return Err(conflict());
        }
        let pcm = super::pcm::compare(
            &input.read(&manifest.audio.full_wav, 64 * 1024 * 1024)?,
            &input.read(&manifest.audio.clip_wav, 64 * 1024 * 1024)?,
            &input.read(&manifest.audio.replay_pcm, 64 * 1024 * 1024)?,
            manifest.audio.clip_start_sample.0,
        )?;
        let samples = frames
            .iter()
            .try_fold(0u64, |sum, frame| sum.checked_add(u64::from(frame.samples)))
            .ok_or_else(invalid)?;
        if samples != pcm.full_samples {
            return Err(conflict());
        }
        let (video_frames, time_base, stream_index) =
            super::video::parse(&text(&input, &manifest.video.frames)?)?;
        let video: VideoExecution = json(&input, &manifest.video.execution)?;
        if video.schema_version != "1"
            || video.source_sha256 != media.sha256
            || video.engine_sha256 != manifest.video.engine.sha256
            || video.frames_sha256 != manifest.video.frames.sha256
            || video.stream_index != stream_index
        {
            return Err(conflict());
        }
        let video_completion = super::original_execution::video(
            &video.trace.contract,
            &text(&input, &video.trace.command)?,
            &text(&input, &video.trace.inputs)?,
            &text(&input, &video.trace.process)?,
            &media.sha256,
            &manifest.video.engine.sha256,
        )?;
        if video.completion != video_completion {
            return Err(conflict());
        }
        let origin = capture(
            &input,
            &manifest.origin_capture,
            media,
            &admitted.capture_engine_sha256,
            &admitted.video_engine_sha256,
            &admitted.capture_cli_sha256,
        )?;
        if origin.requested_source_us != 0 || origin.time_base != time_base {
            return Err(conflict());
        }
        let start_capture = capture(
            &input,
            &manifest.start_capture,
            media,
            &admitted.capture_engine_sha256,
            &admitted.video_engine_sha256,
            &admitted.capture_cli_sha256,
        )?;
        let end_capture = capture(
            &input,
            &manifest.end_capture,
            media,
            &admitted.capture_engine_sha256,
            &admitted.video_engine_sha256,
            &admitted.capture_cli_sha256,
        )?;
        let facts = TimelineObservations {
            video_cadence: None,
            case: case.clone(),
            media: media.clone(),
            reference: reference.clone(),
            mapping_revision: manifest.mapping_revision,
            time_base,
            origin: DecodedOrigin {
                pts: origin.actual_pts,
                time_base: origin.time_base,
                source_sha256: media.sha256.clone(),
                observation_sha256: manifest.origin_capture.sha256,
            },
            video_frames,
            pcm: PcmObservations {
                source_sha256: media.sha256.clone(),
                original_data_sha256: pcm.full_data_sha256,
                replay_data_sha256: pcm.replay_data_sha256,
                replay_engine_sha256: admitted.audio_engine_sha256.clone(),
                sample_rate: 16000,
                frames,
                clip_start_sample: pcm.clip_start_sample,
                clip_samples: pcm.clip_samples,
                original_slice_sha256: pcm.clip_data_sha256.clone(),
                clip_data_sha256: pcm.clip_data_sha256,
            },
            start_capture,
            end_capture,
        };
        facts.validate().map_err(|_| invalid())?;
        input.check_current()?;
        Ok(Box::new(Lease { input, facts }))
    }
}
fn engine(
    input: &RetainedReviewEvidence,
    reference: &EvidenceReference,
    expected: &str,
) -> Result<(), ApiError> {
    if reference.sha256 != expected {
        return Err(conflict());
    }
    input.inspect(reference)?;
    Ok(())
}
fn capture(
    input: &RetainedReviewEvidence,
    reference: &EvidenceReference,
    media: &MediaReference,
    expected_engine: &str,
    expected_probe: &str,
    expected_cli: &str,
) -> Result<CaptureObservation, ApiError> {
    let record: Capture = json(input, reference)?;
    if record.schema_version != "1"
        || record.source_sha256 != media.sha256
        || !record.complete
        || record.exit_code != 0
    {
        return Err(conflict());
    }
    engine(input, &record.engine, expected_engine)?;
    let png = input.read(&record.png, 64 * 1024 * 1024)?;
    let dimensions = crate::infrastructure::frame_capture::validate_png(&png)?;
    let time_base = TimeBase::parse(&record.time_base).map_err(|_| invalid())?;
    super::capture_execution::verify(
        &record.trace.contract,
        &input.read(&record.trace.inputs, 65536)?,
        &input.read(&record.trace.request, 65536)?,
        &input.read(&record.trace.response, 65536)?,
        &input.read(&record.trace.process, 65536)?,
        &super::capture_execution::CaptureExpectation {
            source_sha: &media.sha256,
            engine_sha: expected_engine,
            probe_sha: expected_probe,
            producer_cli_sha: expected_cli,
            requested_us: record.requested_source_us.0,
            actual_pts: record.actual_pts,
            time_base,
            png_sha: &record.png.sha256,
            png_bytes: png.len() as u64,
            dimensions,
        },
    )?;
    Ok(CaptureObservation {
        requested_source_us: record.requested_source_us.0,
        actual_pts: record.actual_pts,
        time_base,
        source_sha256: record.source_sha256,
        png_sha256: record.png.sha256,
        engine_sha256: record.engine.sha256,
    })
}
fn json<T: DeserializeOwned>(
    input: &RetainedReviewEvidence,
    reference: &EvidenceReference,
) -> Result<T, ApiError> {
    serde_json::from_slice(&input.read(reference, 65536)?).map_err(|_| invalid())
}
fn text(input: &RetainedReviewEvidence, reference: &EvidenceReference) -> Result<String, ApiError> {
    String::from_utf8(input.read(reference, 12 * 1024 * 1024)?).map_err(|_| invalid())
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid nominated timeline evidence",
    )
}
fn conflict() -> ApiError {
    ApiError::new(
        ErrorCode::RevisionConflict,
        "Timeline evidence source, tool or execution binding differs",
    )
}
#[cfg(test)]
mod tests;
