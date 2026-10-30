//! Shared manual-export acceptance fixture: evidence and original host lifetime.
use super::*;
use std::cell::{Cell, RefCell};

#[path = "manual_fixture/readiness.rs"]
mod readiness;

pub(super) const SEGMENT: &str = "manual-gate";
pub(super) type Check<T> = Result<T, Box<CliFailure>>;

pub(super) struct Scenario {
    pub(super) fixture: Fixture,
    pub(super) input: Inputs,
    original: Deadline,
    started: Instant,
}

impl Scenario {
    pub(super) fn new(name: &str) -> Result<Self, ScenarioFailure> {
        let mut fixture = Fixture::named(name);
        fixture.keep_evidence();
        let before = |stage, error| {
            ScenarioFailure::host(CliFailure::not_spawned(stage, error, &fixture.root))
        };
        if std::env::var("JOB_CLI_SHA256").is_err() {
            return Err(before(
                "fixed default CLI",
                invalid("Explicit CLI SHA is required"),
            ));
        }
        let input = Inputs::environment().map_err(|error| before("media inputs", error))?;
        input
            .verify_resolution()
            .map_err(|error| before("resolution", error))?;
        let original = Deadline::new(
            Instant::now(),
            Duration::from_secs(90),
            Duration::from_secs(10),
        )
        .map_err(|error| before("original deadline", error))?;
        Ok(Self {
            fixture,
            input,
            original,
            started: Instant::now(),
        })
    }

    /// Every host uses the same original deadline, including the NoMedia restart.
    pub(super) fn with_host<T>(
        &self,
        label: &str,
        action: impl FnOnce(&Calls<'_>, &Cell<Option<DecimalU64>>) -> Check<T>,
    ) -> Result<T, ScenarioFailure> {
        let before = |stage, error| {
            ScenarioFailure::host(CliFailure::not_spawned(stage, error, &self.fixture.root))
        };
        let host_root = self.fixture.root.join("hosts").join(label);
        fs::create_dir_all(host_root.parent().expect("host parent"))
            .and_then(|_| fs::create_dir(&host_root))
            .map_err(|error| before("host evidence", error))?;
        let ready = kinekino_process::phase_deadline(self.original, Duration::from_secs(10))
            .map_err(|error| before("ready deadline", error))?;
        let host = OwnedHost::launch(
            self.fixture.cli.clone(),
            vec![
                "--serve-native".into(),
                "--catalog-root".into(),
                self.input.catalog.clone().into_os_string(),
                "--mpv-executable".into(),
                self.input.mpv.clone().into_os_string(),
                "--mpv-sha256".into(),
                self.input.mpv_hash.clone().into(),
            ],
            &host_root,
            self.original,
        )
        .map_err(ScenarioFailure::host)?;
        let (host, endpoint) = host
            .ready(
                ready,
                &self.fixture.root.join(format!("logs/{label}-ready.stdout")),
                |bytes| {
                    let value: Value = serde_json::from_slice(bytes).map_err(io::Error::other)?;
                    if value["event"] != "native_host_ready"
                        || value["engine_sha256"] != self.input.mpv_hash
                        || value["playback_verified"] != false
                    {
                        return Err(invalid("Original ready identity differs"));
                    }
                    value["endpoint"]
                        .as_str()
                        .filter(|value| !value.is_empty())
                        .map(str::to_owned)
                        .ok_or_else(|| invalid("Missing endpoint"))
                },
            )
            .map_err(ScenarioFailure::host)?;
        let expected_engine = Cell::new(None);
        let returned = RefCell::new(None);
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let calls = self.calls(&endpoint);
            calls.record(
                &format!("{label}-inputs.json"),
                &json!({"catalog":self.input.catalog,
                    "resolution_json":self.input.resolution_json,
                    "resolution_sha256":self.input.resolution_hash,
                    "resolution_artifact":self.input.artifact,
                    "mpv":self.input.mpv,"mpv_sha256":self.input.mpv_hash,
                    "target_us":self.input.target,"tolerance_us":self.input.tolerance,
                    "endpoint":endpoint,"original_total_ms":90000,"cleanup_ms":10000,
                    "request_cap_ms":5000,"deadline_renewed":false}),
            )?;
            returned.replace(Some(action(&calls, &expected_engine)?));
            Ok(())
        }));
        let outcome = kinekino_process::close_request(outcome);
        let stopped = host
            .finish(
                Duration::from_secs(10),
                &self.fixture.root.join(format!("logs/{label}-stop.stdout")),
                &self.fixture.root.join(format!("logs/{label}-host.stderr")),
            )
            .and_then(|terminal| {
                let failed = |error| {
                    CliFailure::terminal(
                        "manual acceptance EOF",
                        error,
                        &host_root,
                        terminal.receipt.clone(),
                    )
                };
                if terminal.receipt.child_exit != Some(0) {
                    return Err(failed(invalid("Original host exit differs")));
                }
                let value: Value = serde_json::from_slice(&terminal.tail)
                    .map_err(|error| failed(io::Error::other(error)))?;
                if expected_engine.get().is_some() {
                    media_stop(&value, expected_engine.get()).map_err(failed)?;
                } else {
                    super::super::validate_no_media_stop(&value).map_err(failed)?;
                }
                Ok(terminal.receipt)
            });
        ScenarioFailure::conclude(outcome, stopped, None)?;
        returned
            .into_inner()
            .ok_or_else(|| before("action result", invalid("Missing result")))
    }

    pub(super) fn calls<'a>(&'a self, endpoint: &'a str) -> Calls<'a> {
        Calls {
            fixture: &self.fixture,
            endpoint,
            original: self.original,
            started: self.started,
        }
    }
}

pub(super) fn ensure(calls: &Calls<'_>, condition: bool, message: &str) -> Check<()> {
    if condition {
        Ok(())
    } else {
        Err(calls.error(invalid(message)))
    }
}

pub(super) fn data<T: DeserializeOwned>(
    calls: &Calls<'_>,
    label: &str,
    command: &str,
    payload: Value,
    revision: Option<DecimalU64>,
) -> Check<T> {
    let mut request = request(command, payload);
    request.context.expected_revision = revision.map(|value| value.0.to_string());
    let response = calls.fixture.try_call_until(
        label,
        &request,
        calls.endpoint,
        calls.original,
        Duration::from_secs(5),
        0,
    )?;
    serde_json::from_value(response["data"].clone())
        .map_err(|error| calls.error(io::Error::other(error)))
}

pub(super) fn rejected(
    calls: &Calls<'_>,
    label: &str,
    command: &str,
    payload: Value,
    revision: Option<DecimalU64>,
    code: ErrorCode,
) -> Check<Value> {
    let mut request = request(command, payload);
    request.context.expected_revision = revision.map(|value| value.0.to_string());
    let response = calls.fixture.try_call_until(
        label,
        &request,
        calls.endpoint,
        calls.original,
        Duration::from_secs(5),
        i32::from(ApiError::new(code, "expected rejection").exit_code()),
    )?;
    let expected =
        serde_json::to_value(code).map_err(|error| calls.error(io::Error::other(error)))?;
    ensure(
        calls,
        response["ok"] == false && response["error"]["code"] == expected,
        "Original expected error differs",
    )?;
    Ok(response)
}

pub(super) fn scope(calls: &Calls<'_>, label: &str) -> Check<v1::NativeIssuerScopeV1> {
    let caps: Value = data(calls, label, "capabilities", json!({}), None)?;
    ensure(
        calls,
        caps["skip_exports"]["available"] == true
            && caps["skip_sessions"]["seek_submission_available"] == false,
        "Explicit export availability/general seek boundary differs",
    )?;
    serde_json::from_value(caps["skip_exports"]["issuer_scope"].clone())
        .map_err(|error| calls.error(io::Error::other(error)))
}

pub(super) fn media(
    calls: &Calls<'_>,
    input: &Inputs,
    expected: &Cell<Option<DecimalU64>>,
) -> Check<SessionView> {
    let opened: OpenedEpisodePlayback = data(
        calls,
        "media-open",
        "playback.open",
        json!({"resolution_artifact_id":input.artifact,"paused":true,"volume_percent":0.}),
        None,
    )?;
    expected.set(Some(opened.engine_instance_id));
    let mut ready = false;
    let mut connected_seen = false;
    for index in 0..32 {
        let sample: EpisodePlaybackObservation = data(
            calls,
            &format!("before-{index}"),
            ENGINE_OBSERVE_COMMAND,
            json!({}),
            None,
        )?;
        if readiness::observe(
            &sample,
            &opened,
            input.target,
            input.tolerance,
            &mut connected_seen,
        )
        .map_err(|message| calls.error(invalid(message)))?
        {
            ready = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    ensure(calls, ready, "Original paused interval is not ready")?;
    data(
        calls,
        "session-open",
        "segments.open",
        json!({"binding":{
        "engine_instance_id":opened.engine_instance_id,
        "resolution_artifact_id":opened.resolution_artifact_id}}),
        None,
    )
}

pub(super) fn export(
    calls: &Calls<'_>,
    label: &str,
    scope: v1::NativeIssuerScopeV1,
    session: &SessionRef,
    revision: u64,
    intent: &str,
    cancelled: bool,
) -> Check<SkipExportResult> {
    data(
        calls,
        label,
        SKIP_EXPORT,
        json!({"issuer_scope":scope,"session":session,
        "input":{"segment_id":SEGMENT,"intent":intent,"cancelled":cancelled}}),
        Some(DecimalU64(revision)),
    )
}

pub(super) fn state(
    calls: &Calls<'_>,
    result: &SkipExportResult,
    session: &SessionRef,
    revision: u64,
    pass: u64,
) -> Check<()> {
    ensure(
        calls,
        result.session_state.revision == DecimalU64(revision)
            && result.session_state.pass_id == DecimalU64(pass)
            && result.session_state.session.skip_session_id == session.skip_session_id
            && result.session_state.session.binding.engine_instance_id
                == session.binding.engine_instance_id
            && result.session_state.session.binding.resolution_artifact_id
                == session.binding.resolution_artifact_id,
        "Original export session state differs",
    )
}

pub(super) fn blocked(calls: &Calls<'_>, result: &SkipExportResult, reason: &str) -> Check<()> {
    let value = serde_json::to_value(&result.result)
        .map_err(|error| calls.error(io::Error::other(error)))?;
    ensure(
        calls,
        value["state"] == "not_submitted"
            && value["decision"]["state"] == "blocked"
            && value["decision"]["reason"] == reason,
        "Expected blocked decision differs",
    )
}

pub(super) fn conflict(calls: &Calls<'_>, result: &SkipExportResult) -> Check<()> {
    let value = serde_json::to_value(&result.result)
        .map_err(|error| calls.error(io::Error::other(error)))?;
    ensure(
        calls,
        value["state"] == "no_submission_returned" && value["error"]["code"] == "revision_conflict",
        "Expected nested conflict differs",
    )
}

pub(super) fn snapshot(
    calls: &Calls<'_>,
    label: &str,
    session: &SessionRef,
    revision: u64,
    pass: u64,
) -> Check<SkipSnapshot> {
    let value: SkipSnapshot = data(calls, label, "segments.inspect", json!(session), None)?;
    ensure(
        calls,
        value.revision == DecimalU64(revision)
            && value.pass_id == DecimalU64(pass)
            && value.binding.engine_instance_id == session.binding.engine_instance_id
            && value.binding.resolution_artifact_id == session.binding.resolution_artifact_id,
        "Original stored snapshot differs",
    )?;
    Ok(value)
}
