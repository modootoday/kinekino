use kinekino_player::application::api::Request;
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::{Arc, Mutex, mpsc},
    thread::JoinHandle,
    time::{Duration, Instant},
};
struct OwnedChild(std::process::Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub struct Receipt {
    pub code: i32,
    pub response: Value,
    pid: u32,
    elapsed_ms: u128,
    spawned_ms: u128,
    stdin_closed_ms: u128,
    stderr_bytes: usize,
    stdout_lines: usize,
    stdout_raw: Vec<u8>,
    stderr_raw: Vec<u8>,
    forced: bool,
}
impl std::fmt::Debug for Receipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CliReceipt").field("pid", &self.pid).field("elapsed_ms", &self.elapsed_ms)
            .field("stdin_closed_ms", &self.stdin_closed_ms).field("code", &self.code).field("stderr_bytes", &self.stderr_bytes)
            .field("spawned_ms", &self.spawned_ms).field("stdout_lines", &self.stdout_lines).field("error", &json!({"code":self.response["error"]["code"],"delivery":self.response["error"]["details"]["delivery"],"diagnostic":self.response["error"]["details"]["diagnostic"]})).finish()
    }
}
pub struct CliRunner {
    supervisor_wait: Duration,
    cancel: mpsc::Sender<()>,
    result: mpsc::Receiver<Receipt>,
    thread: Option<JoinHandle<()>>,
    receipt: Option<Receipt>,
    progress: Arc<Mutex<(Option<u32>, &'static str)>>,
}
impl CliRunner {
    pub fn start(endpoint: &str, request: &Request) -> Self {
        Self::start_with_transport_timeout(endpoint, request, None)
    }
    pub fn start_with_transport_timeout(
        endpoint: &str,
        request: &Request,
        transport_ms: Option<u64>,
    ) -> Self {
        if let Some(ms) = transport_ms {
            assert!((10..=30000).contains(&ms));
        }
        let supervisor = Duration::from_millis(transport_ms.map_or(15000, |ms| ms + 5000));
        if let Ok(expected) = std::env::var("JOB_CLI_SHA256") {
            let mut executable = std::fs::File::open(crate::cli_exe::kinekino_cli_exe()).unwrap();
            let (_, actual) = kinekino_player::infrastructure::content_digest::sha256(
                &mut executable,
                128 * 1024 * 1024,
            )
            .unwrap();
            assert_eq!(
                actual, expected,
                "Actual CLI executable must match the fixed pin"
            );
        }
        let endpoint = endpoint.to_owned();
        let bytes = serde_json::to_vec(request).unwrap();
        let (cancel, cancellation) = mpsc::channel();
        let (send, result) = mpsc::sync_channel(1);
        let progress = Arc::new(Mutex::new((None, "runner_spawned")));
        let worker_progress = progress.clone();
        let thread = std::thread::spawn(move || {
            let start = Instant::now();
            let mut command = Command::new(crate::cli_exe::kinekino_cli_exe());
            command.args(["--endpoint", &endpoint, "--input", "-"]);
            if let Some(ms) = transport_ms {
                command.args(["--transport-timeout-ms", &ms.to_string()]);
            }
            let child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let pid = child.id();
            let spawned_ms = start.elapsed().as_millis();
            *worker_progress.lock().unwrap() = (Some(pid), "child_spawned");
            // Drain both streams while supervising the owned process; cancellation never waits on a full pipe.
            std::thread::scope(|scope| {
                let mut owned = OwnedChild(child);
                let child = &mut owned.0;
                let stdout = child.stdout.take().unwrap();
                let stderr = child.stderr.take().unwrap();
                let out = scope.spawn(move || {
                    let mut b = Vec::new();
                    stdout
                        .take(
                            (kinekino_player::application::local_host::MAX_RESPONSE_BYTES + 8192)
                                as u64,
                        )
                        .read_to_end(&mut b)
                        .unwrap();
                    b
                });
                let err = scope.spawn(move || {
                    let mut b = Vec::new();
                    stderr.take(8192).read_to_end(&mut b).unwrap();
                    b
                });
                let mut stdin = child.stdin.take().unwrap();
                let input_progress = worker_progress.clone();
                let input = scope.spawn(move || {
                    let _ = stdin.write_all(&bytes);
                    drop(stdin);
                    *input_progress.lock().unwrap() = (Some(pid), "stdin_closed");
                    start.elapsed().as_millis()
                });
                let mut forced = false;
                let status = loop {
                    if cancellation.try_recv().is_ok() || start.elapsed() > supervisor {
                        forced = true;
                        let _ = child.kill();
                        break child.wait().unwrap();
                    }
                    if let Some(status) = child.try_wait().unwrap() {
                        break status;
                    }
                    std::thread::sleep(Duration::from_millis(1));
                };
                let stdout = out.join().unwrap();
                let stderr = err.join().unwrap();
                let stdin_closed_ms = input.join().unwrap();
                let response = serde_json::from_slice(&stdout)
                    .unwrap_or_else(|_| json!({"fixture_error":"invalid_or_absent_cli_json"}));
                *worker_progress.lock().unwrap() = (Some(pid), "child_exited");
                let _ = send.send(Receipt {
                    code: status.code().unwrap_or(-1),
                    response,
                    pid,
                    elapsed_ms: start.elapsed().as_millis(),
                    spawned_ms,
                    stdin_closed_ms,
                    stderr_bytes: stderr.len(),
                    stdout_lines: String::from_utf8_lossy(&stdout).lines().count(),
                    stdout_raw: stdout,
                    stderr_raw: stderr,
                    forced,
                });
            });
        });
        Self {
            supervisor_wait: supervisor + Duration::from_secs(1),
            cancel,
            result,
            thread: Some(thread),
            receipt: None,
            progress,
        }
    }
    pub fn poll(&mut self) -> Option<&Receipt> {
        if self.receipt.is_none() {
            self.receipt = self.result.try_recv().ok();
        }
        self.receipt.as_ref()
    }
    pub fn progress(&self) -> (Option<u32>, &'static str) {
        *self.progress.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub fn finish(&mut self) -> &Receipt {
        self.finish_inner(None)
    }
    pub fn finish_preserving(&mut self, directory: &std::path::Path) -> &Receipt {
        self.finish_inner(Some(directory))
    }
    fn finish_inner(&mut self, directory: Option<&std::path::Path>) -> &Receipt {
        if self.receipt.is_none() {
            self.receipt = Some(
                self.result
                    .recv_timeout(self.supervisor_wait)
                    .expect("owned CLI supervisor did not finish"),
            );
        }
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
        let receipt = self.receipt.as_ref().unwrap();
        if let Some(directory) = directory {
            std::fs::write(directory.join("stdout.raw"), &receipt.stdout_raw).unwrap();
            std::fs::write(directory.join("stderr.raw"), &receipt.stderr_raw).unwrap();
            std::fs::write(
                directory.join("process.json"),
                serde_json::to_vec_pretty(&json!({
                    "pid":receipt.pid,"exit_code":receipt.code,"elapsed_ms":receipt.elapsed_ms,
                    "spawned_ms":receipt.spawned_ms,"stdin_closed_ms":receipt.stdin_closed_ms,
                    "stdout_bytes":receipt.stdout_raw.len(),"stderr_bytes":receipt.stderr_raw.len(),
                "stdout_lines":receipt.stdout_lines,"supervisor_joined":true,"forced":receipt.forced
                }))
                .unwrap(),
            )
            .unwrap();
        }
        assert!(!receipt.forced, "{receipt:?}");
        assert_eq!(receipt.stderr_bytes, 0, "{receipt:?}");
        assert_eq!(receipt.stdout_lines, 1, "{receipt:?}");
        assert!(
            receipt.response.get("fixture_error").is_none(),
            "{receipt:?}"
        );
        receipt
    }
}
impl Drop for CliRunner {
    fn drop(&mut self) {
        let _ = self.cancel.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
            if let Ok(receipt) = self.result.try_recv() {
                eprintln!("owned CLI cleanup: {receipt:?}");
            }
        }
    }
}
pub fn invoke(endpoint: &str, request: &Request) -> (i32, Value) {
    let mut runner = CliRunner::start(endpoint, request);
    let receipt = runner.finish();
    eprintln!("{receipt:?}");
    (receipt.code, receipt.response.clone())
}
