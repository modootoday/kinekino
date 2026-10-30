use crate::application::api::{ApiError, ErrorCode};
use std::{
    io::Read,
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    sync::{Mutex, mpsc},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[cfg(test)]
mod tests;

fn failed() -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        "Historical verifier process failed",
    )
}

trait Control: Send {
    fn id(&self) -> u32;
    fn poll(&mut self) -> std::io::Result<Option<ExitStatus>>;
    fn kill(&mut self) -> std::io::Result<()>;
    fn wait(&mut self) -> std::io::Result<ExitStatus>;
}
impl Control for Child {
    fn id(&self) -> u32 {
        self.id()
    }
    fn poll(&mut self) -> std::io::Result<Option<ExitStatus>> {
        self.try_wait()
    }
    fn kill(&mut self) -> std::io::Result<()> {
        self.kill()
    }
    fn wait(&mut self) -> std::io::Result<ExitStatus> {
        self.wait()
    }
}

enum Stream {
    Output,
    Diagnostics,
}
struct Running {
    child: Box<dyn Control>,
    readers: Vec<JoinHandle<()>>,
    events: mpsc::Receiver<(Stream, Result<Vec<u8>, ApiError>)>,
    output: Option<Vec<u8>>,
    received: usize,
    exited: bool,
    exit_success: bool,
    failure: Option<ApiError>,
}
impl Running {
    fn poll(&mut self) {
        while let Ok((stream, result)) = self.events.try_recv() {
            self.received += 1;
            match result {
                Ok(bytes) => {
                    if matches!(stream, Stream::Output) {
                        self.output = Some(bytes);
                    }
                }
                Err(error) => {
                    self.failure.get_or_insert(error);
                }
            }
        }
        match self.child.poll() {
            Ok(Some(status)) => {
                self.exited = true;
                self.exit_success = status.success();
            }
            Err(_) => {
                self.failure.get_or_insert_with(failed);
            }
            Ok(None) => {}
        }
        if self.exited && self.received == 2 && !self.exit_success {
            self.failure.get_or_insert_with(failed);
        }
    }
    fn ready(&self) -> bool {
        self.exited && self.readers.iter().all(JoinHandle::is_finished)
    }
    fn stopped(&mut self) -> bool {
        if self.exited {
            return true;
        }
        if self.child.kill().is_err() {
            self.poll();
            return self.exited;
        }
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            self.poll();
            if self.exited {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
    fn cleanup_error(&self) -> ApiError {
        let mut error = ApiError::new(
            self.failure
                .as_ref()
                .map_or(ErrorCode::EngineFailed, |e| e.code),
            "Historical verifier cleanup is incomplete; retry cleanup before another verification",
        );
        error.details =
            serde_json::json!({"cleanup_complete":false,"owned_process_id":self.child.id()});
        error
    }
}

#[derive(Default)]
pub(super) struct Runner(Mutex<Option<Running>>);
impl Runner {
    pub(super) fn recover(&self) -> Result<(), ApiError> {
        let mut slot = self.0.try_lock().map_err(|_| {
            ApiError::new(ErrorCode::RevisionConflict, "Historical verifier is busy")
        })?;
        if let Some(running) = slot.as_mut() {
            running.stopped();
            Self::finish(&mut slot).map(|_| ())
        } else {
            Ok(())
        }
    }
    /// The pinned verifier source does not spawn descendants. This owner retains
    /// the child and pipe readers after failed cleanup; no second process starts.
    pub(super) fn inspect(
        &self,
        executable: &Path,
        root: &Path,
        label: &str,
        item: &str,
    ) -> Result<Vec<u8>, ApiError> {
        let mut command = Command::new(executable);
        command
            .args(["inspect-import", label, item])
            .current_dir(root);
        crate::infrastructure::os_policy::hide_console_window(&mut command);
        self.run(command, Duration::from_secs(15), |child| Box::new(child))
    }
    fn run(
        &self,
        mut command: Command,
        timeout: Duration,
        wrap: impl FnOnce(Child) -> Box<dyn Control>,
    ) -> Result<Vec<u8>, ApiError> {
        let mut slot = self.0.try_lock().map_err(|_| {
            ApiError::new(ErrorCode::RevisionConflict, "Historical verifier is busy")
        })?;
        if let Some(running) = slot.as_mut() {
            running.stopped();
            return Self::finish(&mut slot);
        }
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| failed())?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let (tx, rx) = mpsc::channel();
        let readers = [
            (
                Stream::Output,
                stdout.map(|pipe| Box::new(pipe) as Box<dyn Read + Send>),
                2_097_152,
            ),
            (
                Stream::Diagnostics,
                stderr.map(|pipe| Box::new(pipe) as Box<dyn Read + Send>),
                8_192,
            ),
        ]
        .into_iter()
        .map(|(stream, pipe, maximum)| {
            let tx = tx.clone();
            thread::spawn(move || {
                let result = pipe
                    .ok_or_else(failed)
                    .and_then(|pipe| bounded(pipe, maximum));
                let _ = tx.send((stream, result));
            })
        })
        .collect();
        *slot = Some(Running {
            child: wrap(child),
            readers,
            events: rx,
            output: None,
            received: 0,
            exited: false,
            exit_success: false,
            failure: None,
        });
        let deadline = Instant::now() + timeout;
        loop {
            let running = slot.as_mut().expect("owned verifier exists");
            running.poll();
            if running.failure.is_some() {
                running.stopped();
                break;
            }
            if running.ready() {
                break;
            }
            if Instant::now() >= deadline {
                running.failure = Some(ApiError::new(
                    ErrorCode::Timeout,
                    "Historical verifier timed out",
                ));
                running.stopped();
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        Self::finish(&mut slot)
    }
    fn finish(slot: &mut Option<Running>) -> Result<Vec<u8>, ApiError> {
        let running = slot.as_mut().expect("owned verifier exists");
        running.poll();
        if !running.ready() || running.child.wait().is_err() {
            running.failure.get_or_insert_with(failed);
            return Err(running.cleanup_error());
        }
        let mut running = slot.take().expect("terminal verifier exists");
        for reader in running.readers {
            if reader.join().is_err() {
                running.failure.get_or_insert_with(failed);
            }
        }
        if let Some(mut error) = running.failure {
            error.details = serde_json::json!({"cleanup_complete":true});
            return Err(error);
        }
        if running.received != 2 {
            return Err(failed());
        }
        running.output.ok_or_else(failed)
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        if let Ok(slot) = self.0.get_mut()
            && let Some(running) = slot.as_mut()
        {
            // Explicit call/retry owns the receipt. Drop is only best effort.
            running.stopped();
        }
    }
}

fn bounded(mut pipe: impl Read, maximum: u64) -> Result<Vec<u8>, ApiError> {
    let mut bytes = Vec::new();
    pipe.by_ref()
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| failed())?;
    if bytes.len() as u64 > maximum {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Historical verifier output exceeds limit",
        ));
    }
    Ok(bytes)
}
