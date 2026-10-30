use super::*;
fn wav(data: &[u8]) -> Vec<u8> {
    let mut bytes = b"RIFF".to_vec();
    bytes.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&[1, 0, 1, 0, 128, 62, 0, 0, 0, 125, 0, 0, 2, 0, 16, 0]);
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(data);
    bytes
}
#[test]
fn pcm_comparison_requires_exact_replay_and_exact_sample_slice() {
    let data = [1, 0, 2, 0, 3, 0, 4, 0];
    let full = wav(&data);
    let clip = wav(&data[2..6]);
    let facts = compare(&full, &clip, &data, 1).unwrap();
    assert_eq!(
        (
            facts.full_samples,
            facts.clip_start_sample,
            facts.clip_samples
        ),
        (4, 1, 2)
    );
    assert_eq!(facts.full_data_sha256, facts.replay_data_sha256);
    assert!(compare(&full, &clip, &[1, 0, 2, 0, 9, 0, 4, 0], 1).is_err());
    assert!(compare(&full, &wav(&[2, 0, 9, 0]), &data, 1).is_err());
    assert!(compare(&full, &clip, &data, 3).is_err());
    assert!(compare(&full, &clip, &data, u64::MAX).is_err());
    assert!(compare(b"not wav", &clip, &data, 1).is_err());
}
#[test]
#[ignore = "explicit central allocation: reads preserved PCM evidence; no decoder execution"]
fn retained_real_pcm_lineage_preserves_originals_and_matches_fixed_sample_range() {
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let full = root.join("runtime/ai-spike/assets/sprite-clips-r1/full-mono-16k.wav");
    let clip = root.join("runtime/ai-spike/assets/sprite-clips-r1/latin-17.wav");
    let replay =
        root.join("runtime/build-cache/tmp/sprite-audio-legacy-replay-datalab-r1/decoded.s16le");
    let retained = RetainedPcmLineage::open(
        PinnedPcmFile {
            path: &full,
            sha256: "872987bc1c6296b33241e833f5a315e8bd656b2d3ed05d14034231cad4ae51a4",
        },
        PinnedPcmFile {
            path: &clip,
            sha256: "d12ec627660e5b3de88579296d45b6f3cf368c7fe6e2419ea3acbeb643c33270",
        },
        PinnedPcmFile {
            path: &replay,
            sha256: "74f30535a9ef581f3a048917941de6f41dad1c40101149c1ebe84352a1300d8b",
        },
        272000,
    )
    .unwrap();
    let facts = retained.facts();
    assert_eq!(
        (
            facts.full_samples,
            facts.clip_start_sample,
            facts.clip_samples
        ),
        (10078204, 272000, 136000)
    );
    assert_eq!(facts.full_data_sha256, facts.replay_data_sha256);
    for path in [&full, &clip, &replay] {
        assert!(std::fs::OpenOptions::new().write(true).open(path).is_err());
    }
}
