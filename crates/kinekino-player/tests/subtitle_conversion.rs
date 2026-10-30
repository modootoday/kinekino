use kinekino_player::application::api::ProposalClient;
#[path = "support/cli_exe.rs"]
mod cli_exe;
use kinekino_player::{
    application::{
        api::{PlayerSdk, Request},
        subtitles::storage::MAX_SUBTITLE_BYTES,
    },
    infrastructure::subtitle_files::SubtitleFiles,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
static NEXT_CALL: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf, bool);
impl Fixture {
    fn new() -> Self {
        let evidence = std::env::var_os("SUBTITLE_CONVERSION_EVIDENCE_DIR").map(PathBuf::from);
        let parent = evidence.clone().unwrap_or_else(std::env::temp_dir);
        assert!(parent.is_absolute());
        let path = parent.join(format!(
            "subtitle-contract-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path, evidence.is_some())
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
    fn request(&self, input: &str, output: &str, source: &str, target: &str) -> Request {
        let mut request = Request::new("subtitles.convert");
        request.payload = json!({"input_path":self.path(input),"output_path":self.path(output),"input_format":source,"output_format":target});
        request
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if !self.1 {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
}

fn cli(request: &Request) -> (i32, Value) {
    let evidence = std::env::var_os("SUBTITLE_CONVERSION_EVIDENCE_DIR").map(|root| {
        let directory = PathBuf::from(root).join(format!(
            "call-{}",
            NEXT_CALL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        fs::write(
            directory.join("request.json"),
            serde_json::to_vec_pretty(request).unwrap(),
        )
        .unwrap();
        directory
    });
    let mut child = Command::new(crate::cli_exe::kinekino_cli_exe())
        .args(["--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = child.id();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(request).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    if let Some(directory) = evidence {
        fs::write(directory.join("stdout.raw"), &output.stdout).unwrap();
        fs::write(directory.join("stderr.raw"), &output.stderr).unwrap();
        let executable = fs::read(crate::cli_exe::kinekino_cli_exe()).unwrap();
        fs::write(directory.join("PROCESS.json"),serde_json::to_vec_pretty(&json!({"pid":pid,"exit_code":output.status.code(),"wait_with_output_returned":true,"cli_sha256":format!("{:x}",Sha256::digest(&executable))})).unwrap()).unwrap();
    }
    assert!(output.stderr.is_empty());
    (
        output.status.code().unwrap(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}

#[test]
fn actual_cli_file_roundtrip_preserves_unicode_time_and_original() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let source = "1\n00:00:01,001 --> 00:00:03,456\n한글 日本語\n둘째\n\n";
    fs::write(fixture.path("원본.srt"), source).unwrap();
    let (code, result) = cli(&fixture.request("원본.srt", "변환.vtt", "srt", "vtt"));
    assert_eq!(code, 0, "{result}");
    assert_eq!(result["data"]["cue_count"], 1);
    assert!(
        fs::read_to_string(fixture.path("변환.vtt"))
            .unwrap()
            .starts_with("WEBVTT\n\n00:00:01.001")
    );
    let (code, result) = cli(&fixture.request("변환.vtt", "왕복.srt", "vtt", "srt"));
    assert_eq!(code, 0, "{result}");
    assert_eq!(
        fs::read_to_string(fixture.path("왕복.srt")).unwrap(),
        source
    );
    assert_eq!(
        fs::read_to_string(fixture.path("원본.srt")).unwrap(),
        source
    );
}

#[test]
fn loss_review_blocks_writes_then_reports_explicit_prefix_conversion() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    fs::write(
        fixture.path("voice.vtt"),
        "WEBVTT\n\n00:01.000 --> 00:02.000 align:start\n<v 화자><b>대사</b></v>\n",
    )
    .unwrap();
    let mut request = fixture.request("voice.vtt", "out.srt", "vtt", "srt");
    let (code, result) = cli(&request);
    assert_eq!(code, 3);
    assert_eq!(result["error"]["code"], "needs_input");
    assert!(!fixture.path("out.srt").exists());
    request.payload["allow_loss"] = json!(true);
    request.payload["speaker_policy"] = json!("prefix");
    let (code, result) = cli(&request);
    assert_eq!(code, 0, "{result}");
    assert_eq!(result["data"]["losses"].as_array().unwrap().len(), 3);
    assert!(
        fs::read_to_string(fixture.path("out.srt"))
            .unwrap()
            .contains("[화자] 대사")
    );
}

#[test]
fn existing_output_same_path_alias_and_hardlink_are_never_overwritten() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let source = "1\n00:00:01,000 --> 00:00:02,000\na\n";
    fs::write(fixture.path("in.srt"), source).unwrap();
    fs::write(fixture.path("out.vtt"), "KEEP").unwrap();
    let (code, result) = cli(&fixture.request("in.srt", "out.vtt", "srt", "vtt"));
    assert_eq!(code, 5, "{result}");
    assert_eq!(fs::read_to_string(fixture.path("out.vtt")).unwrap(), "KEEP");
    let (code, _) = cli(&fixture.request("in.srt", "in.srt", "srt", "vtt"));
    assert_eq!(code, 2);
    fs::hard_link(fixture.path("in.srt"), fixture.path("alias.srt")).unwrap();
    let (code, _) = cli(&fixture.request("in.srt", "alias.srt", "srt", "vtt"));
    assert_eq!(code, 5);
    assert_eq!(fs::read_to_string(fixture.path("in.srt")).unwrap(), source);
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 3);
}

#[test]
fn size_encoding_format_unknown_fields_and_timing_are_rejected() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    fs::write(fixture.path("in.srt"), vec![b'x'; MAX_SUBTITLE_BYTES + 1]).unwrap();
    assert_eq!(
        cli(&fixture.request("in.srt", "out.vtt", "srt", "vtt")).0,
        7
    );
    fs::write(fixture.path("in.srt"), [0xff, 0xfe]).unwrap();
    assert_eq!(
        cli(&fixture.request("in.srt", "out.vtt", "srt", "vtt")).0,
        2
    );
    fs::write(
        fixture.path("in.srt"),
        "1\n-00:00:01,000 --> 00:00:02,000\na",
    )
    .unwrap();
    assert_eq!(
        cli(&fixture.request("in.srt", "out.vtt", "srt", "vtt")).0,
        2
    );
    assert_eq!(
        cli(&fixture.request("in.srt", "out.vtt", "ssa", "vtt")).0,
        3
    );
    let mut request = fixture.request("in.srt", "out.vtt", "srt", "vtt");
    request.payload["overwrite"] = json!(true);
    assert_eq!(cli(&request).0, 2);
    assert!(!fixture.path("out.vtt").exists());
}

#[test]
fn configured_sdk_and_cli_discovery_match_conversion_support() {
    kinekino_platform::install();
    let sdk = PlayerSdk::new(SubtitleFiles).with_frame_capture(
        kinekino_player::infrastructure::frame_capture::FfmpegFrameCapture::discover(),
    );
    let sdk = sdk
        .with_snapshot_export(kinekino_player::infrastructure::snapshot_export::PngSnapshotExport);
    #[cfg(windows)]
    let sdk = sdk.with_speaker_documents(
        kinekino_player::infrastructure::speaker_annotation_files::LocalSpeakerFiles,
    );
    let request = Request::new("capabilities");
    let sdk = serde_json::to_value(sdk.execute(request.clone())).unwrap();
    let (code, remote) = cli(&request);
    assert_eq!(code, 0);
    assert_eq!(sdk, remote);
    assert!(
        remote["data"]["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["command"] == "subtitles.convert" && v["status"] == "supported")
    );
    let mut request = Request::new("schema");
    request.payload = json!({"command":"subtitles.convert"});
    let (_, schema) = cli(&request);
    assert_eq!(schema["data"]["status"], "supported");
    assert_eq!(
        schema["data"]["request"]["properties"]["payload"]["properties"]["input_format"]["enum"],
        json!(["srt", "vtt", "ass"])
    );
}

#[test]
fn concurrent_exports_have_one_winner_and_leave_no_temporary_files() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    fs::write(
        fixture.path("in.srt"),
        "1\n00:00:01,000 --> 00:00:02,000\na\n",
    )
    .unwrap();
    let request = fixture.request("in.srt", "out.vtt", "srt", "vtt");
    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(|| cli(&request).0);
        let second = scope.spawn(|| cli(&request).0);
        (first.join().unwrap(), second.join().unwrap())
    });
    assert!(matches!((first, second), (0, 5) | (5, 0)));
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 2);
}

#[test]
fn context_guarantees_without_implementation_are_rejected_before_writing() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    fs::write(
        fixture.path("in.srt"),
        "1\n00:00:01,000 --> 00:00:02,000\na\n",
    )
    .unwrap();
    let mut request = fixture.request("in.srt", "out.vtt", "srt", "vtt");
    request.context.idempotency_key = Some("key".into());
    assert_eq!(cli(&request).0, 3);
    assert!(!fixture.path("out.vtt").exists());
}

#[test]
fn object_safe_client_on_worker_matches_cli_typed_conversion() {
    kinekino_platform::install();
    use kinekino_player::application::{
        api::ApplicationClientHandle, subtitles::convert::ConvertSubtitles,
    };
    let fixture = Fixture::new();
    fs::write(
        fixture.path("in.srt"),
        "1\n00:00:01,001 --> 00:00:02,002\n공통 API\n",
    )
    .unwrap();
    let request = fixture.request("in.srt", "typed.vtt", "srt", "vtt");
    let payload: ConvertSubtitles = serde_json::from_value(request.payload).unwrap();
    let sdk = PlayerSdk::new(SubtitleFiles).with_frame_capture(
        kinekino_player::infrastructure::frame_capture::FfmpegFrameCapture::discover(),
    );
    let sdk = sdk
        .with_snapshot_export(kinekino_player::infrastructure::snapshot_export::PngSnapshotExport);
    #[cfg(windows)]
    let sdk = sdk.with_speaker_documents(
        kinekino_player::infrastructure::speaker_annotation_files::LocalSpeakerFiles,
    );
    let client: ApplicationClientHandle = std::sync::Arc::new(sdk);
    let handle = client.clone();
    let result = std::thread::spawn(move || handle.convert_subtitles(request.context, payload))
        .join()
        .unwrap()
        .unwrap();
    let (code, response) = cli(&fixture.request("in.srt", "cli.vtt", "srt", "vtt"));
    assert_eq!(code, 0);
    assert_eq!(
        serde_json::to_value(&result).unwrap()["cue_count"],
        response["data"]["cue_count"]
    );
    assert_eq!(
        fs::read(fixture.path("typed.vtt")).unwrap(),
        fs::read(fixture.path("cli.vtt")).unwrap()
    );
    let request = Request::new("capabilities");
    assert_eq!(
        serde_json::to_value(client.execute(request.clone())).unwrap(),
        cli(&request).1
    );
}

#[test]
fn text_path_only_port_reports_unavailable_without_side_effects() {
    kinekino_platform::install();
    use kinekino_player::application::subtitles::path_picker::{
        SubtitlePathPickerHandle, TextPathOnly,
    };
    use kinekino_player::domain::subtitles::SubtitleFormat;
    let picker: SubtitlePathPickerHandle = std::rc::Rc::new(TextPathOnly);
    assert_eq!(picker.open_subtitle().unwrap_err().exit_code(), 3);
    assert_eq!(
        picker
            .save_subtitle(SubtitleFormat::Vtt)
            .unwrap_err()
            .exit_code(),
        3
    );
}

#[test]
#[ignore = "Requires original externally downloaded runtime/ai-spike/source/sintel_ko.srt corpus"]
fn actual_sintel_corpus_with_leading_blank_line_roundtrips_without_preprocessing() {
    kinekino_platform::install();
    use kinekino_player::domain::subtitles::{SpeakerPolicy, SubtitleFormat, parser::parse};
    let fixture = Fixture::new();
    let corpus = std::env::var_os("SUBTITLE_CORPUS_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
                .join("runtime/ai-spike/source/sintel_ko.srt")
        });
    assert!(corpus.is_absolute());
    let original = fs::read(&corpus).unwrap();
    if std::env::var_os("SUBTITLE_CORPUS_PATH").is_some() {
        assert_eq!(
            format!("{:x}", Sha256::digest(&original)),
            std::env::var("SUBTITLE_CORPUS_SHA256")
                .expect("explicit corpus SHA required")
                .to_ascii_lowercase()
        );
    }
    assert_eq!(original.first(), Some(&b'\n'));
    let text = std::str::from_utf8(&original).unwrap();
    let expected = parse(text, SubtitleFormat::Srt, SpeakerPolicy::Omit).unwrap();
    assert!(expected.cues.len() > 20);
    let mut forward = fixture.request("unused", "corpus.vtt", "srt", "vtt");
    forward.payload["input_path"] = json!(corpus);
    let (code, response) = cli(&forward);
    assert_eq!(code, 0, "{response}");
    let (code, response) = cli(&fixture.request("corpus.vtt", "corpus.srt", "vtt", "srt"));
    assert_eq!(code, 0, "{response}");
    let result = fs::read_to_string(fixture.path("corpus.srt")).unwrap();
    assert_eq!(
        expected.cues,
        parse(&result, SubtitleFormat::Srt, SpeakerPolicy::Omit)
            .unwrap()
            .cues
    );
    assert_eq!(fs::read(&corpus).unwrap(), original);
}
