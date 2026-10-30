//! Bind archivable metadata and range plans back to the original live evidence.
use super::*;
use crate::domain::{
    api_values::DecimalU64,
    review_source::{
        UserRange,
        timeline_mapping::{TimelineMappingRecord, TimelineRatioRecord},
    },
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct TimelineRangePlan {
    pub range: UserRange,
    pub first_frame_pts: String,
    pub end_frame_pts: String,
}
/// Mapping custody, not runtime/seek completion or a stored Verified mutation.
/// ```compile_fail
/// use kinekino_player::application::review_source::timeline_evidence::RetainedMappingRange;
/// let forged = serde_json::from_str::<RetainedMappingRange<'static, 'static>>("{}").unwrap();
/// ```
/// ```compile_fail
/// use kinekino_player::{application::review_source::timeline_evidence::RetainedTimelineEvidence,
/// domain::review_source::{UserRange, timeline_mapping::TimelineMappingRecord}};
/// fn release_before_use(proof: RetainedTimelineEvidence<'_>, record: &TimelineMappingRecord, range: &UserRange) {
///     let retained_range = proof.retain_range(record, range).unwrap();
///     drop(proof);
///     retained_range.check_current().unwrap();
/// }
/// ```
pub struct RetainedMappingRange<'proof, 'custody> {
    proof: &'proof RetainedTimelineEvidence<'custody>,
    record: TimelineMappingRecord,
    plan: TimelineRangePlan,
}
impl RetainedMappingRange<'_, '_> {
    pub fn record(&self) -> &TimelineMappingRecord {
        &self.record
    }
    pub fn plan(&self) -> &TimelineRangePlan {
        &self.plan
    }
    pub fn check_current(&self) -> Result<(), ApiError> {
        self.proof.check_record(&self.record)
    }
}
impl<'custody> RetainedTimelineEvidence<'custody> {
    /// This serializable record is data; only this original retained object is authority.
    pub fn mapping_record(&self) -> Result<TimelineMappingRecord, ApiError> {
        self.check_current()?;
        let facts = &self.snapshot;
        let record = TimelineMappingRecord {
            case: facts.case.clone(),
            media: facts.media.clone(),
            evidence: facts.reference.clone(),
            mapping_revision: facts.mapping_revision,
            source_time_base: TimelineRatioRecord::from_time_base(facts.time_base),
            origin_pts: facts.origin.pts.to_string(),
            origin_time_base: TimelineRatioRecord::from_time_base(facts.origin.time_base),
            origin_observation_sha256: facts.origin.observation_sha256.clone(),
            source_start_us: DecimalU64(self.mapping.source_start_us),
            source_end_us: DecimalU64(self.mapping.source_end_us),
            start_frame_pts: self.mapping.start_frame_pts.to_string(),
            end_frame_pts: self.mapping.end_frame_pts.to_string(),
        };
        record.validate().map_err(|_| invalid())?;
        Ok(record)
    }
    pub fn check_record(&self, record: &TimelineMappingRecord) -> Result<(), ApiError> {
        record.validate().map_err(|_| invalid())?;
        if self.mapping_record()? != *record {
            return Err(binding());
        }
        Ok(())
    }
    pub fn retain_range<'proof>(
        &'proof self,
        record: &TimelineMappingRecord,
        range: &UserRange,
    ) -> Result<RetainedMappingRange<'proof, 'custody>, ApiError> {
        self.check_record(record)?;
        if range.start_us.0 >= range.end_us.0 {
            return Err(invalid());
        }
        if range.start_us.0 < self.mapping.source_start_us
            || range.end_us.0 > self.mapping.source_end_us
        {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Range requires additional retained timeline coverage",
            ));
        }
        let frame = |micros| -> Result<String, ApiError> {
            let threshold = self
                .snapshot
                .time_base
                .ticks_at_or_after(micros)
                .map_err(|_| invalid())?;
            let threshold = i64::try_from(threshold).map_err(|_| invalid())?;
            self.snapshot
                .video_frames
                .iter()
                .find(|frame| frame.pts >= threshold)
                .map(|frame| frame.pts.to_string())
                .ok_or_else(invalid)
        };
        let plan = TimelineRangePlan {
            range: range.clone(),
            first_frame_pts: frame(range.start_us.0)?,
            end_frame_pts: frame(range.end_us.0)?,
        };
        self.check_record(record)?;
        Ok(RetainedMappingRange {
            proof: self,
            record: record.clone(),
            plan,
        })
    }
}
