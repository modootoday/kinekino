//! Project storage and `projects.export` through the CLI on the build host (roadmap M4-2).
//! Uses only a local project fixture with a stored locator that is never opened.
use serde_json::{Value, json};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

struct Fixture {
    base: PathBuf,
    projects: PathBuf,
    out: PathBuf,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let base = std::env::temp_dir().join(format!("kk-projexp-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (projects, out) = (base.join("projects"), base.join("out"));
        std::fs::create_dir_all(&projects).unwrap();
        std::fs::create_dir_all(&out).unwrap();
        Self {
            base,
            projects,
            out,
        }
    }
    fn call(&self, request: Value) -> (Option<i32>, Value) {
        call(
            &["--project-root", self.projects.to_str().unwrap()],
            &request,
        )
    }
    fn ok(&self, request: Value) -> Value {
        let (code, response) = self.call(request);
        assert_eq!(response["ok"], true, "{response}");
        assert_eq!(code, Some(0), "{response}");
        response["data"].clone()
    }
    fn error(&self, request: Value) -> String {
        let (code, response) = self.call(request);
        assert_eq!(response["ok"], false, "{response}");
        assert_ne!(code, Some(0), "{response}");
        response["error"]["code"].as_str().unwrap().to_owned()
    }
    fn output_entries(&self) -> Vec<String> {
        let mut names: Vec<_> = std::fs::read_dir(&self.out)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        names
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

fn call(extra: &[&str], request: &Value) -> (Option<i32>, Value) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kinekino"))
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
    let output = child.wait_with_output().unwrap();
    (
        output.status.code(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}

fn request(command: &str, payload: Value) -> Value {
    json!({"context":{"api_version":"1","request_id":command},"command":command,"payload":payload})
}

fn with_expected(mut request: Value, expected: &str) -> Value {
    request["context"]["expected_revision"] = json!(expected);
    request
}

fn content(f: &Fixture, label: &str) -> Value {
    let locator = f.base.join("not-opened.mkv");
    json!({"watching":{"label":label,"source":{"kind":"local_video","source_id":"fixture","path":locator}},"proposals":[]})
}

fn export(project_id: &str, revision: Value, output: &Path) -> Value {
    request(
        "projects.export",
        json!({"project_id":project_id,"revision":revision,"output_path":output}),
    )
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn create_two_revisions(f: &Fixture) -> (Value, Value) {
    let first = f.ok(request(
        "projects.create",
        json!({"project_id":"work","title":"First","content":content(f, "one")}),
    ));
    let second = f.ok(with_expected(
        request(
            "projects.save_revision",
            json!({"project_id":"work","title":"Second","content":content(f, "two")}),
        ),
        "1",
    ));
    (first, second)
}

#[test]
fn project_root_and_export_are_supported_on_this_host() {
    let f = Fixture::new("caps");
    let caps = f.ok(request("capabilities", json!({})));
    for name in [
        "projects.create",
        "projects.inspect",
        "projects.save_revision",
        "projects.export",
    ] {
        let status = caps["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["command"] == name)
            .unwrap_or_else(|| panic!("{name} listed"))["status"]
            .clone();
        assert_eq!(status, "supported", "{name}");
    }
    let schema = f.ok(request("schema", json!({"command":"projects.export"})));
    assert_eq!(schema["status"], "supported");
    let payload = &schema["request"]["properties"]["payload"];
    assert_eq!(
        payload["required"],
        json!(["project_id", "revision", "output_path"])
    );
    // Without a project root the command stays explicitly unsupported.
    let (_, bare) = call(&[], &request("capabilities", json!({})));
    let status = bare["data"]["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["command"] == "projects.export")
        .unwrap()["status"]
        .clone();
    assert_eq!(status, "unsupported");
    let (_, bare) = call(&[], &export("work", json!("1"), &f.out.join("x.json")));
    assert_eq!(bare["error"]["code"], "unsupported", "{bare}");
}

#[test]
fn export_writes_the_selected_fixed_revision_as_a_new_file() {
    let f = Fixture::new("ok");
    let (first, second) = create_two_revisions(&f);
    assert_eq!(second["latest_revision"], "2");
    let target = f.out.join("work-r1.json");
    let receipt = f.ok(export("work", json!("1"), &target));
    let bytes = std::fs::read(&target).unwrap();
    let document: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(document["format"], "kinekino.project-export");
    assert_eq!(document["format_version"], "1");
    assert_eq!(document["revision_sha256"], first["revision_sha256"]);
    assert_eq!(document["project"], first["project"]);
    assert_eq!(document["project"]["title"], "First");
    assert_eq!(receipt["project_id"], "work");
    assert_eq!(receipt["revision"], "1");
    assert_eq!(receipt["revision_sha256"], first["revision_sha256"]);
    assert_eq!(receipt["output_path"], json!(target));
    assert_eq!(receipt["output_sha256"], sha256(&bytes));
    assert_eq!(receipt["output_bytes"], bytes.len().to_string());
    assert_eq!(receipt["collision_policy"], "reject");
    assert_eq!(receipt["overwrite"], false);
    // Export is read-only for the store: the head revision is unchanged.
    let head = f.ok(request("projects.inspect", json!({"project_id":"work"})));
    assert_eq!(head["latest_revision"], "2");
    assert_eq!(head["revision_sha256"], second["revision_sha256"]);
    assert_eq!(f.output_entries(), vec!["work-r1.json"]);
}

#[test]
fn export_never_overwrites_and_preserves_existing_files() {
    let f = Fixture::new("collide");
    create_two_revisions(&f);
    let target = f.out.join("work.json");
    f.ok(export("work", json!("1"), &target));
    let published = std::fs::read(&target).unwrap();
    assert_eq!(
        f.error(export("work", json!("2"), &target)),
        "revision_conflict"
    );
    assert_eq!(std::fs::read(&target).unwrap(), published);
    let foreign = f.out.join("foreign.json");
    std::fs::write(&foreign, b"keep me").unwrap();
    assert_eq!(
        f.error(export("work", json!("2"), &foreign)),
        "revision_conflict"
    );
    assert_eq!(std::fs::read(&foreign).unwrap(), b"keep me");
    assert_eq!(f.output_entries(), vec!["foreign.json", "work.json"]);
}

#[test]
fn export_rejects_invalid_requests_without_writing() {
    let f = Fixture::new("errors");
    create_two_revisions(&f);
    let target = f.out.join("never.json");
    let cases = [
        (export("missing", json!("1"), &target), "not_found"),
        (export("work", json!("9"), &target), "not_found"),
        (export("work", json!("0"), &target), "invalid_argument"),
        (export("../work", json!("1"), &target), "invalid_argument"),
        (
            request(
                "projects.export",
                json!({"project_id":"work","output_path":target}),
            ),
            "invalid_argument",
        ),
        (
            request(
                "projects.export",
                json!({"project_id":"work","revision":"1","output_path":"relative.json"}),
            ),
            "invalid_argument",
        ),
        (
            request(
                "projects.export",
                json!({"project_id":"work","revision":"1","output_path":""}),
            ),
            "invalid_argument",
        ),
        (
            request(
                "projects.export",
                json!({"project_id":"work","revision":"1","output_path":target,"overwrite":true}),
            ),
            "invalid_argument",
        ),
        (
            with_expected(export("work", json!("1"), &target), "2"),
            "invalid_argument",
        ),
        (
            export("work", json!("1"), &f.out.join("../out/never.json")),
            "invalid_argument",
        ),
        (export("work", json!("1"), &f.out), "revision_conflict"),
        (
            export("work", json!("1"), &f.out.join("absent-dir/never.json")),
            "not_found",
        ),
    ];
    for (request, code) in cases {
        assert_eq!(f.error(request.clone()), code, "{request}");
    }
    assert!(f.output_entries().is_empty(), "{:?}", f.output_entries());
}
