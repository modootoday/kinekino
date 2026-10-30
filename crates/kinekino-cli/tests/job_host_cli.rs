//! Explicit `--serve-jobs` host plus `--endpoint` clients on every OS (plan P5).
//! The OS transport is chosen by the platform adapter; this test only uses the CLI surface.
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
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

fn submit(endpoint: &str, id: &str, key: &str, dir: &Path) -> Value {
    let request = json!({"command":"jobs.submit","context":{"api_version":"1","request_id":id,"idempotency_key":key},
        "payload":{"operation":"subtitles.convert","parameters":{"input_path":dir.join("in.srt"),"output_path":dir.join("out.vtt"),"input_format":"srt","output_format":"vtt"}}});
    call(&["--endpoint", endpoint], &request)
}

#[test]
fn serve_jobs_runs_subtitle_job_and_stops_on_stdin_eof() {
    let base = std::env::temp_dir().join(format!("kk-jobhost-{}", std::process::id()));
    let (root, dir) = (base.join("jobs"), base.join("work"));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("in.srt"),
        "1\n00:00:01,000 --> 00:00:02,000\nHello\n\n",
    )
    .unwrap();
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

    let first = submit(&endpoint, "s1", "k1", &dir);
    assert_eq!(first["ok"], true, "{first}");
    assert_eq!(first["data"]["created"], true);
    let job_id = first["data"]["job"]["job_id"].as_str().unwrap().to_owned();
    let replay = submit(&endpoint, "s2", "k1", &dir);
    assert_eq!(replay["data"]["created"], false, "{replay}");
    assert_eq!(replay["data"]["job"]["job_id"], job_id.as_str());

    let get = json!({"command":"jobs.get","context":{"api_version":"1","request_id":"g"},"payload":{"job_id":job_id}});
    let deadline = Instant::now() + Duration::from_secs(20);
    let state = loop {
        let got = call(&["--endpoint", &endpoint], &get);
        let state = got["data"]["state"].as_str().unwrap_or_default().to_owned();
        if state == "completed" || Instant::now() > deadline {
            break state;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    assert_eq!(state, "completed");
    let vtt = std::fs::read_to_string(dir.join("out.vtt")).unwrap();
    assert!(vtt.starts_with("WEBVTT"), "{vtt}");

    drop(host.stdin.take());
    let stopped: Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
    assert_eq!(stopped["event"], "job_host_stopped", "{stopped}");
    assert_eq!(stopped["receipt"]["completed_total"], 1);
    assert!(host.wait().unwrap().success());

    let root_arg = root.to_str().unwrap();
    let list = call(
        &["--job-root", root_arg],
        &json!({"command":"jobs.list","context":{"api_version":"1","request_id":"l"},"payload":{}}),
    );
    assert_eq!(list["data"]["jobs"][0]["job_id"], job_id.as_str(), "{list}");
    let _ = std::fs::remove_dir_all(&base);
}
