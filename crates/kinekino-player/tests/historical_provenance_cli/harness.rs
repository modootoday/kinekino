use kinekino_player::application::api::Request;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const VERIFIER_PIN: &str = "e00c1726603ef7d0167c6d58deb79e694071044c20a2723de8abe8a504f45e41";
pub fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn inventory(root: &Path) -> BTreeMap<PathBuf, String> {
    fn visit(root: &Path, at: &Path, files: &mut BTreeMap<PathBuf, String>) {
        for entry in fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            assert!(!metadata.file_type().is_symlink());
            if metadata.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    sha(&fs::read(path).unwrap()),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}
fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).unwrap();
        assert!(!metadata.file_type().is_symlink());
        let target = to.join(entry.file_name());
        if metadata.is_dir() {
            copy_tree(&path, &target);
        } else {
            fs::copy(path, target).unwrap();
        }
    }
}

pub struct Fixture {
    evidence: PathBuf,
    workspace: PathBuf,
    executable: PathBuf,
    verifier: PathBuf,
    original_inputs: BTreeMap<PathBuf, String>,
    pub proposals: PathBuf,
    pub projects: PathBuf,
}
pub struct Outcome {
    pub code: i32,
    pub value: Value,
}
impl Fixture {
    pub fn new() -> Self {
        let evidence = PathBuf::from(
            std::env::var_os("HISTORICAL_CLI_EVIDENCE_DIR")
                .expect("explicit new evidence directory required"),
        );
        assert!(evidence.is_absolute());
        fs::create_dir(&evidence).unwrap();
        let verifier = PathBuf::from(
            std::env::var_os("HISTORICAL_VERIFIER_EXE").expect("explicit verifier path required"),
        );
        assert!(verifier.is_absolute());
        assert_eq!(sha(&fs::read(&verifier).unwrap()), VERIFIER_PIN);
        let executable = evidence.join(kinekino_player::domain::product::PRODUCT.cli_executable);
        fs::copy(crate::cli_exe::kinekino_cli_exe(), &executable).unwrap();
        let workspace = evidence.join("workspace");
        fs::create_dir(&workspace).unwrap();
        // Explicit read-only original inputs for isolated source workspaces.
        // Product imports still consume only the fresh copied verifier workspace.
        let source = std::env::var_os("HISTORICAL_INPUT_WORKSPACE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")));
        assert!(source.is_absolute());
        let mut original_inputs = BTreeMap::new();
        let agents = source.join("AGENTS.md");
        original_inputs.insert(agents.clone(), sha(&fs::read(&agents).unwrap()));
        fs::copy(agents, workspace.join("AGENTS.md")).unwrap();
        let relative = Path::new(".spec/artifacts/quality-review/sdk-text-results-r3");
        let package = source.join(relative);
        for (name, hash) in inventory(&package) {
            original_inputs.insert(package.join(name), hash);
        }
        copy_tree(&package, &workspace.join(relative));
        for label in [
            "text-structured-gemma270-smoke-cap-r1",
            "text-structured-llama1b-smoke-r1",
            "text-structured-qwen06-smoke-r1",
        ] {
            let relative = Path::new(".spec/evidence/ai/model-spikes-2026-10-18/results")
                .join(label)
                .join("manifest.json");
            let from = source.join(&relative);
            original_inputs.insert(from.clone(), sha(&fs::read(&from).unwrap()));
            let to = workspace.join(relative);
            fs::create_dir_all(to.parent().unwrap()).unwrap();
            fs::copy(from, to).unwrap();
        }
        let proposals = evidence.join("proposals");
        let projects = evidence.join("projects");
        fs::create_dir(&proposals).unwrap();
        fs::create_dir(&projects).unwrap();
        fs::write(evidence.join("input-pins.json"), serde_json::to_vec_pretty(&json!({"cli_sha256":sha(&fs::read(&executable).unwrap()),"verifier_sha256":VERIFIER_PIN,"original_inputs":original_inputs})).unwrap()).unwrap();
        Self {
            evidence,
            workspace,
            executable,
            verifier,
            original_inputs,
            proposals,
            projects,
        }
    }
    pub fn package(&self) -> PathBuf {
        self.workspace
            .join(".spec/artifacts/quality-review/sdk-text-results-r3")
    }
    pub fn sdk(&self) -> impl kinekino_player::application::api::ApplicationClient {
        let reader =
            kinekino_player::infrastructure::historical_evidence::HistoricalVerifier::open(
                &self.verifier,
                VERIFIER_PIN.into(),
                &self.workspace,
                "sdk-text-results-r3".into(),
            )
            .unwrap();
        let store =
            kinekino_player::infrastructure::proposals::ProposalFiles::open(&self.proposals)
                .unwrap()
                .with_historical_reader(std::sync::Arc::new(reader));
        kinekino_player::application::api::PlayerSdk::default().with_proposal_store(store)
    }
    pub fn companion(&self) -> PathBuf {
        self.package().join("historical-evidence-draft.json")
    }
    pub fn assert_original_inputs_unchanged(&self) {
        for (path, hash) in &self.original_inputs {
            assert_eq!(&sha(&fs::read(path).unwrap()), hash);
        }
    }
    pub fn invoke(
        &self,
        name: &str,
        command: &str,
        payload: Value,
        expected: Option<&str>,
        configured: bool,
    ) -> Outcome {
        let mut request = Request::new(command);
        request.context.expected_revision = expected.map(str::to_owned);
        request.payload = payload;
        let mut command = Command::new(&self.executable);
        command
            .args(["--input", "-", "--proposal-root"])
            .arg(&self.proposals)
            .arg("--project-root")
            .arg(&self.projects);
        if configured {
            command
                .arg("--historical-verifier")
                .arg(&self.verifier)
                .args([
                    "--historical-verifier-sha256",
                    VERIFIER_PIN,
                    "--historical-workspace",
                ])
                .arg(&self.workspace)
                .args(["--historical-package", "sdk-text-results-r3"]);
        }
        let mut child = command
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
            .write_all(&serde_json::to_vec(&request).unwrap())
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let output = thread::spawn(move || {
            let mut bytes = Vec::new();
            stdout.take(8_388_609).read_to_end(&mut bytes).unwrap();
            bytes
        });
        let diagnostics = thread::spawn(move || {
            let mut bytes = Vec::new();
            stderr.take(65_537).read_to_end(&mut bytes).unwrap();
            bytes
        });
        let deadline = Instant::now() + Duration::from_secs(90);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("owned CLI deadline exceeded");
            }
            thread::sleep(Duration::from_millis(5));
        };
        child.wait().unwrap();
        let stdout = output.join().unwrap();
        let stderr = diagnostics.join().unwrap();
        fs::write(self.evidence.join(format!("{name}.stdout")), &stdout).unwrap();
        fs::write(self.evidence.join(format!("{name}.stderr")), &stderr).unwrap();
        assert!(stdout.len() <= 8_388_608 && stderr.len() <= 65_536);
        let code = status.code().unwrap();
        fs::write(
            self.evidence.join(format!("{name}.receipt.json")),
            serde_json::to_vec_pretty(
                &json!({"request":request,"pid":pid,"exit_code":code,"reaped":true}),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(stderr.is_empty());
        Outcome {
            code,
            value: serde_json::from_slice(&stdout).unwrap(),
        }
    }
    pub fn ok(&self, name: &str, command: &str, payload: Value, expected: Option<&str>) -> Value {
        let result = self.invoke(name, command, payload, expected, true);
        assert_eq!(result.code, 0, "{name}: {}", result.value["error"]);
        assert_eq!(result.value["ok"], true);
        result.value["data"].clone()
    }
}
