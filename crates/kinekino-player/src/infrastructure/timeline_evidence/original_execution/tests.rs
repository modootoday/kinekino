use super::*;
use crate::infrastructure::timeline_evidence::raw::RetainedRaw;
use std::path::Path;
#[test]
fn required_contract_state_cannot_be_defaulted_from_missing_json() {
    assert!(serde_json::from_str::<ExecutionContract>("{}").is_err());
    assert!(
        serde_json::from_str::<ExecutionContract>(r#"{"kind":"video_first_stream_frame_window"}"#)
            .is_err()
    );
}
#[test]
#[ignore = "explicit central allocation: reads original command/input/process evidence only"]
fn pinned_original_commands_match_explicit_contracts_and_reject_mismatches() {
    let root =
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join("runtime/verification");
    let mut guards = Vec::new();
    let mut read = |path: &str, sha: &str| {
        let mut file = RetainedRaw::open(&root.join(path), sha).unwrap();
        let text = file.read().unwrap();
        guards.push(file);
        text
    };
    let command = read(
        "sprite-audio-legacy-replay-datalab-r1/command.txt",
        "cbf1aa5d1251aea411b05ee93a4b25a4452f0f44bf5a0ded11d169137e31f822",
    );
    let inputs = read(
        "sprite-audio-legacy-replay-datalab-r1/INPUT.txt",
        "e24832b322bc0af67084870a7e877084ce3499cf4b8c2b2d0d93916f72f41a08",
    );
    let process = read(
        "sprite-audio-legacy-replay-datalab-r1/process.txt",
        "90cf6f699cbd2699a58b6836cbfbaa95017c96a446ed54dcb7c7196fe046e628",
    );
    let source = "7cfc8b8c6eeb19685048a289aca208a01356f5082b470fb9f4d933fe4b2087ef";
    let engine = "71414777922d05f2639a5cf4a0f9a405f169809cf459000a6f976c9add61f060";
    let full = "872987bc1c6296b33241e833f5a315e8bd656b2d3ed05d14034231cad4ae51a4";
    let contract = ExecutionContract::AudioCopyTsMono16kS16FirstStream;
    audio(&contract, &command, &inputs, &process, source, engine, full).unwrap();
    assert!(
        audio(
            &contract,
            &command.replace("async=0", "async=1"),
            &inputs,
            &process,
            source,
            engine,
            full
        )
        .is_err()
    );
    assert!(
        audio(
            &contract,
            &command.replace("0:a:0", "0:a:1"),
            &inputs,
            &process,
            source,
            engine,
            full
        )
        .is_err()
    );
    assert!(
        audio(
            &contract,
            &command,
            &inputs,
            &process.replace("terminal=true", "terminal=false"),
            source,
            engine,
            full
        )
        .is_err()
    );
    let command = read(
        "sprite-frame-pts-datalab-r1/command.txt",
        "daf80946f9b2bbe5db37d0c52a93b1bf700fb0b3c9627e03ab184714998ff5b1",
    );
    let inputs = read(
        "sprite-frame-pts-datalab-r1/INPUT.txt",
        "54da2231adcf268a21cf195d46432caf4730f5c906794e6ce685685d72788ada",
    );
    let process = read(
        "sprite-frame-pts-datalab-r1/process.txt",
        "5a35569a7f072d4a3ac33e158fac2968530d52460291205d0ca41aaa6911e5aa",
    );
    let probe = "a6618e99bb58869ded3c6f37b53aa1a8d701c3591dbb7b5b317d47369c112be2";
    video(
        &ExecutionContract::VideoFirstStreamFrameWindow {
            interval: "16%26".into(),
        },
        &command,
        &inputs,
        &process,
        source,
        probe,
    )
    .unwrap();
    assert!(
        video(
            &ExecutionContract::VideoFirstStreamFrameWindow {
                interval: "17%26".into()
            },
            &command,
            &inputs,
            &process,
            source,
            probe
        )
        .is_err()
    );
    assert!(
        video(
            &ExecutionContract::VideoFirstStreamFrameWindow {
                interval: "16%26".into()
            },
            &command.replace("v:0", "v:1"),
            &inputs,
            &process,
            source,
            probe
        )
        .is_err()
    );
}
