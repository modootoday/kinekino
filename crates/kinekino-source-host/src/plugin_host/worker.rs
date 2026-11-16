//! Long-running per-plugin catalog worker (ADR 20261106051332 §9): one OwnedFamily,
//! JSON lines on stdin/stdout, one in-flight request, call/idle timeouts.
use super::{PluginManifest, wire};
use kinekino_core::platform::process::{Deadline, OwnedFamily, SpawnRequest};
use kinekino_core::source::{SourceCapability, SourceError};
use kinekino_process::infrastructure::os_policy::process;
use std::fs::File;
use std::io::{BufRead, BufReader, Write, pipe};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Production default: wait this long for one response line after writing a request.
pub const DEFAULT_CALL_TIMEOUT: Duration = Duration::from_secs(30);
/// Production default: after the last response, stop the worker if idle this long.
pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

const MAX_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;
const SPAWN_TOTAL: Duration = Duration::from_secs(60 * 60 * 24);
const SPAWN_CLEANUP: Duration = Duration::from_secs(10);
const IDLE_TERMINATE_WAIT: Duration = Duration::from_millis(200);

enum State {
    Stopped,
    Ready(Ready),
}

struct Ready {
    family: Box<dyn OwnedFamily>,
    stdin: File,
    stdout: File,
    pid: u32,
    last_response_at: Instant,
}

pub struct PluginWorker {
    plugin: PluginManifest,
    state: State,
    call_timeout: Duration,
    idle_timeout: Duration,
    epoch: u64,
}

enum Exchange {
    /// Wire success or plugin `Error{...}`; worker stays unless the child already exited.
    Protocol(Result<wire::Response, SourceError>),
    /// IO, timeout, death, or malformed JSON; worker must stop.
    Transport(SourceError),
    /// The process was already gone before answering (write failed or stdout hit EOF with
    /// no bytes); a reused worker may be respawned and the request sent once more.
    Closed(SourceError),
}

impl PluginWorker {
    pub fn new(plugin: PluginManifest, call_timeout: Duration, idle_timeout: Duration) -> Self {
        Self {
            plugin,
            state: State::Stopped,
            call_timeout,
            idle_timeout,
            epoch: 0,
        }
    }

    pub fn pid(&self) -> Option<u32> {
        match &self.state {
            State::Ready(ready) => Some(ready.pid),
            State::Stopped => None,
        }
    }

    pub fn call(
        &mut self,
        capability: SourceCapability,
        request: &wire::Request,
        idle_home: Option<Arc<Mutex<PluginWorker>>>,
    ) -> Result<wire::Response, SourceError> {
        if !self.plugin.source.supports(capability) {
            return Err(SourceError::Unsupported(capability));
        }
        let reused = self.ensure_ready(idle_home.clone())?;
        let mut exchange = self.exchange(capability, request);
        if reused && let Exchange::Closed(_) = exchange {
            // A one-shot plugin may exit right after its reply, before the host can see it.
            self.force_stop();
            self.ensure_ready(idle_home.clone())?;
            exchange = self.exchange(capability, request);
        }
        match exchange {
            Exchange::Protocol(result) => {
                self.touch_after_response(idle_home);
                if self.child_exited() {
                    self.force_stop();
                }
                result
            }
            Exchange::Transport(err) | Exchange::Closed(err) => {
                self.force_stop();
                Err(err)
            }
        }
    }

    fn touch_after_response(&mut self, idle_home: Option<Arc<Mutex<PluginWorker>>>) {
        if let State::Ready(ready) = &mut self.state {
            ready.last_response_at = Instant::now();
        }
        self.epoch = self.epoch.wrapping_add(1);
        let epoch = self.epoch;
        if let Some(home) = idle_home {
            schedule_idle(home, epoch, self.idle_timeout);
        }
    }

    fn child_exited(&mut self) -> bool {
        let State::Ready(ready) = &mut self.state else {
            return true;
        };
        matches!(ready.family.poll_child(), Ok(Some(_)))
    }

    /// Returns `true` when an already running worker is reused, `false` after a fresh spawn.
    fn ensure_ready(
        &mut self,
        idle_home: Option<Arc<Mutex<PluginWorker>>>,
    ) -> Result<bool, SourceError> {
        if let State::Ready(ready) = &self.state
            && ready.last_response_at.elapsed() >= self.idle_timeout
        {
            self.graceful_stop();
        }
        if matches!(self.state, State::Ready(_)) && self.child_exited() {
            self.force_stop();
        }
        if matches!(self.state, State::Ready(_)) {
            return Ok(true);
        }
        self.spawn_ready()?;
        self.epoch = self.epoch.wrapping_add(1);
        if let Some(home) = idle_home {
            schedule_idle(home, self.epoch, self.idle_timeout);
        }
        Ok(false)
    }

    fn spawn_ready(&mut self) -> Result<(), SourceError> {
        let unavailable = |e: std::io::Error| SourceError::Unavailable(e.to_string());
        let executable = self.plugin.executable.canonicalize().map_err(unavailable)?;
        let cwd = self.plugin.dir.canonicalize().map_err(unavailable)?;
        let (stdout_reader, stdout_writer) = pipe().map_err(unavailable)?;
        let (stdin_reader, stdin_writer) = pipe().map_err(unavailable)?;
        let deadline = Deadline::new(Instant::now(), SPAWN_TOTAL, SPAWN_CLEANUP)
            .map_err(|e| SourceError::Unavailable(e.to_string()))?;
        let family = process()
            .spawn(
                SpawnRequest {
                    executable,
                    cwd,
                    arguments: Vec::new(),
                    environment: None,
                    stdin: Some(super::stdio::file_from_pipe_reader(stdin_reader)),
                    stdout: super::stdio::file_from_pipe_writer(stdout_writer),
                    stderr: super::stdio::null_sink().map_err(unavailable)?,
                    deadline,
                    source: None,
                    fault: None,
                    // Unsandboxed until the OS sandbox adapters land (ADR 20261109051610).
                    sandbox: None,
                },
                None,
            )
            .map_err(|failure| SourceError::Unavailable(format!("spawn: {}", failure.error)))?;
        let pid = family.identity().pid;
        self.state = State::Ready(Ready {
            family,
            stdin: super::stdio::file_from_pipe_writer(stdin_writer),
            stdout: super::stdio::file_from_pipe_reader(stdout_reader),
            pid,
            last_response_at: Instant::now(),
        });
        Ok(())
    }

    fn exchange(&mut self, capability: SourceCapability, request: &wire::Request) -> Exchange {
        let State::Ready(ready) = &mut self.state else {
            return Exchange::Transport(SourceError::Unavailable("worker stopped".into()));
        };
        let line = match serde_json::to_string(request) {
            Ok(line) => line,
            Err(e) => return Exchange::Transport(SourceError::Invalid(e.to_string())),
        };
        if let Err(e) = (|| {
            ready.stdin.write_all(line.as_bytes())?;
            ready.stdin.write_all(b"\n")?;
            ready.stdin.flush()
        })() {
            return Exchange::Closed(SourceError::Unavailable(e.to_string()));
        }

        let timeout = self.call_timeout;
        let placeholder = match super::stdio::null_sink() {
            Ok(file) => file,
            Err(e) => return Exchange::Transport(SourceError::Unavailable(e.to_string())),
        };
        let stdout = std::mem::replace(&mut ready.stdout, placeholder);
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let limited = std::io::Read::take(stdout, MAX_RESPONSE_BYTES);
            let mut reader = BufReader::new(limited);
            let mut reply = String::new();
            let result = reader.read_line(&mut reply).map(|_| reply);
            let file = reader.into_inner().into_inner();
            let _ = tx.send((result, file));
        });
        let (read_result, stdout) = match rx.recv_timeout(timeout) {
            Ok(pair) => pair,
            Err(_) => {
                let _ = ready.family.terminate_owned_family();
                let _ = rx.recv_timeout(Duration::from_secs(2));
                return Exchange::Transport(SourceError::Unavailable("call timeout".into()));
            }
        };
        ready.stdout = stdout;
        let reply = match read_result {
            Ok(reply) => reply,
            Err(e) => return Exchange::Transport(SourceError::Unavailable(e.to_string())),
        };
        if reply.is_empty() {
            return Exchange::Closed(SourceError::Unavailable("empty plugin reply".into()));
        }
        if reply.trim().is_empty() {
            return Exchange::Transport(SourceError::Unavailable("empty plugin reply".into()));
        }
        let response: wire::Response = match serde_json::from_str(reply.trim()) {
            Ok(response) => response,
            Err(e) => {
                return Exchange::Transport(SourceError::Invalid(format!("plugin reply: {e}")));
            }
        };
        Exchange::Protocol(match response {
            wire::Response::Error { kind, message } => Err(match kind.as_str() {
                "not_found" => SourceError::NotFound,
                "unsupported" => SourceError::Unsupported(capability),
                "invalid" => SourceError::Invalid(message),
                _ => SourceError::Unavailable(message),
            }),
            other => Ok(other),
        })
    }

    fn graceful_stop(&mut self) {
        let State::Ready(mut ready) = std::mem::replace(&mut self.state, State::Stopped) else {
            return;
        };
        drop(ready.stdin);
        thread::sleep(IDLE_TERMINATE_WAIT);
        let _ = ready.family.terminate_owned_family();
        let _ = ready.family.poll_child();
    }

    fn force_stop(&mut self) {
        let State::Ready(mut ready) = std::mem::replace(&mut self.state, State::Stopped) else {
            return;
        };
        let _ = ready.family.terminate_owned_family();
        let _ = ready.family.poll_child();
    }
}

impl Drop for PluginWorker {
    fn drop(&mut self) {
        self.force_stop();
    }
}

fn schedule_idle(home: Arc<Mutex<PluginWorker>>, epoch: u64, idle: Duration) {
    thread::spawn(move || {
        thread::sleep(idle);
        let Ok(mut guard) = home.lock() else {
            return;
        };
        if guard.epoch != epoch {
            return;
        }
        if let State::Ready(ready) = &guard.state
            && ready.last_response_at.elapsed() >= idle
        {
            guard.graceful_stop();
        }
    });
}
