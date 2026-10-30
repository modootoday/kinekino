#![cfg(windows)]
#[path = "support/owned_process_evidence.rs"]
pub mod owned_process_evidence;
use kinekino_player::infrastructure::content_digest;
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn hash(bytes: &[u8]) -> String {
    content_digest::sha256(
        &mut std::io::Cursor::new(bytes),
        kinekino_player::application::local_host::MAX_RESPONSE_BYTES as u64,
    )
    .unwrap()
    .1
}
struct Fixture {
    root: PathBuf,
    executable: PathBuf,
    sha256: String,
    bytes: u64,
}
impl Fixture {
    fn new() -> Self {
        let base = PathBuf::from(
            std::env::var_os("VAD_PRODUCT_CLI_EVIDENCE_ROOT").expect("Explicit evidence root"),
        );
        let executable = PathBuf::from(
            std::env::var_os("VAD_PRODUCT_CLI_BIN").expect("Pin normal player-cli, never libtest"),
        );
        assert!(base.is_absolute() && executable.is_absolute());
        let sha256 = std::env::var("VAD_PRODUCT_CLI_SHA256").expect("Normal CLI SHA256");
        let bytes = std::env::var("VAD_PRODUCT_CLI_BYTES")
            .expect("Normal CLI bytes")
            .parse()
            .unwrap();
        fs::create_dir_all(&base).unwrap();
        let root = base.join(format!(
            "run-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let fixture = Self {
            root,
            executable,
            sha256,
            bytes,
        };
        fixture.check_binary();
        fs::write(fixture.root.join("binary.json"), serde_json::to_vec_pretty(&json!({"path":fixture.executable,"sha256":fixture.sha256,"bytes":fixture.bytes,"kind":"externally_pinned_normal_cli"})).unwrap()).unwrap();
        fixture
    }
    fn check_binary(&self) {
        assert_eq!(fs::metadata(&self.executable).unwrap().len(), self.bytes);
        assert_eq!(
            content_digest::sha256(
                &mut fs::File::open(&self.executable).unwrap(),
                128 * 1024 * 1024
            )
            .unwrap()
            .1,
            self.sha256
        );
    }
    fn call(
        &self,
        name: &str,
        extra: Vec<OsString>,
        exit: u32,
        prepare_job_root: bool,
    ) -> Vec<Value> {
        self.check_binary();
        let evidence = self.root.join(name);
        fs::create_dir(&evidence).unwrap();
        if prepare_job_root {
            fs::create_dir(evidence.join("jobs")).unwrap();
        }
        let mut args = vec![
            "--serve-jobs".into(),
            "--job-root".into(),
            evidence.join("jobs").into_os_string(),
        ];
        args.extend(extra);
        fs::write(
            evidence.join("arguments.json"),
            serde_json::to_vec_pretty(&args).unwrap(),
        )
        .unwrap();
        let output = match owned_process_evidence::run_process_with_arguments(
            self.executable.clone(),
            &evidence,
            b"",
            args,
            Duration::from_secs(15),
            false,
        ) {
            Ok(output) => output,
            Err(failure) => {
                // Retain the original carrier before any fallible diagnostic or panic.
                let retained = Box::leak(failure.closeout());
                panic!("Original CLI failure retained: {retained:?}");
            }
        };
        owned_process_evidence::write_json(&evidence.join("observation.json"), &json!({"receipt":owned_process_evidence::receipt_json(&output.receipt),"stdout":output.stdout_evidence,"stderr":output.stderr_evidence,"elapsed_ms":output.elapsed_ms})).unwrap();
        assert_eq!(output.receipt.child_exit, Some(exit));
        assert!(!output.receipt.forced);
        assert!(output.receipt.stdin_eof_requested);
        assert!(output.stderr.is_empty());
        output.check_deadline(&evidence).unwrap();
        self.check_binary();
        std::str::from_utf8(&output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
    fn reject(&self, name: &str, args: Vec<OsString>, code: &str, exit: u32) {
        let rows = self.call(name, args, exit, false);
        assert_eq!(rows.len(), 1, "No ready or endpoint before failure");
        assert_eq!(rows[0]["ok"], false);
        assert_eq!(rows[0]["error"]["code"], code);
        assert!(rows[0].get("event").is_none());
        assert!(rows[0].get("endpoint").is_none());
        assert!(!self.root.join(name).join("jobs").exists());
    }
}
#[test]
#[ignore = "Central slot plus pinned normal CLI and evidence directory required"]
fn normal_cli_rejects_partial_duplicate_and_relative_vad_addresses() {
    kinekino_platform::install();
    let f = Fixture::new();
    let release = f.root.join("release").into_os_string();
    let workspace = f.root.join("workspace").into_os_string();
    f.reject(
        "partial-root",
        vec!["--vad-root".into(), release.clone()],
        "invalid_argument",
        2,
    );
    f.reject(
        "partial-workspace",
        vec!["--vad-workspace".into(), workspace.clone()],
        "invalid_argument",
        2,
    );
    f.reject(
        "duplicate",
        vec![
            "--vad-root".into(),
            release.clone(),
            "--vad-root".into(),
            release,
            "--vad-workspace".into(),
            workspace.clone(),
        ],
        "invalid_argument",
        2,
    );
    f.reject(
        "relative",
        vec![
            "--vad-root".into(),
            "relative".into(),
            "--vad-workspace".into(),
            workspace,
        ],
        "invalid_argument",
        2,
    );
}
#[test]
#[ignore = "Central slot plus pinned normal CLI and evidence directory required"]
fn normal_cli_unconfigured_product_needs_input_before_ready() {
    kinekino_platform::install();
    let f = Fixture::new();
    f.reject(
        "unconfigured",
        vec![
            "--vad-root".into(),
            f.root.join("release").into_os_string(),
            "--vad-workspace".into(),
            f.root.join("workspace").into_os_string(),
        ],
        "needs_input",
        3,
    );
}
#[test]
#[ignore = "Central slot plus pinned normal CLI and evidence directory required"]
fn normal_cli_default_jobs_ready_then_eof_has_both_shutdown_receipts() {
    kinekino_platform::install();
    let f = Fixture::new();
    let rows = f.call("default-eof", vec![], 0, true);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["event"], "job_host_ready");
    assert_eq!(rows[0]["stop"], "stdin_eof");
    assert!(
        rows[0]["endpoint"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    let operations = rows[0]["execution_operations"].as_array().unwrap();
    assert_eq!(operations.len(), 2);
    assert!(operations.contains(&json!("subtitles.convert")));
    assert!(operations.contains(&json!("subtitles.export_range")));
    assert_eq!(rows[1]["event"], "job_host_stopped");
    assert_eq!(
        rows[1]["receipt"],
        json!({"completed_total":0,"cancelled_total":0,"interrupted_total":0})
    );
    assert_eq!(
        rows[1]["local_host_shutdown"],
        json!({"pipe":"joined","sdk":"joined","dispatch":"idle","dispatch_poisoned":false})
    );
}
