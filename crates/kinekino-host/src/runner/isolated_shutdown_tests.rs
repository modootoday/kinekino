use super::*;
use kinekino_player::infrastructure::local_host::{EndpointClient, LocalHost, TransportOptions};
use std::{
    os::windows::process::CommandExt,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const CHILD_TEST: &str = "runner::isolated_shutdown_tests::child_entry";
const CASE_ENV: &str = "KINEKINO_TEST_ISOLATED_SHUTDOWN_CASE";
const PARENT_ENV: &str = "KINEKINO_TEST_ISOLATED_SHUTDOWN_PARENT";
const ROOT_ENV: &str = "KINEKINO_TEST_ISOLATED_SHUTDOWN_ROOT";
const PHASE_ENV: &str = "KINEKINO_TEST_ISOLATED_SHUTDOWN_PHASE";
const CHILD_OUTPUT_LIMIT: u64 = 64 * 1024;
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
struct Root(std::path::PathBuf);
impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "Central slot only: real child process and production 30-second observation budget"]
fn production_boundary_uses_isolated_children_for_timeout_panic_and_job_eof() {
    assert!(
        std::env::var(PHASE_ENV).is_err(),
        "isolated child must never spawn another parent"
    );
    for (case, expected_exit) in [("timeout", 8), ("panic", 6), ("job-eof", 0)] {
        let temp_parent = std::env::temp_dir();
        let root = temp_parent.join(format!(
            "kinekino-isolated-host-{}-{}-{case}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let resolved_root = root.canonicalize().unwrap();
        let resolved_parent = temp_parent.canonicalize().unwrap();
        assert_eq!(resolved_root.parent(), Some(resolved_parent.as_path()));
        let root = Root(root);
        let executable = std::env::current_exe().unwrap();
        let started = Instant::now();
        let mut child = OwnedChild(
            Command::new(&executable)
                .creation_flags(0x08000000)
                .args([
                    "--exact",
                    CHILD_TEST,
                    "--ignored",
                    "--nocapture",
                    "--test-threads=1",
                ])
                .env(CASE_ENV, case)
                .env(PHASE_ENV, "child")
                .env(PARENT_ENV, std::process::id().to_string())
                .env(ROOT_ENV, &root.0)
                .env("RUST_BACKTRACE", "0")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        let pid = child.0.id();
        let deadline = started + Duration::from_secs(45);
        let mut timed_out = false;
        let mut wait_issue = None;
        let status = loop {
            match child.0.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Ok(None) => {
                    timed_out = true;
                    let _ = child.0.kill();
                    break child
                        .0
                        .wait()
                        .expect("original timed-out child handle could not be reaped");
                }
                Err(error) => {
                    wait_issue = Some(error.to_string());
                    let _ = child.0.kill();
                    break child
                        .0
                        .wait()
                        .expect("original child handle could not be reaped after wait error");
                }
            }
        };
        // Output is deliberately small; drain only after this exact child exited.
        let mut stdout = String::new();
        let mut stderr = String::new();
        child
            .0
            .stdout
            .take()
            .unwrap()
            .take(CHILD_OUTPUT_LIMIT + 1)
            .read_to_string(&mut stdout)
            .unwrap();
        child
            .0
            .stderr
            .take()
            .unwrap()
            .take(CHILD_OUTPUT_LIMIT + 1)
            .read_to_string(&mut stderr)
            .unwrap();
        let elapsed_ms = started.elapsed().as_millis();
        eprintln!(
            "{}",
            serde_json::json!({"event":"isolated_child_receipt","case":case,
            "pid":pid,"exit_code":status.code(),"elapsed_ms":elapsed_ms,"executable":executable,
            "stdout":stdout,"stderr":stderr,"timed_out":timed_out,"wait_issue":wait_issue})
        );
        assert!(!timed_out, "owned child watchdog: case={case}, pid={pid}");
        assert!(
            wait_issue.is_none(),
            "owned child wait error: {wait_issue:?}"
        );
        assert!(
            stdout.len() as u64 <= CHILD_OUTPUT_LIMIT && stderr.len() as u64 <= CHILD_OUTPUT_LIMIT,
            "isolated child exceeded its bounded diagnostic output"
        );
        assert_eq!(
            status.code(),
            Some(expected_exit),
            "case={case}; stdout={stdout}; stderr={stderr}"
        );
        let events: Vec<serde_json::Value> = stdout
            .lines()
            .filter_map(|line| {
                line.find('{')
                    .and_then(|start| serde_json::from_str(&line[start..]).ok())
            })
            .collect();
        if case == "job-eof" {
            assert!(
                events
                    .iter()
                    .any(|event| event["event"] == "job_host_ready")
            );
            let stopped = events
                .iter()
                .find(|event| event["event"] == "job_host_stopped")
                .unwrap();
            assert_eq!(
                stopped["local_host_shutdown"],
                serde_json::json!({
                    "pipe":"joined","sdk":"joined","dispatch":"idle","dispatch_poisoned":false
                })
            );
            assert!(
                !events
                    .iter()
                    .any(|event| event["event"] == "host_shutdown_incomplete")
            );
            continue;
        }
        assert!(!stdout.contains("ISOLATED_DEPENDENCY_DROPPED"));
        assert!(!stderr.contains("ISOLATED_DEPENDENCY_DROPPED"));
        assert!(!events.iter().any(|event| {
            event["event"]
                .as_str()
                .is_some_and(|name| name.ends_with("_stopped"))
        }));
        let before = events
            .iter()
            .find(|event| event["event"] == "isolated_before_boundary")
            .unwrap();
        assert_eq!(before["pid"], pid);
        assert_eq!(before["dependency_drops"], 0);
        assert_eq!(before["sdk_started"], true);
        let diagnostic = events
            .iter()
            .find(|event| event["event"] == "host_shutdown_incomplete")
            .unwrap();
        assert_eq!(diagnostic["cleanup_complete"], false);
        assert_eq!(diagnostic["sdk_cancellation_confirmed"], false);
        assert!(diagnostic.get("host_lifetime_ended").is_none());
        if case == "timeout" {
            assert_eq!(diagnostic["reason"], "ObservationBudgetExpired");
            assert_eq!(diagnostic["budget_expired"], true);
            assert_eq!(diagnostic["local_host_shutdown"]["sdk"], "pending");
            assert!(diagnostic["first_error"].is_null());
            assert!(before["observed_ms"].as_u64().unwrap() >= 30_000);
        } else {
            assert_eq!(diagnostic["reason"], "WorkerPanicked");
            assert_eq!(diagnostic["local_host_shutdown"]["sdk"], "panicked");
            assert_eq!(diagnostic["first_error"]["code"], "permission_denied");
            assert_eq!(
                diagnostic["first_error"]["details"],
                serde_json::json!({"original":true})
            );
        }
    }
}

struct BlockedSdk {
    gate: Arc<(Mutex<bool>, Condvar)>,
    started: mpsc::Sender<()>,
    panic_on_drop: bool,
}
impl ApplicationClient for BlockedSdk {
    fn execute(&self, request: Request) -> Response {
        self.started.send(()).unwrap();
        let (lock, changed) = &*self.gate;
        drop(
            changed
                .wait_while(lock.lock().unwrap(), |released| !*released)
                .unwrap(),
        );
        PlayerSdk::default().execute(request)
    }
}
impl Drop for BlockedSdk {
    fn drop(&mut self) {
        if self.panic_on_drop {
            panic!("isolated SDK owner drop fixture");
        }
    }
}
struct Dependency(Arc<AtomicUsize>);
impl Drop for Dependency {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
        eprintln!("ISOLATED_DEPENDENCY_DROPPED");
    }
}
struct InputFailure(bool);
impl Read for InputFailure {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        if self.0 {
            Err(std::io::Error::other("original isolated read failure"))
        } else {
            Ok(0)
        }
    }
}

#[test]
#[ignore = "Entry point only for the explicitly selected isolated parent"]
fn child_entry() {
    assert_eq!(std::env::var(PHASE_ENV).as_deref(), Ok("child"));
    let case = std::env::var(CASE_ENV).expect("isolated marker required");
    let parent: u32 = std::env::var(PARENT_ENV)
        .expect("parent marker required")
        .parse()
        .unwrap();
    assert_ne!(parent, std::process::id());
    let args: Vec<_> = std::env::args().collect();
    assert!(args.iter().any(|arg| arg == "--exact"));
    assert!(args.iter().any(|arg| arg == CHILD_TEST));
    assert!(matches!(case.as_str(), "timeout" | "panic" | "job-eof"));
    if case == "job-eof" {
        let root = std::env::var(ROOT_ENV).expect("fresh owned job root required");
        let path = std::path::Path::new(&root);
        assert!(path.is_absolute());
        let resolved = path.canonicalize().unwrap();
        let temp_parent = std::env::temp_dir().canonicalize().unwrap();
        assert_eq!(resolved.parent(), Some(temp_parent.as_path()));
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(
            name.starts_with(&format!("kinekino-isolated-host-{parent}-"))
                && name.ends_with("-job-eof")
        );
        assert!(
            std::fs::read_dir(path).unwrap().next().is_none(),
            "job fixture root must be fresh and empty"
        );
        let args = vec!["--serve-jobs".into(), "--job-root".into(), root];
        let code = finish_host(crate::job_host::run(&args), "job-host");
        assert_eq!(code, 0);
        return;
    }
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let (started, receiver) = mpsc::channel();
    let local = LocalHost::start(
        Arc::new(BlockedSdk {
            gate: gate.clone(),
            started,
            panic_on_drop: case == "panic",
        }),
        TransportOptions::default(),
    )
    .unwrap();
    let client = EndpointClient::new(local.endpoint(), TransportOptions::default()).unwrap();
    let _client = std::thread::spawn(move || client.execute(Request::new("version")));
    receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    if case == "panic" {
        let (lock, changed) = &*gate;
        *lock.lock().unwrap() = true;
        changed.notify_all();
    }
    let drops = Arc::new(AtomicUsize::new(0));
    let observed = Instant::now();
    let outcome = crate::host_shutdown::serve_transport(
        local,
        Dependency(drops.clone()),
        &mut InputFailure(case == "panic"),
        &mut Vec::new(),
        &serde_json::json!({"event":"isolated_ready"}),
        |_, _| {
            let mut error = ApiError::new(
                ErrorCode::PermissionDenied,
                "original isolated read failure",
            );
            error.details = serde_json::json!({"original":true});
            error
        },
        || {},
    );
    let owner = match outcome {
        crate::host_shutdown::TransportOutcome::Incomplete(owner) => owner,
        crate::host_shutdown::TransportOutcome::Ready(_) => {
            panic!("fixture must reach abnormal boundary")
        }
    };
    println!(
        "{}",
        serde_json::json!({"event":"isolated_before_boundary","pid":std::process::id(),
        "dependency_drops":drops.load(Ordering::SeqCst),"sdk_started":true,
        "observed_ms":observed.elapsed().as_millis()})
    );
    let code = finish_host(
        crate::host_shutdown::HostRunOutcome::Incomplete(owner),
        "isolated-host",
    );
    panic!("abnormal boundary returned code {code}");
}
