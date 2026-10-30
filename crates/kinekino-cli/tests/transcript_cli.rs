//! P5: ASR transcript JSON → subtitle cues through the CLI (in-process) and the jobs host.
//! No inference runs here; the transcript file has the `kinekino-inference` chunk shape.
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_kinekino"))
}

fn call(extra: &[&str], request: &Value) -> Value {
    let mut child = bin()
        .args(["--input", "-"])
        .args(extra)
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
    serde_json::from_slice(&child.wait_with_output().unwrap().stdout).unwrap()
}

const TRANSCRIPT: &str = r#"{"text":"안녕 하세요 네","chunks":[{"start":0.0,"end":0.5,"text":"안녕"},{"start":0.5,"end":1.0,"text":"하세요"},{"start":4.0,"end":4.2,"text":"네"}]}"#;

#[test]
fn transcript_to_styled_ass_in_process_and_as_job() {
    let base = std::env::temp_dir().join(format!("kk-transcript-{}", std::process::id()));
    let (root, dir) = (base.join("jobs"), base.join("work"));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("t.json");
    std::fs::write(&input, TRANSCRIPT).unwrap();

    let schema = call(
        &[],
        &json!({"command":"schema","context":{"api_version":"1","request_id":"s"},"payload":{"command":"subtitles.from_transcript"}}),
    );
    assert_eq!(schema["data"]["status"], "supported", "{schema}");

    let direct = call(
        &[],
        &json!({"command":"subtitles.from_transcript","context":{"api_version":"1","request_id":"d"},
            "payload":{"input_path":input,"output_path":dir.join("direct.ass"),"output_format":"ass",
                "ass_style":{"font_name":"Noto Sans KR","font_size":48,"alignment":8}}}),
    );
    assert_eq!(direct["ok"], true, "{direct}");
    assert_eq!(direct["data"]["cue_count"], 2);
    let ass = std::fs::read_to_string(dir.join("direct.ass")).unwrap();
    assert!(ass.contains("Style: Default,Noto Sans KR,48,"), "{ass}");
    assert!(
        ass.contains("0:00:00.00,0:00:01.00,Default,,0,0,0,,안녕 하세요"),
        "{ass}"
    );

    let bad = call(
        &[],
        &json!({"command":"subtitles.from_transcript","context":{"api_version":"1","request_id":"b"},
            "payload":{"input_path":input,"output_path":dir.join("bad.srt"),"output_format":"srt","ass_style":{}}}),
    );
    assert_eq!(bad["error"]["code"], "invalid_argument", "{bad}");
    assert!(!dir.join("bad.srt").exists());

    let mut host = bin()
        .args(["--serve-jobs", "--job-root"])
        .arg(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(host.stdout.take().unwrap()).lines();
    let ready: Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
    assert_eq!(ready["event"], "job_host_ready", "{ready}");
    let endpoint = ready["endpoint"].as_str().unwrap().to_owned();
    let submit = call(
        &["--endpoint", &endpoint],
        &json!({"command":"jobs.submit","context":{"api_version":"1","request_id":"j","idempotency_key":"t1"},
            "payload":{"operation":"subtitles.from_transcript","parameters":{"input_path":input,"output_path":dir.join("job.vtt"),"output_format":"vtt"}}}),
    );
    assert_eq!(submit["ok"], true, "{submit}");
    let job_id = submit["data"]["job"]["job_id"].as_str().unwrap().to_owned();
    let get = json!({"command":"jobs.get","context":{"api_version":"1","request_id":"g"},"payload":{"job_id":job_id}});
    let deadline = Instant::now() + Duration::from_secs(20);
    let got = loop {
        let got = call(&["--endpoint", &endpoint], &get);
        if got["data"]["state"] == "completed" || Instant::now() > deadline {
            break got;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    assert_eq!(got["data"]["state"], "completed", "{got}");
    let vtt = std::fs::read_to_string(dir.join("job.vtt")).unwrap();
    assert!(
        vtt.starts_with("WEBVTT\n\n00:00:00.000 --> 00:00:01.000\n안녕 하세요\n"),
        "{vtt}"
    );
    drop(host.stdin.take());
    let _ = lines.next();
    assert!(host.wait().unwrap().success());
    let _ = std::fs::remove_dir_all(&base);
}
