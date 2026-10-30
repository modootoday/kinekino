use super::{
    process::{before, failed, receipt_json},
    *,
};
use crate::{
    application::review_source::MutationControl,
    domain::{
        media_timeline::TimeBase,
        review_source::timeline_evidence::{CaptureObservation, DecodedOrigin, VideoFrame},
    },
    infrastructure::{
        content_digest,
        frame_capture::{self, plan::Selection},
        timeline_evidence::raw::RetainedRaw,
    },
};
use kinekino_process::domain::observation::ProcessReceipt;
#[derive(Clone, Copy, Debug)]
pub enum TimelineCapturePoint {
    Origin,
    AtOrAfter { source_us: u64 },
}
pub struct ProducedTimelineCapture<'s, 'a> {
    pub(super) session: &'s TimelineProductionSession<'a>,
    pub(super) _frames: &'s ProducedVideoFrames<'s, 'a>,
    pub(super) _tool: &'s PinnedTimelineCaptureDecoder,
    _png: RetainedRaw,
    _metadata: RetainedRaw,
    pub(super) point: TimelineCapturePoint,
    observation: CaptureObservation,
    metadata_sha: String,
    dimensions: (u32, u32),
    receipt: ProcessReceipt,
    root: PathBuf,
    _records: Vec<File>,
}
impl ProducedTimelineCapture<'_, '_> {
    pub fn observation(&self) -> &CaptureObservation {
        &self.observation
    }
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }
    pub fn receipt(&self) -> &ProcessReceipt {
        &self.receipt
    }
    pub fn output_root(&self) -> &Path {
        &self.root
    }
    pub fn check_current(&self) -> Result<(), ApiError> {
        self.session.source.check_current()
    }
    pub fn decoded_origin(&self) -> Result<DecodedOrigin, ApiError> {
        if !matches!(self.point, TimelineCapturePoint::Origin) {
            return Err(invalid("Endpoint capture is not a decoded origin"));
        }
        self.check_current()?;
        Ok(DecodedOrigin {
            pts: self.observation.actual_pts,
            time_base: self.observation.time_base,
            source_sha256: self.observation.source_sha256.clone(),
            observation_sha256: self.metadata_sha.clone(),
        })
    }
}
fn selection(
    frames: &[VideoFrame],
    base: TimeBase,
    point: TimelineCapturePoint,
) -> Result<(Selection, Option<i64>), ApiError> {
    match point {
        TimelineCapturePoint::Origin => Ok((Selection::FirstDecoded, None)),
        TimelineCapturePoint::AtOrAfter { source_us } => {
            let threshold = base
                .ticks_at_or_after(source_us)
                .map_err(|_| invalid("Capture threshold overflow"))?;
            let tick = i64::try_from(threshold)
                .map_err(|_| invalid("Capture threshold exceeds signed clock"))?;
            if threshold > 1u128 << 53 || frames.first().is_none_or(|frame| tick <= frame.pts) {
                return Err(invalid(
                    "Endpoint requires an observed preceding frame and exact threshold",
                ));
            }
            let expected = frames
                .iter()
                .find(|frame| frame.pts >= tick)
                .ok_or_else(|| invalid("Endpoint exceeds retained frame window"))?
                .pts;
            Ok((Selection::AtOrAfter(threshold), Some(expected)))
        }
    }
}
impl<'a> TimelineProductionSession<'a> {
    pub fn capture_frame<'s>(
        &'s self,
        tool: &'s PinnedTimelineCaptureDecoder,
        frames: &'s ProducedVideoFrames<'s, 'a>,
        point: TimelineCapturePoint,
        control: &dyn MutationControl,
    ) -> Result<ProducedTimelineCapture<'s, 'a>, Box<TimelineProcessFailure<'s, 'a>>> {
        if !std::ptr::eq(self, frames.session) {
            return Err(before(ApiError::new(
                ErrorCode::SourceExpired,
                "Capture video window belongs to another source session",
            )));
        }
        frames.check_current().map_err(before)?;
        let (selected, expected) =
            selection(frames.frames().frames(), frames.frames().time_base(), point)
                .map_err(before)?;
        let arguments = frame_capture::plan::arguments(
            self.source.path(),
            crate::infrastructure::timeline_evidence::live_contract::demuxer(
                &self.source.identity().classification,
            ),
            selected,
            None,
            1,
        )
        .map_err(before)?;
        let component = if matches!(point, TimelineCapturePoint::Origin) {
            "capture-origin"
        } else {
            "capture-endpoint"
        };
        let mut owner = self.execute(&tool.0, component, arguments, None, Some(frames), control)?;
        let Some(receipt) = owner.receipt().cloned() else {
            return Err(failed(invalid("Capture process receipt missing"), owner));
        };
        let result = (|| {
            owner
                .paths
                .check_limit(self.max_output_bytes)
                .map_err(io_error)?;
            owner._records.push(write_new(
                &owner.root.join("PROCESS.json"),
                &receipt_json(&receipt),
            )?);
            let mut png_file = owner.paths.stdout.open_read().map_err(io_error)?;
            let (_, png_sha) = content_digest::sha256(&mut png_file, self.max_output_bytes)?;
            let mut png =
                RetainedRaw::open_bounded(owner.paths.stdout.path(), &png_sha, 64 * 1024 * 1024)?;
            let dimensions = frame_capture::validate_png(&png.read_bytes()?)?;
            let mut metadata_file = owner.paths.stderr.open_read().map_err(io_error)?;
            let (_, metadata_sha) =
                content_digest::sha256(&mut metadata_file, self.max_output_bytes)?;
            let mut metadata = RetainedRaw::open(owner.paths.stderr.path(), &metadata_sha)?;
            let (pts, base, actual_us) = frame_capture::selected_pts(&metadata.read_bytes()?)?;
            let pts: i64 = pts
                .parse()
                .map_err(|_| invalid("Capture PTS exceeds signed clock"))?;
            let base = TimeBase::parse(&base).map_err(|_| invalid("Capture time base invalid"))?;
            let requested_us = match point {
                TimelineCapturePoint::Origin => 0,
                TimelineCapturePoint::AtOrAfter { source_us } => source_us,
            };
            if base != frames.frames().time_base()
                || expected.is_some_and(|expected| expected != pts)
                || matches!(point, TimelineCapturePoint::Origin)
                    && frames
                        .frames()
                        .frames()
                        .first()
                        .is_none_or(|frame| pts > frame.pts)
                || actual_us < requested_us
            {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Decoded capture differs from retained frame window or requested source PTS",
                ));
            }
            let observation = CaptureObservation {
                requested_source_us: requested_us,
                actual_pts: pts,
                time_base: base,
                source_sha256: self.source.identity().media.sha256.clone(),
                png_sha256: png_sha,
                engine_sha256: tool.sha256().to_owned(),
            };
            control.checkpoint()?;
            self.source.check_current()?;
            if self.deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "Capture verification exceeded original deadline",
                ));
            }
            let (n, d) = base.ratio();
            owner._records.push(write_new(&owner.root.join("RETURN.json"),&serde_json::json!({
                "source":self.source.identity(),"requested_source_us":requested_us,"actual_pts":pts,"time_base":format!("{n}/{d}"),
                "decoded_origin":matches!(point,TimelineCapturePoint::Origin),"png_sha256":observation.png_sha256,"metadata_sha256":metadata_sha,
                "width":dimensions.0,"height":dimensions.1,"child":receipt_json(&receipt),"capture_validated":true,
                "authority_granted":false,"verified_timeline":false,"quality_approved":false,
                "limitations":["CPU RGB24; no HDR/ICC/player color parity or GUI current-frame claim","negative source timing unsupported by existing SDK parser"]
            }))?);
            control.checkpoint()?;
            self.source.check_current()?;
            if self.deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "Capture evidence write exceeded original deadline; validated raw evidence may exist",
                ));
            }
            Ok::<_, ApiError>((png, metadata, observation, metadata_sha, dimensions))
        })();
        match result {
            Ok((png, metadata, observation, metadata_sha, dimensions)) => {
                Ok(ProducedTimelineCapture {
                    session: self,
                    _frames: frames,
                    _tool: tool,
                    _png: png,
                    _metadata: metadata,
                    point,
                    observation,
                    metadata_sha,
                    dimensions,
                    receipt,
                    root: owner.root.clone(),
                    _records: std::mem::take(&mut owner._records),
                })
            }
            Err(error) => Err(failed(error, owner)),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endpoint_requires_preceding_observation_and_selects_first_exact_tick() {
        let frames = [
            VideoFrame {
                pts: 196608,
                duration: 512,
            },
            VideoFrame {
                pts: 208896,
                duration: 512,
            },
            VideoFrame {
                pts: 209408,
                duration: 512,
            },
        ];
        let base = TimeBase::new(1, 12288).unwrap();
        assert_eq!(
            selection(
                &frames,
                base,
                TimelineCapturePoint::AtOrAfter {
                    source_us: 17_000_000
                }
            )
            .unwrap()
            .1,
            Some(208896)
        );
        assert_eq!(
            selection(
                &frames,
                base,
                TimelineCapturePoint::AtOrAfter {
                    source_us: 17_000_001
                }
            )
            .unwrap()
            .1,
            Some(209408)
        );
        for time in [0, 16_000_000, 18_000_000, u64::MAX] {
            assert!(
                selection(
                    &frames,
                    base,
                    TimelineCapturePoint::AtOrAfter { source_us: time }
                )
                .is_err()
            );
        }
    }
}
