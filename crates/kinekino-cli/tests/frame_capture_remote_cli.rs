//! `production.capture_frame` remote-input refusal through `kinekino --input -` (roadmap M4-4).
//! Frame capture only reads absolute local MP4/MKV/WebM files. These fixtures lock the Linux CLI
//! contract: URLs, protocol prefixes, UNC and relative paths, playlists and playlists disguised as
//! media are refused with nothing published, and a local listener proves no connection is made.
use serde_json::{Value, json};
use std::{
    io::{ErrorKind, Write},
    net::TcpListener,
    path::PathBuf,
    process::{Command, Stdio},
};

struct Fixture {
    dir: PathBuf,
    output: PathBuf,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("kk-capture-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self {
            output: dir.join("out.png"),
            dir,
        }
    }
    fn payload(&self, source: Value) -> Value {
        json!({"source_path": source, "source_pts_us": "0", "output_path": &self.output})
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// A local TCP listener standing in for a remote host; `assert_untouched` fails if the CLI
/// opened any connection to it.
struct Sentinel(TcpListener);

impl Sentinel {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        Self(listener)
    }
    fn authority(&self) -> String {
        self.0.local_addr().unwrap().to_string()
    }
    fn assert_untouched(&self) {
        match self.0.accept() {
            Err(error) if error.kind() == ErrorKind::WouldBlock => {}
            other => panic!("capture contacted the remote sentinel: {other:?}"),
        }
    }
}

fn call(request: &Value) -> (Option<i32>, Value) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kinekino"))
        .args(["--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(request.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    (
        output.status.code(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}

fn request(command: &str, request_id: &str, payload: Value) -> Value {
    json!({"command":command,"context":{"request_id":request_id,"api_version":"1"},"payload":payload})
}

fn capture(payload: Value) -> (Option<i32>, Value) {
    call(&request("production.capture_frame", "capture", payload))
}

fn ffmpeg_available() -> bool {
    let (_, capabilities) = call(&request("capabilities", "caps", json!({})));
    capabilities["data"]["frame_capture"]["dependency"]["available"] == true
}

#[test]
fn remote_protocol_unc_and_relative_sources_are_refused_without_connecting() {
    let fixture = Fixture::new("remote");
    let sentinel = Sentinel::new();
    let host = sentinel.authority();
    let local = fixture.dir.join("input.mp4");
    let sources = [
        format!("http://{host}/input.mp4"),
        format!("https://{host}/input.mp4"),
        format!("rtmp://{host}/live/input.mp4"),
        format!("rtsp://{host}/input.mp4"),
        format!("tcp://{host}/input.mp4"),
        format!("udp://{host}/input.mp4"),
        format!("ftp://{host}/input.mp4"),
        format!("file://{}", local.display()),
        format!("concat:{}|{}", local.display(), local.display()),
        format!("{}/http://{host}/input.mp4", fixture.dir.display()),
        "data:video/mp4;base64,AAAA.mp4".to_owned(),
        r"\\server\share\input.mp4".to_owned(),
        "//server/share/input.mp4:".to_owned(),
        "relative.mp4".to_owned(),
        "".to_owned(),
    ];
    for source in sources {
        let (code, response) = capture(fixture.payload(json!(source)));
        assert_eq!(code, Some(2), "{source}: {response}");
        assert_eq!(response["ok"], false, "{source}: {response}");
        assert_eq!(
            response["error"]["code"], "invalid_argument",
            "{source}: {response}"
        );
        assert!(!fixture.output.exists(), "{source}");
    }
    sentinel.assert_untouched();
}

#[test]
fn playlist_sources_and_remote_outputs_are_refused_before_any_fetch() {
    let fixture = Fixture::new("playlist");
    let sentinel = Sentinel::new();
    let host = sentinel.authority();
    let playlist = format!("#EXTM3U\n#EXT-X-TARGETDURATION:1\nhttp://{host}/segment.ts\n");
    for name in ["live.m3u8", "live.m3u", "list.txt", "input.mp4.m3u8"] {
        let path = fixture.dir.join(name);
        std::fs::write(&path, &playlist).unwrap();
        let (code, response) = capture(fixture.payload(json!(path)));
        assert_eq!(code, Some(2), "{name}: {response}");
        assert_eq!(response["error"]["code"], "invalid_argument", "{response}");
        assert!(!fixture.output.exists(), "{name}");
    }

    std::fs::write(fixture.dir.join("input.mp4"), b"\0\0\0\x18ftypisom").unwrap();
    for output in [
        format!("http://{host}/out.png"),
        format!("file://{}", fixture.dir.join("remote.png").display()),
        "relative.png".to_owned(),
    ] {
        let mut payload = fixture.payload(json!(fixture.dir.join("input.mp4")));
        payload["output_path"] = json!(output);
        let (code, response) = capture(payload);
        assert_eq!(code, Some(2), "{output}: {response}");
        assert_eq!(response["error"]["code"], "invalid_argument", "{response}");
    }
    assert!(!fixture.dir.join("remote.png").exists());
    sentinel.assert_untouched();
}

#[test]
fn playlists_disguised_as_local_media_are_refused_without_connecting() {
    let fixture = Fixture::new("disguised");
    let sentinel = Sentinel::new();
    let playlist = format!(
        "#EXTM3U\n#EXT-X-TARGETDURATION:1\nhttp://{}/segment.ts\n",
        sentinel.authority()
    );
    let target = fixture.dir.join("real.m3u8");
    std::fs::write(&target, &playlist).unwrap();
    let mut sources = Vec::new();
    for name in ["input.mp4", "input.mkv", "input.webm"] {
        let path = fixture.dir.join(name);
        std::fs::write(&path, &playlist).unwrap();
        sources.push(path);
    }
    let link = fixture.dir.join("linked.mp4");
    if std::process::Command::new("ln")
        .args(["-s", &target.to_string_lossy(), &link.to_string_lossy()])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
    {
        sources.push(link);
    }

    // Without FFmpeg the adapter refuses as unsupported before reading; with it, the container
    // signature check refuses the playlist text. Either way nothing is fetched or published.
    let expected = if ffmpeg_available() {
        "invalid_argument"
    } else {
        "unsupported"
    };
    for source in sources {
        let (code, response) = capture(fixture.payload(json!(source)));
        assert_ne!(code, Some(0), "{}: {response}", source.display());
        assert_eq!(
            response["error"]["code"],
            expected,
            "{}: {response}",
            source.display()
        );
        assert!(!fixture.output.exists(), "{}", source.display());
        assert_eq!(std::fs::read_to_string(&source).unwrap(), playlist);
    }
    sentinel.assert_untouched();
}

#[test]
fn cli_schema_and_capabilities_declare_local_only_capture() {
    let (code, schema) = call(&request(
        "schema",
        "schema",
        json!({"command": "production.capture_frame"}),
    ));
    assert_eq!(code, Some(0), "{schema}");
    assert_eq!(schema["data"]["command"], "production.capture_frame");
    let source = &schema["data"]["request"]["properties"]["payload"]["properties"]["source_path"];
    assert!(
        source["description"]
            .as_str()
            .is_some_and(|text| text.contains("no URLs")),
        "{source}"
    );

    let (code, capabilities) = call(&request("capabilities", "caps", json!({})));
    assert_eq!(code, Some(0), "{capabilities}");
    let frame = &capabilities["data"]["frame_capture"];
    assert_eq!(frame["network"], false, "{frame}");
    assert_eq!(frame["overwrite"], false, "{frame}");
    assert_eq!(frame["formats"], json!(["mp4", "mkv", "webm"]), "{frame}");
    let status = capabilities["data"]["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["command"] == "production.capture_frame")
        .unwrap()["status"]
        .clone();
    if frame["dependency"]["available"] == true {
        assert_eq!(status, "supported", "{capabilities}");
    }
}
