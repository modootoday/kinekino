//! Real pinned CLI/typed SDK through the existing owned host/request harness.
use super::{
    fixture::{Fixture, request},
    kinekino_process::{
        CliFailure, OwnedHost, ScenarioFailure, close_request, phase_deadline, transport_at,
    },
    provider::try_terminal_until,
};
use kinekino_player::application::api::ClipClient;
use kinekino_player::{
    application::{
        api::RequestContext,
        clip_projects::ClipProjectStore,
        clip_render::presets::{BuiltinClipPresets, ClipPresetOrigin},
        clip_render::{ClipRenderRequest, InspectClipSource, SavedClipProjectPin},
        jobs::{JobId, JobState},
    },
    domain::api_values::DecimalU64,
    infrastructure::{clip_projects::ClipProjectFiles, clip_render_jobs::ClipRenderHostConfig},
};
use kinekino_process::domain::{deadline::Deadline, observation::FamilyObservation};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
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
#[ignore = "Pinned CLI, existing saved project/source/tools and fresh evidence; no GUI/network/models"]
fn configured_cli_and_typed_sdk_render_through_original_jobs_and_owned_eof()
-> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    run_configured_cli(false)
}
#[test]
#[ignore = "Pinned CLI/source/tools/common fonts and fresh evidence; manual captions, no GUI/network/models"]
fn configured_subtitle_cli_and_typed_sdk_render_through_original_jobs_and_owned_eof()
-> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    run_configured_cli(true)
}
fn run_configured_cli(with_subtitles: bool) -> Result<(), ScenarioFailure> {
    let mut fixture = Fixture::named(if with_subtitles {
        "clip-render-subtitles"
    } else {
        "clip-render"
    });
    fixture.keep_evidence();
    let recipe_projects = PathBuf::from(
        std::env::var_os("CLIP_RENDER_PROJECT_ROOT").expect("Existing saved project root"),
    );
    let saved = ClipProjectFiles::open(&recipe_projects)
        .unwrap()
        .load("jobs-render-one", Some(DecimalU64(1)))
        .unwrap();
    let projects = fixture.root.join("projects");
    fs::create_dir(&projects).unwrap();
    let artifacts = fixture.root.join("artifacts");
    fs::create_dir(&artifacts).unwrap();
    let mut config = json!({"schema_version":"clip-render-host/1","project_root":projects,"sources":{"source":{"path":std::env::var("CLIP_RENDER_SOURCE").unwrap(),"container":"matroska"}},"render":ClipRenderHostConfig { encoder: PathBuf::from(std::env::var_os("CLIP_RENDER_ENCODER").unwrap()), encoder_sha256: std::env::var("CLIP_RENDER_ENCODER_SHA256").unwrap(), encoder_inventory_sha256: std::env::var("CLIP_RENDER_INVENTORY_SHA256").unwrap(), probe: PathBuf::from(std::env::var_os("CLIP_RENDER_PROBE").unwrap()), probe_sha256: std::env::var("CLIP_RENDER_PROBE_SHA256").unwrap(), temporary_root: std::env::temp_dir(), artifact_root: artifacts, declared_bt709_source_sha256: BTreeSet::from([saved.publication.document.sources[0].sha256.clone()]) }});
    let preset_id = if with_subtitles {
        let mut definition = BuiltinClipPresets::entries().unwrap().remove(0).definition;
        definition.preset_id = "cli-local-preset".into();
        definition.origin = ClipPresetOrigin::HostManual;
        definition.label = "Local CLI preset".into();
        definition.max_duration_us.0 = 1_000_000;
        let path = fixture.root.join("preset.json");
        fs::write(&path, serde_json::to_vec_pretty(&definition).unwrap()).unwrap();
        config["preset_files"] = json!({definition.preset_id.clone():path});
        definition.preset_id
    } else {
        BuiltinClipPresets::entries()
            .unwrap()
            .remove(0)
            .preset
            .catalog
            .artifact_id
    };
    let caption_path = fixture.root.join("captions.json");
    let style_pin = if with_subtitles {
        let original = fs::read(PathBuf::from(
            std::env::var_os("CLIP_SUBTITLE_STYLE").expect("Pinned common-font style"),
        ))
        .unwrap();
        assert_eq!(
            hash(&original),
            std::env::var("CLIP_SUBTITLE_STYLE_SHA256").unwrap()
        );
        let mut style: Value = serde_json::from_slice(&original).unwrap();
        style["style_id"] = json!("cli-subtitle-style");
        style["font_px"] = json!(32);
        style["line_height_px"] = json!(48);
        let bytes = serde_json::to_vec_pretty(&style).unwrap();
        let path = fixture.root.join("style.json");
        fs::write(&path, &bytes).unwrap();
        config["subtitle_assets"] =
            json!({"documents":{"captions":caption_path},"styles":{"cli-subtitle-style":path}});
        Some(hash(&bytes))
    } else {
        None
    };
    let bytes = serde_json::to_vec_pretty(&config).unwrap();
    let config_pin = hash(&bytes);
    let path = fixture.root.join("clip-host.json");
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
        "--clip-render-config".into(),
        path.into_os_string(),
        "--clip-render-config-sha256".into(),
        config_pin.clone().into(),
    ];
    let cutoff = phase_deadline(original, Duration::from_secs(15)).unwrap();
    let (host, endpoint) = OwnedHost::launch(fixture.cli.clone(), args, &fixture.root, original)
        .and_then(|host| {
            host.ready(cutoff, &fixture.root.join("logs/ready.stdout"), |bytes| {
                let ready: Value = serde_json::from_slice(bytes)?;
                let ops: BTreeSet<_> = ready["execution_operations"]
                    .as_array()
                    .ok_or_else(|| io::Error::other("Missing operations"))?
                    .iter()
                    .filter_map(Value::as_str)
                    .collect();
                if ready["event"] != "job_host_ready"
                    || ready["clip_render_configured"] != true
                    || ready["clip_render_config_sha256"] != config_pin
                    || ops
                        != BTreeSet::from([
                            "subtitles.convert",
                            "subtitles.export_range",
                            "subtitles.from_transcript",
                            "production.render_short",
                        ])
                {
                    return Err(io::Error::other("Configured clip host ready differs"));
                }
                ready["endpoint"]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| io::Error::other("Missing endpoint"))
            })
        })
        .map_err(ScenarioFailure::host)?;
    let action = close_request(catch_unwind(AssertUnwindSafe(
        || -> Result<(), Box<CliFailure>> {
            let call = |label: &str, request: &kinekino_player::application::api::Request, exit| {
                fixture.try_call_until(
                    label,
                    request,
                    &endpoint,
                    original,
                    Duration::from_secs(15),
                    exit,
                )
            };
            let caps = call("capabilities", &request("capabilities", json!({})), 0)?;
            assert_eq!(caps["data"]["clip_projects"]["backend_available"], true);
            assert_eq!(caps["data"]["clip_sources"]["backend_available"], true);
            assert_eq!(
                caps["data"]["clip_presets"]["engine_availability_verified"],
                false
            );
            let selected_preset = caps["data"]["clip_presets"]["presets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|entry| entry["preset"]["catalog"]["artifact_id"] == preset_id)
                .unwrap()
                .clone();
            assert_eq!(
                selected_preset["definition"]["origin"],
                if with_subtitles {
                    "host_manual"
                } else {
                    "application_proposal"
                }
            );
            assert_eq!(selected_preset["platform_requirements_verified"], false);
            let typed_catalog = transport_at(
                &endpoint,
                phase_deadline(original, Duration::from_secs(15)).unwrap(),
            )
            .unwrap()
            .clip_preset_catalog(RequestContext::default())
            .unwrap();
            assert_eq!(
                serde_json::to_value(typed_catalog).unwrap(),
                caps["data"]["clip_presets"]
            );
            let source_schema = call(
                "source-schema",
                &request("schema", json!({"command":"clip_sources.inspect"})),
                0,
            )?;
            assert_eq!(
                source_schema["data"]["constraints"]["observation_is_render_authority"],
                false
            );
            let observed = call(
                "probe-source",
                &request("clip_sources.inspect", json!({"media_id":"source"})),
                0,
            )?;
            assert_eq!(
                observed["data"]["source"]["sha256"],
                saved.publication.document.sources[0].sha256
            );
            assert_eq!(
                observed["data"]["duration_basis"],
                "container_reported_span"
            );
            assert_eq!(
                call(
                    "deny-source-path",
                    &request(
                        "clip_sources.inspect",
                        json!({"media_id":"source","path":"C:/arbitrary.mkv"})
                    ),
                    2
                )?["error"]["code"],
                "invalid_argument"
            );
            assert_eq!(
                call(
                    "deny-invalid-source-id",
                    &request("clip_sources.inspect", json!({"media_id":"../escape"})),
                    2
                )?["error"]["code"],
                "invalid_argument"
            );
            let typed_probe = transport_at(
                &endpoint,
                phase_deadline(original, Duration::from_secs(15)).unwrap(),
            )
            .unwrap()
            .inspect_clip_source(
                RequestContext::default(),
                InspectClipSource {
                    media_id: "source".into(),
                },
            )
            .unwrap();
            assert_eq!(serde_json::to_value(typed_probe).unwrap(), observed["data"]);
            // Existing fixture supplies only the manual edit recipe. The new project's
            // source pin is the observation returned by the running product CLI.
            let mut document = serde_json::to_value(&saved.publication.document).unwrap();
            document["document_id"] = json!("cli-probed-project");
            document["sources"] = json!([observed["data"]["source"]]);
            document["preset"] = selected_preset["preset"].clone();
            document["segments"][0]["crop_keys"].as_array_mut().unwrap().push(json!({"source_us":"15500000","crop":{"left":250000,"top":0,"width":500000,"height":1000000},"fit":"fill"}));
            if let Some(style_pin) = &style_pin {
                let captions = json!({"schema_version":"clip-subtitle-document/1","media":observed["data"]["source"],"time_basis":"media_relative_us","origin":"explicit_manual_snapshot","cues":{"source_id":"captions","source_revision":"1","source_sha256":"c".repeat(64),"cues":[{"cue_id":"first","text":"첫 줄 <>& あ","start_us":"15000000","end_us":"15500000"},{"cue_id":"second","text":"겹친 둘째 줄","start_us":"15250000","end_us":"16000000"}]}});
                let bytes = serde_json::to_vec_pretty(&captions).unwrap();
                fs::write(&caption_path, &bytes).unwrap();
                document["segments"][0]["subtitles"] = json!({"document":{"artifact":{"artifact_id":"captions","revision":"1","sha256":hash(&bytes)},"source_sha256":observed["data"]["source"]["sha256"]},"cue_ids":["second","first"],"style":{"artifact_id":"cli-subtitle-style","revision":"1","sha256":style_pin}});
            }
            let mut bad_preset = document.clone();
            bad_preset["document_id"] = json!("preset-mismatch-project");
            bad_preset["preset"]["width"] =
                json!(bad_preset["preset"]["width"].as_u64().unwrap() - 2);
            let bad_saved = call(
                "create-preset-mismatch",
                &request("clip_projects.create", json!({"document":bad_preset})),
                0,
            )?;
            let mut bad_submit = request(
                "jobs.submit",
                json!({"operation":"production.render_short","parameters":{"project":{"document_id":"preset-mismatch-project","publication_revision":"1","publication_sha256":bad_saved["data"]["publication_sha256"]},"review_artifact_only":true},"execution_timeout_ms":"120000"}),
            );
            bad_submit.context.idempotency_key = Some("deny-preset-mismatch".into());
            assert_eq!(
                call("deny-preset-mismatch", &bad_submit, 8)?["error"]["code"],
                "source_expired"
            );
            let created = call(
                "create-probed-project",
                &request("clip_projects.create", json!({"document":document})),
                0,
            )?;
            assert_eq!(created["data"]["publication"]["publication_revision"], "1");
            document["revision"] = json!("2");
            document["segments"][0]["audio"]["volume_percent"] = json!(50);
            let mut save = request("clip_projects.save_revision", json!({"document":document}));
            save.context.expected_revision = Some("1".into());
            let stored = call("save-probed-project", &save, 0)?;
            let input = ClipRenderRequest {
                project: SavedClipProjectPin {
                    document_id: "cli-probed-project".into(),
                    publication_revision: DecimalU64(2),
                    publication_sha256: stored["data"]["publication_sha256"]
                        .as_str()
                        .unwrap()
                        .into(),
                },
                review_artifact_only: true,
            };
            let schema = call(
                "job-schema",
                &request("schema", json!({"command":"jobs.submit"})),
                0,
            )?;
            assert!(
                serde_json::to_string(&schema)
                    .unwrap()
                    .contains("production.render_short")
            );
            let inspect = call(
                "inspect-project",
                &request(
                    "clip_projects.inspect",
                    json!({"document_id":input.project.document_id}),
                ),
                0,
            )?;
            assert_eq!(
                inspect["data"]["publication_sha256"],
                input.project.publication_sha256
            );
            let mut submit = request(
                "jobs.submit",
                json!({"operation":"production.render_short","parameters":input,"execution_timeout_ms":"120000"}),
            );
            if with_subtitles {
                let mut arbitrary = submit.clone();
                arbitrary.context.idempotency_key = Some("deny-subtitle-path".into());
                arbitrary.payload["parameters"]["subtitle_path"] = json!("C:/arbitrary.json");
                assert_eq!(
                    call("deny-subtitle-path", &arbitrary, 2)?["error"]["code"],
                    "invalid_argument"
                );
            }
            submit.context.idempotency_key = Some("clip-cli-once".into());
            let admitted = call("submit-clip", &submit, 0)?;
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
            let output = done.output.unwrap();
            let bytes = fs::read(&output.path).unwrap();
            assert_eq!(hash(&bytes), output.sha256);
            let manifest: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(manifest["execution"]["job_id"], id.as_str());
            assert_eq!(manifest["quality_approved"], false);
            let render: Value = serde_json::from_slice(
                &fs::read(
                    std::path::Path::new(&output.path)
                        .parent()
                        .unwrap()
                        .join("render.json"),
                )
                .unwrap(),
            )
            .unwrap();
            assert_eq!(render["plan"]["crop_sections"].as_array().unwrap().len(), 2);
            assert_eq!(render["plan"]["crop_sections"][1]["threshold_pts"], "15500");
            assert_eq!(render["plan"]["crop_sections"][1]["frame_offset"], "12");
            assert_eq!(
                render["preset_definition"]["pin"],
                document["preset"]["catalog"]
            );
            assert_eq!(
                render["preset_definition"]["definition"],
                selected_preset["definition"]
            );
            assert_eq!(
                render["preset_definition"]["platform_requirements_verified"],
                false
            );
            if with_subtitles {
                assert_eq!(render["recipe"], "manual-mp4-seekable-stdout/3");
                assert_eq!(render["subtitles"]["runs"].as_array().unwrap().len(), 3);
                assert_eq!(
                    render["subtitles"]["runs"][1]["image"]["cue_ids"],
                    json!(["first", "second"])
                );
                assert_eq!(
                    render["subtitles"]["runs"][1]["image"]["document"],
                    document["segments"][0]["subtitles"]["document"]
                );
                assert_eq!(
                    render["subtitles"]["runs"][1]["image"]["style"]["sha256"],
                    style_pin.as_ref().unwrap().as_str()
                );
            } else {
                assert!(render.get("subtitles").is_none());
            }
            let get = call("get-clip", &request("jobs.get", json!({"job_id":id})), 0)?;
            assert_eq!(get["data"]["output"]["sha256"], output.sha256);
            let replay = call("replay-clip", &submit, 0)?;
            assert_eq!(replay["data"]["created"], false);
            assert_eq!(replay["data"]["job"]["job_id"], id.as_str());
            let mut wrong = submit.clone();
            wrong.context.idempotency_key = Some("clip-cli-wrong".into());
            wrong.payload["parameters"]["project"]["publication_sha256"] = json!("0".repeat(64));
            assert_eq!(
                call("wrong-project-pin", &wrong, 8)?["error"]["code"],
                "source_expired"
            );
            let typed = transport_at(
                &endpoint,
                phase_deadline(original, Duration::from_secs(15)).unwrap(),
            )
            .unwrap();
            let context = RequestContext {
                idempotency_key: Some("clip-typed-once".into()),
                ..RequestContext::default()
            };
            let second = typed
                .submit_clip_render(context, input.clone(), Some(DecimalU64(120_000)))
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
            let second_output = second.output.as_ref().unwrap();
            if with_subtitles {
                let typed_render: Value = serde_json::from_slice(
                    &fs::read(
                        std::path::Path::new(&second_output.path)
                            .parent()
                            .unwrap()
                            .join("render.json"),
                    )
                    .unwrap(),
                )
                .unwrap();
                let typed_runs = typed_render["subtitles"]["runs"].as_array().unwrap();
                let cli_runs = render["subtitles"]["runs"].as_array().unwrap();
                assert_eq!(typed_runs.len(), cli_runs.len());
                for (typed_run, cli_run) in typed_runs.iter().zip(cli_runs) {
                    assert_eq!(typed_run["frames"], cli_run["frames"]);
                    for field in [
                        "recipe",
                        "cue_ids",
                        "cue_text_sha256",
                        "cue_separator",
                        "text_sha256",
                        "document",
                        "style",
                        "fonts",
                        "width",
                        "height",
                        "png_sha256",
                        "alpha_mode",
                        "layout",
                    ] {
                        assert_eq!(
                            typed_run["image"][field], cli_run["image"][field],
                            "{field}"
                        );
                    }
                    assert_eq!(typed_run["image"]["asset_delivery"]["rejected"], 0);
                    assert_eq!(typed_run["image"]["asset_delivery"]["failures"], json!([]));
                }
            }
            fs::write(fixture.root.join("clip-summary.json"), serde_json::to_vec_pretty(&json!({"cli_job":id,"typed_job":second.job_id,"manifest_sha256":output.sha256,"selected_subtitles_verified":with_subtitles,"caption_text_is_manual_fixture":with_subtitles,"preset_definition_verified":true,"preset_settings_mismatch_rejected_before_job_creation":true,"typed_catalog_verified":true,"cli_calls":if with_subtitles {16} else {15},"typed_probe_and_render":true,"quality_approved":false,"color_verified":false,"direct_render_command_supported":false})).unwrap()).unwrap();
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
                    return Err(io::Error::other("Original host did not exit gracefully"));
                }
                let stopped: Value = serde_json::from_slice(&terminal.tail)?;
                if stopped["event"] != "job_host_stopped"
                    || stopped["receipt"]["completed_total"] != 2
                    || stopped["local_host_shutdown"]["pipe"] != "joined"
                    || stopped["local_host_shutdown"]["sdk"] != "joined"
                {
                    return Err(io::Error::other("Clip SDK/transport/Jobs shutdown differs"));
                }
                Ok(())
            })();
            check.map_err(|error| {
                CliFailure::terminal(
                    "clip host EOF",
                    error,
                    &fixture.root,
                    terminal.receipt.clone(),
                )
            })?;
            Ok(terminal.receipt)
        });
    ScenarioFailure::conclude(action, stopped, None)
}
