#![cfg(windows)]
#[path = "support/local_host_cli.rs"]
pub mod cli;
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "sprite_capture_cli/custody.rs"]
mod custody;
#[path = "execution_host_cli/receipt_fault.rs"]
mod receipt_fault;
#[path = "execution_host_cli/submission_rejections.rs"]
mod submission_rejections;
use kinekino_player::application::api::Request;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
struct Owner(std::process::Child);
impl Drop for Owner {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
#[ignore = "central normal CLI execution submit/peek/ACK/EOF with pinned local tools; no capture/model/network/GUI"]
fn actual_execution_cli_admission_terminal_ack_and_eof() {
    kinekino_platform::install();
    run(false, 0);
}
#[test]
#[ignore = "central normal CLI unacknowledged result retains incomplete shutdown; no capture/model/network/GUI"]
fn actual_execution_cli_unacknowledged_eof_is_not_cleanup_success() {
    kinekino_platform::install();
    run(true, 0);
}
#[test]
#[ignore = "central actual normal CLI/probe immutable receipt collision and explicit recovery; no inference/network/GUI"]
fn actual_execution_cli_frame_receipt_recovery_keeps_first_error() {
    kinekino_platform::install();
    run(false, 1);
}
#[test]
#[ignore = "central actual CLI two immutable receipt collisions, stale generation rejection and original retry; no inference/network/GUI"]
fn actual_execution_cli_repeated_receipt_failures_require_new_generation() {
    kinekino_platform::install();
    run(false, 2);
}
#[test]
#[ignore = "central actual CLI sixteen receipt generations exhausted; expected incomplete EOF; no inference/network/GUI"]
fn actual_execution_cli_receipt_budget_exhaustion_never_reports_cleanup_success() {
    kinekino_platform::install();
    run(true, 17);
}
fn run(unacknowledged: bool, collisions: usize) {
    let frame_fault = collisions > 0;
    let root = PathBuf::from(std::env::var_os("EXECUTION_CLI_OUTPUT").unwrap());
    fs::create_dir(&root).unwrap();
    let source = custody::Guard::acquire(&PathBuf::from(
        std::env::var_os("EXECUTION_CLI_SOURCE").unwrap(),
    ));
    let producer =
        custody::Guard::acquire(std::path::Path::new(crate::cli_exe::kinekino_cli_exe()));
    fs::write(root.join("WITNESS.json"),serde_json::to_vec_pretty(&json!({"source":source.evidence(),"producer":producer.evidence(),"unacknowledged":unacknowledged,"quality_approved":false})).unwrap()).unwrap();
    let child = Command::new(crate::cli_exe::kinekino_cli_exe())
        .args(["--serve-execution", "--frame-ffmpeg"])
        .arg(std::env::var_os("KINEKINO_TIMELINE_CAPTURE_FFMPEG").unwrap())
        .args([
            "--frame-ffmpeg-sha256",
            "09948d4cdd0650da6ff5a87577469f2a218dc2615ae379f8f734d24c49de0f73",
            "--frame-ffprobe",
        ])
        .arg(std::env::var_os("KINEKINO_TIMELINE_FFPROBE").unwrap())
        .args([
            "--frame-ffprobe-sha256",
            "a6618e99bb58869ded3c6f37b53aa1a8d701c3591dbb7b5b317d47369c112be2",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut owner = Owner(child);
    let pid = owner.0.id();
    let stdout = owner.0.stdout.take().unwrap();
    let stderr = owner.0.stderr.take().unwrap();
    let (tx, rx) = mpsc::channel();
    let out = thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let _ = tx.send(line.clone());
        let mut tail = String::new();
        reader.read_to_string(&mut tail).unwrap();
        format!("{line}{tail}")
    });
    let err = thread::spawn(move || {
        let mut bytes = Vec::new();
        BufReader::new(stderr).read_to_end(&mut bytes).unwrap();
        bytes
    });
    let ready: Value =
        serde_json::from_str(&rx.recv_timeout(Duration::from_secs(15)).unwrap()).unwrap();
    fs::write(
        root.join("READY.json"),
        serde_json::to_vec_pretty(&ready).unwrap(),
    )
    .unwrap();
    assert_eq!(ready["host"], "execution");
    assert_eq!(ready["pid"], pid);
    let endpoint = ready["endpoint"].as_str().unwrap();
    let call = |label: &str, command: &str, payload: Value| {
        let directory = root.join(label);
        fs::create_dir(&directory).unwrap();
        let mut request = Request::new(command);
        request.payload = payload;
        fs::write(
            directory.join("request.json"),
            serde_json::to_vec_pretty(&request).unwrap(),
        )
        .unwrap();
        let mut runner = cli::CliRunner::start(endpoint, &request);
        let receipt = runner.finish_preserving(&directory);
        assert_eq!(receipt.code, 0, "{receipt:?}");
        assert_eq!(receipt.response["ok"], true);
        receipt.response["data"].clone()
    };
    let capabilities = call("capabilities", "capabilities", json!({}));
    assert_eq!(
        call("host-version", "version", json!({}))["host_protocol"],
        "execution-host/1"
    );
    let commands = capabilities["commands"].as_array().unwrap();
    assert_eq!(commands.len(), 10);
    assert_eq!(ready["commands"].as_array().unwrap().len(), commands.len());
    for (index, command) in commands.iter().enumerate() {
        assert_eq!(command["status"], "supported");
        assert_eq!(&ready["commands"][index], &command["command"]);
        let schema = call(
            &format!("schema-{index}"),
            "schema",
            json!({"command":command["command"]}),
        );
        assert_eq!(schema["command"], command["command"]);
        assert_eq!(schema["host_protocol"], "execution-host/1");
        assert_eq!(schema["status"], "supported");
        assert_eq!(
            schema["request"]["properties"]["command"]["const"],
            command["command"]
        );
    }
    let fault = frame_fault.then(|| receipt_fault::prepare(pid, collisions));
    let inner = if frame_fault {
        receipt_fault::request(&root)
    } else {
        Request::new("version")
    };
    let submission =
        json!({"host_scope":ready["host_scope"],"submission_id":"explicit-normal-cli-1"});
    let correlated = !frame_fault && !unacknowledged;
    let admitted = if correlated {
        // Deliberately discard the first application response; actual pipe loss
        // is tested separately by the raw authenticated-pipe characterization.
        call(
            "submit",
            "execution.submit",
            json!({"request":inner,"submission":submission}),
        );
        let found = call(
            "submission-inspect",
            "execution.submission.inspect",
            json!({"submission":submission}),
        );
        let repeated = call(
            "submission-repeat",
            "execution.submit",
            json!({"request":inner,"submission":submission}),
        );
        assert_eq!(repeated["operation"], found["operation"]);
        assert_eq!(repeated["replayed"], true);
        assert_eq!(repeated["acknowledged"], false);
        submission_rejections::check(&root, endpoint, &submission, &inner);
        let preserved = call(
            "submission-after-rejections",
            "execution.submission.inspect",
            json!({"submission":submission}),
        );
        assert_eq!(preserved["operation"], found["operation"]);
        assert_eq!(preserved["request_sha256"], found["request_sha256"]);
        assert_eq!(
            call("list-after-rejections", "execution.list", json!({}))["operations"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        found
    } else {
        let receipt = call("submit", "execution.submit", json!({"request":inner}));
        assert_eq!(receipt["completion"], "admitted");
        receipt
    };
    let operation = admitted["operation"].clone();
    if let Some(fault) = fault {
        receipt_fault::recover(&root, fault, &operation, endpoint, &call);
    }
    let cutoff = Instant::now() + Duration::from_secs(5);
    for poll in 0.. {
        if collisions == 17 {
            break;
        }
        let state = call(
            &format!("peek-{poll}"),
            "execution.peek",
            json!({"operation":operation}),
        );
        if state["state"]["phase"] == "terminal" {
            assert_eq!(state["state"]["response"]["ok"], !frame_fault);
            if frame_fault {
                receipt_fault::verify_terminal(&root, &state["state"]["response"]);
            }
            break;
        }
        assert!(Instant::now() < cutoff);
    }
    call("list", "execution.list", json!({}));
    if !unacknowledged {
        call(
            "ack",
            "execution.acknowledge",
            json!({"operation":operation}),
        );
        if correlated {
            let old = call(
                "submission-after-ack",
                "execution.submission.inspect",
                json!({"submission":submission}),
            );
            assert_eq!(old["acknowledged"], true);
            let repeated = call(
                "submission-repeat-after-ack",
                "execution.submit",
                json!({"request":inner,"submission":submission}),
            );
            assert_eq!(repeated["acknowledged"], true);
            assert_eq!(repeated["operation"], operation);
            assert!(
                call("list-after-repeat", "execution.list", json!({}))["operations"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
        }
    }
    drop(owner.0.stdin.take());
    let cutoff = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = owner.0.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < cutoff);
        thread::sleep(Duration::from_millis(5));
    };
    let stdout = out.join().unwrap();
    let stderr = err.join().unwrap();
    fs::write(root.join("stdout.raw"), stdout).unwrap();
    fs::write(root.join("stderr.raw"), &stderr).unwrap();
    fs::write(root.join("PROCESS.json"),serde_json::to_vec_pretty(&json!({"pid":pid,"exit_code":status.code(),"stdin_eof":true,"original_child_waited":true,"streams_joined":true,"forced":false,"host_reported_success":status.success()})).unwrap()).unwrap();
    if unacknowledged {
        assert_eq!(status.code(), Some(10));
        let failure: Value = serde_json::from_slice(&stderr).unwrap();
        assert_eq!(failure["cleanup_complete"], false);
    } else {
        assert_eq!(status.code(), Some(0));
        assert!(stderr.is_empty());
    }
    source.assert_unchanged();
    producer.assert_unchanged();
}
