use super::*;
use crate::infrastructure::timeline_evidence::raw::RetainedRaw;
use std::path::Path;
#[test]
fn capture_contract_must_be_explicit() {
    assert!(serde_json::from_str::<CaptureContract>("{}").is_err());
    assert!(serde_json::from_str::<CaptureContract>("null").is_err());
}
#[test]
#[ignore = "explicit central allocation: reads preserved SDK capture records/PNG only"]
fn preserved_three_capture_envelopes_match_source_tool_cli_and_png() {
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .join("runtime/build-cache/tmp/sprite-capture-cli-11384");
    let mut guards = Vec::new();
    let mut read = |name: &str, sha: &str| {
        let mut file = RetainedRaw::open(&root.join(name), sha).unwrap();
        let bytes = file.read_bytes().unwrap();
        guards.push(file);
        bytes
    };
    let inputs = read(
        "INPUT.json",
        "2f0f6c895148cd365fa7fbc4bbc215da0a0745f9f49065a81949dfbf05b882d6",
    );
    for (label, us, ticks, request_sha, response_sha, process_sha, png_sha) in [
        (
            "0s",
            0,
            0,
            "26e522acf499184862ab198be374d4e2ac364ff87b1258725fc0ca9662d51a16",
            "ebb57c402bb1d07c86e29a846797005915b8394905acda590d22bcb3adfeccfc",
            "862c5c47c71e8ef048f80491a0b3d7764a46b646b1f91265196a5b6d0da841b3",
            "6c2ac82030e7a1a584e85ef3053863287c376a00133f2e9462f480e693baa38f",
        ),
        (
            "17s",
            17000000,
            208896,
            "a4aa2dbd7c3ced1bfd8dbd7252d03f402f1c3221e27001f6ce561c06dd783550",
            "b6244762d523e75705b990c51e9dee384c6db71b347167836b0db4b7019a3463",
            "fc997cd6cbe1bc41c4f6842b477c0b31feaa2576b2d923b5b02162db2dae1c28",
            "ac5d6d071c716a943852261bad441a695aa7cb53583340a8a8d6349ff108eed0",
        ),
        (
            "25-5s",
            25500000,
            313344,
            "0605e5f5e4cef0cacc307f31eeff476aed0d2801370a659a5c97e1f494e27c7f",
            "d00367a456be8a9a0ab6cb18c1d17c4d5b80dbd31f48baefbc2bf8c84d2c8c54",
            "601ff39334f85bb13757c68d6a61135f57fa920920ac5835cedecee29c7afcea",
            "bf0e3d8ccfbec31fe03745fc45521bbaeff0d69253f594e324643e431b2e360f",
        ),
    ] {
        let request = read(&format!("{label}.request.json"), request_sha);
        let response = read(&format!("{label}.stdout.json"), response_sha);
        let process = read(&format!("{label}.process.json"), process_sha);
        let png = read(&format!("{label}.png"), png_sha);
        let expected = CaptureExpectation {
            source_sha: "7cfc8b8c6eeb19685048a289aca208a01356f5082b470fb9f4d933fe4b2087ef",
            engine_sha: "09948d4cdd0650da6ff5a87577469f2a218dc2615ae379f8f734d24c49de0f73",
            probe_sha: "a6618e99bb58869ded3c6f37b53aa1a8d701c3591dbb7b5b317d47369c112be2",
            producer_cli_sha: "8d06f1919d05abedcdaa799c9a7b43fa42cf8f14b5c092fd6eb4d6d058cf99f9",
            requested_us: us,
            actual_pts: ticks,
            time_base: TimeBase::new(1, 12288).unwrap(),
            png_sha,
            png_bytes: png.len() as u64,
            dimensions: crate::infrastructure::frame_capture::validate_png(&png).unwrap(),
        };
        verify(
            &CaptureContract::SdkFirstSourceFrameAtOrAfter,
            &inputs,
            &request,
            &response,
            &process,
            &expected,
        )
        .unwrap();
        let mut changed: Value = serde_json::from_slice(&request).unwrap();
        changed["payload"]["source_pts_us"] = (us + 1).to_string().into();
        assert!(
            verify(
                &CaptureContract::SdkFirstSourceFrameAtOrAfter,
                &inputs,
                &serde_json::to_vec(&changed).unwrap(),
                &response,
                &process,
                &expected
            )
            .is_err()
        );
    }
}
