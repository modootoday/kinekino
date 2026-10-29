//! Local fixture server (and optional ffmpeg-generated H.264/AAC clip) for contract runs.
//! Never touches external sites.
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const PAGE: &str = r#"<!doctype html><html><body style="background:#FF5A1F">
<h1 id="t">kinekino engine contract fixture</h1>
<script src="/blocked.js"></script>
<video id="v" src="/clip.mp4" muted playsinline autoplay></video>
</body></html>"#;

/// Generates a 2 s H.264 + AAC MP4 locally with ffmpeg (no download).
pub fn make_clip(dir: &Path) -> Option<PathBuf> {
    let out = dir.join("clip.mp4");
    let ok = Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=320x240:rate=25:duration=2",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=2",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-movflags",
            "+faststart",
            "-shortest",
        ])
        .arg(&out)
        .status()
        .ok()?
        .success();
    ok.then_some(out)
}

/// Serves PAGE at `/`, the clip at `/clip.mp4`, and a marker script at `/blocked.js`.
pub fn serve(clip: Option<PathBuf>) -> String {
    serve_on("127.0.0.1:0", clip)
}

/// Same as [`serve`] on a fixed address (e.g. for a child process demo).
pub fn serve_on(bind: &str, clip: Option<PathBuf>) -> String {
    let listener = TcpListener::bind(bind).expect("bind");
    let addr = listener.local_addr().expect("addr");
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let clip = clip.clone();
            std::thread::spawn(move || {
                let mut reader = BufReader::new(&stream);
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() {
                    return;
                }
                loop {
                    let mut h = String::new();
                    if reader.read_line(&mut h).is_err() || h.is_empty() || h == "\r\n" {
                        break;
                    }
                }
                let path = line.split_whitespace().nth(1).unwrap_or("/");
                let (ctype, body): (&str, Vec<u8>) = match path {
                    "/clip.mp4" => (
                        "video/mp4",
                        clip.and_then(|p| std::fs::read(p).ok()).unwrap_or_default(),
                    ),
                    "/blocked.js" => ("text/javascript", b"window.__blockedRan = true;".to_vec()),
                    _ => ("text/html", PAGE.as_bytes().to_vec()),
                };
                let mut s = &stream;
                let _ = write!(
                    s,
                    "HTTP/1.1 200 OK\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = s.write_all(&body);
            });
        }
    });
    format!("http://localhost:{}", addr.port())
}
