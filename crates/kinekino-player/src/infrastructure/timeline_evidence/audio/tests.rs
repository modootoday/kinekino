use super::*;
#[test]
fn live_side_diagnostics_preserve_frames_and_do_not_weaken_archive_or_frame_validation() {
    let first = row(0, 0, 2);
    let second = row(1, 2, 3);
    let matrix = "[Parsed_ashowinfo_2 @ 001] side data - matrix encoding: none";
    let downmix = "[Parsed_ashowinfo_2 @ 001] side data - downmix: preferred downmix type - unknown Mix levels: center 0.707107";
    let raw = format!("{first}\n{matrix}\n{downmix}\n{second}");
    assert_eq!(
        parse_live(&raw).unwrap(),
        parse(&format!("{first}\n{second}")).unwrap()
    );
    assert!(parse(&raw).is_err());
    for invalid in [
        format!("{matrix}\n{first}"),
        format!("{first}\n{matrix}\n{matrix}"),
        format!("{first}\n{matrix} n:1"),
        format!("{first}\n[Parsed_ashowinfo_2 @ 001] side data - unknown: x"),
        format!("{first}\n{}", second.replace("n:1 ", "")),
        format!("{first}\n{matrix}\n{}", row(1, 3, 3)),
    ] {
        assert!(parse_live(&invalid).is_err(), "accepted {invalid}");
    }
}
fn row(n: u64, pts: i64, samples: u32) -> String {
    format!(
        "[Parsed_ashowinfo_2 @ 001] n:{n} pts:{pts} pts_time:ignored fmt:s16 channels:1 rate:16000 nb_samples:{samples}"
    )
}
#[test]
fn integer_clock_retains_negative_origin_and_ignores_float_time() {
    let frames = parse(&format!("banner\n{}\n{}", row(0, -2, 2), row(1, 0, 3))).unwrap();
    assert_eq!(
        frames,
        vec![
            AudioFrame {
                sequence: 0,
                pts: -2,
                samples: 2
            },
            AudioFrame {
                sequence: 1,
                pts: 0,
                samples: 3
            }
        ]
    );
}
#[test]
fn malformed_duplicate_missing_and_discontinuous_rows_are_rejected() {
    let first = row(0, 0, 2);
    for bad in [
        row(1, 3, 3),
        row(2, 2, 3),
        row(1, 2, 0),
        row(1, 2, 3).replace("rate:16000", "rate:44100"),
        format!("{} pts:2", row(1, 2, 3)),
        row(1, 2, 3).replace("pts:2", "pts:+2"),
        row(1, 2, 3).replace("pts:2", "pts:2.0"),
        row(1, 2, 3).replace("n:1 ", ""),
    ] {
        assert!(parse(&format!("{first}\n{bad}")).is_err(), "accepted {bad}");
    }
    assert!(parse("no observations").is_err());
}
#[test]
fn oversized_row_is_rejected_before_frame_allocation() {
    assert!(parse(&format!("{} {}", row(0, 0, 2), "x".repeat(4096))).is_err());
}
#[test]
#[ignore = "explicit central allocation: reads preserved DATALAB decoder evidence; no decoder execution"]
fn retained_real_legacy_audio_observation_matches_preserved_clock() {
    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .join("runtime/verification/sprite-audio-legacy-replay-datalab-r1/decode.stderr");
    let hash = "ec1a5907637c54a4da4a5184fbb9f72b7f9e1d1dacca229d1b1d3a17a755345c";
    let evidence = RetainedAudioClock::open(&path, hash).unwrap();
    assert_eq!(evidence.frames().len(), 27128);
    assert_eq!(evidence.frames()[0].pts, 0);
    assert_eq!(
        evidence
            .frames()
            .iter()
            .map(|frame| u64::from(frame.samples))
            .sum::<u64>(),
        10078204
    );
    assert_eq!(evidence.sha256(), hash);
    assert!(
        std::fs::OpenOptions::new().write(true).open(&path).is_err(),
        "custody must deny writers"
    );
    assert!(RetainedAudioClock::open(&path, &"a".repeat(64)).is_err());
}
