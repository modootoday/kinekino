//! mpv playback backend over its JSON IPC socket (`--input-ipc-server`).
//! The binary is never bundled: it is found at an explicit path, `$KINEKINO_MPV`, or on `PATH`.
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use kinekino_core::playback::{BackendStatus, PlaybackBackend, PlaybackControl};
use kinekino_core::port::error::{ErrorCode, PortError, PortResult};
use serde_json::{Value, json};

#[cfg(test)]
mod tests;

static NEXT_SOCKET: AtomicU64 = AtomicU64::new(0);
const TIMEOUT: Duration = Duration::from_secs(10);

fn err(code: ErrorCode, m: impl Into<String>) -> PortError {
    PortError::new(code, m.into())
}

/// Locate mpv: `explicit`, then `$KINEKINO_MPV`, then each `PATH` entry.
pub fn find_mpv(explicit: Option<&Path>) -> PortResult<PathBuf> {
    if let Some(p) = explicit {
        return p.is_file().then(|| p.to_path_buf()).ok_or_else(|| {
            err(
                ErrorCode::Unavailable,
                format!("mpv not found at {}", p.display()),
            )
        });
    }
    if let Some(p) = std::env::var_os("KINEKINO_MPV").filter(|p| !p.is_empty()) {
        return find_mpv(Some(Path::new(&p)));
    }
    std::env::var_os("PATH")
        .iter()
        .flat_map(std::env::split_paths)
        .map(|d| d.join("mpv"))
        .find(|p| p.is_file())
        .ok_or_else(|| {
            err(
                ErrorCode::Unavailable,
                "mpv not found on PATH (see docs: mpv install)",
            )
        })
}

/// Options for the spawned mpv process.
#[derive(Debug, Clone, Default)]
pub struct MpvOptions {
    pub binary: Option<PathBuf>,
    /// Use `--vo=null --ao=null` (tests, CI, audio-less servers).
    pub headless: bool,
}

pub struct MpvBackend {
    child: Option<Child>,
    writer: UnixStream,
    reader: BufReader<UnixStream>,
    socket: PathBuf,
    next_id: u64,
    loaded: bool,
}

impl MpvBackend {
    pub fn spawn(options: &MpvOptions) -> PortResult<Self> {
        let bin = find_mpv(options.binary.as_deref())?;
        let socket = std::env::temp_dir().join(format!(
            "kinekino-mpv-{}-{}.sock",
            std::process::id(),
            NEXT_SOCKET.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_file(&socket);
        let mut cmd = Command::new(&bin);
        cmd.args([
            "--idle=yes",
            "--pause",
            "--no-config",
            "--no-terminal",
            "--keep-open=yes",
        ])
        .arg(format!("--input-ipc-server={}", socket.display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
        if options.headless {
            cmd.args(["--vo=null", "--ao=null"]);
        }
        let mut child = cmd.spawn().map_err(|e| {
            err(
                ErrorCode::Unavailable,
                format!("spawn {}: {e}", bin.display()),
            )
        })?;
        let start = Instant::now();
        let stream = loop {
            match UnixStream::connect(&socket) {
                Ok(s) => break s,
                Err(_) if start.elapsed() < TIMEOUT => {
                    if let Ok(Some(st)) = child.try_wait() {
                        return Err(err(ErrorCode::Unavailable, format!("mpv exited: {st}")));
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(err(ErrorCode::Timeout, format!("mpv IPC socket: {e}")));
                }
            }
        };
        let io = |e: std::io::Error| err(ErrorCode::Io, e.to_string());
        stream.set_read_timeout(Some(TIMEOUT)).map_err(io)?;
        let reader = BufReader::new(stream.try_clone().map_err(io)?);
        Ok(Self {
            child: Some(child),
            writer: stream,
            reader,
            socket,
            next_id: 1,
            loaded: false,
        })
    }

    fn live(&self) -> PortResult<()> {
        if self.child.is_none() {
            return Err(err(ErrorCode::Unavailable, "Backend is closed"));
        }
        Ok(())
    }

    fn read_msg(&mut self) -> PortResult<Value> {
        let mut line = String::new();
        let n = self
            .reader
            .read_line(&mut line)
            .map_err(|e| err(ErrorCode::Timeout, format!("mpv IPC read: {e}")))?;
        if n == 0 {
            return Err(err(ErrorCode::Unavailable, "mpv IPC closed"));
        }
        serde_json::from_str(&line)
            .map_err(|e| err(ErrorCode::Internal, format!("mpv IPC json: {e}")))
    }

    /// Send a command and return its `data`; events seen meanwhile are passed to `on_event`.
    fn command_with(&mut self, args: Value, mut on_event: impl FnMut(&str)) -> PortResult<Value> {
        self.live()?;
        let id = self.next_id;
        self.next_id += 1;
        let line = json!({ "command": args, "request_id": id }).to_string() + "\n";
        self.writer
            .write_all(line.as_bytes())
            .map_err(|e| err(ErrorCode::Io, format!("mpv IPC write: {e}")))?;
        loop {
            let msg = self.read_msg()?;
            if let Some(ev) = msg.get("event").and_then(Value::as_str) {
                on_event(ev);
                continue;
            }
            if msg.get("request_id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            return match msg.get("error").and_then(Value::as_str) {
                Some("success") => Ok(msg.get("data").cloned().unwrap_or(Value::Null)),
                Some("property unavailable") => Ok(Value::Null),
                other => Err(err(
                    ErrorCode::Internal,
                    format!("mpv: {}", other.unwrap_or("no status")),
                )),
            };
        }
    }

    fn command(&mut self, args: Value) -> PortResult<Value> {
        self.command_with(args, |_| {})
    }

    /// Issue `args`, then block until `event` arrives (it may arrive before the reply).
    fn command_until(&mut self, args: Value, event: &str, fail: &str) -> PortResult<()> {
        let (mut seen, mut failed) = (false, false);
        self.command_with(args, |e| {
            seen |= e == event;
            failed |= e == fail;
        })?;
        let start = Instant::now();
        while !seen && !failed {
            if start.elapsed() > TIMEOUT {
                return Err(err(ErrorCode::Timeout, format!("mpv: no {event}")));
            }
            match self.read_msg()?.get("event").and_then(Value::as_str) {
                Some(e) if e == event => seen = true,
                Some(e) if e == fail => failed = true,
                _ => {}
            }
        }
        if failed {
            return Err(err(
                ErrorCode::InvalidInput,
                "mpv could not load the source",
            ));
        }
        Ok(())
    }

    fn prop(&mut self, name: &str) -> PortResult<Value> {
        self.command(json!(["get_property", name]))
    }
}

fn secs(us: u64) -> f64 {
    us as f64 / 1e6
}

fn us(v: &Value) -> Option<u64> {
    v.as_f64()
        .filter(|s| s.is_finite() && *s >= 0.0)
        .map(|s| (s * 1e6).round() as u64)
}

impl PlaybackBackend for MpvBackend {
    fn open(&mut self, source: &str, start_us: u64) -> PortResult<()> {
        self.live()?;
        if source.is_empty() {
            return Err(err(ErrorCode::InvalidInput, "Source is empty"));
        }
        self.command(json!(["set_property", "pause", true]))?;
        let opts = format!("start={},pause=yes", secs(start_us));
        self.command_until(
            json!(["loadfile", source, "replace", -1, opts]),
            "playback-restart",
            "end-file",
        )?;
        self.loaded = true;
        Ok(())
    }

    fn control(&mut self, control: &PlaybackControl) -> PortResult<()> {
        control
            .validate()
            .map_err(|m| err(ErrorCode::InvalidInput, m))?;
        self.live()?;
        if !self.loaded {
            return Err(err(ErrorCode::Unavailable, "No media is open"));
        }
        match control {
            PlaybackControl::Play => self
                .command(json!(["set_property", "pause", false]))
                .map(drop),
            PlaybackControl::Pause => self
                .command(json!(["set_property", "pause", true]))
                .map(drop),
            PlaybackControl::Seek { position_us } => self.command_until(
                json!(["seek", secs(position_us.0), "absolute+exact"]),
                "playback-restart",
                "end-file",
            ),
            PlaybackControl::SetVolume { volume_percent } => self
                .command(json!(["set_property", "volume", volume_percent]))
                .map(drop),
            PlaybackControl::SetMuted { muted } => self
                .command(json!(["set_property", "mute", muted]))
                .map(drop),
        }
    }

    fn status(&mut self) -> PortResult<BackendStatus> {
        self.live()?;
        if !self.loaded {
            return Err(err(ErrorCode::Unavailable, "No media is open"));
        }
        Ok(BackendStatus {
            source: self.prop("path")?.as_str().map(str::to_owned),
            position_us: us(&self.prop("time-pos")?).unwrap_or(0),
            duration_us: us(&self.prop("duration")?),
            paused: self.prop("pause")?.as_bool().unwrap_or(true),
            volume_percent: self.prop("volume")?.as_f64().unwrap_or(100.0),
            muted: self.prop("mute")?.as_bool().unwrap_or(false),
            eof: self.prop("eof-reached")?.as_bool().unwrap_or(false),
        })
    }

    fn close(&mut self) -> PortResult<()> {
        if self.child.is_none() {
            return Ok(());
        }
        let _ = self.command(json!(["quit"]));
        if let Some(mut child) = self.child.take() {
            let start = Instant::now();
            while matches!(child.try_wait(), Ok(None)) && start.elapsed() < Duration::from_secs(3) {
                std::thread::sleep(Duration::from_millis(20));
            }
            if matches!(child.try_wait(), Ok(None)) {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
        self.loaded = false;
        let _ = std::fs::remove_file(&self.socket);
        Ok(())
    }
}

impl Drop for MpvBackend {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
