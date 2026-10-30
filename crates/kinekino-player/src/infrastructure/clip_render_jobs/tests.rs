use super::*;
use crate::{
    application::{
        clip_projects,
        jobs::{GetJob, JobState, SubmitJob},
    },
    infrastructure::{
        clip_projects::ClipProjectFiles,
        clip_sources::{ClipContainer, FixedClipSource},
        jobs,
    },
};
use kinekino_jobs::infrastructure::WorkerShutdownOutcome;
use std::{collections::BTreeMap, fs};
mod cancellation;
struct Observe;
impl MutationControl for Observe {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
#[test]
#[ignore = "Pinned existing source/tools, original Jobs host, fresh evidence and exclusive CPU allocation"]
fn original_jobs_capture_render_publish_replay_and_reopen() {
    run_original_jobs(false, false);
}
#[test]
#[ignore = "Pinned existing media/tools/common fonts, original Jobs, fresh evidence; manual fixture captions, no GUI/models"]
fn original_jobs_selected_subtitles_render_publish_replay_and_reopen() {
    run_original_jobs(true, false);
}
#[test]
#[ignore = "Actual pinned render and original Jobs cancellation before publication CAS, fresh evidence, no GUI/models"]
fn original_jobs_cancel_before_publication_releases_new_inputs_and_preserves_partial_artifacts() {
    run_original_jobs(true, true);
}
fn run_original_jobs(with_subtitles: bool, cancel_before_publication: bool) {
    let root = PathBuf::from(std::env::var_os("CLIP_JOBS_EVIDENCE_DIR").expect("Fresh evidence"));
    fs::create_dir(&root).unwrap();
    let source_path = PathBuf::from(std::env::var_os("CLIP_RENDER_SOURCE").unwrap());
    let probe = PathBuf::from(std::env::var_os("CLIP_RENDER_PROBE").unwrap());
    let probe_sha256 = std::env::var("CLIP_RENDER_PROBE_SHA256").unwrap();
    let encoder = PathBuf::from(std::env::var_os("CLIP_RENDER_ENCODER").unwrap());
    let encoder_sha256 = std::env::var("CLIP_RENDER_ENCODER_SHA256").unwrap();
    let sources = Arc::new(
        FfmpegClipSources::new(
            BTreeMap::from([(
                "source".into(),
                FixedClipSource {
                    path: source_path,
                    container: ClipContainer::Matroska,
                },
            )]),
            &probe,
            &probe_sha256,
            crate::domain::frame_capture::MAX_MEDIA_BYTES,
            Duration::from_secs(120),
        )
        .unwrap(),
    );
    let initial = sources.inspect_source("source", &Observe).unwrap();
    let source = &initial.facts().source;
    let mut document: ClipDocument = serde_json::from_value(json!({"schema_version":"1","document_id":"jobs-render-one","revision":"1","sources":[source],"segments":[{"segment_id":"one","media_id":"source","range":{"start_us":"15000000","end_us":"16000000"},"video_stream_index":0,"crop_keys":[{"source_us":"15000000","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"selected","stream_index":1,"volume_percent":75,"muted":false},"subtitles":null,"subject_ref":null}],"preset":{"catalog":{"artifact_id":"fixture-manual-profile","revision":"1","sha256":"c".repeat(64)},"width":1080,"height":1920,"frame_rate":{"numerator":30,"denominator":1},"container":"mp4","video_codec":"h264","audio_codec":"aac","color":"sdr_bt709"},"cut_mode":"exact_reencode"})).unwrap();
    document.preset = crate::application::clip_render::presets::BuiltinClipPresets::entries()
        .unwrap()
        .remove(0)
        .preset;
    let subtitle_assets = if with_subtitles {
        let original_style = fs::read(PathBuf::from(
            std::env::var_os("CLIP_SUBTITLE_STYLE").expect("Pinned style"),
        ))
        .unwrap();
        assert_eq!(
            hash(&original_style),
            std::env::var("CLIP_SUBTITLE_STYLE_SHA256").unwrap()
        );
        let mut style: Value = serde_json::from_slice(&original_style).unwrap();
        style["style_id"] = json!("job-subtitle-style");
        style["font_px"] = json!(32);
        style["line_height_px"] = json!(48);
        let captions = json!({"schema_version":"clip-subtitle-document/1","media":source,"time_basis":"media_relative_us","origin":"explicit_manual_snapshot","cues":{"source_id":"captions","source_revision":"1","source_sha256":"c".repeat(64),"cues":[{"cue_id":"first","text":"첫 줄 <>& あ","start_us":"15000000","end_us":"15500000"},{"cue_id":"second","text":"겹친 둘째 줄","start_us":"15250000","end_us":"16000000"}]}});
        let text_bytes = serde_json::to_vec_pretty(&captions).unwrap();
        let style_bytes = serde_json::to_vec_pretty(&style).unwrap();
        let text_path = root.join("captions.json");
        let style_path = root.join("style.json");
        fs::write(&text_path, &text_bytes).unwrap();
        fs::write(&style_path, &style_bytes).unwrap();
        document.segments[0].subtitles=Some(serde_json::from_value(json!({"document":{"artifact":{"artifact_id":"captions","revision":"1","sha256":hash(&text_bytes)},"source_sha256":source.sha256},"cue_ids":["second","first"],"style":{"artifact_id":"job-subtitle-style","revision":"1","sha256":hash(&style_bytes)}})).unwrap());
        let assets = SubtitleAssetFiles::open(
            BTreeMap::from([("captions".into(), text_path)]),
            BTreeMap::from([("job-subtitle-style".into(), style_path)]),
        )
        .unwrap();
        let empty = SubtitleAssetFiles::open(BTreeMap::new(), BTreeMap::new()).unwrap();
        assert_eq!(
            empty
                .require_selection_available(document.segments[0].subtitles.as_ref().unwrap())
                .unwrap_err()
                .code,
            ErrorCode::NeedsInput
        );
        Some(assets)
    } else {
        None
    };
    let temporary = std::env::temp_dir().join(format!(
        "clip-job-inputs-{}",
        hash(root.to_string_lossy().as_bytes())
    ));
    fs::create_dir(&temporary).unwrap();
    fs::write(
        root.join("temporary-root.json"),
        serde_json::to_vec_pretty(&temporary).unwrap(),
    )
    .unwrap();
    for name in ["projects", "artifacts", "jobs"] {
        fs::create_dir(root.join(name)).unwrap();
    }
    let projects = Arc::new(ClipProjectFiles::open(&root.join("projects")).unwrap());
    let saved = clip_projects::create(projects.as_ref(), &document).unwrap();
    let request = ClipRenderRequest {
        project: SavedClipProjectPin {
            document_id: document.document_id.clone(),
            publication_revision: DecimalU64(1),
            publication_sha256: saved.publication_sha256,
        },
        review_artifact_only: true,
    };
    let executor = ClipRenderJobExecutor::open(
        projects.clone(),
        sources,
        ClipRenderHostConfig {
            encoder,
            encoder_sha256,
            encoder_inventory_sha256: std::env::var("CLIP_RENDER_INVENTORY_SHA256").unwrap(),
            probe,
            probe_sha256,
            temporary_root: temporary.clone(),
            artifact_root: root.join("artifacts"),
            declared_bt709_source_sha256: BTreeSet::from([source.sha256.clone()]),
        },
    )
    .unwrap();
    let executor = Arc::new(if let Some(assets) = subtitle_assets {
        assert_eq!(
            executor.capture(&request).err().unwrap().code,
            ErrorCode::Unsupported
        );
        let before = executor.host_sha256.clone();
        let configured = executor.with_subtitle_assets(assets).unwrap();
        assert_ne!(before, configured.host_sha256);
        configured
    } else {
        executor
    });
    let (gate, executor): (_, Arc<dyn JobExecutor>) = if cancel_before_publication {
        let gate = Arc::new(cancellation::Gate::new());
        (
            Some(gate.clone()),
            Arc::new(cancellation::Paused {
                inner: executor,
                gate,
            }),
        )
    } else {
        (None, executor)
    };
    let host =
        jobs::start_retaining_host_with_executor(&root.join("jobs"), executor.clone()).unwrap();
    let client = host.client();
    let submit = SubmitJob {
        operation: RENDER_OPERATION.into(),
        parameters: serde_json::to_value(&request).unwrap(),
        expected_source_sha256: None,
        execution_timeout_ms: DecimalU64(120_000),
    };
    let captured = client.capture(submit).unwrap();
    fs::write(
        root.join("captured-request.json"),
        serde_json::to_vec_pretty(captured.request()).unwrap(),
    )
    .unwrap();
    assert_eq!(fs::read_dir(root.join("artifacts")).unwrap().count(), 0);
    let captured = captured.into_request();
    let mut wrong = captured.clone();
    wrong.expected_source_sha256 = Some("0".repeat(64));
    assert_eq!(
        client.submit(wrong, "wrong-pin").unwrap_err().code,
        ErrorCode::SourceExpired
    );
    let submitted = client
        .submit(captured.clone(), "clip-original-job")
        .unwrap();
    if let Some(gate) = &gate {
        gate.wait();
        let inputs = fs::read_dir(&temporary)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        let cancellation = client.cancel(crate::application::jobs::CancelJob {
            job_id: submitted.job.job_id.clone(),
            expected_attempt: submitted.job.attempt,
        });
        gate.release();
        fs::write(
            root.join("cancel-request.json"),
            serde_json::to_vec_pretty(&cancellation.as_ref().unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            inputs.len(),
            2,
            "Original PNG and verified MP4 owners must still be live before CAS"
        );
        fs::write(
            root.join("live-inputs-before-cancel.json"),
            serde_json::to_vec_pretty(&inputs).unwrap(),
        )
        .unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(120);
    let done = loop {
        let job = client
            .get(GetJob {
                job_id: submitted.job.job_id.clone(),
            })
            .unwrap();
        fs::write(
            root.join("last-job.json"),
            serde_json::to_vec_pretty(&job).unwrap(),
        )
        .unwrap();
        if job.state.terminal() {
            break job;
        }
        assert!(Instant::now() < deadline, "Original Jobs did not finish");
        std::thread::sleep(Duration::from_millis(20));
    };
    let replay = client.submit(captured, "clip-original-job").unwrap();
    assert!(!replay.created);
    assert_eq!(replay.job.job_id, submitted.job.job_id);
    // Persist terminal evidence before orderly original-owner shutdown even on failure.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut pending = host.begin_shutdown(deadline);
    loop {
        match pending.poll(deadline) {
            WorkerShutdownOutcome::Complete(done) => {
                fs::write(root.join("shutdown.json"), serde_json::to_vec_pretty(&json!({"receipt":done.receipt,"error":done.errors.first(),"secondary_count":done.errors.secondary().len()})).unwrap()).unwrap();
                assert!(done.errors.first().is_none());
                break;
            }
            WorkerShutdownOutcome::Pending(original) => {
                pending = original;
                assert!(
                    Instant::now() < deadline,
                    "Original worker shutdown remains pending"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    if cancel_before_publication {
        cancellation::verify(&root, &temporary, &done);
        drop(client);
        cancellation::retry_after_orderly_restart(&root, &temporary, &done, executor);
        return;
    }
    assert_eq!(done.state, JobState::Completed, "{:?}", done.failure);
    let output = done.output.as_ref().unwrap();
    let bytes = fs::read(&output.path).unwrap();
    assert_eq!(hash(&bytes), output.sha256);
    let manifest: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(manifest["execution"]["job_id"], done.job_id.as_str());
    assert_eq!(manifest["execution"]["input_sha256"], done.input_sha256);
    assert_eq!(
        manifest["execution"]["source_blob_sha256"],
        done.input.source.sha256
    );
    assert_eq!(manifest["quality_approved"], false);
    let reopened = jobs::read_only(&root.join("jobs"))
        .unwrap()
        .get(GetJob {
            job_id: done.job_id.clone(),
        })
        .unwrap();
    assert_eq!(reopened.output.unwrap().sha256, output.sha256);
    let artifacts = ClipArtifactStore::open(&root.join("artifacts")).unwrap();
    let artifact = artifacts
        .inspect(
            manifest["artifact_id"].as_str().unwrap(),
            &output.sha256,
            &Observe,
        )
        .unwrap();
    assert_eq!(
        artifact.execution().unwrap()["job_id"],
        done.job_id.as_str()
    );
    if with_subtitles {
        let render = artifact.render_facts();
        assert_eq!(render["recipe"], "manual-mp4-seekable-stdout/3");
        assert_eq!(render["subtitles"]["runs"].as_array().unwrap().len(), 3);
        assert_eq!(
            render["subtitles"]["runs"][1]["image"]["cue_ids"],
            json!(["first", "second"])
        );
        assert_eq!(
            render["subtitles"]["runs"][1]["image"]["document"]["artifact"]["sha256"],
            document.segments[0]
                .subtitles
                .as_ref()
                .unwrap()
                .document
                .artifact
                .sha256
        );
        assert_eq!(artifact.verification()["decode_integrity_verified"], true);
    }
    fs::write(root.join("verification.json"), serde_json::to_vec_pretty(&json!({"original_jobs_completed":true,"selected_subtitles_verified":with_subtitles,"caption_text_is_manual_fixture":with_subtitles,"manifest_sha256":output.sha256,"original_worker_shutdown_confirmed":true,"reopened_output_verified":true,"cli_verified":false,"quality_approved":false,"color_verified":false})).unwrap()).unwrap();
}
