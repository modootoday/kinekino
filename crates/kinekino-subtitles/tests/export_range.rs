use kinekino_media_contracts::errors::ErrorCode;
use kinekino_subtitles::{
    application::{convert, export_range},
    infrastructure::subtitle_files::SubtitleFiles,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const SOURCE: &str =
    "1\n00:00:01,000 --> 00:00:03,000\n한글 日本語\n\n2\n00:00:04,000 --> 00:00:06,000\n둘째\n\n";

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "subtitle-range-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("source.srt"), SOURCE).unwrap();
        Self(path)
    }
    fn request(&self, start: &str, end: &str) -> Value {
        json!({"input_path":self.0.join("source.srt"),"output_path":self.0.join("out.vtt"),"input_format":"srt","output_format":"vtt","range":{"in_us":start,"out_us":end}})
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn real_files_clip_rebase_preserve_original_and_refuse_overwrite() {
    let fixture = Fixture::new();
    let request = fixture.request("2000000", "5000000");
    let result = export_range::execute(&SubtitleFiles, &request).unwrap();
    assert_eq!(result["cue_count"], 2);
    assert_eq!(result["overwrite"], false);
    let output = fs::read_to_string(fixture.0.join("out.vtt")).unwrap();
    assert_eq!(
        output,
        "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\n한글 日本語\n\n00:00:02.000 --> 00:00:03.000\n둘째\n\n"
    );
    assert_eq!(
        export_range::execute(&SubtitleFiles, &request)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        fs::read_to_string(fixture.0.join("out.vtt")).unwrap(),
        output
    );
    assert_eq!(
        fs::read_to_string(fixture.0.join("source.srt")).unwrap(),
        SOURCE
    );
}

#[test]
fn precision_loss_requires_approval_before_publication() {
    let fixture = Fixture::new();
    let mut request = fixture.request("1000001", "5000000");
    let error = export_range::execute(&SubtitleFiles, &request).unwrap_err();
    assert_eq!(error.code, ErrorCode::NeedsInput);
    assert_eq!(
        error.details["losses"][0]["code"],
        "timestamp_precision_reduced"
    );
    assert!(!fixture.0.join("out.vtt").exists());
    request["allow_loss"] = json!(true);
    let result = export_range::execute(&SubtitleFiles, &request).unwrap();
    assert_eq!(result["losses"], error.details["losses"]);
    assert_eq!(
        fs::read_to_string(fixture.0.join("source.srt")).unwrap(),
        SOURCE
    );
}

#[test]
fn invalid_empty_collapsed_unknown_and_numeric_bounds_never_publish() {
    let fixture = Fixture::new();
    for (start, end) in [
        ("3000000", "4000000"),
        ("1000000", "1000000"),
        ("1000001", "1000999"),
        ("0", "18446744073709551616"),
    ] {
        let mut request = fixture.request(start, end);
        request["allow_loss"] = json!(true);
        assert_eq!(
            export_range::execute(&SubtitleFiles, &request)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert!(!fixture.0.join("out.vtt").exists());
    }
    let mut request = fixture.request("0", "5000000");
    request["range"]["in_us"] = json!(0);
    assert_eq!(
        export_range::execute(&SubtitleFiles, &request)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    request["range"]["in_us"] = json!("0");
    request["unexpected"] = json!(true);
    assert_eq!(
        export_range::execute(&SubtitleFiles, &request)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert!(!fixture.0.join("out.vtt").exists());
}

#[test]
fn conversion_contract_and_loss_gate_remain_unchanged() {
    let fixture = Fixture::new();
    let mut request = fixture.request("0", "5000000");
    request.as_object_mut().unwrap().remove("range");
    let result = convert::execute(&SubtitleFiles, &request).unwrap();
    assert_eq!(result["cue_count"], 2);
    assert!(
        fs::read_to_string(fixture.0.join("out.vtt"))
            .unwrap()
            .contains("00:00:01.000 --> 00:00:03.000")
    );
    fs::remove_file(fixture.0.join("out.vtt")).unwrap();
    fs::write(
        fixture.0.join("source.srt"),
        SOURCE.replace("한글", "<b>한글</b>"),
    )
    .unwrap();
    assert_eq!(
        convert::execute(&SubtitleFiles, &request).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert!(!fixture.0.join("out.vtt").exists());
    let range_request = fixture.request("2000000", "5000000");
    assert_eq!(
        export_range::execute(&SubtitleFiles, &range_request)
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    assert!(!fixture.0.join("out.vtt").exists());
}
