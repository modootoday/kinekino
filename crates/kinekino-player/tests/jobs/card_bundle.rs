//! Actual configured CLI; reuses the existing original host/request ownership harness.
use kinekino_player::application::api::CardClient;
#[path = "card_bundle/batches.rs"]
mod batches;
#[path = "card_bundle/roles.rs"]
mod roles;
use super::{
    fixture::{Fixture, request},
    kinekino_process::{
        CliFailure, OwnedHost, ScenarioFailure, close_request, phase_deadline, transport_at,
    },
    provider::try_terminal_until,
};
use kinekino_player::{
    application::{
        api::{ApplicationClient, RequestContext},
        card_bundle::{CardBundleRequest, SavedCardProjectPin, SelectedCardFrame},
        card_projects,
        jobs::{JobId, JobState, SubmitJob},
    },
    domain::{api_values::DecimalU64, card_document::CardDocument},
    infrastructure::{card_export::CardBundleFiles, card_projects::CardProjectFiles},
};
use kinekino_process::domain::{deadline::Deadline, observation::FamilyObservation};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs, io,
    panic::{AssertUnwindSafe, catch_unwind},
    time::{Duration, Instant},
};

fn setup(fixture: &Fixture) -> (std::path::PathBuf, CardBundleRequest) {
    let projects = fixture.root.join("projects");
    fs::create_dir(&projects).unwrap();
    let store = CardProjectFiles::open(&projects).unwrap();
    let mut frame = Vec::new();
    {
        let mut e = png::Encoder::new(&mut frame, 8, 8);
        e.set_color(png::ColorType::Rgba);
        let mut w = e.write_header().unwrap();
        w.write_image_data(&[100, 50, 20, 255].repeat(64)).unwrap();
        w.finish().unwrap();
    }
    let frame_path = fixture.root.join("frame.png");
    fs::write(&frame_path, &frame).unwrap();
    let sha = kinekino_player::infrastructure::content_digest::sha256(
        &mut std::io::Cursor::new(&frame),
        1024 * 1024,
    )
    .unwrap()
    .1;
    let document:CardDocument=serde_json::from_value(json!({"schema_version":"1","document_id":"cli-card-project","revision":"1","source":{"source_id":"cue-source","source_revision":"1","source_sha256":"a".repeat(64),"cues":[{"cue_id":"cue-1","text":"원문","start_us":null,"end_us":null}]},"layout":{"canvas":CardBundleFiles::current_proposed_canvas(),"cards":[{"card_id":"first","role":"cover","text_kind":"summary","content":{"front":"제목","back":"요약 초안","source_refs":["cue-1"]},"frame":{"artifact_id":"frame-1","png_sha256":sha,"source_sha256":"a".repeat(64),"declared_source_pts_us":null},"alt_text_draft":"설명 초안"}],"caption_draft":"게시문 초안"}})).unwrap();
    let saved = card_projects::create(&store, &document).unwrap();
    (
        projects,
        CardBundleRequest {
            project: SavedCardProjectPin {
                document_id: document.document_id,
                publication_revision: DecimalU64(1),
                publication_sha256: saved.publication_sha256,
            },
            frames: vec![SelectedCardFrame {
                artifact_id: "frame-1".into(),
                path: frame_path.to_str().unwrap().into(),
                png_sha256: sha,
            }],
            output_directory: fixture.root.join("cli-output").to_str().unwrap().into(),
            review_artifact_only: true,
        },
    )
}
fn manifest(path: &str, expected_sha256: &str, expected_bytes: u64) -> Value {
    let bytes = fs::read(path).unwrap();
    let (length, sha) = kinekino_player::infrastructure::content_digest::sha256(
        &mut std::io::Cursor::new(&bytes),
        1024 * 1024,
    )
    .unwrap();
    assert_eq!(length, expected_bytes);
    assert_eq!(sha, expected_sha256);
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let root = std::path::Path::new(path).parent().unwrap();
    for entry in value["files"].as_array().unwrap() {
        let file = fs::File::open(root.join(entry["name"].as_str().unwrap())).unwrap();
        let (bytes, sha) = kinekino_player::infrastructure::content_digest::sha256(
            &mut std::io::BufReader::new(file),
            64 * 1024 * 1024,
        )
        .unwrap();
        assert_eq!(sha, entry["sha256"]);
        assert_eq!(bytes.to_string(), entry["bytes"]);
    }
    value
}

#[test]
#[ignore = "Explicit pinned card-review-export CLI and fresh Jobs evidence; no GUI/models/network"]
fn configured_cli_card_bundle_and_typed_submission_share_durable_jobs_and_owned_eof()
-> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    let mut fixture = Fixture::named("card-bundle");
    fixture.keep_evidence();
    let (projects, input) = setup(&fixture);
    let original = Deadline::new(
        Instant::now(),
        Duration::from_secs(100),
        Duration::from_secs(10),
    )
    .unwrap();
    let args: Vec<OsString> = vec![
        "--serve-jobs".into(),
        "--job-root".into(),
        fixture.root.join("store").into_os_string(),
        "--card-project-root".into(),
        projects.into_os_string(),
    ];
    let cutoff = phase_deadline(original, Duration::from_secs(12)).unwrap();
    let (host, endpoint) = OwnedHost::launch(fixture.cli.clone(), args, &fixture.root, original)
        .and_then(|host| {
            host.ready(cutoff, &fixture.root.join("logs/ready.stdout"), |bytes| {
                let ready: Value = serde_json::from_slice(bytes)?;
                if ready["event"] != "job_host_ready"
                    || ready["card_bundle_configured"] != true
                    || ready["scope"] != "current_user_same_privilege"
                {
                    return Err(io::Error::other("Configured card host ready differs"));
                }
                let operations: BTreeSet<_> = ready["execution_operations"]
                    .as_array()
                    .ok_or_else(|| io::Error::other("Missing operations"))?
                    .iter()
                    .filter_map(Value::as_str)
                    .collect();
                if operations
                    != BTreeSet::from([
                        "subtitles.convert",
                        "subtitles.export_range",
                        "subtitles.from_transcript",
                        "production.export_cards",
                    ])
                {
                    return Err(io::Error::other(
                        "Existing subtitle and new card operations must all be present",
                    ));
                }
                ready["endpoint"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .ok_or_else(|| io::Error::other("Missing endpoint"))
            })
        })
        .map_err(ScenarioFailure::host)?;
    let action = close_request(catch_unwind(AssertUnwindSafe(
        || -> Result<(), Box<CliFailure>> {
            let cap = fixture.try_call_until(
                "capabilities",
                &request("capabilities", json!({})),
                &endpoint,
                original,
                Duration::from_secs(10),
                0,
            )?;
            assert_eq!(cap["data"]["card_projects"]["backend_available"], true);
            assert_eq!(cap["data"]["commands"].as_array().unwrap().len(), 119);
            let templates = &cap["data"]["card_templates"];
            assert_eq!(templates["backend_available"], true);
            assert_eq!(templates["quality_approved"], false);
            assert_eq!(templates["platform_requirements_verified"], false);
            let catalog = templates["templates"].as_array().unwrap();
            assert_eq!(catalog.len(), 13);
            assert_eq!(
                catalog
                    .iter()
                    .filter(|entry| entry["frame_required"] == false)
                    .count(),
                6
            );
            assert!(
                catalog
                    .iter()
                    .any(|entry| entry["canvas"]["width"] == 1920
                        && entry["canvas"]["height"] == 1080)
            );
            assert!(
                catalog
                    .iter()
                    .all(|entry| entry["common_font_sha256"].as_array().unwrap().len() == 3)
            );
            let schema = fixture.try_call_until(
                "card-job-schema",
                &request("schema", json!({"command":"jobs.submit"})),
                &endpoint,
                original,
                Duration::from_secs(10),
                0,
            )?;
            let variants = schema["data"]["request"]["properties"]["payload"]["oneOf"]
                .as_array()
                .unwrap();
            let card = variants
                .iter()
                .find(|variant| {
                    variant["properties"]["operation"]["const"] == "production.export_cards"
                })
                .unwrap();
            assert_eq!(
                card["properties"]["parameters"],
                CardBundleRequest::schema()
            );
            let project = fixture.try_call_until(
                "inspect-project",
                &request(
                    "card_projects.inspect",
                    json!({"document_id":input.project.document_id}),
                ),
                &endpoint,
                original,
                Duration::from_secs(10),
                0,
            )?;
            assert_eq!(
                project["data"]["publication_sha256"],
                input.project.publication_sha256
            );
            let mut submit = request(
                "jobs.submit",
                serde_json::to_value(SubmitJob {
                    operation: "production.export_cards".into(),
                    parameters: serde_json::to_value(&input).unwrap(),
                    expected_source_sha256: None,
                    execution_timeout_ms: DecimalU64(30000),
                })
                .unwrap(),
            );
            submit.context.idempotency_key = Some("cli-card-once".into());
            let admitted = fixture.try_call_until(
                "submit-card",
                &submit,
                &endpoint,
                original,
                Duration::from_secs(10),
                0,
            )?;
            assert_eq!(admitted["data"]["created"], true);
            let id: JobId =
                serde_json::from_value(admitted["data"]["job"]["job_id"].clone()).unwrap();
            let completed = try_terminal_until(
                &endpoint,
                &id,
                original,
                Duration::from_secs(20),
                &fixture.root,
            )?;
            assert_eq!(
                completed.state,
                JobState::Completed,
                "{:?}",
                completed.failure
            );
            let output = completed.output.as_ref().unwrap();
            let first_manifest = manifest(&output.path, &output.sha256, output.bytes.0);
            assert_eq!(first_manifest["quality_approved"], false);
            let get = fixture.try_call_until(
                "get-card",
                &request("jobs.get", json!({"job_id":id})),
                &endpoint,
                original,
                Duration::from_secs(10),
                0,
            )?;
            assert_eq!(get["data"]["output"]["sha256"], output.sha256);
            let replay = fixture.try_call_until(
                "replay-card",
                &submit,
                &endpoint,
                original,
                Duration::from_secs(10),
                0,
            )?;
            assert_eq!(replay["data"]["created"], false);
            assert_eq!(replay["data"]["job"]["job_id"], id.as_str());
            let mut wrong = submit.clone();
            wrong.context.idempotency_key = Some("wrong-frame".into());
            let wrong_output = fixture.root.join("wrong-frame-output");
            wrong.payload["parameters"]["output_directory"] = json!(wrong_output);
            wrong.payload["parameters"]["frames"][0]["png_sha256"] = json!("e".repeat(64));
            let denied = fixture.try_call_until(
                "wrong-frame",
                &wrong,
                &endpoint,
                original,
                Duration::from_secs(10),
                5,
            )?;
            assert_eq!(denied["error"]["code"], "revision_conflict");
            assert!(!wrong_output.exists());
            let mut missing_key = submit.clone();
            missing_key.context.idempotency_key = None;
            let missing_output = fixture.root.join("missing-key-output");
            missing_key.payload["parameters"]["output_directory"] = json!(missing_output);
            let denied = fixture.try_call_until(
                "missing-key",
                &missing_key,
                &endpoint,
                original,
                Duration::from_secs(10),
                2,
            )?;
            assert_eq!(denied["error"]["code"], "invalid_argument");
            assert!(!missing_output.exists());
            let direct = fixture.try_call_until(
                "direct-export-unsupported",
                &request(
                    "production.export_cards",
                    serde_json::to_value(&input).unwrap(),
                ),
                &endpoint,
                original,
                Duration::from_secs(10),
                3,
            )?;
            assert_eq!(direct["error"]["code"], "unsupported");
            let client = transport_at(
                &endpoint,
                phase_deadline(original, Duration::from_secs(10)).unwrap(),
            )
            .unwrap();
            let mut typed = input.clone();
            let typed_catalog = client
                .card_template_catalog(RequestContext::default())
                .unwrap();
            assert_eq!(serde_json::to_value(&typed_catalog).unwrap(), *templates);
            for (template_index, template) in typed_catalog
                .templates
                .iter()
                .filter(|template| !template.frame_required && !template.role_layouts_and_pages)
                .enumerate()
            {
                let mut blank_document: CardDocument =
                    serde_json::from_value(project["data"]["publication"]["document"].clone())
                        .unwrap();
                blank_document.document_id = format!("cli-text-only-project-{template_index}");
                blank_document.revision = DecimalU64(1);
                blank_document.layout.canvas = template.canvas.clone();
                for card in &mut blank_document.layout.cards {
                    card.frame = None;
                }
                let blank_saved = client
                    .create_card_project(RequestContext::default(), blank_document)
                    .unwrap();
                let blank_input = CardBundleRequest {
                    project: SavedCardProjectPin {
                        document_id: blank_saved.publication.document.document_id,
                        publication_revision: blank_saved.publication.publication_revision,
                        publication_sha256: blank_saved.publication_sha256,
                    },
                    frames: Vec::new(),
                    output_directory: fixture
                        .root
                        .join(format!("text-only-output-{template_index}"))
                        .to_str()
                        .unwrap()
                        .into(),
                    review_artifact_only: true,
                };
                let mut blank_submit = request(
                    "jobs.submit",
                    serde_json::to_value(SubmitJob {
                        operation: "production.export_cards".into(),
                        parameters: serde_json::to_value(&blank_input).unwrap(),
                        expected_source_sha256: None,
                        execution_timeout_ms: DecimalU64(30000),
                    })
                    .unwrap(),
                );
                blank_submit.context.idempotency_key =
                    Some(format!("text-only-once-{template_index}"));
                let blank = fixture.try_call_until(
                    &format!("submit-text-only-{template_index}"),
                    &blank_submit,
                    &endpoint,
                    original,
                    Duration::from_secs(10),
                    0,
                )?;
                let blank_id: JobId =
                    serde_json::from_value(blank["data"]["job"]["job_id"].clone()).unwrap();
                let blank_result = try_terminal_until(
                    &endpoint,
                    &blank_id,
                    original,
                    Duration::from_secs(20),
                    &fixture.root,
                )?;
                assert_eq!(
                    blank_result.state,
                    JobState::Completed,
                    "{:?}",
                    blank_result.failure
                );
                let blank_output = blank_result.output.unwrap();
                let blank_manifest = manifest(
                    &blank_output.path,
                    &blank_output.sha256,
                    blank_output.bytes.0,
                );
                assert_eq!(blank_manifest["source_frames"], json!([]));
                assert_eq!(blank_manifest["template"]["frame_required"], false);
                let directory = std::path::Path::new(&blank_output.path).parent().unwrap();
                let png = fs::read(directory.join("001-cover.png")).unwrap();
                let mut decoder = png::Decoder::new(std::io::Cursor::new(&png))
                    .read_info()
                    .unwrap();
                assert_eq!(
                    (decoder.info().width, decoder.info().height),
                    (template.canvas.width, template.canvas.height)
                );
                let mut pixels = vec![0; decoder.output_buffer_size().unwrap()];
                decoder.next_frame(&mut pixels).unwrap();
                decoder.finish().unwrap();
                let provenance: Value = serde_json::from_slice(
                    &fs::read(directory.join("001-cover.provenance.json")).unwrap(),
                )
                .unwrap();
                assert!(provenance["frame_path"].is_null());
                assert!(provenance["frame_sha256"].is_null());
                assert!(provenance["frame_size"].is_null());
                assert!(
                    blank_manifest["files"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|entry| !entry["name"].as_str().unwrap().starts_with("frame-"))
                );
            }
            let role_document: CardDocument =
                serde_json::from_value(project["data"]["publication"]["document"].clone()).unwrap();
            roles::RoleScenario {
                fixture: &fixture,
                client: &client,
                endpoint: &endpoint,
                deadline: original,
                document: &role_document,
                frames: &input.frames,
            }
            .execute(&typed_catalog.templates)?;
            typed.output_directory = fixture.root.join("typed-output").to_str().unwrap().into();
            let context = RequestContext {
                idempotency_key: Some("typed-card-once".into()),
                ..RequestContext::default()
            };
            let admitted = client.submit_card_bundle(context, typed, None).unwrap();
            assert!(admitted.created);
            let second = try_terminal_until(
                &endpoint,
                &admitted.job.job_id,
                original,
                Duration::from_secs(20),
                &fixture.root,
            )?;
            assert_eq!(second.state, JobState::Completed, "{:?}", second.failure);
            let second_output = second.output.as_ref().unwrap();
            let second_manifest = manifest(
                &second_output.path,
                &second_output.sha256,
                second_output.bytes.0,
            );
            assert_eq!(first_manifest["project"], second_manifest["project"]);
            let png = |m: &Value| {
                m["files"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["name"] == "001-cover.png")
                    .unwrap()["sha256"]
                    .clone()
            };
            assert_eq!(png(&first_manifest), png(&second_manifest));
            let source = fixture.root.join("legacy.srt");
            let converted = fixture.root.join("legacy.vtt");
            fs::write(
                &source,
                "1\n00:00:01,000 --> 00:00:02,000\n기존 자막 작업\n\n",
            )
            .unwrap();
            let mut legacy = request(
                "jobs.submit",
                json!({"operation":"subtitles.convert","parameters":{"input_path":source,"output_path":converted,"input_format":"srt","output_format":"vtt"},"execution_timeout_ms":"10000"}),
            );
            legacy.context.idempotency_key = Some("card-host-legacy-subtitle".into());
            let legacy = fixture.try_call_until(
                "legacy-subtitle",
                &legacy,
                &endpoint,
                original,
                Duration::from_secs(10),
                0,
            )?;
            let legacy_id: JobId =
                serde_json::from_value(legacy["data"]["job"]["job_id"].clone()).unwrap();
            assert_eq!(
                try_terminal_until(
                    &endpoint,
                    &legacy_id,
                    original,
                    Duration::from_secs(15),
                    &fixture.root
                )?
                .state,
                JobState::Completed
            );
            assert!(fs::read_to_string(&converted).unwrap().contains("WEBVTT"));
            fs::write(fixture.root.join("card-summary.json"),serde_json::to_vec_pretty(&json!({"cli_job":id,"typed_job":admitted.job.job_id,"project":input.project,"pixel_sha_equal":true,"quality_approved":false,"direct_export_supported":false})).unwrap()).unwrap();
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
                    || stopped["receipt"]["completed_total"] != 12
                    || stopped["receipt"]["cancelled_total"] != 0
                    || stopped["receipt"]["interrupted_total"] != 0
                    || stopped["local_host_shutdown"]["pipe"] != "joined"
                    || stopped["local_host_shutdown"]["sdk"] != "joined"
                    || stopped["local_host_shutdown"]["dispatch_poisoned"] != false
                {
                    return Err(io::Error::other(
                        "Original SDK/transport/jobs shutdown differs",
                    ));
                }
                Ok(())
            })();
            check.map_err(|error| {
                CliFailure::terminal(
                    "card bundle host EOF",
                    error,
                    &fixture.root,
                    terminal.receipt.clone(),
                )
            })?;
            Ok(terminal.receipt)
        });
    ScenarioFailure::conclude(action, stopped, None)
}
