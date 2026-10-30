//! Consumer contracts plus an ignored, separately allocated NoMedia CLI lifecycle scenario.
#[cfg(windows)]
#[path = "native_skip_cli/actual_export.rs"]
mod actual_export;
#[cfg(windows)]
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[cfg(windows)]
#[path = "jobs/fixture.rs"]
#[expect(
    dead_code,
    reason = "Original shared CLI fixture also exposes helpers used by other targets"
)]
mod fixture;
#[cfg(windows)]
#[path = "support/fixture_plugin.rs"]
mod fixture_plugin;
#[cfg(windows)]
#[path = "jobs/kinekino_process.rs"]
#[expect(
    dead_code,
    reason = "Original owned-process harness also exposes helpers used by other targets"
)]
mod kinekino_process;
#[cfg(windows)]
#[path = "native_skip_cli/ready_observation.rs"]
mod ready_observation;
use kinekino_player::application::{
    api::{
        ApiError, ApplicationClient, ErrorCode, Request, RequestContext, Response,
        SkipSegmentsClient,
    },
    playback_control::ControlReceipt,
    playback_session::{ControlEpisodePlayback, v1},
    skip_segments::{SkipObservation, SkipPlaybackPort, host::*},
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

struct NeverPlayback;
impl SkipPlaybackPort for NeverPlayback {
    fn observe(&self) -> Result<SkipObservation, ApiError> {
        panic!("No observation for closed export integration")
    }
    fn submit(&self, _: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        panic!("No native submission")
    }
}
fn host() -> SkipHostRegistry {
    SkipHostRegistry::new(
        "consumer-contract".into(),
        Arc::new(NeverPlayback),
        2,
        false,
    )
    .unwrap()
}
fn origin() -> Value {
    json!({"session":{"skip_session_id":"skip-original-1","binding":{"engine_instance_id":"7","resolution_artifact_id":"a".repeat(64)}},"pass_id":"1","request_revision":"3","segment_id":"opening"})
}
fn request(command: &str) -> Request {
    let mut request = Request::new(command);
    request.payload =
        json!({"issuer_scope":"nsi1.abcdefabcdefabcdefabcdefabcdefab","origin":origin()});
    if command != EXPORT_LIST {
        request.payload["reference"] =
            json!("nsr1.abcdefabcdefabcdefabcdefabcdefab.0000000000000001");
    }
    request
}
#[test]
fn strict_codec_and_shape_precede_context_and_unsupported_without_native_work() {
    kinekino_platform::install();
    let host = host();
    for scope in [
        "nsi1.00000000000000000000000000000000",
        "nsi1.ABCDEFABCDEFABCDEFABCDEFABCDEFAB",
        "nsi2.abcdefabcdefabcdefabcdefabcdefab",
    ] {
        let mut request = request(EXPORT_LIST);
        request.context.expected_revision = Some("1".into());
        request.payload["issuer_scope"] = json!(scope);
        assert_eq!(
            host.execute(&request).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    let mut request = request(EXPORT_PEEK);
    request.payload["target_us"] = json!("1");
    assert_eq!(
        host.execute(&request).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert!(!host.export_capability().available);
}
#[test]
fn valid_read_context_is_checked_before_closed_integration_and_never_mutates_skip() {
    kinekino_platform::install();
    let host = host();
    for command in [EXPORT_LIST, EXPORT_PEEK, EXPORT_CONSUME] {
        let mut request = request(command);
        assert_eq!(
            host.execute(&request).unwrap_err().code,
            ErrorCode::Unsupported
        );
        request.context.expected_revision = Some("1".into());
        let error = host.execute(&request).unwrap_err();
        assert!(error.message.contains("expected_revision"));
        request.context.expected_revision = None;
        request.context.timeout_ms = Some(10);
        assert!(
            host.execute(&request)
                .unwrap_err()
                .message
                .contains("timeout")
        );
    }
}
#[test]
fn submit_requires_revision_and_preserves_cancel_shape_without_exposing_target_authority() {
    kinekino_platform::install();
    let host = host();
    let mut request = Request::new(SKIP_EXPORT);
    request.payload = json!({"issuer_scope":"nsi1.abcdefabcdefabcdefabcdefabcdefab","session":origin()["session"],"input":{"segment_id":"opening","intent":"manual","cancelled":true}});
    assert_eq!(
        host.execute(&request).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    request.context.expected_revision = Some("3".into());
    assert_eq!(
        host.execute(&request).unwrap_err().code,
        ErrorCode::Unsupported
    );
    request.payload["input"]["actual_target_us"] = json!("100");
    assert_eq!(
        host.execute(&request).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
}
#[test]
fn all_new_schemas_remain_unsupported_and_reads_have_null_revision() {
    kinekino_platform::install();
    for command in EXPORT_COMMANDS {
        let schema = schema::describe(
            &json!({"command":command}),
            SkipHostCapability {
                sessions: true,
                seek_submission: true,
            },
        )
        .unwrap();
        assert_eq!(schema["status"], "unsupported");
        let payload = &schema["request"]["properties"]["payload"];
        assert_eq!(payload["additionalProperties"], false);
        assert_eq!(payload["properties"]["issuer_scope"]["maxLength"], 37);
        if *command != SKIP_EXPORT {
            assert_eq!(
                schema["request"]["properties"]["context"]["properties"]["expected_revision"]["type"],
                "null"
            );
        }
    }
    assert_eq!(COMMANDS.len(), 9); // Global/legacy registration has not been expanded here.
}
struct RecordingClient(Mutex<Vec<Request>>);
impl ApplicationClient for RecordingClient {
    fn execute(&self, request: Request) -> Response {
        let id = request.context.request_id.clone();
        self.0.lock().unwrap().push(request);
        Response::failure(
            id,
            ApiError::new(ErrorCode::Unsupported, "original unsupported"),
        )
    }
}
#[test]
fn typed_client_preserves_commands_origin_and_context_without_fallback_or_retry() {
    kinekino_platform::install();
    let client = RecordingClient(Mutex::new(Vec::new()));
    let context = RequestContext {
        request_id: "original-request".into(),
        ..Default::default()
    };
    let list: ListSkipExports = serde_json::from_value(request(EXPORT_LIST).payload).unwrap();
    let read: ReadSkipExport = serde_json::from_value(request(EXPORT_PEEK).payload).unwrap();
    assert_eq!(
        client
            .list_skip_exports(context.clone(), list)
            .unwrap_err()
            .message,
        "original unsupported"
    );
    assert!(
        client
            .peek_skip_export(context.clone(), read.clone())
            .is_err()
    );
    assert!(client.consume_skip_export(context, read).is_err());
    let calls = client.0.lock().unwrap();
    assert_eq!(
        calls
            .iter()
            .map(|call| call.command.as_str())
            .collect::<Vec<_>>(),
        [EXPORT_LIST, EXPORT_PEEK, EXPORT_CONSUME]
    );
    assert!(
        calls
            .iter()
            .all(|call| call.context.request_id == "original-request"
                && call.context.expected_revision.is_none())
    );
    assert!(calls.iter().all(|call| call.payload["origin"] == origin()));
}
#[test]
fn bounded_error_projection_omits_unbounded_details_and_full_snapshot() {
    kinekino_platform::install();
    let error = ApiError::new(ErrorCode::EngineFailed, "한\\\"\n".repeat(4000));
    let projected = v1::NativeExportCoreErrorV1::from_api(&error);
    let result = SkipExportResult {
        session_state: SkipExportSessionState {
            session: serde_json::from_value(origin()["session"].clone()).unwrap(),
            revision: kinekino_player::domain::api_values::DecimalU64(3),
            pass_id: kinekino_player::domain::api_values::DecimalU64(1),
        },
        result: SkipExportAction::NoSubmissionReturned {
            error: projected,
            origin: serde_json::from_value(origin()).unwrap(),
        },
    };
    let bytes = v1::encode_native_export_v1(&result).unwrap();
    assert!(bytes.len() <= v1::NATIVE_EXPORT_JSON_BYTES);
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["result"]["error"]["truncated"], true);
    assert_eq!(value["result"]["error"]["details_omitted"], true);
    assert!(value.get("snapshot").is_none());
    assert!(value["session_state"].get("segments").is_none());
}

#[cfg(windows)]
#[test]
#[ignore = "Separate central NoMedia CLI allocation: original OwnedHost/Deadline/EOF; no playback.open or browser"]
fn actual_no_media_standalone_export_host_uses_original_issuer_and_owned_eof()
-> Result<(), crate::kinekino_process::ScenarioFailure> {
    kinekino_platform::install();
    use crate::kinekino_process::{
        CliFailure, OwnedHost, ScenarioFailure, close_request, phase_deadline,
    };
    use ::kinekino_process::domain::deadline::Deadline;
    use sha2::{Digest, Sha256};
    use std::{
        io,
        panic::{AssertUnwindSafe, catch_unwind},
        time::{Duration, Instant},
    };
    let mut fixture = fixture::Fixture::named("native-export-no-media");
    fixture.keep_evidence();
    let catalog = fixture.root.join("catalog");
    std::fs::create_dir(&catalog).unwrap();
    let executable = fixture.root.join("mpv.exe");
    let bytes = b"Verified identity fixture; never execute or open media";
    std::fs::write(&executable, bytes).unwrap();
    let hash = format!("{:x}", Sha256::digest(bytes));
    let original = Deadline::new(
        Instant::now(),
        Duration::from_secs(30),
        Duration::from_secs(5),
    )
    .unwrap();
    let ready = phase_deadline(original, Duration::from_secs(10)).unwrap();
    let host = OwnedHost::launch(
        fixture.cli.clone(),
        vec![
            "--serve-native".into(),
            "--catalog-root".into(),
            catalog.into_os_string(),
            "--mpv-executable".into(),
            executable.into_os_string(),
            "--mpv-sha256".into(),
            hash.clone().into(),
        ],
        &fixture.root,
        original,
    )
    .map_err(ScenarioFailure::host)?;
    let (host, endpoint) = host
        .ready(
            ready,
            &fixture.root.join("logs/export-ready.stdout"),
            |bytes| {
                let value: Value = serde_json::from_slice(bytes).map_err(io::Error::other)?;
                if value["event"] != "native_host_ready"
                    || value["engine_sha256"] != hash
                    || value["playback_verified"] != false
                {
                    return Err(io::Error::other(
                        "Original native host readiness identity mismatch",
                    ));
                }
                value["endpoint"]
                    .as_str()
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
                    .ok_or_else(|| io::Error::other("Missing original endpoint"))
            },
        )
        .map_err(ScenarioFailure::host)?;
    let action = catch_unwind(AssertUnwindSafe(|| {
        let capabilities = fixture.try_call_until(
            "export-capabilities",
            &fixture::request("capabilities", json!({})),
            &endpoint,
            original,
            Duration::from_secs(5),
            0,
        )?;
        assert_eq!(capabilities["data"]["skip_exports"]["available"], true);
        assert_eq!(
            capabilities["data"]["skip_sessions"]["seek_submission_available"],
            false
        );
        let scope = capabilities["data"]["skip_exports"]["issuer_scope"]
            .as_str()
            .unwrap();
        assert!(scope.parse::<v1::NativeIssuerScopeV1>().is_ok());
        let schema = fixture.try_call_until(
            "export-schema",
            &fixture::request("schema", json!({"command":EXPORT_LIST})),
            &endpoint,
            original,
            Duration::from_secs(5),
            0,
        )?;
        assert_eq!(schema["data"]["status"], "supported");
        let list = fixture.try_call_until(
            "export-empty-list",
            &fixture::request(EXPORT_LIST, json!({"issuer_scope":scope,"origin":origin()})),
            &endpoint,
            original,
            Duration::from_secs(5),
            0,
        )?;
        assert_eq!(list["ok"], true);
        assert!(list["data"]["items"].as_array().unwrap().is_empty());
        Ok(())
    }));
    let action = close_request(action);
    let stopped = host
        .finish(
            Duration::from_secs(10),
            &fixture.root.join("logs/export-stop.stdout"),
            &fixture.root.join("logs/export-host.stderr"),
        )
        .and_then(|terminal| {
            let failed = |error| {
                CliFailure::terminal(
                    "native export EOF proof",
                    error,
                    &fixture.root,
                    terminal.receipt.clone(),
                )
            };
            let value: Value = serde_json::from_slice(&terminal.tail)
                .map_err(|error| failed(io::Error::other(error)))?;
            if terminal.receipt.child_exit != Some(0) {
                return Err(failed(io::Error::other(
                    "Original no-media native child exit was not successful",
                )));
            }
            validate_no_media_stop(&value).map_err(failed)?;
            Ok(terminal.receipt)
        });
    ScenarioFailure::conclude(action, stopped, None)
}

fn validate_no_media_stop(value: &Value) -> Result<(), std::io::Error> {
    let shutdown = &value["local_host_shutdown"];
    if value["event"] != "native_host_stopped"
        || value["host_lifetime_ended"] != true
        || value.get("receipt") != Some(&Value::Null)
        || shutdown["pipe"] != "joined"
        || shutdown["sdk"] != "joined"
        || shutdown.get("dispatch_poisoned") != Some(&Value::Bool(false))
        || !matches!(
            shutdown.get("dispatch").and_then(Value::as_str),
            Some("idle" | "completed")
        )
    {
        return Err(std::io::Error::other(
            "Original no-media native stop receipt missing or incomplete",
        ));
    }
    Ok(())
}

#[test]
fn no_media_stop_requires_explicit_null_receipt_and_complete_unpoisoned_dispatch() {
    kinekino_platform::install();
    let complete = json!({"event":"native_host_stopped","host_lifetime_ended":true,"receipt":null,"local_host_shutdown":{"pipe":"joined","sdk":"joined","dispatch":"idle","dispatch_poisoned":false}});
    validate_no_media_stop(&complete).unwrap();
    let mut completed = complete.clone();
    completed["local_host_shutdown"]["dispatch"] = json!("completed");
    validate_no_media_stop(&completed).unwrap();
    for key in [
        "receipt",
        "event",
        "host_lifetime_ended",
        "local_host_shutdown",
    ] {
        let mut missing = complete.clone();
        missing.as_object_mut().unwrap().remove(key);
        assert!(
            validate_no_media_stop(&missing).is_err(),
            "accepted missing {key}"
        );
    }
    for key in ["pipe", "sdk", "dispatch", "dispatch_poisoned"] {
        let mut missing = complete.clone();
        missing["local_host_shutdown"]
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(
            validate_no_media_stop(&missing).is_err(),
            "accepted missing shutdown {key}"
        );
    }
    for dispatch in [
        Value::Null,
        json!("accepted"),
        json!("running"),
        json!("unknown"),
    ] {
        let mut pending = complete.clone();
        pending["local_host_shutdown"]["dispatch"] = dispatch;
        assert!(validate_no_media_stop(&pending).is_err());
    }
    let mut poisoned = complete.clone();
    poisoned["local_host_shutdown"]["dispatch_poisoned"] = json!(true);
    assert!(validate_no_media_stop(&poisoned).is_err());
    let mut active = complete.clone();
    active["receipt"] = json!({"engine_exited":true});
    assert!(validate_no_media_stop(&active).is_err());
    for worker in ["pipe", "sdk"] {
        for state in ["pending", "panicked"] {
            let mut incomplete = complete.clone();
            incomplete["local_host_shutdown"][worker] = json!(state);
            assert!(validate_no_media_stop(&incomplete).is_err());
        }
    }
}
