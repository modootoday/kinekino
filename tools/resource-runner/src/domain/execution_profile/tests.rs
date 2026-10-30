use super::*;

#[test]
fn default_selection_preserves_first_two_and_noncontiguous_masks() {
    for mask in [0, 1, 2, 3, 0b101010101, usize::MAX] {
        assert_eq!(
            Profile::Conservative2.select(mask),
            (mask != 0).then(|| first_two(mask))
        );
    }
    assert_eq!(Profile::Build4.select(0b101010101), Some(0b1010101));
    assert_eq!(Profile::Build4.select(0b10101), None);
}
#[test]
fn profiles_are_explicit_and_unknown_names_never_fall_back() {
    assert_eq!(Profile::parse("build4"), Some(Profile::Build4));
    assert_eq!(
        Profile::parse("conservative2"),
        Some(Profile::Conservative2)
    );
    for unknown in ["", "Build4", "build8", "--topology"] {
        assert_eq!(Profile::parse(unknown), None);
    }
}
#[test]
fn strict_cpu_boundary_and_inclusive_memory_boundary_are_exact() {
    let sample = Interval {
        busy: 49,
        total: 100,
        minimum_free_bytes: MIN_FREE_BYTES,
        elapsed_ms: 1000,
    };
    assert_eq!(sample.rejection(), None);
    assert_eq!(
        Interval { busy: 50, ..sample }.rejection(),
        Some("cpu_not_below_50_percent")
    );
    assert_eq!(
        Interval {
            minimum_free_bytes: MIN_FREE_BYTES - 1,
            ..sample
        }
        .rejection(),
        Some("free_ram_below_16_gib")
    );
    assert_eq!(
        Interval {
            elapsed_ms: 999,
            ..sample
        }
        .rejection(),
        Some("sample_interval_too_short")
    );
    assert_eq!(
        Interval {
            busy: u64::MAX / 2,
            total: u64::MAX,
            ..sample
        }
        .rejection(),
        None
    );
}
#[test]
fn reset_zero_overflow_and_inconsistent_cpu_counters_are_rejected() {
    let start = Snapshot {
        idle: 10,
        kernel: 20,
        user: 30,
        free_bytes: MIN_FREE_BYTES,
    };
    assert!(Interval::between(start, start, 1000).is_none());
    assert!(
        Interval::between(
            start,
            Snapshot {
                kernel: 19,
                ..start
            },
            1000
        )
        .is_none()
    );
    assert!(
        Interval::between(
            start,
            Snapshot {
                idle: 100,
                kernel: 21,
                ..start
            },
            1000
        )
        .is_none()
    );
    assert!(
        Interval::between(
            Snapshot {
                idle: 0,
                kernel: 0,
                user: 0,
                ..start
            },
            Snapshot {
                idle: 0,
                kernel: u64::MAX,
                user: 1,
                ..start
            },
            1000
        )
        .is_none()
    );
}
