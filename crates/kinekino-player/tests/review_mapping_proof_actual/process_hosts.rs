//! Explicit ignored process fixture. The nomination remains pinned trusted Rust test input.
use super::*;
use kinekino_player::{
    assembly::review_playback::start_transport, infrastructure::local_host::TransportOptions,
};
use std::{
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn output() -> PathBuf {
    let path = PathBuf::from(std::env::var("KINEKINO_REVIEW_OUTPUT_DIR").unwrap());
    assert!(path.starts_with(
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join("runtime/verification")
    ));
    path
}
fn admission() -> TrustedTimelineAdmission {
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .join("runtime/verification/sprite-timeline-actual-r1");
    let guard = custody::Guard::acquire(&root.join("NOMINATION.json"));
    assert_eq!(
        guard.sha256,
        "b1a47d17ad348d6ac7c4c5ec9eb246a54f561babc442cf4310274cdb349be201"
    );
    let data: Value =
        serde_json::from_slice(&fs::read(root.join("NOMINATION.json")).unwrap()).unwrap();
    let result = trusted_admission(
        serde_json::from_value(data["case"].clone()).unwrap(),
        serde_json::from_value(data["media"].clone()).unwrap(),
        serde_json::from_value(data["evidence"].clone()).unwrap(),
    );
    guard.assert_unchanged();
    result
}

#[test]
#[ignore = "private child of explicitly allocated process EOF/restart test"]
fn review_host_process_child() {
    kinekino_platform::install();
    let epoch = std::env::var("KINEKINO_REVIEW_HOST_EPOCH").unwrap();
    assert!(matches!(
        epoch.as_str(),
        "first" | "second" | "broken-stdout"
    ));
    let output = output();
    let folder = output.join(&epoch);
    let source = PathBuf::from(std::env::var("KINEKINO_REVIEW_SOURCE_RECEIPT_DIR").unwrap())
        .join("SOURCE.md");
    let source_guard = custody::Guard::acquire(&source);
    assert_eq!(
        source_guard.sha256,
        std::env::var("KINEKINO_REVIEW_SOURCE_RECEIPT_SHA256")
            .unwrap()
            .to_ascii_lowercase()
    );
    let producer = custody::Guard::acquire(&std::env::current_exe().unwrap());
    let workspace = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let (mut mask, mut system) = (0usize, 0usize);
    unsafe {
        windows::Win32::System::Threading::GetProcessAffinityMask(
            windows::Win32::System::Threading::GetCurrentProcess(),
            &mut mask,
            &mut system,
        )
        .unwrap();
    }
    let config = sessions::fixed_config(
        &workspace.join("runtime/verification/sprite-timeline-actual-r1"),
        &output.join("user-review"),
        admission(),
        output.join("process-runtime"),
        workspace.join("runtime/toolchains/mpv-dde0671072-review-r1/mpv.exe"),
        kinekino_process::infrastructure::family::JobBudget {
            max_committed_bytes: 1024 * 1024 * 1024,
            affinity_mask: mask.isolate_lowest_one(),
        },
    );
    let tool = custody::Guard::acquire(&config.executable);
    assert_eq!(tool.sha256, config.profile.engine_sha256);
    let assembly = kinekino_player::assembly::review_playback::assemble_sdk(
        PlayerSdk::default(),
        config,
        false,
        4,
    )
    .unwrap();
    let host = start_transport(
        assembly.client,
        assembly.sessions,
        TransportOptions::default(),
    )
    .unwrap();
    let ready = json!({"endpoint":host.endpoint(),"pid":std::process::id(),"source":source_guard.evidence(),"producer":producer.evidence(),"new_model_execution":false});
    write_new(&folder.join("READY.json"), &ready);
    let input = std::io::stdin();
    let output_stream = std::io::stdout();
    let outcome = if epoch == "broken-stdout" {
        let mut gate = GateWriter {
            input: input.lock(),
            output: output_stream.lock(),
            waited: false,
        };
        kinekino_player::assembly::review_playback::serve_stdio(
            host,
            &ready,
            &mut std::io::empty(),
            &mut gate,
        )
    } else {
        kinekino_player::assembly::review_playback::serve_stdio(
            host,
            &ready,
            &mut input.lock(),
            &mut output_stream.lock(),
        )
    };
    let done = match outcome {
        kinekino_player::assembly::review_playback::ReviewStdioOutcome::Complete(done) => done,
        kinekino_player::assembly::review_playback::ReviewStdioOutcome::Incomplete { .. } => {
            panic!("original stdio host cleanup incomplete")
        }
    };
    write_new(
        &folder.join("STOPPED.json"),
        &json!({"pid":std::process::id(),"reason":if epoch=="broken-stdout" {"ready_output_failure"}else{"stdin_eof"},"pipe":format!("{:?}",done.shutdown.transport.pipe),"sdk":format!("{:?}",done.shutdown.transport.sdk),"sessions":done.shutdown.sessions,"complete":true,"first_error":done.first_error(),"succeeded":done.succeeded()}),
    );
    source_guard.assert_unchanged();
    producer.assert_unchanged();
    tool.assert_unchanged();
    if epoch == "broken-stdout" {
        assert!(!done.succeeded());
        assert_eq!(done.first_error().unwrap().details["io_kind"], "BrokenPipe");
        // All original owners were physically completed before this expected
        // failure exit; avoid a test harness write to the intentionally closed pipe.
        std::process::exit(i32::from(done.first_error().unwrap().exit_code()));
    }
    assert!(done.succeeded(), "{done:?}");
}

struct GateWriter<R, W> {
    input: R,
    output: W,
    waited: bool,
}
impl<R: std::io::Read, W: std::io::Write> std::io::Write for GateWriter<R, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if !self.waited {
            let mut signal = [0u8];
            if self.input.read(&mut signal)? != 0 {
                return Err(std::io::ErrorKind::InvalidData.into());
            }
            self.waited = true;
        }
        self.output.write(bytes)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.output.flush()
    }
}

struct ProcessHost {
    child: Child,
    folder: PathBuf,
    finished: bool,
    started: Instant,
}
impl ProcessHost {
    fn start(epoch: &str) -> (Self, String) {
        let folder = output().join(epoch);
        fs::create_dir(&folder).unwrap();
        let file = |name| {
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(folder.join(name))
                .unwrap()
        };
        let child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "process_hosts::review_host_process_child",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("KINEKINO_REVIEW_HOST_EPOCH", epoch)
            .stdin(Stdio::piped())
            .stdout(if epoch == "broken-stdout" {
                Stdio::piped()
            } else {
                Stdio::from(file("stdout.raw"))
            })
            .stderr(file("stderr.raw"))
            .spawn()
            .unwrap();
        let mut owned = Self {
            child,
            folder,
            finished: false,
            started: Instant::now(),
        };
        let cutoff = Instant::now() + Duration::from_secs(15);
        loop {
            let ready = owned.folder.join("READY.json");
            if ready.is_file() {
                // Creation precedes write; retry only incomplete JSON while this original child lives.
                if let Ok(data) = serde_json::from_slice::<Value>(&fs::read(&ready).unwrap()) {
                    assert_eq!(data["pid"].as_u64().unwrap(), u64::from(owned.child.id()));
                    let endpoint = data["endpoint"].as_str().unwrap().to_owned();
                    return (owned, endpoint);
                }
            }
            assert!(
                owned.child.try_wait().unwrap().is_none(),
                "original host exited before readiness; see {}",
                owned.folder.display()
            );
            assert!(Instant::now() < cutoff, "original host readiness timed out");
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn eof_and_wait(&mut self) -> Value {
        self.eof_and_wait_expected(true)
    }
    fn eof_and_wait_expected(&mut self, success: bool) -> Value {
        drop(self.child.stdin.take());
        let cutoff = Instant::now() + Duration::from_secs(15);
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < cutoff,
                "original host did not exit after EOF"
            );
            thread::sleep(Duration::from_millis(10));
        };
        self.finished = true;
        let stopped: Value =
            serde_json::from_slice(&fs::read(self.folder.join("STOPPED.json")).unwrap()).unwrap();
        write_new(
            &self.folder.join("PROCESS.json"),
            &json!({"pid":self.child.id(),"exit_code":status.code(),"forced":false,"elapsed_ms":self.started.elapsed().as_millis(),"stopped":stopped}),
        );
        assert_eq!(status.success(), success);
        assert_eq!(stopped["pipe"], "Joined");
        assert_eq!(stopped["sdk"], "Joined");
        assert_eq!(stopped["sessions"]["complete"], true);
        assert_eq!(stopped["sessions"]["owners"], 1);
        stopped
    }
}
impl Drop for ProcessHost {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.child.kill();
            let status = self.child.wait();
            write_new(
                &self.folder.join("FAILED-OWNED-CLEANUP.json"),
                &json!({"pid":self.child.id(),"forced":true,"wait":format!("{status:?}")}),
            );
        }
    }
}
fn call(endpoint: &str, folder: &Path, label: &str, request: &Request, ok: bool) -> Value {
    let directory = folder.join(label);
    fs::create_dir(&directory).unwrap();
    write_new(
        &directory.join("request.json"),
        &serde_json::to_value(request).unwrap(),
    );
    let mut runner = cli::CliRunner::start(endpoint, request);
    let result = runner.finish_preserving(&directory);
    assert_eq!(result.code == 0, ok, "{result:?}");
    assert_eq!(result.response["ok"], ok);
    result.response.clone()
}
fn open_and_wait(endpoint: &str, folder: &Path) -> Value {
    let mut request = Request::new("reviews.playback.open");
    request.context.idempotency_key = Some("open".into());
    request.payload = json!({"selection":{"case":admission().case,"association_id":"sealed-sprite","association_revision":"2"}});
    let opened = call(endpoint, folder, "open", &request, true);
    request = Request::new("reviews.playback.status");
    request.payload = json!({"session":opened["data"]["session"]});
    let cutoff = Instant::now() + Duration::from_secs(20);
    for index in 0..100 {
        let status = call(endpoint, folder, &format!("status-{index}"), &request, true);
        assert_eq!(
            status["data"]["observation"]["has_error"], false,
            "{status}"
        );
        if status["data"]["observation"]["connected"] == true {
            return status["data"]["session"].clone();
        }
        assert!(Instant::now() < cutoff, "original session did not connect");
        thread::sleep(Duration::from_millis(100));
    }
    panic!("session status observation exhausted");
}
fn media_receipts() -> Vec<Value> {
    let mut result = Vec::new();
    for entry in fs::read_dir(output().join("process-runtime")).unwrap() {
        let file = entry.unwrap().path().join("PROCESS.json");
        if file.is_file() {
            let record: Value = serde_json::from_slice(&fs::read(file).unwrap()).unwrap();
            assert_eq!(record["forced"], false);
            assert_eq!(record["child_exit"], 0);
            assert_eq!(record["family_empty"], true);
            assert_eq!(record["ipc_exited"], true);
            assert!(record["first_error"].is_null());
            result.push(record);
        }
    }
    result
}
pub(super) fn verify_broken_stdout() {
    let producer = custody::Guard::acquire(&std::env::current_exe().unwrap());
    let source = custody::Guard::acquire(
        &PathBuf::from(std::env::var("KINEKINO_REVIEW_SOURCE_RECEIPT_DIR").unwrap())
            .join("SOURCE.md"),
    );
    let cli_producer = custody::Guard::acquire(Path::new(crate::cli_exe::kinekino_cli_exe()));
    write_new(
        &output().join("PROCESS-HOST-IO-WITNESS.json"),
        &json!({"source":source.evidence(),"producer":producer.evidence(),"cli":cli_producer.evidence(),"requested_owned_host_processes":1,"requested_media_processes":1,"fault":"parent closes real child stdout reader after media connects; gate releases on stdin EOF","new_model_execution":false}),
    );
    fs::create_dir(output().join("process-runtime")).unwrap();
    let (mut host, endpoint) = ProcessHost::start("broken-stdout");
    let session = open_and_wait(&endpoint, &host.folder);
    let reader = host
        .child
        .stdout
        .take()
        .expect("original piped stdout reader");
    drop(reader);
    write_new(
        &host.folder.join("STDOUT-CLOSE.json"),
        &json!({"pid":host.child.id(),"session":session,"after_connected":true,"real_parent_read_handle_closed":true,"stdout_prefix_not_captured":true}),
    );
    let stopped = host.eof_and_wait_expected(false);
    let receipt: Value =
        serde_json::from_slice(&fs::read(host.folder.join("PROCESS.json")).unwrap()).unwrap();
    assert_eq!(receipt["exit_code"], 8);
    assert_eq!(stopped["first_error"]["code"], "engine_failed");
    assert_eq!(stopped["first_error"]["details"]["io_kind"], "BrokenPipe");
    assert!(
        ["ready_write", "ready_flush"]
            .contains(&stopped["first_error"]["details"]["phase"].as_str().unwrap())
    );
    assert_eq!(stopped["succeeded"], false);
    let media = media_receipts();
    assert_eq!(media.len(), 1);
    producer.assert_unchanged();
    source.assert_unchanged();
    cli_producer.assert_unchanged();
    write_new(
        &output().join("PROCESS-HOST-IO-RETURN.json"),
        &json!({"source":source.evidence(),"producer":producer.evidence(),"cli":cli_producer.evidence(),"host":receipt,"stopped":stopped,"media":media,"new_decode_execution":true,"actual_broken_stdout":true,"first_error_preserved":true,"physical_cleanup_complete":true,"forced":false,"quality_approved":false,"actual9_binding":false}),
    );
}
pub(super) fn verify() {
    let producer = custody::Guard::acquire(&std::env::current_exe().unwrap());
    let source = custody::Guard::acquire(
        &PathBuf::from(std::env::var("KINEKINO_REVIEW_SOURCE_RECEIPT_DIR").unwrap())
            .join("SOURCE.md"),
    );
    let cli_producer = custody::Guard::acquire(Path::new(crate::cli_exe::kinekino_cli_exe()));
    write_new(
        &output().join("PROCESS-HOST-WITNESS.json"),
        &json!({"source":source.evidence(),"producer":producer.evidence(),"cli":cli_producer.evidence(),"requested_owned_host_processes":2,"requested_serial_media_processes":2,"new_model_execution":false,"new_capture_execution":false}),
    );
    fs::create_dir(output().join("process-runtime")).unwrap();
    let (mut first, endpoint1) = ProcessHost::start("first");
    let old = open_and_wait(&endpoint1, &first.folder);
    let stopped1 = first.eof_and_wait();
    let (mut second, endpoint2) = ProcessHost::start("second");
    assert_ne!(first.child.id(), second.child.id());
    assert_ne!(endpoint1, endpoint2);
    let mut request = Request::new("reviews.playback.status");
    request.payload = json!({"session":old});
    let rejected = call(&endpoint2, &second.folder, "stale-session", &request, false);
    assert_eq!(rejected["error"]["code"], "revision_conflict");
    let new = open_and_wait(&endpoint2, &second.folder);
    assert_eq!(old["ordinal"], new["ordinal"]);
    assert_ne!(old["host_scope"], new["host_scope"]);
    let stopped2 = second.eof_and_wait();
    let media_processes = media_receipts();
    assert_eq!(media_processes.len(), 2);
    producer.assert_unchanged();
    source.assert_unchanged();
    cli_producer.assert_unchanged();
    write_new(
        &output().join("PROCESS-HOST-RETURN.json"),
        &json!({"source":source.evidence(),"producer":producer.evidence(),"cli":cli_producer.evidence(),"first_endpoint":endpoint1,"second_endpoint":endpoint2,"old":old,"new":new,"first_stopped":stopped1,"second_stopped":stopped2,"media_processes":media_processes,"new_decode_execution":true,"owned_host_processes":2,"stale_reference_denied":true,"same_key_is_new_host_session":true,"forced":false,"quality_approved":false,"actual9_binding":false}),
    );
}
