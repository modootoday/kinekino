use kinekino_player::application::api::Request;
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub struct Fixture {
    pub root: PathBuf,
    evidence: Option<PathBuf>,
    calls: AtomicU64,
}
impl Fixture {
    pub fn new(case: &str) -> Self {
        let evidence = std::env::var_os("OUTPUT_REVIEW_EVIDENCE_DIR").map(|path| {
            let path = PathBuf::from(path).join(case);
            assert!(path.is_absolute());
            fs::create_dir(&path).unwrap();
            path
        });
        let root = evidence
            .as_ref()
            .map(|path| path.join("store"))
            .unwrap_or_else(|| {
                std::env::temp_dir().join(format!(
                    "output-review-cli-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ))
            });
        fs::create_dir(&root).unwrap();
        Self {
            root,
            evidence,
            calls: AtomicU64::new(0),
        }
    }
    pub fn invoke(&self, request: &Value, rooted: bool) -> (i32, Value) {
        let index = self.calls.fetch_add(1, Ordering::SeqCst);
        let mut command = Command::new(crate::cli_exe::kinekino_cli_exe());
        command.args(["--input", "-"]);
        if rooted {
            command.arg("--proposal-root").arg(&self.root);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(request).unwrap())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        let code = output.status.code().unwrap();
        if let Some(path) = &self.evidence {
            fs::write(path.join(format!("{index:04}.json")),serde_json::to_vec_pretty(&json!({"request":request,"response":response,"exit_code":code,"rooted":rooted,"executable":crate::cli_exe::kinekino_cli_exe()})).unwrap()).unwrap();
        }
        (code, response)
    }
    pub fn ok(&self, command: &str, payload: Value, expected: Option<&str>) -> Value {
        let (code, response) = self.invoke(&request(command, payload, expected), true);
        assert_eq!(code, 0, "{response}");
        assert_eq!(response["ok"], true);
        response["data"].clone()
    }
    pub fn reject(&self, request: &Value, code: &str) {
        let (exit, response) = self.invoke(request, true);
        assert_ne!(exit, 0);
        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], code, "{response}");
    }
    pub fn seed(&self, id: &str, task: &str) -> Value {
        let output = match task {
            "correction" => {
                json!({"task":"correction","edits":[{"cue_id":"cue-1","text":"Synthetic source!"}]})
            }
            "cards" => {
                json!({"task":"cards","cards":[{"front":"Synthetic question?","back":"Synthetic answer.","source_refs":["cue-1"]}]})
            }
            "highlights" => {
                json!({"task":"highlights","highlights":[{"title":"Synthetic selection","source_refs":["cue-1"]}]})
            }
            _ => panic!("unsupported fixture task"),
        };
        self.ok("proposals.create",json!({"proposal_id":id,"source":{"source_id":"source-1","source_revision":"7","source_sha256":"a".repeat(64),"cues":[{"cue_id":"cue-1","text":"Synthetic source.","start_us":null,"end_us":null}]},"generated":{"task":task,"raw_text":serde_json::to_string(&output).unwrap()}}),None);
        self.ok(
            "proposals.accept",
            json!({"proposal_id":id,"target_source_revision":"7"}),
            Some("1"),
        );
        self.inspect(id)
    }
    pub fn inspect(&self, id: &str) -> Value {
        self.ok(
            "proposals.inspect_output_review_target",
            json!({"proposal_id":id}),
            None,
        )
    }
    pub fn history(&self, id: &str) -> Value {
        self.ok(
            "proposals.output_decisions",
            json!({"proposal_id":id}),
            None,
        )
    }
    pub fn decision_count(&self, id: &str) -> usize {
        let path = self.root.join("output-decisions").join(format!("p-{id}"));
        if !path.exists() {
            return 0;
        }
        fs::read_dir(path).unwrap().count()
    }
    pub fn revision_file(&self, id: &str, revision: u64) -> PathBuf {
        fs::read_dir(self.root.join(format!("p-{id}/r-{revision}")))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
    }
    pub fn record_files(&self, id: &str) -> Vec<(PathBuf, Vec<u8>)> {
        let mut result = Vec::new();
        let path = self.root.join("output-decisions").join(format!("p-{id}"));
        if !path.exists() {
            return result;
        }
        for dir in fs::read_dir(path).unwrap() {
            for file in fs::read_dir(dir.unwrap().path()).unwrap() {
                let path = file.unwrap().path();
                let bytes = fs::read(&path).unwrap();
                result.push((path, bytes));
            }
        }
        result.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        result
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        eprintln!("fresh CLI calls: {}", self.calls.load(Ordering::SeqCst));
        if self.evidence.is_none() {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
pub fn request(command: &str, payload: Value, expected: Option<&str>) -> Value {
    let mut request = Request::new(command);
    request.payload = payload;
    request.context.expected_revision = expected.map(str::to_string);
    serde_json::to_value(request).unwrap()
}
pub fn record_payload(view: &Value, decision_id: &str) -> Value {
    json!({"proposal_id":view["target"]["proposal_id"],"decision":{"decision_id":decision_id,"target":view["target"],"criteria":view["criteria_identity"],"verdict":"deferred","note":"Synthetic persistence fixture; no actual quality approval.","declared_reviewer":{"label":"Declared synthetic reviewer","reviewer_id":"fixture"},"claimed_review_unix_ms":"19"}})
}
