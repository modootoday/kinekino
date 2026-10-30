//! Real pre-existing local sample and explicit native tools, centrally scheduled only.
use kinekino_player::application::api::PlaybackClient;
#[path = "../support/review_discovery_config.rs"]
mod config;
use super::fixture::{Fixture, Host, request};
use kinekino_player::{
    application::api::RequestContext,
    domain::{api_values::DecimalU64, frame_capture::CaptureFrame},
    infrastructure::content_digest,
};
use serde_json::json;
use std::{fs, path::Path};
fn sha(path: &Path) -> String {
    content_digest::sha256(&mut fs::File::open(path).unwrap(), 512 * 1024 * 1024)
        .unwrap()
        .1
}

#[test]
#[ignore = "Central allocation: existing pinned local media/FFmpeg; no download, inference or GUI"]
fn explicit_job_host_frame_cli_and_typed_client_share_pins_and_eof_cleanup() {
    kinekino_platform::install();
    let fixture = Fixture::named("frame-host");
    let source = std::path::PathBuf::from(
        std::env::var_os("FRAME_HOST_MEDIA").expect("explicit existing media"),
    );
    assert_eq!(
        sha(&source),
        "0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483"
    );
    let options = config::frame_options();
    assert!(!options.is_empty(), "explicit tool pins are required");
    let host = Host::start_with_options(&fixture, "frame", &fixture.root.join("store"), &options);
    let ready: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.root.join("logs/frame-ready.stdout")).unwrap())
            .unwrap();
    assert_eq!(ready["frame_capture_configured"], true);
    let caps = fixture.call(
        "frame-caps",
        &request("capabilities", json!({})),
        Some(&host.endpoint),
        None,
        0,
    );
    assert_eq!(
        caps["data"]["frame_capture"]["dependency"]["available"],
        true
    );
    assert_eq!(caps["data"]["jobs"]["execution_host_available"], true);
    let output = fixture.root.join("frame.png");
    let capture = request(
        "production.capture_frame",
        json!({"source_path":source,"source_pts_us":"500000","output_path":output}),
    );
    let response = fixture.call("capture", &capture, Some(&host.endpoint), None, 0);
    assert_eq!(response["data"]["actual_source_pts_us"], "500000");
    assert_eq!(response["data"]["width"], 720);
    assert_eq!(response["data"]["height"], 1280);
    let expected = "0b73162584f488c517833254e029ccb4d26f6c966d126d7de7e63f1dd7b9de2a";
    assert_eq!(response["data"]["output_sha256"], expected);
    assert_eq!(sha(&output), expected);
    assert_eq!(
        fixture.call("existing", &capture, Some(&host.endpoint), None, 5)["error"]["code"],
        "revision_conflict"
    );
    let client = fixture.endpoint(&host);
    let typed = client
        .capture_frame(
            RequestContext::default(),
            CaptureFrame {
                source_path: source.to_string_lossy().into(),
                source_pts_us: DecimalU64(500000),
                output_path: fixture.root.join("typed.png").to_string_lossy().into(),
            },
        )
        .unwrap();
    assert_eq!(typed.output_sha256.as_deref(), Some(expected));
    assert_eq!(sha(&fixture.root.join("typed.png")), expected);
    assert_eq!(
        sha(&source),
        "0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483"
    );
    assert_eq!(sha(&output), expected);
    drop(client);
    host.stop();
}
