//! Consistency of unchanged observed PTS/durations. A rate hint is never authority.
use super::timeline_evidence::{EvidenceError, MAX_OBSERVED_FRAMES, VideoFrame};
use crate::domain::media_timeline::TimeBase;
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ObservedCadence {
    pub rate_numerator: u64,
    pub rate_denominator: u64,
}
pub fn validate(
    frames: &[VideoFrame],
    base: TimeBase,
    cadence: Option<ObservedCadence>,
) -> Result<(), EvidenceError> {
    if frames.is_empty() || frames.len() > MAX_OBSERVED_FRAMES {
        return Err(EvidenceError::InvalidVideoWindow);
    }
    let Some(rate) = cadence else {
        let mut previous_end = None;
        for frame in frames {
            if frame.duration == 0 || previous_end.is_some_and(|end| end != i128::from(frame.pts)) {
                return Err(EvidenceError::InvalidVideoWindow);
            }
            previous_end = Some(i128::from(frame.pts) + i128::from(frame.duration));
        }
        return Ok(());
    };
    if frames.len() < 2
        || rate.rate_numerator == 0
        || rate.rate_denominator == 0
        || u128::from(rate.rate_numerator) > 240 * u128::from(rate.rate_denominator)
    {
        return Err(EvidenceError::InvalidVideoWindow);
    }
    let (bn, bd) = base.ratio();
    let p = u128::from(rate.rate_denominator)
        .checked_mul(bd)
        .and_then(|value| i128::try_from(value).ok())
        .ok_or(EvidenceError::Overflow)?;
    let q = u128::from(rate.rate_numerator)
        .checked_mul(bn)
        .and_then(|value| i128::try_from(value).ok())
        .ok_or(EvidenceError::Overflow)?;
    if q == 0 || p < q {
        return Err(EvidenceError::InvalidVideoWindow);
    }
    let floor = p / q;
    let ceil = floor + i128::from(p % q != 0);
    // There must be ONE common phase phi in [0,q), not a new tolerance for each frame:
    // (pts[n]-pts[0])*q <= phi+n*p < ((pts[n]-pts[0])+1)*q.
    let (mut lower, mut upper) = (0i128, q);
    let mut previous = None;
    for (index, frame) in frames.iter().enumerate() {
        if ![floor, ceil].contains(&i128::from(frame.duration))
            || previous.is_some_and(|pts| frame.pts <= pts)
        {
            return Err(EvidenceError::InvalidVideoWindow);
        }
        let delta = (i128::from(frame.pts) - i128::from(frames[0].pts))
            .checked_mul(q)
            .and_then(|value| {
                (index as i128)
                    .checked_mul(p)
                    .and_then(|ideal| value.checked_sub(ideal))
            })
            .ok_or(EvidenceError::Overflow)?;
        lower = lower.max(delta);
        upper = upper.min(delta.checked_add(q).ok_or(EvidenceError::Overflow)?);
        if lower >= upper {
            return Err(EvidenceError::InvalidVideoWindow);
        }
        previous = Some(frame.pts);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn raw() -> Vec<VideoFrame> {
        [98958, 99000, 99042, 99083, 99125, 99167]
            .into_iter()
            .map(|pts| VideoFrame { pts, duration: 41 })
            .collect()
    }
    #[test]
    fn quantized_phase_proof_preserves_raw_and_does_not_relax_legacy() {
        let frames = raw();
        let before = frames.clone();
        let base = TimeBase::new(1, 1000).unwrap();
        assert!(validate(&frames, base, None).is_err());
        validate(
            &frames,
            base,
            Some(ObservedCadence {
                rate_numerator: 24,
                rate_denominator: 1,
            }),
        )
        .unwrap();
        assert_eq!(frames, before);
        let exact = [
            VideoFrame {
                pts: 0,
                duration: 1000,
            },
            VideoFrame {
                pts: 1000,
                duration: 1000,
            },
        ];
        validate(&exact, TimeBase::new(1, 24000).unwrap(), None).unwrap();
    }
    #[test]
    fn missing_duplicate_drift_wrong_duration_rate_and_overflow_are_rejected() {
        let base = TimeBase::new(1, 1000).unwrap();
        let rate = Some(ObservedCadence {
            rate_numerator: 24,
            rate_denominator: 1,
        });
        let mut missing = raw();
        missing.remove(2);
        assert!(validate(&missing, base, rate).is_err());
        let mut duplicate = raw();
        duplicate.insert(2, duplicate[1].clone());
        assert!(validate(&duplicate, base, rate).is_err());
        let mut drift = raw();
        drift[3].pts += 2;
        assert!(validate(&drift, base, rate).is_err());
        let mut duration = raw();
        duration[3].duration = 40;
        assert!(validate(&duration, base, rate).is_err());
        assert!(
            validate(
                &raw(),
                base,
                Some(ObservedCadence {
                    rate_numerator: 25,
                    rate_denominator: 1
                })
            )
            .is_err()
        );
        assert!(validate(&raw(), TimeBase::new(u128::MAX, 1).unwrap(), rate).is_err());
    }
}
