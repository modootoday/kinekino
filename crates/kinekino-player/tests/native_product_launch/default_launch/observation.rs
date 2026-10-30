use super::{Failure, Mode, Probe, Stage, owner, window};
use kinekino_player::application::api::ProjectClient;
use kinekino_player::{
    application::{api::RequestContext, projects::InspectProject},
    domain::api_values::DecimalU64,
    infrastructure::local_host::{EndpointClient, TransportOptions},
};
use kinekino_process::{
    application::{
        raw_records::{RecordCursor, RecordPoll},
        supervisor::Supervisor,
    },
    domain::deadline::Deadline,
};
use serde_json::Value;
use std::{
    io::Read,
    time::{Duration, Instant},
};
pub(super) struct Observations {
    pub(super) events: Vec<Value>,
    pub(super) window: Option<window::Window>,
    pub(super) sdk_read: bool,
    pub(super) close_sent: bool,
    pub(super) close_attempted: bool,
}
impl Observations {
    pub(super) fn new() -> Self {
        Self {
            events: Vec::new(),
            window: None,
            sdk_read: false,
            close_sent: false,
            close_attempted: false,
        }
    }
    pub(super) fn records(
        &mut self,
        supervisor: &Supervisor<
            owner::NoForce<kinekino_process::infrastructure::family::OwnedProcessFamily>,
        >,
        cursor: &mut RecordCursor,
        terminal: bool,
    ) -> Result<(), Failure> {
        supervisor
            .evidence()
            .check_limit(1024 * 1024)
            .map_err(|e| Failure::new(Stage::Raw, e))?;
        loop {
            match cursor
                .poll(&supervisor.evidence().stdout, 256 * 1024, terminal)
                .map_err(|e| Failure::new(Stage::Raw, e))?
            {
                RecordPoll::Pending => return Ok(()),
                RecordPoll::Record { bytes, .. } => {
                    if self.events.len() >= 128 {
                        return Err(Failure::new(Stage::Raw, "Event bound exceeded"));
                    }
                    let event =
                        serde_json::from_slice(&bytes).map_err(|e| Failure::new(Stage::Raw, e))?;
                    self.events.push(event);
                }
            }
        }
    }
    pub(super) fn inspect(&mut self, probe: &Probe, deadline: Deadline) -> Result<bool, Failure> {
        let Some(endpoint) = self
            .events
            .iter()
            .find(|e| e["kind"] == "local-host-ready")
            .and_then(|e| e["data"]["endpoint"].as_str())
        else {
            return Ok(false);
        };
        let budget = deadline.request_budget(Instant::now(), Duration::from_millis(500));
        if budget < Duration::from_millis(10) {
            return Err(Failure::new(Stage::Deadline, "No SDK request budget"));
        }
        let client = EndpointClient::new(endpoint, TransportOptions { timeout: budget })
            .map_err(|e| Failure::debug(Stage::Sdk, e))?;
        let stored = client
            .inspect_project(
                RequestContext::default(),
                InspectProject {
                    project_id: probe.id.clone(),
                    revision: Some(DecimalU64(probe.revision)),
                },
            )
            .map_err(|e| Failure::debug(Stage::Sdk, e))?;
        if stored.project.project_id != probe.id
            || stored.project.revision.0 != probe.revision
            || stored.revision_sha256 != probe.sha256
        {
            return Err(Failure::new(Stage::Sdk, "Pinned project identity mismatch"));
        }
        if deadline.operation_remaining(Instant::now()).is_zero() {
            return Err(Failure::new(
                Stage::Deadline,
                "SDK returned after operation cutoff",
            ));
        }
        self.sdk_read = true;
        Ok(true)
    }
    pub(super) fn check_report(&self, mode: Mode) -> Result<(), Failure> {
        if mode == Mode::NoArgs {
            return Ok(());
        }
        let data = self
            .events
            .iter()
            .find(|e| e["kind"] == "native-viewer-outcome")
            .map(|e| &e["data"])
            .ok_or_else(|| Failure::new(Stage::Report, "Missing Native outcome"))?;
        if !self.sdk_read
            || data["success"] != true
            || data["normal_os_exit"] != true
            || data["owners"]["stage"] != "native_return"
            || data["owners"]["result"] != "Ok(())"
            || data["cleanup"] != "WorkersQuiescent(Closed)"
            || data["error"] != "None"
            || data["acceptance_error"] != "None"
        {
            return Err(Failure::new(
                Stage::Report,
                "Original Native outcome is not normal Closed",
            ));
        }
        Ok(())
    }
    pub(super) fn final_raw_eof(&mut self, supervisor: &owner::Original) -> Result<(), Failure> {
        let mut lengths = Vec::new();
        for raw in [&supervisor.evidence().stdout, &supervisor.evidence().stderr] {
            let reader = raw.open_read().map_err(|e| Failure::new(Stage::Raw, e))?;
            let mut bytes = Vec::new();
            reader
                .take(1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| Failure::new(Stage::Raw, e))?;
            if bytes.len() > 1024 * 1024 {
                return Err(Failure::new(Stage::Raw, "Final raw file exceeds bound"));
            }
            lengths.push(bytes.len());
        }
        self.events
            .push(serde_json::json!({"kind":"final-regular-file-eof",
            "stdout_bytes":lengths[0],"stderr_bytes":lengths[1],"reader_workers":0,
            "pipe_eof_observed":false}));
        Ok(())
    }
}
