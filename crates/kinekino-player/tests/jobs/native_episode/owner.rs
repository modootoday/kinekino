//! Windows native-episode playback host; the bounded owner in `playback_owner` is portable.
use super::super::fixture::Fixture;
use super::super::playback_owner::{validate_ready, validate_stop};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use std::{
    io::{BufRead, BufReader, Read},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};
pub(super) struct NativeOwner {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    root: PathBuf,
    pub(super) endpoint: String,
    stopped: bool,
}
impl NativeOwner {
    pub(super) fn start(
        fixture: &Fixture,
        catalog: &Path,
        executable: &Path,
        sha256: &str,
    ) -> Self {
        let jobs = fixture.root.join("transition-jobs");
        let profile = fixture.root.join("transition-profile");
        fs::create_dir(&jobs).unwrap();
        fs::create_dir(&profile).unwrap();
        let mut command = Command::new(&fixture.cli);
        kinekino_player::infrastructure::os_policy::hide_console_window(&mut command);
        let mut child = command
            .arg("--serve-playback")
            .arg("--job-root")
            .arg(&jobs)
            .arg("--browser-profile-root")
            .arg(&profile)
            .arg("--catalog-root")
            .arg(catalog)
            .arg("--mpv-executable")
            .arg(executable)
            .arg("--mpv-sha256")
            .arg(sha256)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(
                fs::File::create(fixture.root.join("logs/native-host.stderr")).unwrap(),
            ))
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut owner = Self {
            child,
            input,
            output,
            root: fixture.root.clone(),
            endpoint: String::new(),
            stopped: false,
        };
        let mut line = String::new();
        owner.output.read_line(&mut line).unwrap();
        fs::write(owner.root.join("logs/native-ready.stdout"), &line).unwrap();
        owner.endpoint = validate_ready(line.as_bytes(), sha256).unwrap();
        owner
    }
    pub(super) fn stop(self, expects_active: bool) {
        self.finish(Some(expects_active), 2, Duration::from_secs(10), false);
    }
    fn finish(
        mut self,
        expected_active: Option<bool>,
        completed_jobs: u64,
        observation_budget: Duration,
        require_join: bool,
    ) {
        drop(self.input.take());
        let deadline = Instant::now()
            .checked_add(observation_budget)
            .expect("Host observation deadline overflow");
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "Native host EOF did not terminate"
            );
            std::thread::sleep(Duration::from_millis(50));
        };
        self.stopped = true;
        let mut output = String::new();
        self.output.read_to_string(&mut output).unwrap();
        fs::write(self.root.join("logs/native-stop.stdout"), &output).unwrap();
        assert!(status.success());
        let stopped: Value = serde_json::from_str(output.trim()).unwrap();
        validate_stop(&stopped, expected_active, completed_jobs, require_join).unwrap();
    }
}
impl Drop for NativeOwner {
    fn drop(&mut self) {
        if self.stopped {
            return;
        }
        drop(self.input.take());
        let deadline = Instant::now() + Duration::from_secs(10);
        while self.child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
        let forced = self.child.try_wait().ok().flatten().is_none();
        if forced {
            let _ = self.child.kill();
        }
        let status = self.child.wait();
        let mut output = String::new();
        let _ = self.output.read_to_string(&mut output);
        let _ = fs::write(self.root.join("logs/native-failure-stop.stdout"), output);
        let _ = fs::write(
            self.root.join("logs/native-failure-cleanup.json"),
            json!({"forced_owned_child_stop":forced,"wait_succeeded":status.is_ok()}).to_string(),
        );
    }
}
