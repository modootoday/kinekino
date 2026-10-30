use super::*;
#[test]
fn rational_validation_and_canonical_equivalence() {
    assert_eq!(
        TimeBase::parse("2/24576").unwrap(),
        TimeBase::new(1, 12288).unwrap()
    );
    assert_eq!(TimeBase::parse("0002/0004").unwrap().ratio(), (1, 2));
    for value in ["0/1", "1/0", "-1/2", "+1/2", "1/2/3", " 1/2", "1/", "/2"] {
        assert_eq!(TimeBase::parse(value), Err(TimelineError::InvalidTimeBase));
    }
    assert_eq!(TimeBase::new(u128::MAX, u128::MAX).unwrap().ratio(), (1, 1));
}
#[test]
fn signed_origin_is_explicit_and_rounding_reports_precision() {
    let transform = TimelineTransform::new(TimeBase::new(1, 90000).unwrap(), -9000);
    assert_eq!(
        transform.pts_to_us(-9000, Rounding::Down).unwrap(),
        Conversion {
            value: 0,
            exact: true
        }
    );
    assert_eq!(
        transform.pts_to_us(0, Rounding::Down).unwrap(),
        Conversion {
            value: 100000,
            exact: true
        }
    );
    assert_eq!(
        transform.pts_to_us(-8999, Rounding::Down).unwrap(),
        Conversion {
            value: 11,
            exact: false
        }
    );
    assert_eq!(
        transform.pts_to_us(-8999, Rounding::Up).unwrap(),
        Conversion {
            value: 12,
            exact: false
        }
    );
    assert_eq!(
        transform.pts_to_us(-9001, Rounding::Up),
        Err(TimelineError::BeforeOrigin)
    );
    assert_eq!(transform.us_to_pts(100000, Rounding::Up).unwrap().value, 0);
}
#[test]
fn capture_threshold_and_inverse_use_the_same_fractional_boundary() {
    let base = TimeBase::new(1, 12288).unwrap();
    let transform = TimelineTransform::new(base, 0);
    for (micros, expected) in [(583333, 7168), (583334, 7169), (500000, 6144)] {
        assert_eq!(base.ticks_at_or_after(micros).unwrap(), expected);
        assert_eq!(
            transform.us_to_pts(micros, Rounding::Up).unwrap().value,
            expected as i64
        );
    }
    assert_eq!(
        transform.us_to_pts(1, Rounding::Down).unwrap(),
        Conversion {
            value: 0,
            exact: false
        }
    );
    assert_eq!(
        transform.us_to_pts(1, Rounding::Up).unwrap(),
        Conversion {
            value: 1,
            exact: false
        }
    );
}
#[test]
fn overflow_is_an_error_and_extreme_signed_differences_do_not_wrap() {
    let transform = TimelineTransform::new(TimeBase::new(1, 1).unwrap(), i64::MIN);
    assert_eq!(
        transform.pts_to_us(i64::MAX, Rounding::Down),
        Err(TimelineError::Overflow)
    );
    assert_eq!(
        TimelineTransform::new(TimeBase::new(1, 1_000_000).unwrap(), i64::MAX)
            .us_to_pts(1, Rounding::Up),
        Err(TimelineError::Overflow)
    );
    assert_eq!(
        TimeBase::new(u128::MAX, 1).unwrap().ticks_at_or_after(1),
        Err(TimelineError::Overflow)
    );
    assert_eq!(
        TimeBase::new(1, u128::MAX)
            .unwrap()
            .ticks_at_or_after(u64::MAX),
        Err(TimelineError::Overflow)
    );
}
#[test]
fn directional_rounding_brackets_requested_time_across_timebases() {
    for base in [
        TimeBase::new(1, 90000).unwrap(),
        TimeBase::new(1001, 30000).unwrap(),
        TimeBase::new(1, 12288).unwrap(),
    ] {
        let transform = TimelineTransform::new(base, -1000);
        for micros in [0, 1, 11, 12, 33366, 33367, 583333, 583334, 1_000_000] {
            let down = transform.us_to_pts(micros, Rounding::Down).unwrap();
            let up = transform.us_to_pts(micros, Rounding::Up).unwrap();
            assert!(down.value <= up.value && up.value - down.value <= 1);
            assert!(
                transform
                    .pts_to_us(down.value, Rounding::Down)
                    .unwrap()
                    .value
                    <= micros
            );
            assert!(transform.pts_to_us(up.value, Rounding::Up).unwrap().value >= micros);
        }
    }
}
