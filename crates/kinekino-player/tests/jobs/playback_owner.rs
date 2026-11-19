//! Shared test-owned playback host; fallback cleanup is not a quiescence proof.
use super::fixture::Fixture;
use super::kinekino_process::{CliFailure, OwnedHost};
use kinekino_process::domain::deadline::Deadline;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(super) fn validate_ready(bytes: &[u8], sha256: &str) -> std::io::Result<String> {
    let ready: Value = serde_json::from_slice(bytes).map_err(std::io::Error::other)?;
    if ready["event"] != "playback_host_ready"
        || ready["engine_sha256"] != sha256
        || ready["playback_verified"] != false
    {
        return Err(std::io::Error::other(
            "Original playback readiness identity/verification mismatch",
        ));
    }
    ready["endpoint"]
        .as_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| std::io::Error::other("Missing playback endpoint"))
}
pub(super) fn validate_stop(
    stopped: &Value,
    expected_active: Option<bool>,
    completed_jobs: u64,
    require_join: bool,
) -> std::io::Result<()> {
    let invalid = || std::io::Error::other("Original playback stop receipt contract failed");
    if stopped["event"] != "playback_host_stopped" || stopped["host_lifetime_ended"] != true {
        return Err(invalid());
    }
    if require_join {
        let joined = &stopped["local_host_shutdown"];
        if joined["pipe"] != "joined"
            || joined["sdk"] != "joined"
            || joined["dispatch_poisoned"] != false
            || !matches!(joined["dispatch"].as_str(), Some("idle" | "completed"))
        {
            return Err(invalid());
        }
    }
    let receipt = &stopped["receipt"];
    if receipt["native"].get("Ok").is_none()
        || receipt["provider"].get("Ok").is_none()
        || receipt["pending_transition_job"].get("Ok").is_none()
        || !receipt["pending_transition_job"]["Ok"].is_null()
        || receipt["provider"]["Ok"]["browser_worker_exited"] != true
        || receipt["provider"]["Ok"]["jobs"]["completed_total"]
            .as_u64()
            .is_none_or(|value| value < completed_jobs)
    {
        return Err(invalid());
    }
    let native = &receipt["native"]["Ok"];
    if expected_active.unwrap_or(!native.is_null()) {
        if [
            "engine_exited",
            "ipc_worker_exited",
            "stderr_worker_exited",
            "stdout_worker_exited",
        ]
        .iter()
        .any(|key| native[key] != true)
        {
            return Err(invalid());
        }
    } else if !native.is_null() {
        return Err(invalid());
    }
    Ok(())
}

pub(super) struct BoundedNativeOwner {
    process: OwnedHost,
    root: PathBuf,
    pub(super) endpoint: String,
}
impl BoundedNativeOwner {
    pub(super) fn process_id(&self) -> u32 {
        self.process.pid()
    }
    pub(super) fn deadline(&self) -> Deadline {
        self.process.deadline()
    }
    pub(super) fn try_start(
        fixture: &Fixture,
        catalog: &Path,
        executable: &Path,
        sha256: &str,
        original: Deadline,
        ready_cap: Duration,
    ) -> Result<Self, Box<CliFailure>> {
        let ready_deadline =
            super::kinekino_process::phase_deadline(original, ready_cap).map_err(|error| {
                CliFailure::not_spawned("playback ready budget", error, &fixture.root)
            })?;
        let jobs = fixture.root.join("transition-jobs");
        let profile = fixture.root.join("transition-profile");
        fs::create_dir(&jobs)
            .and_then(|_| fs::create_dir(&profile))
            .map_err(|error| {
                CliFailure::not_spawned("playback directories", error, &fixture.root)
            })?;
        let arguments = vec![
            "--serve-playback".into(),
            "--job-root".into(),
            jobs.into_os_string(),
            "--browser-profile-root".into(),
            profile.into_os_string(),
            "--catalog-root".into(),
            catalog.as_os_str().to_owned(),
            "--mpv-executable".into(),
            executable.as_os_str().to_owned(),
            "--mpv-sha256".into(),
            sha256.into(),
        ];
        let process = OwnedHost::launch(fixture.cli.clone(), arguments, &fixture.root, original)?;
        let (process, endpoint) = process.ready(
            ready_deadline,
            &fixture.root.join("logs/native-ready.stdout"),
            |bytes| validate_ready(bytes, sha256),
        )?;
        if let Err(error) =
            process.export_stderr(&fixture.root.join("logs/native-host.stderr"), false)
        {
            let deadline = process.active_cap();
            return Err(process.fail("ready stderr evidence", error, deadline));
        }
        Ok(Self {
            process,
            root: fixture.root.clone(),
            endpoint,
        })
    }
    pub(super) fn try_stop_chain(
        self,
        completed_jobs: u64,
        cap: Duration,
    ) -> Result<kinekino_process::domain::observation::ProcessReceipt, Box<CliFailure>> {
        let terminal = self.process.finish(
            cap,
            &self.root.join("logs/native-stop.stdout"),
            &self.root.join("logs/native-host.stderr"),
        )?;
        let failed =
            |stage, error| CliFailure::terminal(stage, error, &self.root, terminal.receipt.clone());
        if terminal.receipt.child_exit != Some(0) {
            return Err(failed(
                "playback exit",
                std::io::Error::other("Playback host exited abnormally"),
            ));
        }
        let stopped = serde_json::from_slice(&terminal.tail)
            .map_err(|error| failed("playback stop JSON", std::io::Error::other(error)))?;
        validate_stop(&stopped, None, completed_jobs, true)
            .map_err(|error| failed("playback stop receipt", error))?;
        if terminal.deadline.remaining(Instant::now()).is_zero() {
            return Err(failed(
                "playback stop validation deadline",
                std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Original closeout deadline elapsed",
                ),
            ));
        }
        Ok(terminal.receipt)
    }
}

#[cfg(test)]
mod bounded_receipt_tests {
    use super::*;

    #[test]
    fn readiness_preserves_engine_identity_and_unverified_playback() {
        kinekino_platform::install();
        let ready = json!({"event":"playback_host_ready","engine_sha256":"original","playback_verified":false,"endpoint":"owned-endpoint"});
        assert_eq!(
            validate_ready(&serde_json::to_vec(&ready).unwrap(), "original").unwrap(),
            "owned-endpoint"
        );
        assert!(validate_ready(&serde_json::to_vec(&ready).unwrap(), "different").is_err());
        let mut claimed = ready;
        claimed["playback_verified"] = json!(true);
        assert!(validate_ready(&serde_json::to_vec(&claimed).unwrap(), "original").is_err());
    }

    #[test]
    fn family_empty_cannot_substitute_product_workers_or_original_completed_jobs() {
        kinekino_platform::install();
        let stopped = json!({"event":"playback_host_stopped","host_lifetime_ended":true,
            "local_host_shutdown":{"pipe":"joined","sdk":"joined","dispatch_poisoned":false,"dispatch":"idle"},
            "receipt":{"native":{"Ok":{"engine_exited":true,"ipc_worker_exited":true,"stderr_worker_exited":true,"stdout_worker_exited":true}},
            "provider":{"Ok":{"browser_worker_exited":true,"jobs":{"completed_total":9}}},"pending_transition_job":{"Ok":null}}});
        assert!(validate_stop(&stopped, None, 9, true).is_ok());
        for key in [
            "engine_exited",
            "ipc_worker_exited",
            "stderr_worker_exited",
            "stdout_worker_exited",
        ] {
            let mut missing = stopped.clone();
            missing["receipt"]["native"]["Ok"][key] = json!(false);
            missing["owned_family"] = json!("Empty");
            assert!(validate_stop(&missing, None, 9, true).is_err());
        }
        assert!(validate_stop(&stopped, None, 10, true).is_err());
        let mut unjoined = stopped;
        unjoined["local_host_shutdown"]["sdk"] = json!("not_applicable");
        assert!(validate_stop(&unjoined, None, 9, true).is_err());
    }
}
