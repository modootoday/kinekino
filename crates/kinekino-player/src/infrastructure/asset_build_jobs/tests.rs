use super::*;
mod cancellation;
mod inspection;
#[test]
fn producer_normalization_accepts_local_absolute_but_rejects_parent_and_relative_paths() {
    // OS-specific prefix forms (verbatim disk, UNC) are covered by the platform adapters'
    // `normalize_local_path` tests; this checks the player policy on top of the port.
    let local = std::env::temp_dir()
        .join("workspaces")
        .join("player-cli.exe");
    assert_eq!(local_producer_path(&local).unwrap(), local);
    let parent = std::env::temp_dir()
        .join("workspaces")
        .join("..")
        .join("player.exe");
    assert!(local_producer_path(&parent).is_err());
    assert!(local_producer_path(std::path::Path::new("relative.exe")).is_err());
}
use crate::{
    application::{
        asset_projects,
        jobs::{GetJob, JobState, SubmitJob},
    },
    domain::asset_generation::*,
    infrastructure::jobs,
};
use kinekino_jobs::infrastructure::WorkerShutdownOutcome;
use serde_json::json;
use std::{
    fs,
    fs::OpenOptions,
    time::{Duration, Instant},
};
#[test]
fn asset_job_request_rejects_host_paths_and_other_operations() {
    let request = json!({"project":{"document_id":"one","publication_revision":"1","publication_sha256":"a".repeat(64)},"missing_frames":"reject","review_artifact_only":true});
    assert!(parse(BUILD_OPERATION, request.clone()).is_ok());
    let mut injected = request;
    injected["output_directory"] = json!("C:/untrusted");
    assert_eq!(
        parse(BUILD_OPERATION, injected).err().unwrap().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        parse("production.render_short", json!({}))
            .err()
            .unwrap()
            .code,
        ErrorCode::Unsupported
    );
}
#[test]
#[ignore = "Fresh evidence, pinned existing PNGs/request, original Core Jobs worker and exclusive allocation; no model/GUI/CLI approval"]
fn original_jobs_build_assets_commit_verify_replay_and_reopen() {
    run_original_assets(None);
}
#[test]
#[ignore = "Original CPU Jobs cancellation before original CAS and orderly restart/manual attempt2; pinned PNGs/fresh evidence"]
fn original_jobs_cancel_before_cas_preserves_partial_and_manual_retry() {
    run_original_assets(Some(cancellation::Point::BeforeCas));
}
#[test]
#[ignore = "Original CPU Jobs late cancellation after actual marker commit; pinned PNGs/fresh evidence"]
fn original_jobs_late_cancel_does_not_undo_committed_output() {
    run_original_assets(Some(cancellation::Point::AfterCommit));
}
fn run_original_assets(point: Option<cancellation::Point>) {
    let root = PathBuf::from(std::env::var_os("ASSET_JOBS_EVIDENCE_DIR").expect("Fresh evidence"));
    fs::create_dir(&root).unwrap();
    let bytes = fs::read(PathBuf::from(
        std::env::var_os("ASSET_PROJECT_REQUEST").unwrap(),
    ))
    .unwrap();
    assert_eq!(
        hash(&bytes),
        std::env::var("ASSET_PROJECT_REQUEST_SHA256").unwrap()
    );
    let frame: AssetFrameRequest = serde_json::from_slice(&bytes).unwrap();
    let image_path = PathBuf::from(std::env::var_os("ASSET_INPUT_IMAGE").unwrap());
    let mask_path = PathBuf::from(std::env::var_os("ASSET_INPUT_MASK").unwrap());
    for name in ["projects", "artifacts", "jobs"] {
        fs::create_dir(root.join(name)).unwrap();
    }
    let mut document = AssetDocument {
        schema_version: "asset-project/1".into(),
        document_id: "original-jobs-assets".into(),
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
    let projects = AssetProjectFiles::open(&root.join("projects")).unwrap();
    let saved = asset_projects::create(&projects, &document).unwrap();
    let original_project = root
        .join("projects")
        .join("asset-original-jobs-assets")
        .join(format!("r-1-{}.json", saved.publication_sha256));
    let original_project_bytes = fs::read(&original_project).unwrap();
    let request = AssetBundleRequest {
        project: SavedAssetProjectPin {
            document_id: document.document_id.clone(),
            publication_revision: DecimalU64(1),
            publication_sha256: saved.publication_sha256,
        },
        missing_frames: MissingAssetFramePolicy::Reject,
        review_artifact_only: true,
    };
    let config = AssetBuildHostConfig {
        producer_sha256: hash(&fs::read(std::env::current_exe().unwrap()).unwrap()),
        project_root: root.join("projects"),
        artifact_root: root.join("artifacts"),
        images: BTreeMap::from([(
            frame.image.blob.artifact_id.clone(),
            FixedAssetImage {
                pin: frame.image.clone(),
                path: image_path.clone(),
            },
        )]),
        masks: BTreeMap::from([(
            frame.mask.blob.artifact_id.clone(),
            FixedAssetMask {
                pin: frame.mask.clone(),
                path: mask_path.clone(),
            },
        )]),
    };
    fs::write(
        root.join("host-config.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
    let executor = Arc::new(AssetBuildJobExecutor::open(config).unwrap());
    let base: Arc<dyn JobExecutor> = executor;
    let gate = point.map(|point| Arc::new(cancellation::Gate::new(point)));
    let selected: Arc<dyn JobExecutor> = match &gate {
        Some(gate) => Arc::new(cancellation::Paused {
            inner: base.clone(),
            gate: gate.clone(),
            root: root.clone(),
        }),
        None => base.clone(),
    };
    let host = jobs::start_retaining_host_with_executor(&root.join("jobs"), selected).unwrap();
    let client = host.client();
    let captured = client
        .capture(SubmitJob {
            operation: BUILD_OPERATION.into(),
            parameters: serde_json::to_value(request).unwrap(),
            expected_source_sha256: None,
            execution_timeout_ms: DecimalU64(120_000),
        })
        .unwrap();
    assert_eq!(fs::read_dir(root.join("artifacts")).unwrap().count(), 0);
    let captured = captured.into_request();
    fs::write(
        root.join("captured-request.json"),
        serde_json::to_vec_pretty(&captured).unwrap(),
    )
    .unwrap();
    let mut wrong = captured.clone();
    wrong.expected_source_sha256 = Some("0".repeat(64));
    assert_eq!(
        client.submit(wrong, "wrong-asset-pin").unwrap_err().code,
        ErrorCode::SourceExpired
    );
    // New head must not rebind the exact earlier publication selected for this job.
    document.revision.0 = 2;
    document.layers[0].transform.pivot.x = 0;
    asset_projects::save(&projects, DecimalU64(1), &document).unwrap();
    drop(projects);
    let submitted = client
        .submit(captured.clone(), "original-asset-job")
        .unwrap();
    if let Some(gate) = &gate {
        cancellation::cancel_at_gate(
            gate,
            &root,
            client.as_ref(),
            &submitted,
            &image_path,
            &mask_path,
            &original_project,
        );
    }
    let until = Instant::now() + Duration::from_secs(120);
    let done = loop {
        let record = client
            .get(GetJob {
                job_id: submitted.job.job_id.clone(),
            })
            .unwrap();
        fs::write(
            root.join("last-job.json"),
            serde_json::to_vec_pretty(&record).unwrap(),
        )
        .unwrap();
        if record.state.terminal() {
            break record;
        }
        assert!(Instant::now() < until, "Original asset worker remains live");
        std::thread::sleep(Duration::from_millis(20));
    };
    let replay = client.submit(captured, "original-asset-job").unwrap();
    assert!(!replay.created);
    assert_eq!(replay.job.job_id, done.job_id);
    let until = Instant::now() + Duration::from_secs(10);
    let mut pending = host.begin_shutdown(until);
    loop {
        match pending.poll(until) {
            WorkerShutdownOutcome::Complete(closed) => {
                fs::write(root.join("shutdown.json"),serde_json::to_vec_pretty(&json!({"receipt":closed.receipt,"error":closed.errors.first(),"secondary_count":closed.errors.secondary().len()})).unwrap()).unwrap();
                assert!(closed.errors.first().is_none());
                break;
            }
            WorkerShutdownOutcome::Pending(original) => {
                pending = original;
                assert!(
                    Instant::now() < until,
                    "Original asset shutdown remains pending"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    if let Some(gate) = &gate {
        assert_eq!(gate.calls(), 1);
    }
    if point == Some(cancellation::Point::BeforeCas) {
        drop(client);
        cancellation::verify_cancel_and_retry(&root, &done, base, &image_path, &mask_path);
        assert_eq!(
            hash(&fs::read(&image_path).unwrap()),
            frame.image.blob.sha256
        );
        assert_eq!(hash(&fs::read(&mask_path).unwrap()), frame.mask.blob.sha256);
        assert_eq!(fs::read(&original_project).unwrap(), original_project_bytes);
        return;
    }
    assert_eq!(done.state, JobState::Completed, "{:?}", done.failure);
    assert!(done.publication_started);
    let output = done.output.as_ref().unwrap();
    let bytes = fs::read(&output.path).unwrap();
    assert_eq!(hash(&bytes), output.sha256);
    let publication: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(publication["schema_version"], "asset-artifact/1");
    assert_eq!(publication["execution"]["job_id"], done.job_id.as_str());
    assert_eq!(publication["execution"]["input_sha256"], done.input_sha256);
    assert_eq!(
        publication["execution"]["source_blob_sha256"],
        done.input.source.sha256
    );
    assert_eq!(publication["execution"]["attempt"], "1");
    assert_eq!(publication["quality_approved"], false);
    let directory = std::path::Path::new(&output.path).parent().unwrap();
    assert_eq!(fs::read_dir(directory).unwrap().count(), 9);
    for file in publication["files"].as_array().unwrap() {
        let bytes = fs::read(directory.join(file["name"].as_str().unwrap())).unwrap();
        assert_eq!(hash(&bytes), file["sha256"].as_str().unwrap());
        assert_eq!(
            bytes.len() as u64,
            file["bytes"].as_str().unwrap().parse::<u64>().unwrap()
        );
    }
    let manifest: AssetBundleManifest =
        serde_json::from_slice(&fs::read(directory.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest.layers.len(), 2);
    assert_eq!(manifest.plan.project.publication_revision.0, 1);
    assert_eq!(
        manifest.layers[0].request.transform.pivot.x,
        frame.transform.pivot.x
    );
    assert!(!manifest.quality_approved);
    let reopened = jobs::read_only(&root.join("jobs"))
        .unwrap()
        .get(GetJob {
            job_id: done.job_id.clone(),
        })
        .unwrap();
    assert_eq!(reopened.output.as_ref().unwrap().sha256, output.sha256);
    assert_eq!(fs::read(&original_project).unwrap(), original_project_bytes);
    assert_eq!(
        hash(&fs::read(&image_path).unwrap()),
        frame.image.blob.sha256
    );
    assert_eq!(hash(&fs::read(&mask_path).unwrap()), frame.mask.blob.sha256);
    assert!(OpenOptions::new().write(true).open(&image_path).is_ok());
    assert!(OpenOptions::new().write(true).open(&mask_path).is_ok());
    fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&json!({"late_cancel_publication_in_progress_verified":point==Some(cancellation::Point::AfterCommit),"original_jobs_completed":true,"original_publication_cas_observed":done.publication_started,"publication_sha256":output.sha256,"data_files":8,"publication_files":1,"selected_old_revision_preserved":true,"replay_same_job":true,"original_worker_shutdown_confirmed":true,"reopened_job_output_same":true,"source_preserved":true,"no_native_child_started":true,"cli_verified":false,"gui_verified":false,"quality_approved":false,"colorimetry_verified":false})).unwrap()).unwrap();
}
