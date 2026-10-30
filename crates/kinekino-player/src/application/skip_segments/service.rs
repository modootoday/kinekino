use super::*;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_session::ControlEpisodePlayback,
    },
    domain::{
        api_values::DecimalU64,
        identifiers::valid_id,
        playback_control::PlaybackControl,
        skip_segments::{
            self, Evaluation, Intent, ManualSegment, PassState, SegmentOrigin, SegmentTimebase,
            SkipPolicy,
        },
    },
};

/// Exclusive owner of an ephemeral playback pass. No durable settings or scheduler.
pub struct SkipSession {
    binding: SkipBinding,
    revision: u64,
    pass_id: u64,
    policy: SkipPolicy,
    segments: Vec<ManualSegment>,
    pass: PassState,
}

impl SkipSession {
    pub(super) fn export_state(&self) -> (DecimalU64, DecimalU64) {
        (DecimalU64(self.revision), DecimalU64(self.pass_id))
    }
    pub(super) fn binding_ref(&self) -> &SkipBinding {
        &self.binding
    }
    /// Read scalars only; core skip still owns validation, reservation and cancellation order.
    pub(super) fn export_selection(
        &self,
        segment_id: &str,
    ) -> (
        DecimalU64,
        DecimalU64,
        Option<crate::application::playback_session::v1::NativeExportAnnotationInputV1>,
    ) {
        let annotation = self
            .segments
            .iter()
            .find(|segment| segment.id == segment_id)
            .map(|segment| {
                crate::application::playback_session::v1::NativeExportAnnotationInputV1 {
                    annotation_revision: segment.revision,
                    verified_revision: segment.verified_revision.unwrap_or(DecimalU64(0)),
                    timebase: segment.timebase,
                }
            });
        (
            DecimalU64(self.revision),
            DecimalU64(self.pass_id),
            annotation,
        )
    }
    pub fn new(binding: SkipBinding) -> Result<Self, ApiError> {
        if binding.engine_instance_id.0 == 0 || !valid_id(&binding.resolution_artifact_id) {
            return Err(invalid("Invalid skip session binding"));
        }
        Ok(Self {
            binding,
            revision: 1,
            pass_id: 1,
            policy: SkipPolicy::default(),
            segments: Vec::new(),
            pass: PassState::default(),
        })
    }
    pub fn inspect(&self) -> SkipSnapshot {
        SkipSnapshot {
            binding: self.binding.clone(),
            revision: DecimalU64(self.revision),
            pass_id: DecimalU64(self.pass_id),
            policy: self.policy.clone(),
            segments: self.segments.clone(),
            durable: false,
            detection_supported: false,
        }
    }
    fn next(&self, expected: DecimalU64) -> Result<u64, ApiError> {
        if expected.0 != self.revision {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Skip session revision changed",
            ));
        }
        self.revision
            .checked_add(1)
            .ok_or_else(|| ApiError::new(ErrorCode::ResourceExhausted, "Skip revision exhausted"))
    }
    fn index(&self, id: &str) -> Result<usize, ApiError> {
        self.segments
            .iter()
            .position(|segment| segment.id == id)
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Manual segment not found"))
    }
    pub fn edit(&mut self, input: EditSegment) -> Result<SkipSnapshot, ApiError> {
        let next = self.next(input.expected_revision)?;
        let previous = self
            .segments
            .iter()
            .position(|segment| segment.id == input.segment_id);
        let revision = match previous {
            Some(index) => self.segments[index]
                .revision
                .0
                .checked_add(1)
                .ok_or_else(|| invalid("Annotation revision exhausted"))?,
            None => 1,
        };
        let segment = ManualSegment {
            id: input.segment_id,
            revision: DecimalU64(revision),
            kind: input.kind,
            in_us: input.in_us,
            out_us: input.out_us,
            origin: SegmentOrigin::Manual,
            timebase: SegmentTimebase::EnginePlayback,
            verified_revision: None,
        };
        segment.validate().map_err(invalid)?;
        if let Some(index) = previous {
            self.segments[index] = segment;
        } else {
            self.segments.push(segment);
        }
        self.revision = next;
        Ok(self.inspect())
    }
    pub fn verify(&mut self, input: VerifySegment) -> Result<SkipSnapshot, ApiError> {
        let next = self.next(input.expected_revision)?;
        let index = self.index(&input.segment_id)?;
        if self.segments[index].revision != input.annotation_revision {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Annotation revision changed",
            ));
        }
        self.segments[index].verified_revision = Some(input.annotation_revision);
        self.revision = next;
        Ok(self.inspect())
    }
    pub fn set_policy(&mut self, input: SetSkipPolicy) -> Result<SkipSnapshot, ApiError> {
        let next = self.next(input.expected_revision)?;
        self.policy = input.policy;
        self.revision = next;
        Ok(self.inspect())
    }
    pub fn evaluate(
        &mut self,
        segment_id: &str,
        intent: Intent,
        port: &impl SkipPlaybackPort,
    ) -> Result<skip_segments::Decision, ApiError> {
        let index = self.index(segment_id)?;
        let observation = port.observe()?;
        if observation.binding != self.binding {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Playback instance or media changed",
            ));
        }
        let authorized = observation.observation_authorized
            && observation.playback.connected
            && !observation.playback.has_error
            && observation.playback.idle == Some(false);
        // Unknown/untrusted observations do not mutate the pass's position history.
        if authorized {
            self.pass.observe_position(observation.playback.position_us);
        }
        let evaluation = Evaluation {
            position_us: observation.playback.position_us,
            duration_us: observation.playback.duration_us,
            playing: observation.playback.paused == Some(false),
            observation_authorized: authorized,
            seekability: observation.seekability,
        };
        Ok(skip_segments::evaluate(
            &self.segments[index],
            &self.segments,
            &self.policy,
            &self.pass,
            &evaluation,
            intent,
        ))
    }
    pub fn skip(
        &mut self,
        input: SkipRequest,
        port: &impl SkipPlaybackPort,
    ) -> Result<SkipResult, ApiError> {
        let next = self.next(input.expected_revision)?;
        self.index(&input.segment_id)?;
        if input.cancelled {
            let status = self.cancel(input.expected_revision, &input.segment_id)?;
            return Ok(SkipResult::Cancelled {
                session_revision: DecimalU64(self.revision),
                status,
            });
        }
        let decision = self.evaluate(&input.segment_id, input.intent, port)?;
        let skip_segments::Decision::Eligible { target_us } = decision else {
            return Ok(SkipResult::NotSubmitted { decision });
        };
        // Reserve before external admission: neither errors nor deferred/ambiguous replies auto-replay.
        self.pass.mark_attempted(&input.segment_id);
        self.revision = next;
        let receipt = port.submit(ControlEpisodePlayback {
            engine_instance_id: self.binding.engine_instance_id,
            control: PlaybackControl::Seek {
                position_us: target_us,
            },
        })?;
        if receipt.engine_instance_id != self.binding.engine_instance_id {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Skip admission returned a different engine; completion unobserved",
            ));
        }
        Ok(SkipResult::Admitted {
            session_revision: DecimalU64(next),
            receipt,
        })
    }
    pub fn cancel(
        &mut self,
        expected_revision: DecimalU64,
        segment_id: &str,
    ) -> Result<CancelStatus, ApiError> {
        let next = self.next(expected_revision)?;
        self.index(segment_id)?;
        let attempted = self.pass.was_attempted(segment_id);
        self.pass.suppress(segment_id);
        self.revision = next;
        Ok(if attempted {
            CancelStatus::TooLateOrUnobserved
        } else {
            CancelStatus::BeforeAdmission
        })
    }
    pub fn manual_seek_override(
        &mut self,
        expected_revision: DecimalU64,
    ) -> Result<SkipSnapshot, ApiError> {
        let next = self.next(expected_revision)?;
        self.pass.override_pass();
        self.revision = next;
        Ok(self.inspect())
    }
    /// Explicit user new-pass action only; a backward observation never calls this.
    pub fn begin_new_pass(
        &mut self,
        expected_revision: DecimalU64,
    ) -> Result<SkipSnapshot, ApiError> {
        let next = self.next(expected_revision)?;
        let pass_id = self
            .pass_id
            .checked_add(1)
            .ok_or_else(|| invalid("Skip pass identity exhausted"))?;
        self.pass = PassState::default();
        self.pass_id = pass_id;
        self.revision = next;
        Ok(self.inspect())
    }
}
fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod export_getter_tests {
    use super::*;
    #[test]
    fn saved_selection_keeps_annotation_revision_independent_of_later_session_edit() {
        let mut session = SkipSession::new(SkipBinding {
            engine_instance_id: DecimalU64(7),
            resolution_artifact_id: "a".repeat(64),
        })
        .unwrap();
        let edit = |revision, out_us| EditSegment {
            expected_revision: DecimalU64(revision),
            segment_id: "op".into(),
            kind: skip_segments::SegmentKind::Opening,
            in_us: DecimalU64(10),
            out_us: DecimalU64(out_us),
        };
        session.edit(edit(1, 20)).unwrap();
        session
            .verify(VerifySegment {
                expected_revision: DecimalU64(2),
                segment_id: "op".into(),
                annotation_revision: DecimalU64(1),
            })
            .unwrap();
        let (request_revision, pass, saved) = session.export_selection("op");
        let saved = saved.unwrap();
        assert_eq!(request_revision, DecimalU64(3));
        assert_eq!(pass, DecimalU64(1));
        assert_eq!(saved.annotation_revision, DecimalU64(1));
        assert_eq!(saved.verified_revision, DecimalU64(1));
        session.edit(edit(3, 25)).unwrap();
        assert_eq!(saved.annotation_revision, DecimalU64(1));
        assert_eq!(saved.verified_revision, DecimalU64(1));
        let (_, _, later) = session.export_selection("op");
        assert_eq!(later.unwrap().annotation_revision, DecimalU64(2));
        assert_eq!(session.export_state(), (DecimalU64(4), DecimalU64(1)));
        assert!(session.export_selection("absent").2.is_none());
    }
}
