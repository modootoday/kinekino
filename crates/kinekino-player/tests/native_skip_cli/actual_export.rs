//! Explicitly allocated media checkpoints, with opt-in endpoint response-loss validation.
use super::{
    fixture::{Fixture, request},
    kinekino_process::{self, CliFailure, OwnedHost, ScenarioFailure},
};
use ::kinekino_process::domain::deadline::Deadline;
use kinekino_player::application::api::EpisodeClient;
use kinekino_player::{
    application::{
        api::{ApiError, ErrorCode},
        episode_resolve::{EpisodeResolutionResults, StoredEpisodeResolution},
        playback_session::{EpisodePlaybackObservation, OpenedEpisodePlayback, v1},
        skip_segments::{SkipSnapshot, host::*},
    },
    domain::{api_values::DecimalU64, identifiers::valid_sha256},
    infrastructure::catalog::CatalogFiles,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{
    fs,
    io::{self, Read},
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    time::{Duration, Instant},
};

const ENGINE_OBSERVE_COMMAND: &str = "playback.engine";

#[cfg(not(feature = "validation-export-delivery-loss"))]
#[path = "actual_export/manual_fixture.rs"]
mod manual_fixture;
#[cfg(not(feature = "validation-export-delivery-loss"))]
#[path = "actual_export/manual_negative.rs"]
mod manual_negative;
#[cfg(not(feature = "validation-export-delivery-loss"))]
#[path = "actual_export/manual_owner.rs"]
mod manual_owner;

#[cfg(not(feature = "validation-export-delivery-loss"))]
#[test]
#[ignore = "Separate default-artifact manual-export negative allocation; original media and owner required"]
fn actual_same_instance_manual_export_negative_contract() -> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    manual_negative::run()
}

#[cfg(not(feature = "validation-export-delivery-loss"))]
#[test]
#[ignore = "Separate default-artifact manual-export override/owner allocation; original media and owner required"]
fn actual_manual_override_and_original_owner_contract() -> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    manual_owner::run()
}

fn trust_read_inputs(path: PathBuf, hash: String) -> io::Result<(PathBuf, String)> {
    if !path.is_absolute()
        || path.file_name().and_then(|name| name.to_str()) != Some("mpv.exe")
        || !valid_sha256(&hash)
    {
        return Err(invalid(
            "Explicit absolute mpv.exe path and SHA256 are required",
        ));
    }
    Ok((path, hash))
}

#[test]
fn trust_read_diagnostic_rejects_relative_path_and_invalid_hash() {
    kinekino_platform::install();
    assert!(trust_read_inputs("mpv.exe".into(), "a".repeat(64)).is_err());
    assert!(trust_read_inputs(PathBuf::from(r"C:\mpv.exe"), "invalid".into()).is_err());
    assert!(trust_read_inputs(PathBuf::from(r"C:\other.exe"), "a".repeat(64)).is_err());
    assert!(trust_read_inputs(PathBuf::from(r"C:\mpv.exe"), "a".repeat(64)).is_ok());
}

/// Measures this read in isolation; it cannot establish the cause of host readiness failure.
#[test]
#[ignore = "Requires separately allocated explicit trusted executable read diagnostic"]
fn diagnostic_trusted_mpv_open_read_only_elapsed() -> io::Result<()> {
    kinekino_platform::install();
    use kinekino_player::infrastructure::mpv::TrustedMpvExecutable;
    use std::io::Write;
    let (path, expected_sha256) = trust_read_inputs(
        std::env::var_os("NATIVE_MPV_EXE")
            .ok_or_else(|| invalid("Explicit NATIVE_MPV_EXE is required"))?
            .into(),
        std::env::var("NATIVE_MPV_SHA256")
            .map_err(|_| invalid("Explicit NATIVE_MPV_SHA256 is required"))?,
    )?;
    let evidence = PathBuf::from(
        std::env::var_os("NATIVE_MPV_TRUST_READ_EVIDENCE")
            .ok_or_else(|| invalid("Explicit fresh diagnostic evidence path is required"))?,
    );
    if !evidence.is_absolute() {
        return Err(invalid("Diagnostic evidence path must be absolute"));
    }
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(evidence)?;
    let started = Instant::now();
    let opened = TrustedMpvExecutable::open(&path, &expected_sha256);
    let elapsed_us = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);
    let (bytes, verified_sha256, error) = match &opened {
        Ok(executable) => (
            Some(fs::metadata(executable.path())?.len()),
            Some(executable.sha256()),
            None,
        ),
        Err(error) => (None, None, Some(error.code)),
    };
    let proof = json!({"schema_version":1,"diagnostic":"trusted_mpv_open_read_only",
        "elapsed_us":elapsed_us,"verified_bytes":bytes,"expected_sha256":expected_sha256,
        "verified_sha256":verified_sha256,"error_code":error,
        "process_started":false,"network_used":false,"host_cause_established":false});
    let mut encoded = serde_json::to_vec(&proof).map_err(io::Error::other)?;
    if encoded.len() >= 2048 {
        return Err(invalid("Diagnostic proof exceeds 2KiB budget"));
    }
    encoded.push(b'\n');
    output.write_all(&encoded)?;
    output.flush()?;
    opened
        .map(|_| ())
        .map_err(|error| invalid(&format!("Original trusted open failed: {error:?}")))
}

fn invalid(message: &str) -> io::Error {
    io::Error::other(message)
}
struct Inputs {
    catalog: PathBuf,
    resolution_json: PathBuf,
    resolution_hash: String,
    artifact: String,
    mpv: PathBuf,
    mpv_hash: String,
    target: DecimalU64,
    tolerance: u64,
}
impl Inputs {
    fn environment() -> io::Result<Self> {
        let value = |name| {
            std::env::var(name).map_err(|_| invalid(&format!("Explicit {name} is required")))
        };
        let input = Self {
            catalog: value("NATIVE_EXPORT_CATALOG_ROOT")?.into(),
            resolution_json: value("NATIVE_EXPORT_RESOLUTION_JSON")?.into(),
            resolution_hash: value("NATIVE_EXPORT_RESOLUTION_SHA256")?,
            artifact: value("NATIVE_EXPORT_RESOLUTION_ARTIFACT")?,
            mpv: value("NATIVE_MPV_EXE")?.into(),
            mpv_hash: value("NATIVE_MPV_SHA256")?,
            target: DecimalU64(
                value("NATIVE_EXPORT_TARGET_US")?
                    .parse()
                    .map_err(|_| invalid("Invalid explicit target"))?,
            ),
            tolerance: value("NATIVE_EXPORT_LANDING_TOLERANCE_US")?
                .parse()
                .map_err(|_| invalid("Invalid explicit landing tolerance"))?,
        };
        if !input.catalog.is_absolute()
            || !input.resolution_json.is_absolute()
            || !input.mpv.is_absolute()
            || ![&input.resolution_hash, &input.artifact, &input.mpv_hash]
                .into_iter()
                .all(|hash| valid_sha256(hash))
            || input.target.0 == 0
            || input.tolerance > 1_000_000
        {
            return Err(invalid("Invalid explicit media inputs"));
        }
        kinekino_player::domain::playback_control::PlaybackControl::Seek {
            position_us: input.target,
        }
        .validate()
        .map_err(invalid)?;
        Ok(input)
    }
    fn verify_resolution(&self) -> io::Result<()> {
        if !self.catalog.is_dir() {
            return Err(invalid("An existing authorized catalog is required"));
        }
        let mut bytes = Vec::new();
        fs::File::open(&self.resolution_json)?
            .take(64 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 64 * 1024 {
            return Err(invalid("Resolution input exceeds explicit 64KiB budget"));
        }
        use sha2::{Digest, Sha256};
        if format!("{:x}", Sha256::digest(&bytes)) != self.resolution_hash {
            return Err(invalid("Original resolution JSON hash differs"));
        }
        let expected: StoredEpisodeResolution =
            serde_json::from_slice(&bytes).map_err(io::Error::other)?;
        expected
            .validate()
            .map_err(|error| invalid(&format!("Original resolution validation: {error:?}")))?;
        let catalog = CatalogFiles::open(&self.catalog)
            .map_err(|error| invalid(&format!("Original catalog: {error:?}")))?;
        let original = EpisodeResolutionResults::read(&catalog, &self.artifact)
            .map_err(|error| invalid(&format!("Original evidence read: {error:?}")))?;
        if serde_json::to_value(expected).map_err(io::Error::other)?
            != serde_json::to_value(original).map_err(io::Error::other)?
        {
            return Err(invalid(
                "Original catalog resolution differs from pinned JSON",
            ));
        }
        Ok(())
    }
}
struct Calls<'a> {
    fixture: &'a Fixture,
    endpoint: &'a str,
    original: Deadline,
    started: Instant,
}
impl Calls<'_> {
    fn error(&self, error: io::Error) -> Box<CliFailure> {
        CliFailure::not_spawned("actual export evidence", error, &self.fixture.root)
    }
    fn call<T: DeserializeOwned>(
        &self,
        label: &str,
        command: &str,
        payload: Value,
        revision: Option<DecimalU64>,
    ) -> Result<T, Box<CliFailure>> {
        let mut request = request(command, payload);
        request.context.expected_revision = revision.map(|value| value.0.to_string());
        let response = self.fixture.try_call_until(
            label,
            &request,
            self.endpoint,
            self.original,
            Duration::from_secs(10),
            0,
        )?;
        serde_json::from_value(response["data"].clone())
            .map_err(|error| self.error(io::Error::other(error)))
    }
    fn record(&self, name: &str, value: &impl Serialize) -> Result<(), Box<CliFailure>> {
        let bytes = v1::encode_native_export_v1(value)
            .map_err(|error| self.error(invalid(&format!("Bounded evidence: {error:?}"))))?;
        fs::write(self.fixture.root.join(name), bytes).map_err(|error| self.error(error))
    }
}
#[derive(Debug)]
struct Admission {
    reference: v1::NativeResultRefV1,
    metadata: v1::NativeExportMetadataV1,
    post_error: Option<v1::NativeExportCoreErrorV1>,
}
fn admission(
    result: SkipExportResult,
    origin: &SkipExportOrigin,
    scope: v1::NativeIssuerScopeV1,
    target: DecimalU64,
) -> io::Result<Admission> {
    let (reference, metadata, post_error) = match result.result {
        SkipExportAction::Admitted {
            reference,
            metadata,
            ..
        } => (reference, metadata, None),
        SkipExportAction::AdmittedWithError {
            reference,
            metadata,
            error,
            ..
        } => (reference, metadata, Some(error)),
        _ => {
            return Err(invalid(
                "No original admission returned; never resubmit this seek",
            ));
        }
    };
    validate_admission_identity(reference, &metadata, origin, scope, target)?;
    Ok(Admission {
        reference,
        metadata,
        post_error,
    })
}
fn validate_admission_identity(
    reference: v1::NativeResultRefV1,
    metadata: &v1::NativeExportMetadataV1,
    origin: &SkipExportOrigin,
    scope: v1::NativeIssuerScopeV1,
    target: DecimalU64,
) -> io::Result<()> {
    let saved = metadata.origin();
    if reference.scope() != scope
        || saved.session() != origin.session.skip_session_id
        || saved.pass() != origin.pass_id
        || saved.segment() != origin.segment_id
        || saved.request_revision() != origin.request_revision
        || saved.engine_instance_id() != origin.session.binding.engine_instance_id
        || saved.resolution_artifact_id() != origin.session.binding.resolution_artifact_id
        || metadata.actual_target_us() != target
    {
        return Err(invalid("Original admission metadata differs"));
    }
    Ok(())
}
fn acknowledged(result: &v1::NativeExportResultV1, original: &Admission) -> io::Result<bool> {
    if result.item.reference() != original.reference || result.item.metadata() != &original.metadata
    {
        return Err(invalid(
            "Original result identity or saved metadata changed",
        ));
    }
    match result.item.state() {
        v1::NativeExportStateV1::Queued
        | v1::NativeExportStateV1::WriteCommitted
        | v1::NativeExportStateV1::WrittenAwaitingAck => Ok(false),
        v1::NativeExportStateV1::CommandAccepted
            if result.write_committed && result.failure.is_none() =>
        {
            Ok(true)
        }
        _ => Err(invalid(
            "Original native command failed or ACK is unobserved",
        )),
    }
}
fn valid_sample(sample: &EpisodePlaybackObservation, opened: &OpenedEpisodePlayback) -> bool {
    sample.engine_instance_id == opened.engine_instance_id
        && sample.resolution_artifact_id == opened.resolution_artifact_id
        && sample.playback.connected
        && !sample.playback.has_error
}
fn landed(position: Option<DecimalU64>, target: DecimalU64, tolerance: u64) -> bool {
    position.is_some_and(|value| value.0.abs_diff(target.0) <= tolerance)
}
fn media_stop(value: &Value, expected_engine: Option<DecimalU64>) -> io::Result<()> {
    let receipt = value
        .get("receipt")
        .filter(|receipt| receipt.is_object())
        .ok_or_else(|| invalid("Missing original media receipt object"))?;
    let engine: DecimalU64 =
        serde_json::from_value(receipt["engine_instance_id"].clone()).map_err(io::Error::other)?;
    let shutdown = &value["local_host_shutdown"];
    if value["event"] != "native_host_stopped"
        || value["host_lifetime_ended"] != true
        || Some(engine) != expected_engine
        || [
            "engine_exited",
            "ipc_worker_exited",
            "stderr_worker_exited",
            "stdout_worker_exited",
        ]
        .iter()
        .any(|field| receipt.get(*field) != Some(&Value::Bool(true)))
        || shutdown["pipe"] != "joined"
        || shutdown["sdk"] != "joined"
        || shutdown.get("dispatch_poisoned") != Some(&Value::Bool(false))
        || !matches!(shutdown["dispatch"].as_str(), Some("idle" | "completed"))
    {
        return Err(invalid("Original media cleanup proof is incomplete"));
    }
    Ok(())
}

#[test]
#[ignore = "Separate actual-media allocation; explicit authorized catalog/resolution JSON and pinned mpv; no browser resolve or implicit retry"]
fn actual_original_guarded_export_ack_landing_and_consumption() -> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    run_media_case(false)
}
#[cfg(feature = "validation-export-delivery-loss")]
#[test]
#[ignore = "Separate actual CLI endpoint-delivery-loss allocation; authorized media and validation-only CLI required"]
fn actual_endpoint_loss_recovers_original_export_without_resubmission()
-> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    run_media_case(true)
}
fn run_media_case(drop_delivery: bool) -> Result<(), ScenarioFailure> {
    let _cli_hash =
        std::env::var("JOB_CLI_SHA256").expect("Explicit fixed normal CLI SHA is required");
    let mut fixture = Fixture::named(if drop_delivery {
        "native-export-delivery-loss"
    } else {
        "native-export-media"
    });
    fixture.keep_evidence();
    let before =
        |stage, error| ScenarioFailure::host(CliFailure::not_spawned(stage, error, &fixture.root));
    #[cfg(not(feature = "validation-export-delivery-loss"))]
    if drop_delivery {
        return Err(before(
            "validation feature",
            invalid("Delivery-loss validation feature is absent"),
        ));
    }
    let input = Inputs::environment().map_err(|error| before("explicit media inputs", error))?;
    input
        .verify_resolution()
        .map_err(|error| before("original resolution evidence", error))?;
    let original = Deadline::new(
        Instant::now(),
        Duration::from_secs(90),
        Duration::from_secs(10),
    )
    .map_err(|error| before("original media deadline", error))?;
    let started = Instant::now();
    let ready = kinekino_process::phase_deadline(original, Duration::from_secs(10))
        .map_err(|error| before("ready deadline", error))?;
    let arguments = vec![
        "--serve-native".into(),
        "--catalog-root".into(),
        input.catalog.clone().into_os_string(),
        "--mpv-executable".into(),
        input.mpv.clone().into_os_string(),
        "--mpv-sha256".into(),
        input.mpv_hash.clone().into(),
    ];
    #[cfg(feature = "validation-export-delivery-loss")]
    let proof_path = fixture.root.join("delivery-proof.jsonl");
    #[cfg(feature = "validation-export-delivery-loss")]
    let arguments = {
        let mut args = arguments;
        if drop_delivery {
            args.extend([
                "--validation-export-drop-request-id".into(),
                LOSS_REQUEST_ID.into(),
                "--validation-export-evidence".into(),
                proof_path.clone().into_os_string(),
            ]);
        }
        args
    };
    let host = OwnedHost::launch(fixture.cli.clone(), arguments, &fixture.root, original)
        .map_err(ScenarioFailure::host)?;
    let (host, endpoint) = host
        .ready(
            ready,
            &fixture.root.join("logs/media-ready.stdout"),
            |bytes| {
                let value: Value = serde_json::from_slice(bytes).map_err(io::Error::other)?;
                if value["event"] != "native_host_ready"
                    || value["engine_sha256"] != input.mpv_hash
                    || value["playback_verified"] != false
                {
                    return Err(invalid("Original media host identity differs"));
                }
                value["endpoint"]
                    .as_str()
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
                    .ok_or_else(|| invalid("Missing original endpoint"))
            },
        )
        .map_err(ScenarioFailure::host)?;
    let expected_engine = std::cell::Cell::new(None);
    #[cfg(feature = "validation-export-delivery-loss")]
    let recovered = std::cell::RefCell::new(None);
    let action = catch_unwind(AssertUnwindSafe(|| {
        let calls = Calls {
            fixture: &fixture,
            endpoint: &endpoint,
            original,
            started,
        };
        calls.record("explicit-inputs.json",&json!({"catalog":input.catalog,"resolution_json":input.resolution_json,"resolution_sha256":input.resolution_hash,"artifact":input.artifact,"mpv_sha256":input.mpv_hash,"target_us":input.target,"landing_tolerance_us":input.tolerance,"endpoint_response_loss_injection":drop_delivery}))?;
        let opened: OpenedEpisodePlayback = calls.call(
            "media-open",
            "playback.open",
            json!({"resolution_artifact_id":input.artifact,"paused":true,"volume_percent":0.}),
            None,
        )?;
        expected_engine.set(Some(opened.engine_instance_id));
        if opened.engine_instance_id.0 == 0 || opened.resolution_artifact_id != input.artifact {
            return Err(calls.error(invalid("Opened original binding differs")));
        }
        let mut observed = None;
        for index in 0..32 {
            let sample: EpisodePlaybackObservation = calls.call(
                &format!("before-{index}"),
                ENGINE_OBSERVE_COMMAND,
                json!({}),
                None,
            )?;
            if !valid_sample(&sample, &opened) {
                return Err(calls.error(invalid(
                    "Original pre-submit binding or native state differs",
                )));
            }
            if let (Some(position), Some(duration)) =
                (sample.playback.position_us, sample.playback.duration_us)
                && sample.playback.paused == Some(true)
                && position
                    .0
                    .checked_add(input.tolerance)
                    .is_some_and(|value| value < input.target.0)
                && input.target < duration
            {
                observed = Some(sample);
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let observed = observed.ok_or_else(|| {
            calls.error(invalid(
                "Original position/duration did not admit explicit manual interval",
            ))
        })?;
        calls.record(
            "before-submit.json",
            &json!({"elapsed_us":calls.started.elapsed().as_micros(),"observation":observed}),
        )?;
        let caps: Value = calls.call("media-capabilities", "capabilities", json!({}), None)?;
        if caps["skip_exports"]["available"] != true {
            return Err(calls.error(invalid("Explicit export host unavailable")));
        }
        let scope: v1::NativeIssuerScopeV1 =
            serde_json::from_value(caps["skip_exports"]["issuer_scope"].clone())
                .map_err(|error| calls.error(io::Error::other(error)))?;
        let view:SessionView=calls.call("segment-open","segments.open",json!({"binding":{"engine_instance_id":opened.engine_instance_id,"resolution_artifact_id":opened.resolution_artifact_id}}),None)?;
        let edited:SkipSnapshot=calls.call("segment-edit","segments.edit",json!({"session":view.session,"input":{"segment_id":"manual-gate","kind":"opening","in_us":"0","out_us":input.target}}),Some(view.snapshot.revision))?;
        let annotation = edited
            .segments
            .iter()
            .find(|segment| segment.id == "manual-gate")
            .ok_or_else(|| calls.error(invalid("Original manual annotation absent")))?;
        let accepted:SkipSnapshot=calls.call("segment-accept","segments.accept_skip",json!({"session":view.session,"input":{"segment_id":"manual-gate","annotation_revision":annotation.revision}}),Some(edited.revision))?;
        let origin = SkipExportOrigin {
            session: view.session,
            pass_id: accepted.pass_id,
            request_revision: accepted.revision,
            segment_id: "manual-gate".into(),
        };
        // Exactly one submission. No outcome, timeout or lost response permits retry here.
        let submit_payload = json!({"issuer_scope":scope,"session":origin.session,"input":{"segment_id":origin.segment_id,"intent":"manual","cancelled":false}});
        let admitted = submit_original_once(
            &calls,
            submit_payload,
            &origin,
            scope,
            input.target,
            drop_delivery,
        )?;
        #[cfg(feature = "validation-export-delivery-loss")]
        {
            recovered.replace(Some((admitted.reference, admitted.metadata.clone())));
        }
        calls.record("original-admission.json",&json!({"elapsed_us":calls.started.elapsed().as_micros(),"reference":admitted.reference,"metadata":admitted.metadata,"post_error":admitted.post_error}))?;
        if admitted.post_error.is_some() {
            return Err(calls.error(invalid(
                "Original admitted-with-error preserved; no resubmission",
            )));
        }
        if admitted.metadata.annotation().annotation_revision() != annotation.revision
            || admitted.metadata.annotation().verified_revision() != annotation.revision
        {
            return Err(calls.error(invalid("Original accepted annotation revision differs")));
        }
        let listed: v1::NativeExportListV1 = calls.call(
            "original-list",
            EXPORT_LIST,
            json!({"issuer_scope":scope,"origin":origin}),
            None,
        )?;
        if listed.scope() != scope
            || listed.items().len() != 1
            || !listed.items().iter().any(|item| {
                item.reference() == admitted.reference && item.metadata() == &admitted.metadata
            })
        {
            return Err(calls.error(invalid("Original admission is absent from issuer list")));
        }
        let payload = json!({"issuer_scope":scope,"origin":origin,"reference":admitted.reference});
        let mut acked = false;
        for index in 0..32 {
            let result: v1::NativeExportResultV1 =
                calls.call(&format!("peek-{index}"), EXPORT_PEEK, payload.clone(), None)?;
            calls.record(
                &format!("ack-{index}.json"),
                &json!({"elapsed_us":calls.started.elapsed().as_micros(),"result":result}),
            )?;
            if acknowledged(&result, &admitted).map_err(|error| calls.error(error))? {
                acked = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        if !acked {
            return Err(calls.error(invalid("Original ACK was not observed in bounded samples")));
        }
        let mut landing = false;
        for index in 0..32 {
            let sample: EpisodePlaybackObservation = calls.call(
                &format!("landing-{index}"),
                ENGINE_OBSERVE_COMMAND,
                json!({}),
                None,
            )?;
            calls.record(&format!("landing-{index}.json"),&json!({"elapsed_us":calls.started.elapsed().as_micros(),"target_us":input.target,"tolerance_us":input.tolerance,"observation":sample}))?;
            if !valid_sample(&sample, &opened) {
                return Err(calls.error(invalid(
                    "Original post-submit binding or native state differs",
                )));
            }
            if sample.playback.paused == Some(true)
                && landed(sample.playback.position_us, input.target, input.tolerance)
            {
                landing = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        if !landing {
            return Err(calls.error(invalid(
                "ACK observed but engine playback landing was not observed",
            )));
        }
        let consumed: v1::NativeExportResultV1 =
            calls.call("consume-once", EXPORT_CONSUME, payload.clone(), None)?;
        if !acknowledged(&consumed, &admitted).map_err(|error| calls.error(error))? {
            return Err(calls.error(invalid("Consumed original terminal result differs")));
        }
        let remaining: v1::NativeExportListV1 = calls.call(
            "after-consume-list",
            EXPORT_LIST,
            json!({"issuer_scope":scope,"origin":origin}),
            None,
        )?;
        if remaining.scope() != scope || !remaining.items().is_empty() {
            return Err(calls.error(invalid("Consumed original row remains listed")));
        }
        let second = request(EXPORT_CONSUME, payload);
        let response = fixture.try_call_until(
            "consume-again-not-found",
            &second,
            &endpoint,
            original,
            Duration::from_secs(5),
            i32::from(ApiError::new(ErrorCode::NotFound, "expected original absence").exit_code()),
        )?;
        if response["ok"] != false || response["error"]["code"] != "not_found" {
            return Err(calls.error(invalid("Second original consume did not preserve NotFound")));
        }
        Ok(())
    }));
    let action = kinekino_process::close_request(action);
    let stopped = host
        .finish(
            Duration::from_secs(10),
            &fixture.root.join("logs/media-stop.stdout"),
            &fixture.root.join("logs/media-host.stderr"),
        )
        .and_then(|terminal| {
            let failed = |error| {
                CliFailure::terminal(
                    "actual media EOF proof",
                    error,
                    &fixture.root,
                    terminal.receipt.clone(),
                )
            };
            if terminal.receipt.child_exit != Some(0) {
                return Err(failed(invalid("Original host child exit differs")));
            }
            let value: Value = serde_json::from_slice(&terminal.tail)
                .map_err(|error| failed(io::Error::other(error)))?;
            media_stop(&value, expected_engine.get()).map_err(failed)?;
            Ok(terminal.receipt)
        });
    #[cfg(feature = "validation-export-delivery-loss")]
    let evidence_error = if drop_delivery {
        verify_delivery_proof(&proof_path, recovered.borrow().as_ref()).err()
    } else {
        None
    };
    #[cfg(not(feature = "validation-export-delivery-loss"))]
    let evidence_error = None;
    ScenarioFailure::conclude(action, stopped, evidence_error)
}

fn submit_original_once(
    calls: &Calls<'_>,
    payload: Value,
    origin: &SkipExportOrigin,
    scope: v1::NativeIssuerScopeV1,
    target: DecimalU64,
    drop_delivery: bool,
) -> Result<Admission, Box<CliFailure>> {
    #[cfg(feature = "validation-export-delivery-loss")]
    if drop_delivery {
        let mut request = request(SKIP_EXPORT, payload);
        request.context.request_id = LOSS_REQUEST_ID.into();
        request.context.expected_revision = Some(origin.request_revision.0.to_string());
        let response = calls.fixture.try_call_until(
            "guarded-export-lost-response",
            &request,
            calls.endpoint,
            calls.original,
            Duration::from_secs(10),
            i32::from(
                ApiError::new(
                    ErrorCode::EngineFailed,
                    "expected response delivery failure",
                )
                .exit_code(),
            ),
        )?;
        if response["ok"] != false
            || response["error"]["details"]["delivery"] != "execution_unknown"
            || response["error"]["details"]["automatic_retry"] != false
        {
            return Err(calls.error(invalid(
                "Original endpoint delivery was not reported unknown",
            )));
        }
        // This input contains no reference and cannot read the private host proof.
        return RecoveryOrigin {
            scope,
            origin: origin.clone(),
        }
        .recover(calls, target);
    }
    let _ = drop_delivery;
    let result: SkipExportResult = calls.call(
        "guarded-export-once",
        SKIP_EXPORT,
        payload,
        Some(origin.request_revision),
    )?;
    admission(result, origin, scope, target).map_err(|error| calls.error(error))
}
#[cfg(feature = "validation-export-delivery-loss")]
const LOSS_REQUEST_ID: &str = "validation-export-original";
#[cfg(feature = "validation-export-delivery-loss")]
struct RecoveryOrigin {
    scope: v1::NativeIssuerScopeV1,
    origin: SkipExportOrigin,
}
#[cfg(feature = "validation-export-delivery-loss")]
impl RecoveryOrigin {
    fn recover(&self, calls: &Calls<'_>, target: DecimalU64) -> Result<Admission, Box<CliFailure>> {
        let listed: v1::NativeExportListV1 = calls.call(
            "recover-reference-from-list",
            EXPORT_LIST,
            json!({"issuer_scope":self.scope,"origin":self.origin}),
            None,
        )?;
        if listed.scope() != self.scope || listed.items().len() != 1 {
            return Err(calls.error(invalid(
                "Original reference recovery did not yield one original row",
            )));
        }
        let item = &listed.items()[0];
        validate_admission_identity(
            item.reference(),
            item.metadata(),
            &self.origin,
            self.scope,
            target,
        )
        .map_err(|error| calls.error(error))?;
        Ok(Admission {
            reference: item.reference(),
            metadata: item.metadata().clone(),
            post_error: None,
        })
    }
}
#[cfg(feature = "validation-export-delivery-loss")]
fn verify_delivery_proof(
    path: &std::path::Path,
    recovered: Option<&(v1::NativeResultRefV1, v1::NativeExportMetadataV1)>,
) -> io::Result<()> {
    use kinekino_player::infrastructure::local_host::validation::TRACE_BYTES;
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(TRACE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    decode_delivery_proof(&bytes, recovered)
}
#[cfg(feature = "validation-export-delivery-loss")]
fn decode_delivery_proof(
    bytes: &[u8],
    recovered: Option<&(v1::NativeResultRefV1, v1::NativeExportMetadataV1)>,
) -> io::Result<()> {
    use kinekino_player::infrastructure::local_host::validation::{
        CounterObservation, DeliveryEvent, DeliveryPhase, EVENT_BYTES, MAX_EVENTS, TRACE_BYTES,
    };
    let (reference, metadata) =
        recovered.ok_or_else(|| invalid("No original reference was recovered"))?;
    if bytes.len() > TRACE_BYTES || !bytes.ends_with(b"\n") {
        return Err(invalid("Validation proof is incomplete or over budget"));
    }
    if bytes
        .split(|byte| *byte == b'\n')
        .any(|line| line.len() >= EVENT_BYTES)
    {
        return Err(invalid("Validation event exceeds original frame budget"));
    }
    let events: Vec<DeliveryEvent> = bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(serde_json::from_slice)
        .collect::<Result<_, _>>()
        .map_err(io::Error::other)?;
    if events.len() > MAX_EVENTS
        || events
            .iter()
            .filter(|event| event.phase == DeliveryPhase::InjectedBeforeFrame)
            .count()
            != 1
    {
        return Err(invalid("Expected exactly one original endpoint fault"));
    }
    for event in &events {
        if event.request_id != LOSS_REQUEST_ID
            || event.command != SKIP_EXPORT
            || event.matched_dispatches != 1
            || event.matched_overflow
            || event.frame_bytes_sent != 0
            || event.reference != Some(*reference)
            || event.metadata.as_ref() != Some(metadata)
            || event.post_admission_error
            || event.before
                != (CounterObservation::Known {
                    attempts: 0,
                    enqueued: 0,
                })
            || event.after
                != (CounterObservation::Known {
                    attempts: 1,
                    enqueued: 1,
                })
        {
            return Err(invalid(
                "Original endpoint fault/count/identity proof differs",
            ));
        }
    }
    if !events
        .last()
        .is_some_and(|event| event.phase == DeliveryPhase::Final && event.injected)
    {
        return Err(invalid("Final original fault proof was not completed"));
    }
    Ok(())
}

#[cfg(feature = "validation-export-delivery-loss")]
#[test]
fn delivery_proof_requires_original_identity_known_counts_and_final_marker() {
    kinekino_platform::install();
    use kinekino_player::infrastructure::local_host::validation::TRACE_BYTES;
    let metadata = json!({"origin":{"registry_generation":"g","session":"s","pass":"1","segment":"op","request_revision":"3","engine_instance_id":"7","resolution_artifact_id":"a".repeat(64)},"annotation":{"annotation_revision":"1","verified_revision":"1","timebase":"engine_playback"},"actual_target_us":"10000"});
    let reference = "nsr1.abcdefabcdefabcdefabcdefabcdefab.0000000000000001";
    let recovered = (
        reference.parse().unwrap(),
        serde_json::from_value(metadata.clone()).unwrap(),
    );
    let first = json!({"phase":"injected_before_frame","request_id":LOSS_REQUEST_ID,"command":SKIP_EXPORT,"matched_dispatches":1,"matched_overflow":false,"injected":false,"frame_bytes_sent":0,"elapsed_us":1,"before":{"state":"known","attempts":0,"enqueued":0},"after":{"state":"known","attempts":1,"enqueued":1},"reference":reference,"metadata":metadata,"post_admission_error":false});
    let mut final_event = first.clone();
    final_event["phase"] = json!("final");
    final_event["injected"] = json!(true);
    let encode = |events: &[Value]| {
        let mut bytes = Vec::new();
        for event in events {
            bytes.extend(serde_json::to_vec(event).unwrap());
            bytes.push(b'\n');
        }
        bytes
    };
    let valid = encode(&[first.clone(), final_event.clone()]);
    assert!(decode_delivery_proof(&valid, Some(&recovered)).is_ok());
    assert!(decode_delivery_proof(&valid, None).is_err());
    assert!(decode_delivery_proof(&valid[..valid.len() - 1], Some(&recovered)).is_err());
    assert!(decode_delivery_proof(&vec![b'x'; TRACE_BYTES + 1], Some(&recovered)).is_err());
    assert!(
        decode_delivery_proof(&encode(std::slice::from_ref(&first)), Some(&recovered)).is_err()
    );
    for (field, changed) in [
        ("matched_dispatches", json!(2)),
        ("frame_bytes_sent", json!(1)),
        ("after", json!({"state":"unknown","code":"source_expired"})),
        ("metadata", json!(null)),
        (
            "reference",
            json!("nsr1.abcdefabcdefabcdefabcdefabcdefab.0000000000000002"),
        ),
    ] {
        let mut wrong = final_event.clone();
        wrong[field] = changed;
        assert!(
            decode_delivery_proof(&encode(&[first.clone(), wrong]), Some(&recovered)).is_err(),
            "{field}"
        );
    }
    assert!(
        decode_delivery_proof(
            &encode(&[first.clone(), first, final_event]),
            Some(&recovered)
        )
        .is_err()
    );
}

#[test]
fn ack_and_landing_are_separate_and_pending_never_means_success() {
    kinekino_platform::install();
    assert!(!landed(None, DecimalU64(10_000), 100));
    assert!(!landed(Some(DecimalU64(9_000)), DecimalU64(10_000), 100));
    assert!(landed(Some(DecimalU64(9_950)), DecimalU64(10_000), 100));
    let metadata:v1::NativeExportMetadataV1=serde_json::from_value(json!({"origin":{"registry_generation":"g","session":"s","pass":"1","segment":"op","request_revision":"3","engine_instance_id":"7","resolution_artifact_id":"a".repeat(64)},"annotation":{"annotation_revision":"1","verified_revision":"1","timebase":"engine_playback"},"actual_target_us":"10000"})).unwrap();
    let original = Admission {
        reference: "nsr1.abcdefabcdefabcdefabcdefabcdefab.0000000000000001"
            .parse()
            .unwrap(),
        metadata,
        post_error: None,
    };
    for state in [
        v1::NativeExportStateV1::Queued,
        v1::NativeExportStateV1::WriteCommitted,
        v1::NativeExportStateV1::WrittenAwaitingAck,
    ] {
        let result = v1::NativeExportResultV1 {
            item: v1::NativeExportItemV1::new(
                original.reference,
                original.metadata.clone(),
                state,
                v1::NativeExportReadModeV1::Standalone,
            )
            .unwrap(),
            write_committed: false,
            failure: None,
        };
        assert!(!acknowledged(&result, &original).unwrap());
    }
    let mut result = v1::NativeExportResultV1 {
        item: v1::NativeExportItemV1::new(
            original.reference,
            original.metadata.clone(),
            v1::NativeExportStateV1::CommandAccepted,
            v1::NativeExportReadModeV1::Standalone,
        )
        .unwrap(),
        write_committed: false,
        failure: None,
    };
    assert!(acknowledged(&result, &original).is_err());
    result.write_committed = true;
    assert!(acknowledged(&result, &original).unwrap());
    result.item = v1::NativeExportItemV1::new(
        original.reference,
        original.metadata.clone(),
        v1::NativeExportStateV1::OutcomeUnobserved,
        v1::NativeExportReadModeV1::Standalone,
    )
    .unwrap();
    assert!(acknowledged(&result, &original).is_err());
}

#[test]
fn missing_admission_never_invents_reference_or_permission_to_retry() {
    kinekino_platform::install();
    let origin:SkipExportOrigin=serde_json::from_value(json!({"session":{"skip_session_id":"s","binding":{"engine_instance_id":"7","resolution_artifact_id":"a".repeat(64)}},"pass_id":"1","request_revision":"3","segment_id":"op"})).unwrap();
    let result:SkipExportResult=serde_json::from_value(json!({"session_state":{"session":origin.session,"revision":"3","pass_id":"1"},"result":{"state":"cancelled","status":"before_admission"}})).unwrap();
    assert!(
        admission(
            result,
            &origin,
            "nsi1.abcdefabcdefabcdefabcdefabcdefab".parse().unwrap(),
            DecimalU64(10000)
        )
        .is_err()
    );
}

#[test]
fn media_stop_requires_original_engine_and_all_exit_and_transport_proofs() {
    kinekino_platform::install();
    let valid = json!({"event":"native_host_stopped","host_lifetime_ended":true,"receipt":{"engine_instance_id":"7","engine_exited":true,"ipc_worker_exited":true,"stderr_worker_exited":true,"stdout_worker_exited":true,"diagnostics":null},"local_host_shutdown":{"pipe":"joined","sdk":"joined","dispatch":"completed","dispatch_poisoned":false}});
    media_stop(&valid, Some(DecimalU64(7))).unwrap();
    assert!(media_stop(&valid, Some(DecimalU64(8))).is_err());
    assert!(media_stop(&valid, None).is_err());
    let mut missing = valid.clone();
    missing.as_object_mut().unwrap().remove("receipt");
    assert!(media_stop(&missing, Some(DecimalU64(7))).is_err());
    for field in [
        "engine_exited",
        "ipc_worker_exited",
        "stderr_worker_exited",
        "stdout_worker_exited",
    ] {
        let mut incomplete = valid.clone();
        incomplete["receipt"][field] = json!(false);
        assert!(media_stop(&incomplete, Some(DecimalU64(7))).is_err());
    }
    let mut poisoned = valid;
    poisoned["local_host_shutdown"]["dispatch_poisoned"] = json!(true);
    assert!(media_stop(&poisoned, Some(DecimalU64(7))).is_err());
}

#[test]
fn admission_preserves_original_payload_identity_and_rejects_changed_target_or_revision() {
    kinekino_platform::install();
    let origin:SkipExportOrigin=serde_json::from_value(json!({"session":{"skip_session_id":"s","binding":{"engine_instance_id":"7","resolution_artifact_id":"a".repeat(64)}},"pass_id":"1","request_revision":"3","segment_id":"op"})).unwrap();
    let metadata:v1::NativeExportMetadataV1=serde_json::from_value(json!({"origin":{"registry_generation":"g","session":"s","pass":"1","segment":"op","request_revision":"3","engine_instance_id":"7","resolution_artifact_id":"a".repeat(64)},"annotation":{"annotation_revision":"1","verified_revision":"1","timebase":"engine_playback"},"actual_target_us":"10000"})).unwrap();
    let reference = "nsr1.abcdefabcdefabcdefabcdefabcdefab.0000000000000001"
        .parse()
        .unwrap();
    let result = || SkipExportResult {
        session_state: SkipExportSessionState {
            session: origin.session.clone(),
            revision: DecimalU64(4),
            pass_id: DecimalU64(1),
        },
        result: SkipExportAction::Admitted {
            receipt: kinekino_player::application::playback_control::ControlReceipt::enqueued(7),
            reference,
            metadata: metadata.clone(),
        },
    };
    let scope = "nsi1.abcdefabcdefabcdefabcdefabcdefab".parse().unwrap();
    let saved = admission(result(), &origin, scope, DecimalU64(10000)).unwrap();
    assert_eq!(saved.reference, reference);
    assert_eq!(saved.metadata, metadata);
    assert!(admission(result(), &origin, scope, DecimalU64(10001)).is_err());
    let mut stale = origin.clone();
    stale.request_revision = DecimalU64(4);
    assert!(admission(result(), &stale, scope, DecimalU64(10000)).is_err());
    let mut foreign = origin.clone();
    foreign.session.binding.engine_instance_id = DecimalU64(8);
    assert!(admission(result(), &foreign, scope, DecimalU64(10000)).is_err());
}

#[test]
fn observation_command_matches_typed_client_and_requires_full_engine_dto() {
    kinekino_platform::install();
    use kinekino_player::application::api::{
        API_VERSION, ApplicationClient, Request, RequestContext, Response,
    };
    struct Client(std::sync::Mutex<Vec<Request>>);
    impl ApplicationClient for Client {
        fn execute(&self, request: Request) -> Response {
            let id = request.context.request_id.clone();
            self.0.lock().unwrap().push(request);
            Response {
                api_version: API_VERSION,
                request_id: id,
                ok: true,
                error: None,
                data: Some(
                    json!({"engine_instance_id":"7","resolution_artifact_id":"a".repeat(64),"playback":{"origin":"native_observation","published_age_ms":"0","connected":true,"paused":true,"idle":false,"position_us":"0","duration_us":"10000","volume_percent":0.0,"muted":false,"video_size":null,"audio_channels":null,"has_error":false},"diagnostics":null,"options":null}),
                ),
            }
        }
    }
    let client = Client(std::sync::Mutex::new(Vec::new()));
    let observed = client
        .observe_episode_playback(RequestContext::default())
        .unwrap();
    assert_eq!(observed.engine_instance_id, DecimalU64(7));
    assert_eq!(observed.resolution_artifact_id, "a".repeat(64));
    let calls = client.0.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].command, ENGINE_OBSERVE_COMMAND);
    assert_eq!(calls[0].payload, json!({}));
    assert!(
        serde_json::from_value::<EpisodePlaybackObservation>(
            serde_json::to_value(observed.playback).unwrap()
        )
        .is_err()
    );
}
