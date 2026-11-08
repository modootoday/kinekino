//! Jobs submit, cancel and retry through the CLI against an explicit `--serve-jobs` host
//! (roadmap M4-1). Uses only local subtitle fixtures; no inference or network.
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdout, Command, Stdio},
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

fn context(id: &str) -> Value {
    json!({"api_version":"1","request_id":id})
}

fn convert(dir: &Path, input: &str, output: &str) -> Value {
    json!({"operation":"subtitles.convert","parameters":{"input_path":dir.join(input),"output_path":dir.join(output),"input_format":"srt","output_format":"vtt"}})
}

struct Host {
    child: Child,
    lines: std::io::Lines<BufReader<ChildStdout>>,
    endpoint: String,
    root: PathBuf,
    work: PathBuf,
    base: PathBuf,
}

impl Host {
    fn start(tag: &str) -> Self {
        let base = std::env::temp_dir().join(format!("kk-joblife-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (root, work) = (base.join("jobs"), base.join("work"));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&work).unwrap();
        let mut child = bin()
            .args(["--serve-jobs", "--job-root"])
            .arg(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let ready: Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
        assert_eq!(ready["event"], "job_host_ready", "{ready}");
        let endpoint = ready["endpoint"].as_str().unwrap().to_owned();
        Self {
            child,
            lines,
            endpoint,
            root,
            work,
            base,
        }
    }
    fn rpc(&self, request: Value) -> Value {
        call(&["--endpoint", &self.endpoint], &request)
    }
    fn submit(&self, key: &str, payload: Value) -> Value {
        let mut context = context(key);
        context["idempotency_key"] = json!(key);
        self.rpc(json!({"command":"jobs.submit","context":context,"payload":payload}))
    }
    fn get(&self, job_id: &str) -> Value {
        self.rpc(json!({"command":"jobs.get","context":context("g"),"payload":{"job_id":job_id}}))
    }
    /// Polls until the job reaches one of `states` (on the given attempt) and returns the record.
    fn wait(&self, job_id: &str, attempt: u64, states: &[&str]) -> Value {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let got = self.get(job_id);
            let data = &got["data"];
            let state = data["state"].as_str().unwrap_or_default();
            let current = data["attempt"]
                .as_str()
                .and_then(|value| value.parse::<u64>().ok())
                .or_else(|| data["attempt"].as_u64());
            if current == Some(attempt) && states.contains(&state) {
                return data.clone();
            }
            assert!(
                Instant::now() < deadline,
                "job never reached {states:?}: {got}"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    fn stop(mut self) -> Value {
        drop(self.child.stdin.take());
        let stopped: Value = serde_json::from_str(&self.lines.next().unwrap().unwrap()).unwrap();
        assert_eq!(stopped["event"], "job_host_stopped", "{stopped}");
        assert!(self.child.wait().unwrap().success());
        stopped
    }
}

fn revision(record: &Value) -> String {
    record["revision"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| record["revision"].to_string())
}

fn error_code(response: &Value) -> &str {
    assert_eq!(response["ok"], false, "{response}");
    response["error"]["code"].as_str().unwrap_or_default()
}

#[test]
fn cancel_of_a_completed_job_is_already_terminal_and_checks_the_attempt() {
    let host = Host::start("cancel");
    std::fs::write(
        host.work.join("in.srt"),
        "1\n00:00:01,000 --> 00:00:02,000\nHello\n\n",
    )
    .unwrap();
    let submitted = host.submit("c1", convert(&host.work, "in.srt", "out.vtt"));
    assert_eq!(submitted["ok"], true, "{submitted}");
    let job_id = submitted["data"]["job"]["job_id"]
        .as_str()
        .unwrap()
        .to_owned();
    host.wait(&job_id, 1, &["completed"]);

    let cancel = |attempt: Value| {
        host.rpc(json!({"command":"jobs.cancel","context":context("c"),"payload":{"job_id":job_id,"expected_attempt":attempt}}))
    };
    let done = cancel(json!("1"));
    assert_eq!(done["ok"], true, "{done}");
    assert_eq!(done["data"]["disposition"], "already_terminal", "{done}");
    assert_eq!(done["data"]["job"]["state"], "completed", "{done}");
    assert_eq!(error_code(&cancel(json!("2"))), "revision_conflict");

    let missing = host
        .rpc(json!({"command":"jobs.cancel","context":context("m"),"payload":{"job_id":job_id}}));
    assert_eq!(error_code(&missing), "invalid_argument");
    let mut keyed = context("k");
    keyed["idempotency_key"] = json!("not-for-cancel");
    let keyed = host.rpc(json!({"command":"jobs.cancel","context":keyed,"payload":{"job_id":job_id,"expected_attempt":"1"}}));
    assert_eq!(error_code(&keyed), "invalid_argument");

    let mut retry = context("r");
    retry["expected_revision"] = json!(revision(&host.get(&job_id)["data"]));
    let retry =
        host.rpc(json!({"command":"jobs.retry","context":retry,"payload":{"job_id":job_id}}));
    assert_eq!(
        error_code(&retry),
        "invalid_argument",
        "completed jobs are not retryable"
    );

    let base = host.base.clone();
    host.stop();
    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn retry_requeues_a_failed_job_as_a_new_attempt_with_revision_checks() {
    let host = Host::start("retry");
    std::fs::write(host.work.join("bad.srt"), "this is not a subtitle file\n").unwrap();
    let submitted = host.submit("r1", convert(&host.work, "bad.srt", "bad.vtt"));
    assert_eq!(submitted["ok"], true, "{submitted}");
    let job_id = submitted["data"]["job"]["job_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let failed = host.wait(&job_id, 1, &["failed", "needs_input"]);
    assert!(
        !host.work.join("bad.vtt").exists(),
        "failed job published output"
    );

    let retry = |expected: Option<String>| {
        let mut context = context("r");
        if let Some(expected) = expected {
            context["expected_revision"] = json!(expected);
        }
        host.rpc(json!({"command":"jobs.retry","context":context,"payload":{"job_id":job_id}}))
    };
    assert_eq!(error_code(&retry(None)), "invalid_argument");
    let stale = revision(&failed).parse::<u64>().unwrap() + 7;
    assert_eq!(
        error_code(&retry(Some(stale.to_string()))),
        "revision_conflict"
    );

    let retried = retry(Some(revision(&failed)));
    assert_eq!(retried["ok"], true, "{retried}");
    assert_eq!(retried["data"]["job_id"], job_id.as_str(), "{retried}");
    let again = host.wait(&job_id, 2, &["failed", "needs_input"]);
    assert!(again["failure"].is_object(), "{again}");

    let base = host.base.clone();
    host.stop();
    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn local_job_root_reads_but_refuses_job_writes() {
    let host = Host::start("local");
    let root = host.root.to_str().unwrap().to_owned();
    host.stop();
    let mut keyed = context("s");
    keyed["idempotency_key"] = json!("local");
    let submit = call(
        &["--job-root", &root],
        &json!({"command":"jobs.submit","context":keyed,"payload":convert(Path::new("/nonexistent"), "in.srt", "out.vtt")}),
    );
    assert_eq!(error_code(&submit), "unsupported", "{submit}");
    let cancel = call(
        &["--job-root", &root],
        &json!({"command":"jobs.cancel","context":context("c"),"payload":{"job_id":"j","expected_attempt":"1"}}),
    );
    assert_eq!(error_code(&cancel), "unsupported", "{cancel}");
    let list = call(
        &["--job-root", &root],
        &json!({"command":"jobs.list","context":context("l"),"payload":{}}),
    );
    assert_eq!(list["ok"], true, "{list}");
    let _ = std::fs::remove_dir_all(Path::new(&root).parent().unwrap());
}
