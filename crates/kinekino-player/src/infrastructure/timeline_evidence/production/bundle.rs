//! Retained live observations, deliberately not a package asset or resolver grant.
use super::*;
mod export;
use crate::infrastructure::timeline_evidence::live_contract::ProducerPin;
use crate::{
    application::review_source::MutationControl,
    domain::review_source::timeline_evidence::{
        DecodedOrigin, MappingWindow, PcmObservations, TimelineContent,
    },
    infrastructure::timeline_evidence::raw::RetainedRaw,
};
pub(crate) use export::ProducedArtifact;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Trusted Rust references to actual producers; no paths, DTOs or deserialization.
#[derive(Clone, Copy)]
pub struct TimelineBundleInputs<'s, 'a> {
    pub video: &'s ProducedVideoFrames<'s, 'a>,
    pub pcm: &'s ProducedPcmLineage<'s, 'a>,
    pub origin: &'s ProducedTimelineCapture<'s, 'a>,
    pub start: &'s ProducedTimelineCapture<'s, 'a>,
    pub end: &'s ProducedTimelineCapture<'s, 'a>,
}
/// Keeps original source/tool/process observations and files alive together.
/// No Deserialize, TimelineEvidenceLease or implicit custody registration.
pub struct ProducedTimelineBundle<'s, 'a> {
    pub(super) session: &'s TimelineProductionSession<'a>,
    pub(super) inputs: TimelineBundleInputs<'s, 'a>,
    origin: DecodedOrigin,
    pcm: PcmObservations,
    mapping: MappingWindow,
    root: PathBuf,
    receipt: RetainedRaw,
    producer: ProducerPin,
    _records: Vec<File>,
}
impl ProducedTimelineBundle<'_, '_> {
    pub fn content(&self) -> TimelineContent<'_> {
        TimelineContent {
            video_cadence: self.inputs.video.frames().cadence(),
            source_sha256: &self.session.source.identity().media.sha256,
            time_base: self.inputs.video.frames().time_base(),
            origin: &self.origin,
            video_frames: self.inputs.video.frames().frames(),
            pcm: &self.pcm,
            start_capture: self.inputs.start.observation(),
            end_capture: self.inputs.end.observation(),
        }
    }
    pub fn mapping(&self) -> MappingWindow {
        self.mapping
    }
    pub fn output_root(&self) -> &Path {
        &self.root
    }
    pub fn receipt_sha256(&self) -> &str {
        self.receipt.sha256()
    }
    pub fn check_current(&self) -> Result<(), ApiError> {
        self.session.source.check_current()?;
        self.inputs.video.check_current()?;
        self.inputs.pcm.check_current()?;
        self.inputs.origin.check_current()?;
        self.inputs.start.check_current()?;
        self.inputs.end.check_current()?;
        if self
            .content()
            .validate()
            .map_err(|_| invalid("Retained bundle content inconsistent"))?
            != self.mapping
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Retained bundle mapping changed",
            ));
        }
        Ok(())
    }
}
struct Publication<'a>(&'a Cell<u8>);
impl Drop for Publication<'_> {
    fn drop(&mut self) {
        self.0.set(0);
    }
}
fn capture_record(capture: &ProducedTimelineCapture<'_, '_>) -> serde_json::Value {
    let observed = capture.observation();
    let (n, d) = observed.time_base.ratio();
    serde_json::json!({"root":capture.output_root(),"requested_source_us":observed.requested_source_us,
        "actual_pts":observed.actual_pts,"time_base":format!("{n}/{d}"),"source_sha256":observed.source_sha256,
        "png_sha256":observed.png_sha256,"engine_sha256":observed.engine_sha256,"dimensions":capture.dimensions()})
}
impl<'a> TimelineProductionSession<'a> {
    pub fn bundle<'s>(
        &'s self,
        inputs: TimelineBundleInputs<'s, 'a>,
        control: &dyn MutationControl,
    ) -> Result<ProducedTimelineBundle<'s, 'a>, ApiError> {
        if self.active.get() != 0 {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Timeline component active or abandoned",
            ));
        }
        self.active.set(1);
        let _publication = Publication(&self.active);
        control.checkpoint()?;
        self.source.check_current()?;
        if !std::ptr::eq(self, inputs.video.session)
            || !std::ptr::eq(self, inputs.pcm.original.clock.session)
            || [inputs.origin, inputs.start, inputs.end]
                .iter()
                .any(|capture| {
                    !std::ptr::eq(self, capture.session)
                        || !std::ptr::eq(inputs.video, capture._frames)
                })
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Bundle producers belong to different source sessions or video observations",
            ));
        }
        inputs.video.check_current()?;
        inputs.pcm.check_current()?;
        inputs.origin.check_current()?;
        inputs.start.check_current()?;
        inputs.end.check_current()?;
        let capture_sha = &inputs.origin.observation().engine_sha256;
        if [
            &inputs.start.observation().engine_sha256,
            &inputs.end.observation().engine_sha256,
        ]
        .iter()
        .any(|sha| *sha != capture_sha)
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Bundle capture engine pins differ",
            ));
        }
        let origin = inputs.origin.decoded_origin()?;
        let facts = inputs.pcm.facts();
        let original = &inputs.pcm.original;
        let pcm = PcmObservations {
            source_sha256: self.source.identity().media.sha256.clone(),
            original_data_sha256: facts.full_data_sha256.clone(),
            replay_data_sha256: facts.replay_data_sha256.clone(),
            replay_engine_sha256: original.clock._tool.sha256().to_owned(),
            sample_rate: original.clock.clock().sample_rate(),
            frames: original.clock.clock().frames().to_vec(),
            clip_start_sample: facts.clip_start_sample,
            clip_samples: facts.clip_samples,
            original_slice_sha256: facts.clip_data_sha256.clone(),
            clip_data_sha256: facts.clip_data_sha256.clone(),
        };
        let content = TimelineContent {
            video_cadence: inputs.video.frames().cadence(),
            source_sha256: &self.source.identity().media.sha256,
            time_base: inputs.video.frames().time_base(),
            origin: &origin,
            video_frames: inputs.video.frames().frames(),
            pcm: &pcm,
            start_capture: inputs.start.observation(),
            end_capture: inputs.end.observation(),
        };
        let mapping = content
            .validate()
            .map_err(|_| invalid("Live source timeline components are inconsistent"))?;
        if self.deadline.operation_remaining(Instant::now()).is_zero() {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Bundle validation exceeded original deadline",
            ));
        }
        let (n, d) = content.time_base.ratio();
        let producer_path = std::env::current_exe().map_err(io_error)?;
        let (parent, mut producer_guards) = rooted::root(
            producer_path
                .parent()
                .ok_or_else(|| invalid("Current producer parent missing"))?,
        )?;
        let producer_path = parent.join(
            producer_path
                .file_name()
                .ok_or_else(|| invalid("Current producer filename missing"))?,
        );
        let mut producer_file = rooted::file(&producer_path)?;
        let (producer_bytes, producer_sha) =
            crate::infrastructure::content_digest::sha256(&mut producer_file, 512 * 1024 * 1024)?;
        let producer = ProducerPin {
            path: producer_path.to_string_lossy().into(),
            sha256: producer_sha,
            bytes: producer_bytes,
            file_identity: super::tool::file_identity(&producer_file)?,
            pid: std::process::id(),
        };
        producer_guards.push(producer_file);
        let record = serde_json::json!({
            "schema_version":"live_observations_2","producer":producer,"source":self.source.identity(),"time_base":format!("{n}/{d}"),
            "video":{"root":inputs.video.output_root(),"frames_sha256":inputs.video.frames().sha256(),"stream_index":inputs.video.frames().stream_index(),
                "cadence":content.video_cadence,"frames":content.video_frames.iter().map(|frame|serde_json::json!({"pts":frame.pts,"duration":frame.duration})).collect::<Vec<_>>()},
            "pcm":{"original_root":original.clock.output_root(),"lineage_root":inputs.pcm.output_root(),"full_samples":facts.full_samples,
                "original_data_sha256":pcm.original_data_sha256,"replay_data_sha256":pcm.replay_data_sha256,"replay_engine_sha256":pcm.replay_engine_sha256,
                "sample_rate":pcm.sample_rate,"clip_start_sample":pcm.clip_start_sample,"clip_samples":pcm.clip_samples,
                "original_slice_sha256":pcm.original_slice_sha256,"clip_data_sha256":pcm.clip_data_sha256,
                "frames":pcm.frames.iter().map(|frame|serde_json::json!({"sequence":frame.sequence,"pts":frame.pts,"samples":frame.samples})).collect::<Vec<_>>()},
            "origin":{"capture":capture_record(inputs.origin),"observation_sha256":origin.observation_sha256},
            "start_capture":capture_record(inputs.start),"end_capture":capture_record(inputs.end),
            "mapping":{"source_start_us":mapping.source_start_us,"source_end_us":mapping.source_end_us,"start_frame_pts":mapping.start_frame_pts,"end_frame_pts":mapping.end_frame_pts},
            "content_validated":true,"custody_registered":false,"authority_granted":false,"verified_timeline":false,"quality_approved":false
        });
        let bytes = serde_json::to_vec_pretty(&record)
            .map_err(|e| ApiError::new(ErrorCode::EngineFailed, e.to_string()))?;
        if bytes.len() as u64 > self.max_output_bytes {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Bundle receipt exceeds output budget",
            ));
        }
        control.checkpoint()?;
        self.source.check_current()?;
        if self.deadline.operation_remaining(Instant::now()).is_zero() {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Bundle encoding exceeded original deadline",
            ));
        }
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| io_error(std::io::Error::other(e)))?
            .as_nanos();
        let root = self
            .root
            .join(format!("bundle-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&root).map_err(io_error)?;
        let mut records = vec![
            rooted::directory(&root)?,
            write_new(&root.join("BUNDLE.json"), &record)?,
        ];
        records.extend(producer_guards);
        let sha = format!("{:x}", Sha256::digest(&bytes));
        let receipt = RetainedRaw::open_bounded(&root.join("BUNDLE.json"), &sha, 64 * 1024 * 1024)?;
        let result = ProducedTimelineBundle {
            session: self,
            inputs,
            origin,
            pcm,
            mapping,
            root,
            receipt,
            producer,
            _records: records,
        };
        control.checkpoint()?;
        result.check_current()?;
        if self.deadline.operation_remaining(Instant::now()).is_zero() {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Bundle write exceeded original deadline; validated raw receipt may exist",
            ));
        }
        Ok(result)
    }
}
