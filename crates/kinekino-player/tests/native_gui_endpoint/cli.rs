use super::{
    executables::Executables,
    fixture::{Fixture, Owned, TestResult},
    native::{self, Native},
};
use kinekino_player::{
    application::{api::Request, playback_status::PlaybackStatus},
    domain::playback_control::PlaybackControl,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
pub struct Run {
    pub fixture: Fixture,
    owned: Owned,
    output: PathBuf,
    sequence: u32,
    engine: Option<String>,
    executables: Executables,
}
impl Run {
    pub fn new(fixture: Fixture, owned: Owned) -> TestResult<Self> {
        let parent = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
            .join("runtime/native-gui-endpoint");
        fs::create_dir_all(&parent).map_err(|_| "Cannot create owned evidence parent")?;
        let output = parent.join(&fixture.run_id);
        fs::create_dir(&output).map_err(|_| "Evidence run already exists or cannot be created")?;
        let executables = Executables::freeze(&fixture, &output)?;
        let run = Self {
            fixture,
            owned,
            output,
            sequence: 0,
            engine: None,
            executables,
        };
        run.save("fixture.json", &run.fixture)?;
        run.save("fixture-validation.json", &run.owned.validation())?;
        run.save("executables-before.json",&json!({"phase":"before_initial_observation_or_control","executables":run.executables.identities,"all_source_copy_reread_verified":true}))?;
        run.save("cli-executable.json",&json!({"path":run.executables.cli(),"original_path":crate::cli_exe::kinekino_cli_exe(),"identity":run.executables.identities[2],"request_transport":"endpoint_only","native_probe":"read_only_get_property","copied_cli_exclusively_executed":true}))?;
        Ok(run)
    }
    pub fn save(&self, name: &str, value: &impl Serialize) -> TestResult<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.output.join(name))
            .map_err(|_| "Cannot create immutable evidence file")?;
        serde_json::to_writer_pretty(&mut file, value).map_err(|_| "Cannot write evidence JSON")?;
        file.write_all(b"\n").map_err(|_| "Cannot flush evidence")?;
        file.sync_all()
            .map_err(|_| "Cannot sync evidence".to_owned())
    }
    pub fn native(&self, label: &str) -> TestResult<Native> {
        self.owned.check()?;
        let value = native::observe(&self.fixture)?;
        self.save(&format!("native-{label}.json"), &value)?;
        Ok(value)
    }
    pub fn call(&mut self, label: &str, command: &str, payload: Value) -> TestResult<Value> {
        self.owned.check()?;
        self.sequence += 1;
        let prefix = format!("{:03}-{label}", self.sequence);
        let mut request = Request::new(command);
        request.context.request_id = format!("native-{}-{}", self.fixture.run_id, self.sequence);
        request.payload = payload;
        self.save(&format!("{prefix}-request.json"), &request)?;
        let mut child = Command::new(self.executables.cli())
            .args(["--endpoint", &self.fixture.endpoint, "--input", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| "Cannot start owned CLI subprocess")?;
        let process_identity = match self.executables.verify_cli_child(&child) {
            Ok(identity) => identity,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error);
            }
        };
        let stdout = child.stdout.take().ok_or("Owned CLI stdout absent")?;
        let stderr = child.stderr.take().ok_or("Owned CLI stderr absent")?;
        let out = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = stdout.take(4_194_305).read_to_end(&mut bytes);
            (result, bytes)
        });
        let err = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = stderr.take(65537).read_to_end(&mut bytes);
            (result, bytes)
        });
        let write = child
            .stdin
            .take()
            .ok_or("Owned CLI stdin absent")?
            .write_all(
                &serde_json::to_vec(&request).map_err(|_| "Cannot serialize owned request")?,
            );
        let start = Instant::now();
        let mut timed_out = false;
        let status = loop {
            if let Some(status) = child
                .try_wait()
                .map_err(|_| "Cannot observe owned CLI exit")?
            {
                break status;
            }
            if start.elapsed() > Duration::from_secs(8) {
                timed_out = true;
                let _ = child.kill();
                break child
                    .wait()
                    .map_err(|_| "Cannot reap timed-out owned CLI")?;
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        let (out_result, stdout) = out.join().map_err(|_| "Owned CLI output reader failed")?;
        let (err_result, stderr) = err.join().map_err(|_| "Owned CLI error reader failed")?;
        self.save(&format!("{prefix}-exit.json"),&json!({"code":status.code(),"timed_out":timed_out,"stderr_empty":stderr.is_empty(),"elapsed_ms":start.elapsed().as_millis(),"executable_identity":process_identity}))?;
        let response: Value =
            serde_json::from_slice(&stdout).map_err(|_| "Owned CLI response is not JSON")?;
        self.save(&format!("{prefix}-response.json"), &response)?;
        if write.is_err()
            || out_result.is_err()
            || err_result.is_err()
            || timed_out
            || stdout.len() > 4_194_304
            || !stderr.is_empty()
            || !status.success()
            || response["ok"] != true
            || response["request_id"] != request.context.request_id
        {
            return Err("Owned CLI request failed; inspect preserved response".into());
        }
        response
            .get("data")
            .cloned()
            .ok_or("Owned CLI data absent".into())
    }
    pub fn status(&mut self) -> TestResult<PlaybackStatus> {
        let data = self.call("status", "playback.status", json!({}))?;
        let status: PlaybackStatus =
            serde_json::from_value(data).map_err(|_| "Invalid native status DTO")?;
        if !status.connected
            || status.idle != Some(false)
            || status.has_error
            || status.published_age_ms.0 > 1500
        {
            return Err("Native status is unavailable, stale or erroneous".into());
        }
        Ok(status)
    }
    pub fn wait(
        &mut self,
        predicate: impl Fn(&PlaybackStatus) -> bool,
    ) -> TestResult<PlaybackStatus> {
        let start = Instant::now();
        loop {
            let status = self.status()?;
            if predicate(&status) {
                return Ok(status);
            }
            if start.elapsed() > Duration::from_secs(3) {
                return Err("Native SDK observation deadline expired".into());
            }
            std::thread::sleep(Duration::from_millis(40));
        }
    }
    pub fn control(&mut self, label: &str, action: PlaybackControl) -> TestResult<()> {
        self.owned.check()?;
        native::observe(&self.fixture)?; // Source identity is checked before every mutation.
        let data = self.call(label, action.command(), action.payload())?;
        if data["acceptance"] != "accepted" || data["completion"] != "unobserved" {
            return Err("Control receipt incorrectly reports execution completion".into());
        }
        let engine = data["engine_instance_id"]
            .as_str()
            .ok_or("Control receipt engine identity absent")?
            .to_owned();
        if !engine.bytes().all(|b| b.is_ascii_digit())
            || engine.is_empty()
            || self.engine.as_ref().is_some_and(|old| old != &engine)
        {
            return Err("Native engine changed during leased fixture".into());
        }
        self.engine = Some(engine);
        Ok(())
    }
    pub fn initial(&mut self) -> TestResult<Native> {
        let native = self.native("initial")?;
        let status = self.status()?;
        if !native.paused
            || status.paused != Some(true)
            || status
                .position_us
                .is_none_or(|p| p.0.abs_diff(native.position_us) > 100_000)
            || status
                .volume_percent
                .is_none_or(|v| (v - native.volume).abs() > 0.1)
            || native.duration_us < 10_000_000
        {
            return Err(
                "Owned fixture must begin paused with matching observable properties".into(),
            );
        }
        self.save("initial-status.json", &status)?;
        Ok(native)
    }
    pub fn finish(&mut self, result: &TestResult<()>, restored: &TestResult<()>) -> TestResult<()> {
        let integrity = self.executables.verify_after();
        self.save("executables-after.json",&json!({"phase":"after_restoration_attempt","executables":self.executables.identities,"all_source_and_copy_hashes_unchanged":integrity.is_ok(),"error":integrity.as_ref().err()}))?;
        integrity?;
        self.save("report.json",&json!({"scenario_passed":result.is_ok(),"restoration_passed":restored.is_ok(),"scenario_error":result.as_ref().err(),"restoration_error":restored.as_ref().err(),"engine_instance_id":self.engine,"receipt_semantics":"enqueue accepted; completion unobserved","independent_observation":"owned mpv read-only get_property, distinct connection","position_tolerance_us":100000,"fixture_processes_left_running_for_ui_owner":true}))
    }
}
