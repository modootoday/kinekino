use super::*;
use crate::{
    application::api::PlayerSdk,
    infrastructure::{
        content_digest, frame_capture::FfmpegFrameCapture, subtitle_files::SubtitleFiles,
    },
};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
fn sha(path: &Path) -> String {
    content_digest::sha256(&mut fs::File::open(path).unwrap(), 512 * 1024 * 1024)
        .unwrap()
        .1
}

#[test]
#[ignore = "Central allocation: pinned Rust held native tool and existing local source/probe; no inference or GUI"]
fn running_frame_sdk_cancel_closes_original_native_family_before_terminal_ack() {
    let variable =
        |name| std::env::var(name).expect("explicit frame control fixture configuration");
    let source = PathBuf::from(variable("FRAME_CONTROL_MEDIA"));
    let marker = PathBuf::from(variable("FRAME_CONTROL_MARKER"));
    assert!(!marker.exists());
    assert!(!marker.with_extension("ready").exists());
    assert_eq!(
        sha(&source),
        "0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483"
    );
    let root = std::env::temp_dir().join(format!("frame-sdk-cancel-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let output = root.join("cancelled.png");
    let port = FfmpegFrameCapture::pinned(
        Path::new(&variable("FRAME_CONTROL_TOOL")),
        &variable("FRAME_CONTROL_TOOL_SHA256"),
        Path::new(&variable("FRAME_CONTROL_PROBE")),
        &variable("FRAME_CONTROL_PROBE_SHA256"),
    )
    .unwrap();
    let client = Arc::new(PlayerSdk::new(SubtitleFiles).with_frame_capture(port));
    let caps = client.execute(Request::new("capabilities"));
    assert_eq!(
        caps.data.unwrap()["frame_capture"]["in_process_sdk_execution_cooperative_cancel"],
        true
    );
    let owner = ExecutionOwner::start(
        client,
        budget(),
        Arc::new(Notify {
            signals: AtomicUsize::new(0),
            fail: false,
        }),
    )
    .unwrap();
    let handle = owner.handle();
    let mut request = Request::new("production.capture_frame");
    request.payload =
        serde_json::json!({"source_path":source,"source_pts_us":"500000","output_path":output});
    let id = handle.submit(request).unwrap();
    let cutoff = Instant::now() + Duration::from_secs(5);
    while !marker.with_extension("ready").exists() {
        assert!(Instant::now() < cutoff, "native capture never entered");
        std::thread::sleep(Duration::from_millis(10));
    }
    use std::os::windows::fs::OpenOptionsExt;
    assert!(
        fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&marker)
            .is_err(),
        "native fixture must own its marker"
    );
    assert_eq!(handle.cancel(&id).unwrap(), CancelDisposition::Requested);
    let next = handle.submit(Request::new("version")).unwrap();
    let done = terminal(&handle, &id);
    assert!(done.cancellation_requested);
    assert!(!done.queued_cancelled);
    let error = done.response.error.as_ref().unwrap();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(error.details["kinekino_process"]["physical_closed"], true);
    assert_eq!(error.details["kinekino_process"]["receipt_recorded"], true);
    let evidence = PathBuf::from(
        error.details["kinekino_process"]["evidence_root"]
            .as_str()
            .unwrap(),
    );
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(evidence.join("receipt-0000.json")).unwrap()).unwrap();
    assert_eq!(receipt["family"], "Empty");
    assert_eq!(receipt["child_exit"], 124);
    assert_eq!(receipt["forced"], true);
    let mut contents = String::new();
    fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&marker)
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();
    assert_eq!(contents, "entered");
    assert!(!output.exists());
    assert_eq!(
        sha(&source),
        "0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483"
    );
    fs::write(
        root.join("cancel-response.json"),
        serde_json::to_vec_pretty(done.response.as_ref()).unwrap(),
    )
    .unwrap();
    assert!(terminal(&handle, &next).response.ok);
    handle.acknowledge(&id).unwrap();
    handle.acknowledge(&next).unwrap();
    close(&handle);
}
