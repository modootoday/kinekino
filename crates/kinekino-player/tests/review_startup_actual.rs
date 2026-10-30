#![cfg(windows)]
#[path = "support/local_host_cli.rs"]
pub mod cli;
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "review_startup_actual/controls.rs"]
mod controls;
#[path = "sprite_capture_cli/custody.rs"]
mod custody;
#[path = "review_startup_actual/matroska.rs"]
mod matroska;
use kinekino_player::application::api::Request;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
fn write_new(path: &Path, value: &Value) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    f.write_all(&serde_json::to_vec_pretty(value).unwrap())
        .unwrap();
    f.sync_all().unwrap();
}
struct Owner(Child);
impl Drop for Owner {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn call(endpoint: &str, root: &Path, name: &str, request: &Request, success: bool) -> Value {
    let root = root.join(name);
    fs::create_dir(&root).unwrap();
    write_new(&root.join("request.json"), &json!(request));
    let mut runner = if std::env::var_os("REVIEW_MATROSKA_TRANSPORT_BUDGET").is_some() {
        cli::CliRunner::start_with_transport_timeout(endpoint, request, Some(30000))
    } else {
        cli::CliRunner::start(endpoint, request)
    };
    let receipt = runner.finish_preserving(&root);
    assert_eq!(receipt.code == 0, success, "{name}: {receipt:?}");
    assert_eq!(
        receipt.response["ok"], success,
        "{name}: {:?}",
        receipt.response
    );
    receipt.response.clone()
}
fn request(command: &str, payload: Value) -> Request {
    let mut r = Request::new(command);
    r.payload = payload;
    r
}
fn fixture_root() -> PathBuf {
    std::env::var_os("KINEKINO_REVIEW_FIXTURE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
                .join("runtime/verification/sprite-timeline-actual-r1")
        })
}
fn playback_tool() -> PathBuf {
    std::env::var_os("KINEKINO_REVIEW_PLAYBACK_TOOL")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
                .join("runtime/toolchains/mpv-dde0671072-review-r1/mpv.exe")
        })
}
fn settings(root: &Path, nomination: &Value, mask: usize) -> Value {
    json!({
        "source_root":fixture_root(),
        "published_root":root.join("published"),"user_root":root.join("user"),"runtime_root":root.join("runtime"),
        "case":nomination["case"],"media":nomination["media"],
        "video":{"path":std::env::var("KINEKINO_TIMELINE_FFPROBE").unwrap(),"sha256":"a6618e99bb58869ded3c6f37b53aa1a8d701c3591dbb7b5b317d47369c112be2"},
        "audio":{"path":std::env::var("KINEKINO_TIMELINE_FFMPEG").unwrap(),"sha256":"71414777922d05f2639a5cf4a0f9a405f169809cf459000a6f976c9add61f060"},
        "capture":{"path":std::env::var("KINEKINO_TIMELINE_CAPTURE_FFMPEG").unwrap(),"sha256":"09948d4cdd0650da6ff5a87577469f2a218dc2615ae379f8f734d24c49de0f73"},
        "playback":{"path":playback_tool(),"sha256":"f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790"},
        "video_start_us":16000000,"video_end_us":26000000,"pcm_start_pts":272000,"pcm_end_pts":408000,"start_us":17000000,"end_us":25500000,
        "job_memory_bytes":1073741824,"affinity_mask":mask,"production_total_ms":60000,"playback_total_ms":35000,"cleanup_ms":2000,
        "max_source_bytes":134217728,"max_output_bytes":67108864,"max_package_bytes":1073741824,"playback_raw_output_bytes":1048576,"startup_timeout_ms":10000,"max_history":4
    })
}
#[test]
#[ignore = "central pinned actual product CLI startup producer/storage/playback/EOF/restart; no model/network/GUI"]
fn actual_product_review_startup_generates_authority_and_restarts_with_fresh_scope() {
    kinekino_platform::install();
    run(false, false, false);
}
#[test]
#[ignore = "central pinned actual product CLI broken ready stdout; no model/network/GUI"]
fn actual_product_review_broken_ready_preserves_error_without_media_child() {
    kinekino_platform::install();
    run(true, false, false);
}
#[test]
#[ignore = "central actual product review CLI control/close/reopen/EOF; no model/network/GUI"]
fn actual_product_review_controls_close_reopen_and_eof_keep_original_owners() {
    kinekino_platform::install();
    run(false, true, false);
}
#[test]
#[ignore = "central actual MKV normal CLI producer/playback/controls/EOF; no model/network/GUI"]
fn actual_matroska_product_cli_controls_and_eof() {
    kinekino_platform::install();
    run(false, true, true);
}
#[test]
#[ignore = "central actual MKV normal CLI fresh authority/restart; no model/network/GUI"]
fn actual_matroska_product_cli_restarts_with_fresh_authority() {
    kinekino_platform::install();
    run(false, false, true);
}
fn run(broken_ready: bool, controls: bool, matroska: bool) {
    let workspace = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let root = PathBuf::from(std::env::var("KINEKINO_REVIEW_OUTPUT_DIR").unwrap());
    assert!(root.starts_with(workspace.join("runtime/verification")));
    fs::create_dir(&root).unwrap();
    for name in ["published", "user", "runtime"] {
        fs::create_dir(root.join(name)).unwrap();
    }
    let source = custody::Guard::acquire(
        &PathBuf::from(std::env::var("KINEKINO_REVIEW_SOURCE_RECEIPT_DIR").unwrap())
            .join("SOURCE.md"),
    );
    assert_eq!(
        source.sha256,
        std::env::var("KINEKINO_REVIEW_SOURCE_RECEIPT_SHA256")
            .unwrap()
            .to_ascii_lowercase()
    );
    let producer = custody::Guard::acquire(Path::new(crate::cli_exe::kinekino_cli_exe()));
    let nomination_path = if matroska {
        matroska::package_path()
    } else {
        fixture_root().join("NOMINATION.json")
    };
    let nomination = custody::Guard::acquire(&nomination_path);
    if !matroska {
        assert_eq!(
            nomination.sha256,
            "b1a47d17ad348d6ac7c4c5ec9eb246a54f561babc442cf4310274cdb349be201"
        );
    }
    let data: Value = if matroska {
        matroska::nomination(&nomination.sha256)
    } else {
        serde_json::from_slice(&fs::read(fixture_root().join("NOMINATION.json")).unwrap()).unwrap()
    };
    let (mut mask, mut system) = (0usize, 0usize);
    unsafe {
        windows::Win32::System::Threading::GetProcessAffinityMask(
            windows::Win32::System::Threading::GetCurrentProcess(),
            &mut mask,
            &mut system,
        )
        .unwrap();
    }
    let mut settings = settings(&root, &data, mask.isolate_lowest_one());
    if matroska {
        matroska::configure(&mut settings);
    }
    let config = root.join("settings.json");
    write_new(&config, &settings);
    let config_guard = custody::Guard::acquire(&config);
    write_new(
        &root.join("WITNESS.json"),
        &json!({"source":source.evidence(),"producer":producer.evidence(),"nomination":nomination.evidence(),"settings":config_guard.evidence(),"explicit_actual9_media_selection":matroska,"historical_media_verified":false,"quality_approved":false}),
    );
    let mut old_session = None;
    let mut old_case = None;
    for &epoch in if broken_ready {
        &[][..]
    } else if controls {
        &["first"][..]
    } else {
        &["first", "second"][..]
    } {
        let output = root.join(epoch);
        fs::create_dir(&output).unwrap();
        let start = Instant::now();
        let child = Command::new(crate::cli_exe::kinekino_cli_exe())
            .args(["--serve-review", "--review-host-config"])
            .arg(&config)
            .args(["--review-host-config-sha256", &config_guard.sha256])
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
            let mut first = String::new();
            reader.read_line(&mut first).unwrap();
            let _ = tx.send(first.clone());
            let mut tail = String::new();
            reader.read_to_string(&mut tail).unwrap();
            format!("{first}{tail}")
        });
        let err = thread::spawn(move || {
            let mut bytes = Vec::new();
            BufReader::new(stderr).read_to_end(&mut bytes).unwrap();
            bytes
        });
        let line = rx
            .recv_timeout(Duration::from_secs(90))
            .expect("original product host did not report ready");
        let ready: Value = serde_json::from_str(&line).unwrap();
        write_new(&output.join("READY.json"), &ready);
        assert_eq!(ready["host"], "review", "{ready}");
        assert_eq!(ready["pid"], pid);
        let endpoint = ready["endpoint"].as_str().unwrap();
        if let Some(old) = &old_session {
            let denied = call(
                endpoint,
                &output,
                "old-session",
                &request("reviews.playback.status", json!({"session":old})),
                false,
            );
            assert_eq!(denied["error"]["code"], "revision_conflict");
            assert_ne!(old_case.as_ref().unwrap(), &ready["case"]);
            let inspected = call(
                endpoint,
                &output,
                "old-stored-association",
                &request(
                    "reviews.association.inspect",
                    json!({"case":old_case,"association_id":"startup"}),
                ),
                true,
            );
            assert_eq!(inspected["data"]["stored"]["association"]["revision"], "2");
        }
        let mut create = request(
            "reviews.association.create",
            json!({"case":ready["case"],"association_id":"startup","media":data["media"],"note":"technical startup validation"}),
        );
        create.context.idempotency_key = Some(format!("{epoch}-create"));
        call(endpoint, &output, "create", &create, true);
        let mut verify = request("reviews.association.revise", create.payload.clone());
        verify.payload["timeline_evidence"] = ready["evidence"].clone();
        verify.context.expected_revision = Some("1".into());
        verify.context.idempotency_key = Some(format!("{epoch}-verify"));
        let verified = call(endpoint, &output, "verify", &verify, true);
        assert_eq!(
            verified["data"]["receipt"]["body"]["association"]["revision"],
            "2"
        );
        assert_eq!(verified["data"]["playback_ready"], false);
        let replay = call(endpoint, &output, "verify-replay", &verify, true);
        assert_eq!(verified["data"]["receipt"], replay["data"]["receipt"]);
        let mut open = request(
            "reviews.playback.open",
            json!({"selection":{"case":ready["case"],"association_id":"startup","association_revision":"2"}}),
        );
        open.context.idempotency_key = Some("startup-open".into());
        let opened = call(endpoint, &output, "open", &open, true);
        let mut session = opened["data"]["session"].clone();
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut poll = 0;
        let connected = loop {
            let status = call(
                endpoint,
                &output,
                &format!("status-{poll}"),
                &request("reviews.playback.status", json!({"session":session})),
                true,
            );
            assert_eq!(
                status["data"]["observation"]["has_error"], false,
                "{status}"
            );
            if status["data"]["observation"]["connected"] == true {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "original session not connected: {status}"
            );
            poll += 1;
            thread::sleep(Duration::from_millis(100));
        };
        if controls {
            session = controls::exercise(
                endpoint,
                &output,
                &session,
                &connected,
                &open,
                if matroska { 107_250_000 } else { 17_000_000 },
                if matroska { 134_000_000 } else { 26_000_000 },
            );
        }
        // EOF while the original media is live must close it through the product driver.
        drop(owner.0.stdin.take());
        let cutoff = Instant::now() + Duration::from_secs(15);
        let status = loop {
            if let Some(status) = owner.0.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < cutoff,
                "original product host did not exit"
            );
            thread::sleep(Duration::from_millis(20));
        };
        let stdout = out.join().unwrap();
        let stderr = err.join().unwrap();
        fs::write(output.join("stdout.raw"), stdout).unwrap();
        fs::write(output.join("stderr.raw"), stderr).unwrap();
        write_new(
            &output.join("PROCESS.json"),
            &json!({"pid":pid,"exit_code":status.code(),"elapsed_ms":start.elapsed().as_millis(),"stdin_eof":true,"original_child_waited":true,"streams_joined":true,"forced":false}),
        );
        assert_eq!(status.code(), Some(0));
        old_session = Some(session);
        old_case = Some(ready["case"].clone());
    }
    if !broken_ready {
        source.assert_unchanged();
        producer.assert_unchanged();
        nomination.assert_unchanged();
        config_guard.assert_unchanged();
        write_new(
            &root.join("RETURN.json"),
            &json!({"product_hosts":if controls {1} else {2},"normal_hosts":if controls {1} else {2},"controls_checked":controls,"restart_checked":!controls,"fresh_producer_each_start":true,"old_session_rejected":if controls {Value::Null} else {json!(true)},"old_stored_association_preserved":if controls {Value::Null} else {json!(true)},"source":source.evidence(),"producer":producer.evidence(),"explicit_actual9_media_selection":matroska,"historical_media_verified":false,"quality_approved":false}),
        );
        return;
    }
    let output = root.join("broken-ready");
    fs::create_dir(&output).unwrap();
    let start = Instant::now();
    let child = Command::new(crate::cli_exe::kinekino_cli_exe())
        .args(["--serve-review", "--review-host-config"])
        .arg(&config)
        .args(["--review-host-config-sha256", &config_guard.sha256])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut owner = Owner(child);
    let pid = owner.0.id();
    // Close the actual reader before the product attempts its ready write.
    drop(owner.0.stdout.take());
    drop(owner.0.stdin.take());
    let stderr = owner.0.stderr.take().unwrap();
    let err = thread::spawn(move || {
        let mut bytes = Vec::new();
        BufReader::new(stderr).read_to_end(&mut bytes).unwrap();
        bytes
    });
    let cutoff = Instant::now() + Duration::from_secs(90);
    let status = loop {
        if let Some(status) = owner.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < cutoff,
            "original broken-ready host did not exit"
        );
        thread::sleep(Duration::from_millis(20));
    };
    let stderr = err.join().unwrap();
    fs::write(output.join("stderr.raw"), &stderr).unwrap();
    write_new(
        &output.join("PROCESS.json"),
        &json!({"pid":pid,"exit_code":status.code(),"elapsed_ms":start.elapsed().as_millis(),"stdout_reader_closed":true,"original_child_waited":true,"streams_joined":true,"forced":false}),
    );
    assert_eq!(status.code(), Some(8));
    let response: Value = serde_json::from_slice(&stderr).unwrap();
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["details"]["phase"], "ready_write");
    assert_eq!(
        fs::read_dir(root.join("runtime"))
            .unwrap()
            .filter(|entry| entry
                .as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("review-session-"))
            .count(),
        0
    );
    source.assert_unchanged();
    producer.assert_unchanged();
    nomination.assert_unchanged();
    config_guard.assert_unchanged();
    write_new(
        &root.join("RETURN.json"),
        &json!({"product_hosts":1,"normal_hosts":0,"broken_ready_host":{"exit_code":8,"primary_phase":"ready_write","media_child_created":false},"source":source.evidence(),"producer":producer.evidence(),"actual9_binding":false,"quality_approved":false}),
    );
}
