use super::SkipRange;

#[test]
fn a_range_needs_a_finite_start_before_its_end() {
    assert!(SkipRange::new(10.0, 90.0).is_some());
    assert!(SkipRange::new(0.0, 0.5).is_some());
    assert!(SkipRange::new(90.0, 90.0).is_none(), "empty");
    assert!(SkipRange::new(90.0, 10.0).is_none(), "reversed");
    assert!(
        SkipRange::new(-1.0, 10.0).is_none(),
        "before the start of the file"
    );
    assert!(SkipRange::new(f64::NAN, 10.0).is_none());
    assert!(SkipRange::new(0.0, f64::INFINITY).is_none());
}

#[test]
fn contains_the_start_but_not_the_end() {
    let r = SkipRange::new(10.0, 90.0).unwrap();
    assert_eq!((r.start(), r.end()), (10.0, 90.0));
    assert!(!r.contains(9.99));
    assert!(r.contains(10.0));
    assert!(r.contains(89.9));
    assert!(
        !r.contains(90.0),
        "at the end there is nothing left to skip"
    );
    assert!(!r.contains(f64::NAN));
}

#[test]
fn milliseconds_round_trip_floors_fractional_seconds() {
    let r = SkipRange::new(10.5, 90.25).unwrap();
    assert_eq!(r.to_ms(), (10_500, 90_250));
    let back = SkipRange::from_ms(10_500, 90_250).unwrap();
    assert_eq!((back.start(), back.end()), (10.5, 90.25));
    // 10.999s → 10999 ms (floor), not 11000.
    assert_eq!(SkipRange::new(10.999, 20.0).unwrap().to_ms().0, 10_999);
    assert_eq!(SkipRange::from_ms(10, 10), None, "empty after ms");
    assert_eq!(SkipRange::from_ms(90, 10), None, "reversed after ms");
}
