//! Actual normal CLI and typed transport through the original pinned host/EOF harness.
use super::{
    fixture::{Fixture, request},
    kinekino_process::{
        CliFailure, OwnedHost, ScenarioFailure, close_request, phase_deadline, transport_at,
    },
    provider::try_terminal_until,
};
use kinekino_player::application::api::AssetClient;
use kinekino_player::{
    application::{
        api::RequestContext,
        asset_bundle::{AssetBundleRequest, MissingAssetFramePolicy, SavedAssetProjectPin},
        asset_projects::InspectAssetProject,
        jobs::{JobId, JobState},
    },
    domain::{api_values::DecimalU64, asset_generation::*},
    infrastructure::{
        asset_build_jobs::AssetBuildHostConfig,
        asset_inputs::{FixedAssetImage, FixedAssetMask},
    },
};
use kinekino_process::domain::{deadline::Deadline, observation::FamilyObservation};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs, io,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    time::{Duration, Instant},
};
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[test]
#[ignore = "Pinned normal CLI, existing static PNG request/pair, fresh evidence and original host/transport/Jobs shutdown; no model/GUI quality approval"]
fn configured_asset_cli_and_typed_sdk_share_original_jobs_and_preserve_pins()
-> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    let mut fixture = Fixture::named("asset-build");
    fixture.keep_evidence();
    let bytes = fs::read(PathBuf::from(
        std::env::var_os("ASSET_PROJECT_REQUEST").unwrap(),
    ))
    .unwrap();
    assert_eq!(
        hash(&bytes),
        std::env::var("ASSET_PROJECT_REQUEST_SHA256").unwrap()
    );
    let frame: AssetFrameRequest = serde_json::from_slice(&bytes).unwrap();
    let mut document = AssetDocument {
        schema_version: "asset-project/1".into(),
        document_id: "cli-assets".into(),
        revision: DecimalU64(1),
        sources: BTreeMap::from([(
            "frame".into(),
            AssetFrameSlot {
                position: AssetFramePosition::StaticPng,
                selected_image: Some(frame.image.clone()),
            },
        )]),
        order_basis: AssetOrderBasis::SingleFrame,
        frames: vec![AssetFrameEntry::Available {
            frame_id: "frame".into(),
            input: Box::new(AssetFrameInput {
                image: frame.image.clone(),
                mask: frame.mask.clone(),
                review: frame.review.clone(),
            }),
        }],
        layers: vec![AssetLayer {
            layer_id: "foreground".into(),
            kind: LayerKind::Foreground,
            visible: true,
            locked: false,
            transform: frame.transform,
        }],
    };
    let mut back = document.layers[0].clone();
    back.layer_id = "background".into();
    back.kind = LayerKind::Background;
    back.transform.alpha_mode = AlphaMode::Premultiplied;
    document.layers.push(back);
    let projects = fixture.root.join("projects");
    let artifacts = fixture.root.join("artifacts");
    fs::create_dir(&projects).unwrap();
    fs::create_dir(&artifacts).unwrap();
    let producer = hash(&fs::read(&fixture.cli).unwrap());
    let config = json!({"schema_version":"asset-build-host/1","build":AssetBuildHostConfig{producer_sha256:producer.clone(),project_root:projects,artifact_root:artifacts,images:BTreeMap::from([(frame.image.blob.artifact_id.clone(),FixedAssetImage{pin:frame.image.clone(),path:PathBuf::from(std::env::var_os("ASSET_INPUT_IMAGE").unwrap())})]),masks:BTreeMap::from([(frame.mask.blob.artifact_id.clone(),FixedAssetMask{pin:frame.mask.clone(),path:PathBuf::from(std::env::var_os("ASSET_INPUT_MASK").unwrap())})])}});
    let bytes = serde_json::to_vec_pretty(&config).unwrap();
    let pin = hash(&bytes);
    let path = fixture.root.join("asset-host.json");
    fs::write(&path, bytes).unwrap();
    let original = Deadline::new(
        Instant::now(),
        Duration::from_secs(240),
        Duration::from_secs(15),
    )
    .unwrap();
    let args: Vec<OsString> = vec![
        "--serve-jobs".into(),
        "--job-root".into(),
        fixture.root.join("store").into_os_string(),
        "--asset-build-config".into(),
        path.clone().into_os_string(),
        "--asset-build-config-sha256".into(),
        pin.clone().into(),
    ];
    let cutoff = phase_deadline(original, Duration::from_secs(15)).unwrap();
    let (host, endpoint) = OwnedHost::launch(fixture.cli.clone(), args, &fixture.root, original)
        .and_then(|host| {
            host.ready(cutoff, &fixture.root.join("logs/ready.stdout"), |bytes| {
                let ready: Value = serde_json::from_slice(bytes)?;
                let ops: BTreeSet<_> = ready["execution_operations"]
                    .as_array()
                    .ok_or_else(|| io::Error::other("Missing asset operations"))?
                    .iter()
                    .filter_map(Value::as_str)
                    .collect();
                if ready["event"] != "job_host_ready"
                    || ready["asset_build_configured"] != true
                    || ready["asset_build_config_sha256"] != pin
                    || ops
                        != BTreeSet::from([
                            "subtitles.convert",
                            "subtitles.export_range",
                            "subtitles.from_transcript",
                            "production.build_assets",
                        ])
                {
                    return Err(io::Error::other("Pinned asset host ready differs"));
                }
                ready["endpoint"]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| io::Error::other("Missing asset endpoint"))
            })
        })
        .map_err(ScenarioFailure::host)?;
    let action = close_request(catch_unwind(AssertUnwindSafe(
        || -> Result<(), Box<CliFailure>> {
            let call = |label: &str, input: &kinekino_player::application::api::Request, exit| {
                fixture.try_call_until(
                    label,
                    input,
                    &endpoint,
                    original,
                    Duration::from_secs(15),
                    exit,
                )
            };
            assert!(
                fs::OpenOptions::new().write(true).open(&path).is_err(),
                "Pinned host document must remain guarded"
            );
            let caps = call("capabilities", &request("capabilities", json!({})), 0)?;
            assert_eq!(caps["data"]["asset_projects"]["backend_available"], true);
            let direct = caps["data"]["commands"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["command"] == "production.build_assets")
                .unwrap();
            assert_eq!(direct["status"], "unsupported");
            let schema = call(
                "job-schema",
                &request("schema", json!({"command":"jobs.submit"})),
                0,
            )?;
            let variant = schema["data"]["request"]["properties"]["payload"]["oneOf"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["properties"]["operation"]["const"] == "production.build_assets")
                .unwrap();
            assert_eq!(
                variant["properties"]["parameters"]["additionalProperties"],
                false
            );
            assert_eq!(
                variant["properties"]["parameters"]["properties"]["review_artifact_only"]["const"],
                true
            );
            let created = call(
                "create-project",
                &request("asset_projects.create", json!({"document":document})),
                0,
            )?;
            let input = AssetBundleRequest {
                project: SavedAssetProjectPin {
                    document_id: "cli-assets".into(),
                    publication_revision: DecimalU64(1),
                    publication_sha256: created["data"]["publication_sha256"]
                        .as_str()
                        .unwrap()
                        .into(),
                },
                missing_frames: MissingAssetFramePolicy::Reject,
                review_artifact_only: true,
            };
            document.revision.0 = 2;
            document.layers[0].transform.pivot.x = 0;
            let mut save = request("asset_projects.save_revision", json!({"document":document}));
            save.context.expected_revision = Some("1".into());
            let saved = call("save-project", &save, 0)?;
            assert_eq!(saved["data"]["publication"]["publication_revision"], "2");
            let old = call(
                "inspect-old-project",
                &request(
                    "asset_projects.inspect",
                    json!({"document_id":"cli-assets","publication_revision":"1"}),
                ),
                0,
            )?;
            assert_eq!(old["data"]["publication"], created["data"]["publication"]);
            let typed = transport_at(
                &endpoint,
                phase_deadline(original, Duration::from_secs(15)).unwrap(),
            )
            .unwrap();
            assert_eq!(
                serde_json::to_value(
                    typed
                        .inspect_asset_project(
                            RequestContext::default(),
                            InspectAssetProject {
                                document_id: "cli-assets".into(),
                                publication_revision: Some(DecimalU64(1))
                            }
                        )
                        .unwrap()
                )
                .unwrap(),
                old["data"]
            );
            let mut submit = request(
                "asset_sources.export",
                json!({"archive_id":"cli-originals","project":input.project,"missing_frames":"reject"}),
            );
            let source_schema = call(
                "source-export-schema",
                &request("schema", json!({"command":"asset_sources.export"})),
                0,
            )?;
            assert_eq!(
                source_schema["data"]["constraints"]["render_units_required"],
                false
            );
            assert_eq!(
                source_schema["data"]["request"]["properties"]["payload"]["additionalProperties"],
                false
            );
            let exported = call("export-original-sources", &submit, 0)?;
            assert_eq!(
                call("deny-source-overwrite", &submit, 5)?["error"]["code"],
                "revision_conflict"
            );
            assert_eq!(
                exported["data"]["manifest"]["originals"]
                    .as_array()
                    .unwrap()
                    .len(),
                2
            );
            let original_pin =
                kinekino_player::application::asset_source_archives::InspectAssetSources {
                    archive_id: "cli-originals".into(),
                    manifest_sha256: exported["data"]["manifest_sha256"].as_str().unwrap().into(),
                };
            let inspected = call(
                "inspect-original-sources",
                &request(
                    "asset_sources.inspect",
                    serde_json::to_value(&original_pin).unwrap(),
                ),
                0,
            )?;
            assert_eq!(inspected["data"], exported["data"]);
            let typed_originals = typed
                .inspect_asset_sources(RequestContext::default(), original_pin.clone())
                .unwrap();
            assert_eq!(
                serde_json::to_value(&typed_originals).unwrap(),
                exported["data"]
            );
            let typed_export = typed
                .export_asset_sources(
                    RequestContext::default(),
                    kinekino_player::application::asset_source_archives::ExportAssetSources {
                        archive_id: "typed-originals".into(),
                        project: input.project.clone(),
                        missing_frames: MissingAssetFramePolicy::Reject,
                    },
                )
                .unwrap();
            assert_eq!(
                typed_export.manifest_sha256,
                typed_originals.manifest_sha256
            );
            document.revision.0 = 3;
            for layer in &mut document.layers {
                layer.visible = false;
            }
            let mut hidden_save =
                request("asset_projects.save_revision", json!({"document":document}));
            hidden_save.context.expected_revision = Some("2".into());
            let hidden = call("save-hidden-source-project", &hidden_save, 0)?;
            let hidden_export = call(
                "export-hidden-sources",
                &request(
                    "asset_sources.export",
                    json!({"archive_id":"hidden-originals","project":{"document_id":"cli-assets","publication_revision":"3","publication_sha256":hidden["data"]["publication_sha256"]},"missing_frames":"reject"}),
                ),
                0,
            )?;
            assert_eq!(
                hidden_export["data"]["manifest"]["bindings"]
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
            assert_eq!(
                hidden_export["data"]["project"]["document"]["layers"][0]["visible"],
                false
            );
            submit.payload["output_directory"] = json!("C:/foreign");
            assert_eq!(
                call("deny-source-path", &submit, 2)?["error"]["code"],
                "invalid_argument"
            );
            let mut wrong = original_pin.clone();
            wrong.manifest_sha256 = "0".repeat(64);
            assert!(
                typed
                    .inspect_asset_sources(RequestContext::default(), wrong)
                    .is_err()
            );
            fs::write(
                fixture.root.join("source-archive-summary.json"),
                serde_json::to_vec_pretty(&typed_originals).unwrap(),
            )
            .unwrap();
            let mut submit = request(
                "jobs.submit",
                json!({"operation":"production.build_assets","parameters":input,"execution_timeout_ms":"120000"}),
            );
            submit.context.idempotency_key = Some("asset-cli-once".into());
            let mut injected = submit.clone();
            injected.context.idempotency_key = Some("deny-path".into());
            injected.payload["parameters"]["output_directory"] = json!("C:/untrusted");
            assert_eq!(
                call("deny-output-path", &injected, 2)?["error"]["code"],
                "invalid_argument"
            );
            let mut wrong = submit.clone();
            wrong.context.idempotency_key = Some("wrong-asset-pin".into());
            wrong.payload["parameters"]["project"]["publication_sha256"] = json!("0".repeat(64));
            assert_eq!(
                call("deny-project-pin", &wrong, 8)?["error"]["code"],
                "source_expired"
            );
            let admitted = call("submit-assets", &submit, 0)?;
            let id: JobId =
                serde_json::from_value(admitted["data"]["job"]["job_id"].clone()).unwrap();
            let done = try_terminal_until(
                &endpoint,
                &id,
                original,
                Duration::from_secs(90),
                &fixture.root,
            )?;
            assert_eq!(done.state, JobState::Completed, "{:?}", done.failure);
            let output = done.output.as_ref().unwrap();
            let bytes = fs::read(&output.path).unwrap();
            assert_eq!(hash(&bytes), output.sha256);
            let publication: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(publication["execution"]["job_id"], id.as_str());
            assert_eq!(publication["quality_approved"], false);
            let directory = std::path::Path::new(&output.path).parent().unwrap();
            assert_eq!(fs::read_dir(directory).unwrap().count(), 9);
            for file in publication["files"].as_array().unwrap() {
                let bytes = fs::read(directory.join(file["name"].as_str().unwrap())).unwrap();
                assert_eq!(hash(&bytes), file["sha256"].as_str().unwrap());
            }
            let manifest: Value =
                serde_json::from_slice(&fs::read(directory.join("manifest.json")).unwrap())
                    .unwrap();
            assert_eq!(manifest["plan"]["project"]["publication_revision"], "1");
            assert_eq!(
                manifest["layers"][0]["request"]["transform"]["pivot"]["x"],
                frame.transform.pivot.x
            );
            let get = call("get-assets", &request("jobs.get", json!({"job_id":id})), 0)?;
            assert_eq!(get["data"]["output"]["sha256"], output.sha256);
            let inspect_input =
                kinekino_player::application::asset_artifacts::InspectAssetArtifact {
                    artifact_id: publication["artifact_id"].as_str().unwrap().into(),
                    publication_sha256: output.sha256.clone(),
                };
            let inspected = call(
                "inspect-asset-artifact",
                &request(
                    "asset_artifacts.inspect",
                    serde_json::to_value(&inspect_input).unwrap(),
                ),
                0,
            )?;
            assert_eq!(inspected["data"]["publication_sha256"], output.sha256);
            assert_eq!(inspected["data"]["manifest"]["quality_approved"], false);
            let typed_inspection = typed
                .inspect_asset_artifact(RequestContext::default(), inspect_input)
                .unwrap();
            assert_eq!(typed_inspection.manifest.layers.len(), 2);
            let replay = call("replay-assets", &submit, 0)?;
            assert_eq!(replay["data"]["created"], false);
            assert_eq!(replay["data"]["job"]["job_id"], id.as_str());
            let second = typed
                .submit_asset_bundle(
                    RequestContext {
                        idempotency_key: Some("asset-typed-once".into()),
                        ..RequestContext::default()
                    },
                    input,
                    Some(DecimalU64(120_000)),
                )
                .unwrap();
            let second = try_terminal_until(
                &endpoint,
                &second.job.job_id,
                original,
                Duration::from_secs(90),
                &fixture.root,
            )?;
            assert_eq!(second.state, JobState::Completed, "{:?}", second.failure);
            assert_ne!(second.output.as_ref().unwrap().path, output.path);
            fs::write(fixture.root.join("asset-summary.json"),serde_json::to_vec_pretty(&json!({"cli_job":id,"typed_job":second.job_id,"publication_sha256":output.sha256,"producer_sha256":producer,"config_sha256":pin,"host_config_guarded":true,"old_revision_preserved":true,"quality_approved":false,"gui_verified":false,"source_pts":null,"direct_production_command_supported":false})).unwrap()).unwrap();
            Ok(())
        },
    )));
    let stopped = host
        .finish(
            Duration::from_secs(15),
            &fixture.root.join("logs/stop.stdout"),
            &fixture.root.join("logs/host.stderr"),
        )
        .and_then(|terminal| {
            let check = (|| -> io::Result<()> {
                if terminal.receipt.child_exit != Some(0)
                    || terminal.receipt.forced
                    || terminal.receipt.family != FamilyObservation::Empty
                    || !terminal.receipt.stdin_eof_requested
                {
                    return Err(io::Error::other(
                        "Original asset host did not exit gracefully",
                    ));
                }
                let stopped: Value = serde_json::from_slice(&terminal.tail)?;
                if stopped["event"] != "job_host_stopped"
                    || stopped["receipt"]["completed_total"] != 2
                    || stopped["local_host_shutdown"]["pipe"] != "joined"
                    || stopped["local_host_shutdown"]["sdk"] != "joined"
                {
                    return Err(io::Error::other(
                        "Original asset SDK/transport/Jobs shutdown differs",
                    ));
                }
                Ok(())
            })();
            check.map_err(|error| {
                CliFailure::terminal(
                    "asset host EOF",
                    error,
                    &fixture.root,
                    terminal.receipt.clone(),
                )
            })?;
            Ok(terminal.receipt)
        });
    ScenarioFailure::conclude(action, stopped, None)
}
