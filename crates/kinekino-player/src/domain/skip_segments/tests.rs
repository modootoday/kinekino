use super::*;
use crate::domain::{api_values::DecimalU64, playback_control::MAX_SEEK_US};

fn segment(id: &str, start: u64, end: u64) -> ManualSegment {
    ManualSegment {
        id: id.into(),
        revision: DecimalU64(1),
        kind: SegmentKind::Opening,
        in_us: DecimalU64(start),
        out_us: DecimalU64(end),
        origin: SegmentOrigin::Manual,
        timebase: SegmentTimebase::EnginePlayback,
        verified_revision: Some(DecimalU64(1)),
    }
}
fn observation(position: u64) -> Evaluation {
    Evaluation {
        position_us: Some(DecimalU64(position)),
        duration_us: Some(DecimalU64(100_000_000)),
        playing: true,
        observation_authorized: true,
        seekability: Seekability::Yes,
    }
}
fn auto() -> SkipPolicy {
    SkipPolicy {
        opening: SkipMode::Auto,
        ending: SkipMode::Manual,
    }
}

#[test]
fn half_open_notice_and_known_seek_precision_boundaries() {
    let segment = segment("op", 10_000_000, 20_000_000);
    let decision = |position| {
        evaluate(
            &segment,
            std::slice::from_ref(&segment),
            &auto(),
            &PassState::default(),
            &observation(position),
            Intent::Automatic,
        )
    };
    assert_eq!(
        decision(6_999_999),
        Decision::Blocked {
            reason: BlockReason::OutsideSegment
        }
    );
    assert_eq!(
        decision(7_000_000),
        Decision::Notice {
            remaining_us: DecimalU64(3_000_000)
        }
    );
    assert_eq!(
        decision(10_000_000),
        Decision::Eligible {
            target_us: DecimalU64(20_000_000)
        }
    );
    assert_eq!(
        decision(20_000_000),
        Decision::Blocked {
            reason: BlockReason::OutsideSegment
        }
    );
    assert!(
        super::tests::segment("max", 0, MAX_SEEK_US)
            .validate()
            .is_ok()
    );
    assert!(
        super::tests::segment("overflow", 0, u64::MAX)
            .validate()
            .is_err()
    );
    assert!(super::tests::segment("empty", 1, 1).validate().is_err());
}

#[test]
fn independent_modes_and_manual_origin_confirmation() {
    let mut segment = segment("ed", 10_000_000, 20_000_000);
    segment.kind = SegmentKind::Ending;
    assert_eq!(
        evaluate(
            &segment,
            std::slice::from_ref(&segment),
            &auto(),
            &PassState::default(),
            &observation(15_000_000),
            Intent::Automatic
        ),
        Decision::Blocked {
            reason: BlockReason::ManualOnly
        }
    );
    assert_eq!(
        evaluate(
            &segment,
            std::slice::from_ref(&segment),
            &auto(),
            &PassState::default(),
            &observation(15_000_000),
            Intent::Manual
        ),
        Decision::Eligible {
            target_us: DecimalU64(20_000_000)
        }
    );
    assert_eq!(segment.origin, SegmentOrigin::Manual);
    assert_eq!(
        evaluate(
            &segment,
            std::slice::from_ref(&segment),
            &SkipPolicy {
                opening: SkipMode::Auto,
                ending: SkipMode::Off
            },
            &PassState::default(),
            &observation(15_000_000),
            Intent::Manual
        ),
        Decision::Blocked {
            reason: BlockReason::Off
        }
    );
    segment.verified_revision = None;
    assert_eq!(
        evaluate(
            &segment,
            std::slice::from_ref(&segment),
            &auto(),
            &PassState::default(),
            &observation(15_000_000),
            Intent::Manual
        ),
        Decision::Blocked {
            reason: BlockReason::Unverified
        }
    );
}

#[test]
fn overlap_blocks_but_adjacent_intervals_do_not() {
    let first = segment("op", 10_000_000, 20_000_000);
    let overlapping = segment("ed", 19_999_999, 30_000_000);
    let adjacent = segment("adjacent", 20_000_000, 30_000_000);
    assert_eq!(
        evaluate(
            &first,
            &[first.clone(), overlapping],
            &auto(),
            &PassState::default(),
            &observation(15_000_000),
            Intent::Automatic
        ),
        Decision::Blocked {
            reason: BlockReason::Overlap
        }
    );
    assert!(matches!(
        evaluate(
            &first,
            &[first.clone(), adjacent],
            &auto(),
            &PassState::default(),
            &observation(15_000_000),
            Intent::Automatic
        ),
        Decision::Eligible { .. }
    ));
}

#[test]
fn unknown_duration_seekability_pause_and_untrusted_observation_block() {
    let segment = segment("op", 10_000_000, 20_000_000);
    for (reason, current) in [
        (
            BlockReason::DurationUnknown,
            Evaluation {
                duration_us: None,
                ..observation(15_000_000)
            },
        ),
        (
            BlockReason::SeekUnknown,
            Evaluation {
                seekability: Seekability::Unknown,
                ..observation(15_000_000)
            },
        ),
        (
            BlockReason::NotSeekable,
            Evaluation {
                seekability: Seekability::No,
                ..observation(15_000_000)
            },
        ),
        (
            BlockReason::PausedOrUnknown,
            Evaluation {
                playing: false,
                ..observation(15_000_000)
            },
        ),
        (
            BlockReason::ObservationUnknown,
            Evaluation {
                observation_authorized: false,
                ..observation(15_000_000)
            },
        ),
    ] {
        assert_eq!(
            evaluate(
                &segment,
                std::slice::from_ref(&segment),
                &auto(),
                &PassState::default(),
                &current,
                Intent::Automatic
            ),
            Decision::Blocked { reason }
        );
    }
}
