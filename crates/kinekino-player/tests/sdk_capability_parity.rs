#[path = "support/cli_exe.rs"]
mod cli_exe;
use kinekino_player::{
    application::api::{ApplicationClientHandle, PlayerSdk, Request},
    infrastructure::{frame_capture::FfmpegFrameCapture, subtitle_files::SubtitleFiles},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
};

fn invoke(executable: &Path, request: &Request) -> (i32, Value) {
    let mut child = Command::new(executable)
        .args(["--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(request).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.stderr.is_empty(), "unexpected CLI stderr");
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap().lines().count(),
        1
    );
    (
        output.status.code().unwrap(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}

fn statuses(data: &Value) -> BTreeMap<String, String> {
    let entries = data["commands"].as_array().unwrap();
    let map = entries
        .iter()
        .map(|entry| {
            (
                entry["command"].as_str().unwrap().to_owned(),
                entry["status"].as_str().unwrap().to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(map.len(), entries.len(), "duplicate capability command");
    map
}

fn record(folder: &Option<PathBuf>, name: &str, value: &Value) {
    if let Some(folder) = folder {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(folder.join(name))
            .unwrap();
        file.write_all(&serde_json::to_vec_pretty(value).unwrap())
            .unwrap();
    }
}

fn output_review_fixture(request: &mut Request) {
    request.payload = json!({"proposal_id":"parity-proposal"});
    if request.command == "proposals.record_output_decision" {
        let criteria = kinekino_player::application::proposals::output_review::criteria_for(
            kinekino_player::domain::proposals::ProposalTask::Correction,
        )
        .unwrap()
        .identity()
        .unwrap();
        request.context.expected_revision = Some("2".into());
        request.payload["decision"] = json!({
            "decision_id":"parity-decision",
            "target":{"schema_version":"1","kind":"accepted_output","proposal_id":"parity-proposal","task":"correction","draft_revision":"1","output_revision":"2","hash_format":"accepted_typed_output_v1","output_sha256":"1".repeat(64),"source_id":"parity-source","source_revision":"1","source_sha256":"2".repeat(64),"generated_raw_sha256":"3".repeat(64),"generated_raw_bytes":"1","historical":null},
            "criteria":criteria,"verdict":"deferred","note":"Synthetic unsupported-adapter fixture; no output has been reviewed",
            "declared_reviewer":{"label":"parity-fixture","reviewer_id":null},"claimed_review_unix_ms":null
        });
        let typed: kinekino_player::application::proposals::output_review::RecordOutputDecision =
            serde_json::from_value(request.payload.clone()).unwrap();
        typed.decision.target.validate_stored().unwrap();
    }
}

fn subtitle_export_fixture(request: &mut Request) {
    use kinekino_player::{
        application::proposals::subtitle_export::{COMMAND, ExportAcceptedSubtitles},
        domain::{proposals::output_review::OutputReviewTarget, subtitles::SubtitleFormat},
    };
    assert_eq!(request.command, COMMAND);
    // Structurally valid synthetic identity only; no store, acceptance or output is created.
    let target: OutputReviewTarget = serde_json::from_value(json!({
        "schema_version":"1","kind":"accepted_output","proposal_id":"parity-proposal",
        "task":"correction","draft_revision":"1","output_revision":"2",
        "hash_format":"accepted_typed_output_v1","output_sha256":"1".repeat(64),
        "source_id":"parity-source","source_revision":"1","source_sha256":"2".repeat(64),
        "generated_raw_sha256":"3".repeat(64),"generated_raw_bytes":"1","historical":null
    }))
    .unwrap();
    target.validate_stored().unwrap();
    let input = ExportAcceptedSubtitles {
        target,
        output_path: "D:/parity-unavailable.srt".into(),
        output_format: SubtitleFormat::Srt,
        allow_loss: false,
    };
    request.payload = serde_json::to_value(input).unwrap();
}

fn card_export_fixture(request: &mut Request) {
    use kinekino_player::{
        application::proposals::card_export::{COMMAND, ExportCardReviewPng, TemplateSelection},
        domain::{api_values::DecimalU64, proposals::output_review::OutputReviewTarget},
    };
    assert_eq!(request.command, COMMAND);
    let target: OutputReviewTarget = serde_json::from_value(json!({
        "schema_version":"1","kind":"accepted_output","proposal_id":"parity-card",
        "task":"cards","draft_revision":"1","output_revision":"2",
        "hash_format":"accepted_typed_output_v1","output_sha256":"1".repeat(64),
        "source_id":"parity-source","source_revision":"1","source_sha256":"2".repeat(64),
        "generated_raw_sha256":"3".repeat(64),"generated_raw_bytes":"1","historical":null
    }))
    .unwrap();
    target.validate_stored().unwrap();
    // Structural unavailable-adapter fixture; it neither reads a frame nor accepts output.
    request.payload = serde_json::to_value(ExportCardReviewPng {
        target,
        position: DecimalU64(0),
        frame_path: if cfg!(windows) {
            "D:/parity-frame.png".into()
        } else {
            "/kinekino-test/parity-frame.png".into()
        },
        frame_sha256: "4".repeat(64),
        template: TemplateSelection {
            id: "parity-unavailable".into(),
            version: "1".into(),
            declaration_sha256: "5".repeat(64),
        },
        output_path: if cfg!(windows) {
            "D:/parity-card-new.png".into()
        } else {
            "/kinekino-test/parity-card-new.png".into()
        },
        review_artifact_only: true,
    })
    .unwrap();
}

fn export_fixture(request: &mut Request) {
    use kinekino_player::{
        application::{
            playback_session::v1,
            skip_segments::{SkipBinding, host::*},
        },
        domain::{api_values::DecimalU64, skip_segments::Intent},
    };
    // Canonical lookup values only: parsing issues no live issuer/control authority.
    // The real unavailable adapter must reject these well-formed DTOs as Unsupported.
    let issuer_scope: v1::NativeIssuerScopeV1 =
        "nsi1.abcdefabcdefabcdefabcdefabcdefab".parse().unwrap();
    let origin = SkipExportOrigin {
        session: SessionRef {
            skip_session_id: "parity-session".into(),
            binding: SkipBinding {
                engine_instance_id: DecimalU64(1),
                resolution_artifact_id: "a".repeat(64),
            },
        },
        pass_id: DecimalU64(1),
        request_revision: DecimalU64(2),
        segment_id: "parity-segment".into(),
    };
    request.payload = match request.command.as_str() {
        SKIP_EXPORT => {
            request.context.expected_revision = Some("2".into());
            serde_json::to_value(SkipExportRequest {
                issuer_scope,
                session: origin.session,
                input: SkipInput {
                    segment_id: origin.segment_id,
                    intent: Intent::Manual,
                    cancelled: false,
                },
            })
            .unwrap()
        }
        EXPORT_LIST => serde_json::to_value(ListSkipExports {
            issuer_scope,
            origin,
        })
        .unwrap(),
        EXPORT_PEEK | EXPORT_CONSUME => serde_json::to_value(ReadSkipExport {
            issuer_scope,
            reference: "nsr1.abcdefabcdefabcdefabcdefabcdefab.0000000000000001"
                .parse()
                .unwrap(),
            origin,
        })
        .unwrap(),
        _ => panic!("Fixture requires an existing export command"),
    };
}

#[test]
fn actual_cli_capabilities_schemas_and_unsupported_dispatch_match_injected_client() {
    kinekino_platform::install();
    let folder = std::env::var_os("SDK_CAPABILITY_EVIDENCE_DIR").map(PathBuf::from);
    if let Some(folder) = &folder {
        assert!(folder.is_absolute());
        fs::create_dir(folder).unwrap();
    }
    let built_executable = Path::new(crate::cli_exe::kinekino_cli_exe());
    let executable = if let Some(folder) = &folder {
        let pinned = folder.join(built_executable.file_name().unwrap());
        fs::copy(built_executable, &pinned).unwrap();
        assert_eq!(
            fs::read(built_executable).unwrap(),
            fs::read(&pinned).unwrap()
        );
        pinned
    } else {
        built_executable.to_owned()
    };
    // CLI-equivalent adapters; desktop additionally injects its live status reader.
    let client = PlayerSdk::new(SubtitleFiles).with_frame_capture(FfmpegFrameCapture::discover());
    let client = client
        .with_snapshot_export(kinekino_player::infrastructure::snapshot_export::PngSnapshotExport);
    // The CLI runner wires local speaker documents on Windows only (kinekino-host runner).
    #[cfg(windows)]
    let client = client.with_speaker_documents(
        kinekino_player::infrastructure::speaker_annotation_files::LocalSpeakerFiles,
    );
    let client: ApplicationClientHandle = Arc::new(client);
    let mut discovery = Request::new("capabilities");
    discovery.context.request_id = "capability-통합".into();
    let sdk = serde_json::to_value(client.execute(discovery.clone())).unwrap();
    let (code, cli) = invoke(&executable, &discovery);
    record(
        &folder,
        "discovery.json",
        &json!({"request":discovery,"sdk":sdk,"cli":cli,"exit_code":code}),
    );
    assert_eq!(code, 0);
    assert_eq!(cli["request_id"], "capability-통합");
    let expected = statuses(&sdk["data"]);
    assert_eq!(statuses(&cli["data"]), expected);
    let unavailable = PlayerSdk::default().execute(Request::new("capabilities"));
    let unavailable = statuses(&unavailable.data.unwrap());
    assert_eq!(
        unavailable
            .values()
            .filter(|status| *status == "supported")
            .count(),
        3
    );
    assert_eq!(expected["subtitles.convert"], "supported");
    let mut rows = Vec::new();
    for (index, (command, status)) in expected.iter().enumerate() {
        let mut schema_request = Request::new("schema");
        schema_request.context.request_id = format!("schema-{index}-요청");
        schema_request.payload = json!({"command":command});
        let sdk_schema = serde_json::to_value(client.execute(schema_request.clone())).unwrap();
        let (code, cli_schema) = invoke(&executable, &schema_request);
        record(
            &folder,
            &format!("schema-{index:03}.json"),
            &json!({"request":schema_request,"sdk":sdk_schema,"cli":cli_schema,"exit_code":code}),
        );
        assert_eq!(code, 0, "schema failed: {command}");
        assert_eq!(cli_schema, sdk_schema, "schema parity: {command}");
        assert_eq!(cli_schema["data"]["status"], *status);
        assert_eq!(cli_schema["data"]["command"], *command);
        let mut row = json!({"command":command,"status":status,"schema_equal":true});
        if status == "unsupported" {
            let mut request = Request::new(command);
            request.context.request_id = format!("unsupported-{index}-요청");
            let export =
                kinekino_player::application::skip_segments::host::is_export_command(command);
            let subtitle_export =
                command == kinekino_player::application::proposals::subtitle_export::COMMAND;
            let card_export =
                command == kinekino_player::application::proposals::card_export::COMMAND;
            // Strict-shape-first commands (unsupported only where the host lacks their backend, e.g. Linux).
            let shape_first = matches!(
                command.as_str(),
                "asset_artifacts.inspect"
                    | "asset_projects.create"
                    | "asset_projects.inspect"
                    | "asset_projects.save_revision"
                    | "clip_projects.create"
                    | "clip_projects.inspect"
                    | "clip_projects.save_revision"
                    | "clip_sources.inspect"
                    | "card_projects.create"
                    | "card_projects.inspect"
                    | "card_projects.save_revision"
                    | "reviews.media.prepare"
                    | "reviews.media.inspect"
                    | "production.export_snapshot"
            );
            if export
                || subtitle_export
                || card_export
                || shape_first
                || matches!(
                    command.as_str(),
                    "proposals.inspect_output_review_target"
                        | "proposals.record_output_decision"
                        | "proposals.output_decisions"
                )
            {
                let malformed_sdk = serde_json::to_value(client.execute(request.clone())).unwrap();
                let (exit, malformed_cli) = invoke(&executable, &request);
                record(
                    &folder,
                    &format!("malformed-{index:03}.json"),
                    &json!({"request":request,"sdk":malformed_sdk,"cli":malformed_cli,"exit_code":exit}),
                );
                assert_eq!(exit, 2, "malformed typed payload exit: {command}");
                assert_eq!(
                    malformed_cli, malformed_sdk,
                    "malformed typed payload parity: {command}"
                );
                assert_eq!(malformed_cli["error"]["code"], "invalid_argument");
                assert_eq!(malformed_cli["ok"], false);
                assert!(malformed_cli["data"].is_null());
                if export {
                    export_fixture(&mut request);
                } else if subtitle_export {
                    subtitle_export_fixture(&mut request);
                } else if card_export {
                    card_export_fixture(&mut request);
                } else if !shape_first {
                    output_review_fixture(&mut request);
                }
            }
            if shape_first {
                // No valid typed fixture exists for these yet; malformed parity above is the checked contract.
                row["unsupported_dispatch_equal"] = json!("shape_first_without_fixture");
                rows.push(row);
                continue;
            }
            let sdk_response = serde_json::to_value(client.execute(request.clone())).unwrap();
            let (exit, cli_response) = invoke(&executable, &request);
            record(
                &folder,
                &format!("unsupported-{index:03}.json"),
                &json!({"request":request,"sdk":sdk_response,"cli":cli_response,"exit_code":exit}),
            );
            assert_eq!(exit, 3, "unsupported exit: {command}");
            assert_eq!(cli_response, sdk_response, "unsupported parity: {command}");
            assert_eq!(cli_response["error"]["code"], "unsupported");
            assert_eq!(cli_response["ok"], false);
            assert!(cli_response["data"].is_null());
            row["unsupported_dispatch_equal"] = json!(true);
        }
        rows.push(row);
    }
    record(
        &folder,
        "summary.json",
        &json!({"cli_executable":executable,"build_output_path":built_executable,"binary_pinned_before_requests":folder.is_some(),"commands":rows,"scope":"actual CLI subprocess vs identically configured object-safe SDK without playback status provider; desktop adds a native status reader; no GUI interaction or supported media action execution; no model inference; dependency availability is host-specific; not a full compiled-source attestation"}),
    );
}

#[test]
fn desktop_status_provider_changes_only_playback_status_capability() {
    kinekino_platform::install();
    use kinekino_player::application::playback_status::native_status_channel;
    let (_publisher, reader) = native_status_channel();
    let cli = PlayerSdk::new(SubtitleFiles).with_frame_capture(FfmpegFrameCapture::discover());
    let desktop = PlayerSdk::new(SubtitleFiles)
        .with_frame_capture(FfmpegFrameCapture::discover())
        .with_playback_status(reader);
    let cli = statuses(&cli.execute(Request::new("capabilities")).data.unwrap());
    let desktop = statuses(&desktop.execute(Request::new("capabilities")).data.unwrap());
    assert_eq!(cli.len(), desktop.len());
    let differences: Vec<_> = cli
        .iter()
        .filter(|(name, status)| desktop.get(*name) != Some(*status))
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(differences, ["playback.status"]);
    assert_eq!(cli["playback.status"], "unsupported");
    assert_eq!(desktop["playback.status"], "supported");
}
