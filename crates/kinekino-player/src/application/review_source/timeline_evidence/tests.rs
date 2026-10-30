use super::*;
use crate::domain::{
    api_values::DecimalU64, media_timeline::TimeBase, review_source::timeline_evidence::*,
};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub(in crate::application::review_source) fn observations() -> TimelineObservations {
    let time_base = TimeBase::new(1, 24).unwrap();
    let capture = |requested_source_us, actual_pts| CaptureObservation {
        requested_source_us,
        actual_pts,
        time_base,
        source_sha256: "f".repeat(64),
        png_sha256: "c".repeat(64),
        engine_sha256: "e".repeat(64),
    };
    TimelineObservations {
        video_cadence: None,
        case: ReviewCase {
            package_sha256: "b".repeat(64),
            item_id: "item".into(),
            source_request_sha256: "c".repeat(64),
        },
        media: MediaReference {
            asset_id: "media".into(),
            sha256: "f".repeat(64),
        },
        reference: EvidenceReference {
            asset_id: "proof".into(),
            sha256: "a".repeat(64),
        },
        mapping_revision: DecimalU64(1),
        time_base,
        origin: DecodedOrigin {
            pts: 0,
            time_base,
            source_sha256: "f".repeat(64),
            observation_sha256: "a".repeat(64),
        },
        video_frames: (384..624)
            .map(|pts| VideoFrame { pts, duration: 1 })
            .collect(),
        pcm: PcmObservations {
            source_sha256: "f".repeat(64),
            original_data_sha256: "a".repeat(64),
            replay_data_sha256: "a".repeat(64),
            replay_engine_sha256: "e".repeat(64),
            sample_rate: 16000,
            frames: vec![
                AudioFrame {
                    sequence: 0,
                    pts: 0,
                    samples: 240000,
                },
                AudioFrame {
                    sequence: 1,
                    pts: 240000,
                    samples: 240000,
                },
            ],
            clip_start_sample: 272000,
            clip_samples: 136000,
            original_slice_sha256: "b".repeat(64),
            clip_data_sha256: "b".repeat(64),
        },
        start_capture: capture(17000000, 408),
        end_capture: capture(25500000, 612),
    }
}

#[test]
fn synthetic_clock_window_is_consistent_without_claiming_authenticity() {
    assert_eq!(
        observations().validate().unwrap(),
        MappingWindow {
            source_start_us: 17000000,
            source_end_us: 25500000,
            start_frame_pts: 408,
            end_frame_pts: 612
        }
    );
}

#[test]
fn synthetic_nonzero_origin_preserves_distinct_audio_and_video_clock_units() {
    let mut facts = observations();
    facts.origin.pts = 24;
    for frame in &mut facts.pcm.frames {
        frame.pts += 16_000;
    }
    for frame in &mut facts.video_frames {
        frame.pts += 24;
    }
    for capture in [&mut facts.start_capture, &mut facts.end_capture] {
        capture.requested_source_us += 1_000_000;
        capture.actual_pts += 24;
    }
    let mapping = facts.validate().unwrap();
    assert_eq!(
        (mapping.source_start_us, mapping.source_end_us),
        (18_000_000, 26_500_000)
    );
}

#[test]
fn synthetic_video_origin_after_earlier_video_is_rejected_in_the_shared_time_base() {
    let mut facts = observations();
    facts.origin.pts = -2;
    for frame in &mut facts.pcm.frames {
        frame.pts -= 1600;
    }
    facts.video_frames = (-12..624)
        .map(|pts| VideoFrame { pts, duration: 1 })
        .collect();
    for capture in [&mut facts.start_capture, &mut facts.end_capture] {
        capture.requested_source_us -= 100_000;
        capture.actual_pts = i64::try_from(
            facts
                .time_base
                .ticks_at_or_after(capture.requested_source_us)
                .unwrap(),
        )
        .unwrap();
    }
    assert_eq!(facts.validate(), Err(EvidenceError::InvalidVideoWindow));
}

#[test]
fn mismatched_content_clock_capture_and_resource_bounds_are_rejected() {
    let mutations: Vec<fn(&mut TimelineObservations)> = vec![
        |v| v.pcm.replay_data_sha256 = "d".repeat(64),
        |v| v.pcm.clip_data_sha256 = "d".repeat(64),
        |v| v.pcm.source_sha256 = "d".repeat(64),
        |v| v.pcm.frames[1].pts += 1,
        |v| v.pcm.frames[1].sequence = 2,
        |v| v.pcm.sample_rate = 0,
        |v| v.video_frames[1].pts += 1,
        |v| v.video_frames[1].duration = 0,
        |v| v.start_capture.actual_pts += 1,
        |v| v.end_capture.requested_source_us += 1,
        |v| v.pcm.clip_start_sample = u64::MAX,
        |v| v.video_frames.clear(),
        |v| {
            v.pcm.frames.resize(
                MAX_OBSERVED_FRAMES + 1,
                AudioFrame {
                    sequence: 0,
                    pts: 0,
                    samples: 1,
                },
            )
        },
    ];
    for mutate in mutations {
        let mut facts = observations();
        mutate(&mut facts);
        assert!(facts.validate().is_err());
    }
}

struct Authority {
    facts: TimelineObservations,
    live: AtomicUsize,
    revoked: AtomicBool,
}
struct Lease<'a>(&'a Authority);
impl Drop for Lease<'_> {
    fn drop(&mut self) {
        self.0.live.fetch_sub(1, Ordering::SeqCst);
    }
}
impl Authority {
    fn lease(&self) -> Lease<'_> {
        self.live.fetch_add(1, Ordering::SeqCst);
        Lease(self)
    }
}
impl CaseLease for Lease<'_> {
    fn case(&self) -> &ReviewCase {
        &self.0.facts.case
    }
    fn check_current(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
impl MediaLease for Lease<'_> {
    fn identity(&self) -> &MediaReference {
        &self.0.facts.media
    }
    fn check_current(&self) -> Result<(), ApiError> {
        Ok(())
    }
    fn verified_duration_us(&self) -> Option<u64> {
        None
    }
}
impl TimelineEvidenceLease for Lease<'_> {
    fn observations(&self) -> &TimelineObservations {
        &self.0.facts
    }
    fn check_current(&self) -> Result<(), ApiError> {
        if self.0.revoked.load(Ordering::SeqCst) {
            Err(binding())
        } else {
            Ok(())
        }
    }
}
impl CaseResolver for Authority {
    fn retain(&self, _: &ReviewCase) -> Result<Box<dyn CaseLease + '_>, ApiError> {
        Ok(Box::new(self.lease()))
    }
}
impl MediaResolver for Authority {
    fn retain(
        &self,
        _: &ReviewCase,
        _: &MediaReference,
    ) -> Result<Box<dyn MediaLease + '_>, ApiError> {
        Ok(Box::new(self.lease()))
    }
}
impl TimelineEvidenceResolver for Authority {
    fn retain(
        &self,
        _: &ReviewCase,
        _: &MediaReference,
        _: &EvidenceReference,
    ) -> Result<Box<dyn TimelineEvidenceLease + '_>, ApiError> {
        Ok(Box::new(self.lease()))
    }
}
#[test]
fn retained_authority_holds_all_leases_and_observes_late_revocation() {
    let owner = Authority {
        facts: observations(),
        live: AtomicUsize::new(0),
        revoked: AtomicBool::new(false),
    };
    let retained = retain(
        &owner,
        &owner,
        &owner,
        &owner.facts.case,
        &owner.facts.media,
        &owner.facts.reference,
    )
    .unwrap();
    assert_eq!(owner.live.load(Ordering::SeqCst), 3);
    assert!(!retained.playback_ready());
    owner.revoked.store(true, Ordering::SeqCst);
    assert!(retained.check_current().is_err());
    drop(retained);
    assert_eq!(owner.live.load(Ordering::SeqCst), 0);
}
#[test]
fn foreign_evidence_reference_releases_all_admission_leases() {
    let owner = Authority {
        facts: observations(),
        live: AtomicUsize::new(0),
        revoked: AtomicBool::new(false),
    };
    let foreign = EvidenceReference {
        asset_id: "foreign".into(),
        sha256: "a".repeat(64),
    };
    assert!(
        retain(
            &owner,
            &owner,
            &owner,
            &owner.facts.case,
            &owner.facts.media,
            &foreign
        )
        .is_err()
    );
    assert_eq!(owner.live.load(Ordering::SeqCst), 0);
}

#[test]
fn mapping_record_is_data_and_live_range_proof_rejects_scope_tampering_and_revocation() {
    use crate::domain::review_source::UserRange;
    let owner = Authority {
        facts: observations(),
        live: AtomicUsize::new(0),
        revoked: AtomicBool::new(false),
    };
    let proof = retain(
        &owner,
        &owner,
        &owner,
        &owner.facts.case,
        &owner.facts.media,
        &owner.facts.reference,
    )
    .unwrap();
    let record = proof.mapping_record().unwrap();
    let decoded = serde_json::from_slice(&serde_json::to_vec(&record).unwrap()).unwrap();
    let range = UserRange {
        start_us: DecimalU64(18_000_000),
        end_us: DecimalU64(19_000_000),
    };
    let plan = proof.retain_range(&decoded, &range).unwrap();
    assert_eq!(plan.plan().first_frame_pts, "432");
    assert_eq!(plan.plan().end_frame_pts, "456");
    let mut foreign = record.clone();
    foreign.media.sha256 = "e".repeat(64);
    assert!(
        matches!(proof.retain_range(&foreign, &range), Err(e) if e.code == ErrorCode::RevisionConflict)
    );
    let mut forged = record.clone();
    forged.source_end_us.0 += 1;
    assert!(
        matches!(proof.retain_range(&forged, &range), Err(e) if e.code == ErrorCode::RevisionConflict)
    );
    let outside = UserRange {
        start_us: DecimalU64(16_999_999),
        end_us: DecimalU64(19_000_000),
    };
    assert!(
        matches!(proof.retain_range(&record, &outside), Err(e) if e.code == ErrorCode::NeedsInput)
    );
    let mut noncanonical = record.clone();
    noncanonical.origin_pts = "-0".into();
    assert!(noncanonical.validate().is_err());
    noncanonical = record.clone();
    noncanonical.source_time_base.numerator = "01".into();
    assert!(noncanonical.validate().is_err());
    owner.revoked.store(true, Ordering::SeqCst);
    assert!(plan.check_current().is_err());
    assert!(proof.mapping_record().is_err());
    drop(plan);
    drop(proof);
    assert_eq!(owner.live.load(Ordering::SeqCst), 0);
}
