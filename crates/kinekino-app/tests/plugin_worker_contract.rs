//! ADR 20261106051332 §9 contract: long-running PluginWorker reuse, idle respawn,
//! crash, call timeout, and one-shot serve regression.
use kinekino_core::source::{SourceError, SourceProvider, SourceRef};
use kinekino_source_host::plugin_host::{PluginManifest, ProcessSource};
use std::path::PathBuf;
use std::time::Duration;

fn install_os() {
    kinekino_platform::install();
}

fn setup(tag: &str, mode: &str) -> (PathBuf, ProcessSource) {
    install_os();
    let root = std::env::temp_dir().join(format!(
        "kinekino-worker-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let dir = root.join("fixture");
    std::fs::create_dir_all(&dir).unwrap();
    let exe = env!("CARGO_BIN_EXE_fake-source-plugin");
    let target = dir.join(format!("plugin{}", std::env::consts::EXE_SUFFIX));
    std::fs::copy(exe, &target).unwrap();
    if !mode.is_empty() {
        std::fs::write(dir.join("fixture-mode.txt"), mode).unwrap();
    }
    let manifest = serde_json::json!({
        "id": "com.example.fixture", "version": "0.1.0", "api": "source/1",
        "capabilities": ["browse", "resolve"], "media_hosts": ["media.example.test"],
        "executable": target.file_name().unwrap().to_str().unwrap(),
    });
    std::fs::write(dir.join("plugin.json"), manifest.to_string()).unwrap();
    let plugin = PluginManifest::parse(&dir, &manifest.to_string()).unwrap();
    let source = ProcessSource::with_timeouts(
        plugin,
        Duration::from_millis(400),
        Duration::from_millis(300),
    );
    (root, source)
}

fn category() -> SourceRef {
    SourceRef {
        provider: "com.example.fixture".into(),
        id: "airing".into(),
    }
}

#[test]
fn two_consecutive_browses_reuse_one_process() {
    let (root, source) = setup("reuse", "");
    let page1 = source.browse(&category(), 1).unwrap();
    let pid1 = source
        .worker_pid()
        .expect("worker ready after first browse");
    assert_eq!(page1.items[0].reference.id, "airing-1");
    let page2 = source.browse(&category(), 2).unwrap();
    let pid2 = source
        .worker_pid()
        .expect("worker ready after second browse");
    assert_eq!(page2.items[0].reference.id, "airing-2");
    assert_eq!(pid1, pid2, "expected one long-running worker process");
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn idle_timeout_then_respawn() {
    let (root, source) = setup("idle", "");
    source.browse(&category(), 1).unwrap();
    let pid1 = source.worker_pid().unwrap();
    std::thread::sleep(Duration::from_millis(450));
    // Idle reaper or ensure_ready must have stopped the worker.
    let page = source.browse(&category(), 2).unwrap();
    let pid2 = source.worker_pid().unwrap();
    assert_eq!(page.items[0].reference.id, "airing-2");
    assert_ne!(pid1, pid2, "idle stop must respawn a new process");
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn crash_returns_unavailable_then_next_call_restarts() {
    let (root, source) = setup("crash", "crash-on-request");
    let err = source.browse(&category(), 1).unwrap_err();
    assert!(
        matches!(err, SourceError::Unavailable(_)),
        "crash must be Unavailable, got {err:?}"
    );
    assert_eq!(source.worker_pid(), None);
    // Switch to a healthy loop for the restart call.
    std::fs::write(root.join("fixture/fixture-mode.txt"), "").unwrap();
    let page = source.browse(&category(), 3).unwrap();
    assert_eq!(page.items[0].reference.id, "airing-3");
    assert!(source.worker_pid().is_some());
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn call_timeout_terminates_worker() {
    let (root, source) = setup("timeout", "sleep-before-ms=2000");
    let err = source.browse(&category(), 1).unwrap_err();
    assert!(
        matches!(err, SourceError::Unavailable(ref m) if m.contains("timeout")),
        "expected call timeout, got {err:?}"
    );
    assert_eq!(source.worker_pid(), None);
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn oneshot_serve_fixture_still_answers_then_respawns() {
    let (root, source) = setup("oneshot", "oneshot");
    let page1 = source.browse(&category(), 1).unwrap();
    assert_eq!(page1.items[0].reference.id, "airing-1");
    // One-shot child exits after the reply; next call must spawn again.
    let page2 = source.browse(&category(), 2).unwrap();
    assert_eq!(page2.items[0].reference.id, "airing-2");
    std::fs::remove_dir_all(root).ok();
}
