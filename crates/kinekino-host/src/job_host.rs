use crate::host_shutdown::{
    DependencyObservation, HostRunOutcome, TransportOutcome, dependency_deadline,
    failed_dependency, finish_dependency, observe_dependency, pending_jobs, serve_transport,
};
use kinekino_jobs::infrastructure::RetainingWorkerHost;
use kinekino_player::application::audio_vad::ProductVadHostOptions;
use kinekino_player::{
    application::api::{API_VERSION, ApiError, ErrorCode, PlayerSdk},
    infrastructure,
};
use std::{io::Write, path::PathBuf, sync::Arc};
struct JobHostInput {
    root: PathBuf,
    vad: Option<ProductVadHostOptions>,
    card_projects: Option<PathBuf>,
    clip_config: Option<(PathBuf, String)>,
    asset_config: Option<(PathBuf, String)>,
}
fn input(args: &[String]) -> Result<JobHostInput, ApiError> {
    let mut root = None;
    let mut vad_root = None;
    let mut vad_workspace = None;
    let mut card_projects = None;
    let mut clip_config = None;
    let mut clip_sha = None;
    let mut asset_config = None;
    let mut asset_sha = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--serve-jobs" => {}
            "--job-root" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Explicit job host requires --job-root ABSOLUTE_DIRECTORY",
                    )
                })?;
                if root.is_some() {
                    return Err(ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Job host root must be given once",
                    ));
                }
                root = Some(PathBuf::from(value));
            }
            flag @ ("--vad-root" | "--vad-workspace") => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::InvalidArgument,
                        "VAD host address value is missing",
                    )
                })?;
                let slot = if flag == "--vad-root" {
                    &mut vad_root
                } else {
                    &mut vad_workspace
                };
                if slot.replace(PathBuf::from(value)).is_some() {
                    return Err(ApiError::new(
                        ErrorCode::InvalidArgument,
                        "VAD host addresses must be given once",
                    ));
                }
            }
            "--card-project-root" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Card project root value missing",
                    )
                })?;
                let path = PathBuf::from(value);
                if !path.is_absolute() || card_projects.replace(path).is_some() {
                    return Err(ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Card project root must be one explicit absolute directory",
                    ));
                }
            }
            flag @ ("--asset-build-config" | "--asset-build-config-sha256") => {
                index += 1;
                let value = args
                    .get(index)
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(|| {
                        ApiError::new(
                            ErrorCode::InvalidArgument,
                            "Asset build host option requires a value",
                        )
                    })?;
                let slot = if flag == "--asset-build-config" {
                    &mut asset_config
                } else {
                    &mut asset_sha
                };
                if slot.replace(value.clone()).is_some() {
                    return Err(ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Duplicate asset build host option",
                    ));
                }
            }
            flag @ ("--clip-render-config" | "--clip-render-config-sha256") => {
                index += 1;
                let value = args
                    .get(index)
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(|| {
                        ApiError::new(
                            ErrorCode::InvalidArgument,
                            "Clip render host option requires a value",
                        )
                    })?;
                let slot = if flag == "--clip-render-config" {
                    &mut clip_config
                } else {
                    &mut clip_sha
                };
                if slot.replace(value.clone()).is_some() {
                    return Err(ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Duplicate clip render host option",
                    ));
                }
            }
            flag if crate::frame_tools::is_option(flag) => {
                index += 1;
                if args.get(index).is_none() {
                    return Err(ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Frame option requires a value",
                    ));
                }
            }
            _ => {
                return Err(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Job host accepts --serve-jobs, --job-root, optional --card-project-root, paired clip/asset configuration paths and SHA256 pins, paired VAD settings and paired frame tool paths/SHA256 pins",
                ));
            }
        }
        index += 1;
    }
    let root = root.filter(|root| root.is_absolute()).ok_or_else(|| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Job host requires an absolute local root",
        )
    })?;
    let vad = match (vad_root, vad_workspace) {
        (None, None) => None,
        (Some(release_root), Some(workspace_root)) => Some(ProductVadHostOptions {
            release_root,
            workspace_root,
        }),
        _ => {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "VAD host requires both --vad-root and --vad-workspace",
            ));
        }
    };
    crate::frame_tools::options(args)?;
    let clip_config = match (clip_config, clip_sha) {
        (None, None) => None,
        (Some(path), Some(sha))
            if PathBuf::from(&path).is_absolute()
                && kinekino_player::domain::identifiers::valid_sha256(&sha) =>
        {
            Some((PathBuf::from(path), sha))
        }
        _ => {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Clip host requires paired absolute --clip-render-config and --clip-render-config-sha256",
            ));
        }
    };
    let asset_config = match (asset_config, asset_sha) {
        (None, None) => None,
        (Some(path), Some(sha))
            if PathBuf::from(&path).is_absolute()
                && kinekino_player::domain::identifiers::valid_sha256(&sha) =>
        {
            Some((PathBuf::from(path), sha))
        }
        _ => {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Asset host requires paired absolute --asset-build-config and --asset-build-config-sha256",
            ));
        }
    };
    if vad.is_some() && (card_projects.is_some() || clip_config.is_some() || asset_config.is_some())
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Combined VAD/card/clip/asset executor composition is not implemented; no adapter is silently omitted",
        ));
    }
    Ok(JobHostInput {
        root,
        vad,
        card_projects,
        clip_config,
        asset_config,
    })
}
struct CardHostConfiguration {
    templates: Arc<dyn kinekino_player::application::card_bundle::CardTemplateCatalogPort>,
    executor: Arc<dyn kinekino_jobs::application::JobExecutor>,
    projects: Arc<dyn kinekino_player::application::card_projects::CardProjectStore>,
}
fn card_configuration(
    root: Option<&std::path::Path>,
) -> Result<Option<CardHostConfiguration>, ApiError> {
    let Some(root) = root else {
        return Ok(None);
    };
    #[cfg(feature = "card-review-export")]
    {
        let projects: Arc<dyn kinekino_player::application::card_projects::CardProjectStore> =
            Arc::new(infrastructure::card_projects::CardProjectFiles::open(root)?);
        let executor =
            infrastructure::card_bundle_jobs::CardBundleJobExecutor::new(projects.clone());
        Ok(Some(CardHostConfiguration {
            templates: Arc::new(infrastructure::card_export::CardBundleFiles),
            executor: Arc::new(executor),
            projects,
        }))
    }
    #[cfg(not(feature = "card-review-export"))]
    {
        let _ = root;
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "This CLI build does not include the explicit card renderer/executor",
        ))
    }
}
fn serve(args: &[String]) -> Result<HostRunOutcome<RetainingWorkerHost>, ApiError> {
    let options = input(args)?;
    let card = card_configuration(options.card_projects.as_deref())?;
    let card_bundle_configured = card.is_some();
    let clip = options
        .clip_config
        .as_ref()
        .map(|(path, sha)| infrastructure::clip_render_jobs::OpenedClipRenderHost::open(path, sha))
        .transpose()?;
    let clip_render_configured = clip.is_some();
    let asset = options
        .asset_config
        .as_ref()
        .map(|(path, sha)| infrastructure::asset_build_jobs::OpenedAssetBuildHost::open(path, sha))
        .transpose()?;
    let asset_build_configured = asset.is_some();
    // Verify the explicit tools before creating a durable job host or publishing ready.
    let frame = crate::frame_tools::configured(args)?;
    let frame_configured = frame.is_some();
    let host = if card.is_some() || clip.is_some() || asset.is_some() {
        let mut executors = vec![infrastructure::jobs::subtitle_executor()];
        if let Some(config) = &card {
            executors.push(config.executor.clone());
        }
        if let Some(config) = &clip {
            executors.push(config.executor.clone());
        }
        if let Some(config) = &asset {
            executors.push(config.executor.clone());
        }
        let registry = kinekino_jobs::application::ExecutorRegistry::new(executors)?;
        infrastructure::jobs::start_retaining_host_with_executor(&options.root, Arc::new(registry))?
    } else {
        match options.vad {
            Some(vad) => {
                infrastructure::vad_product_host::start_product_vad_jobs(&options.root, vad)?
            }
            None => infrastructure::jobs::start_retaining_host(&options.root)?,
        }
    };
    let jobs = host.client();
    let batch_store = match infrastructure::job_batches::BatchFiles::open(&options.root) {
        Ok(store) => Arc::new(store),
        Err(error) => {
            let deadline = dependency_deadline();
            return Ok(failed_dependency(
                pending_jobs(host, deadline),
                error,
                deadline,
            ));
        }
    };
    let batches = Arc::new(
        kinekino_player::application::job_batches::service::BatchService::new(
            batch_store,
            jobs.clone(),
        ),
    );
    let execution_operations = jobs.capability().operations;
    let client = PlayerSdk::new(infrastructure::subtitle_files::SubtitleFiles)
        .with_jobs(jobs)
        .with_job_batches(batches);
    let client = if let Some(config) = &asset {
        client
            .with_shared_asset_project_store(config.projects.clone())
            .with_asset_artifact_inspector(config.artifacts.clone())
            .with_asset_source_archives(config.sources.clone())
    } else {
        client
    };
    let client = if let Some(config) = card {
        client
            .with_shared_card_project_store(config.projects)
            .with_card_template_catalog(config.templates)
    } else {
        client
    };
    let client: kinekino_player::application::api::ApplicationClientHandle =
        if let Some(config) = &clip {
            let client = client
                .with_shared_clip_project_store(config.projects.clone())
                .with_clip_source_inspector(config.sources.clone())
                .with_clip_preset_catalog(config.presets.clone());
            if let Some(frame) = frame {
                Arc::new(client.with_frame_capture(frame))
            } else {
                Arc::new(client)
            }
        } else if let Some(frame) = frame {
            Arc::new(client.with_frame_capture(frame))
        } else {
            Arc::new(client)
        };
    let local = match infrastructure::local_host::LocalHost::start(
        client,
        infrastructure::local_host::TransportOptions::default(),
    ) {
        Ok(local) => local,
        Err(error) => {
            let deadline = dependency_deadline();
            return Ok(failed_dependency(
                pending_jobs(host, deadline),
                error,
                deadline,
            ));
        }
    };
    let ready = serde_json::json!({"event":"job_host_ready","api_version":API_VERSION,"endpoint":local.endpoint(),"scope":"current_user_same_privilege","execution_operations":execution_operations,"frame_capture_configured":frame_configured,"card_bundle_configured":card_bundle_configured,"clip_render_configured":clip_render_configured,"clip_render_config_sha256":clip.as_ref().map(|config| &config.config_sha256),"asset_build_configured":asset_build_configured,"asset_build_config_sha256":asset.as_ref().map(|config|&config.config_sha256),"stop":"stdin_eof"});
    let outcome = serve_transport(
        local,
        host,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        &ready,
        |_, error| ApiError::new(ErrorCode::EngineFailed, error.to_string()),
        || {},
    );
    let transport = match outcome {
        TransportOutcome::Incomplete(owner) => return Ok(HostRunOutcome::Incomplete(owner)),
        TransportOutcome::Ready(ready) => ready,
    };
    let (host, first_error, local_status) = transport.into_parts();
    let deadline = dependency_deadline();
    let completion = match observe_dependency(
        pending_jobs(host, deadline),
        first_error,
        Some(local_status),
        deadline,
    ) {
        DependencyObservation::Complete(result) => result,
        DependencyObservation::Incomplete(owner) => {
            return Ok(HostRunOutcome::DependencyIncomplete(owner));
        }
    };
    let result = (|| {
        let receipt = finish_dependency(completion)?;
        writeln!(
        std::io::stdout().lock(),
        "{}",
        serde_json::json!({"event":"job_host_stopped","receipt":receipt,"local_host_shutdown":crate::host_shutdown::LocalHostShutdownView::from(local_status)})
    )
    .map_err(|error| ApiError::new(ErrorCode::EngineFailed, error.to_string()))?;
        Ok(())
    })();
    Ok(HostRunOutcome::Complete {
        result,
        local_host_shutdown: Some(local_status),
    })
}
pub(crate) fn run(args: &[String]) -> HostRunOutcome<RetainingWorkerHost> {
    match serve(args) {
        Ok(outcome) => outcome,
        Err(error) => HostRunOutcome::Complete {
            result: Err(error),
            local_host_shutdown: None,
        },
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    /// Test fixtures write `C:/name`; map it to an absolute path on the running OS.
    fn abs(name: &str) -> PathBuf {
        std::env::temp_dir().join(name)
    }
    fn args(values: &[&str]) -> Vec<String> {
        values
            .iter()
            .map(|s| match s.strip_prefix("C:/") {
                Some(rest) => abs(rest).to_string_lossy().into_owned(),
                None => (*s).into(),
            })
            .collect()
    }
    #[test]
    fn legacy_job_host_remains_without_vad_opt_in() {
        let parsed = input(&args(&["--serve-jobs", "--job-root", "C:/jobs"])).unwrap();
        assert!(parsed.vad.is_none());
        assert!(parsed.card_projects.is_none());
        assert!(parsed.asset_config.is_none());
    }
    #[test]
    fn asset_config_requires_one_absolute_path_and_one_valid_pin() {
        let good = args(&[
            "--serve-jobs",
            "--job-root",
            "C:/jobs",
            "--asset-build-config",
            "C:/asset.json",
            "--asset-build-config-sha256",
            &"a".repeat(64),
        ]);
        assert!(input(&good).unwrap().asset_config.is_some());
        let mut missing = good.clone();
        missing.truncate(missing.len() - 2);
        assert!(input(&missing).is_err());
        let mut relative = good.clone();
        relative[4] = "relative.json".into();
        assert!(input(&relative).is_err());
        let mut duplicate = good.clone();
        duplicate.extend(args(&["--asset-build-config", "C:/other.json"]));
        assert!(input(&duplicate).is_err());
        let mut bad = good.clone();
        *bad.last_mut().unwrap() = "bad-pin".into();
        assert!(input(&bad).is_err());
        let mut vad = good;
        vad.extend(args(&[
            "--vad-root",
            "C:/vad",
            "--vad-workspace",
            "C:/media",
        ]));
        assert_eq!(input(&vad).err().unwrap().code, ErrorCode::Unsupported);
    }
    #[test]
    fn card_root_is_explicit_unique_and_not_silently_combined_with_vad() {
        let parsed = input(&args(&[
            "--serve-jobs",
            "--job-root",
            "C:/jobs",
            "--card-project-root",
            "C:/cards",
        ]))
        .unwrap();
        assert_eq!(parsed.card_projects, Some(abs("cards")));
        assert!(
            input(&args(&[
                "--serve-jobs",
                "--job-root",
                "C:/jobs",
                "--card-project-root",
                "relative"
            ]))
            .is_err()
        );
        assert!(
            input(&args(&[
                "--serve-jobs",
                "--job-root",
                "C:/jobs",
                "--card-project-root",
                "C:/cards",
                "--card-project-root",
                "C:/other"
            ]))
            .is_err()
        );
        let error = input(&args(&[
            "--serve-jobs",
            "--job-root",
            "C:/jobs",
            "--card-project-root",
            "C:/cards",
            "--vad-root",
            "C:/vad",
            "--vad-workspace",
            "C:/media",
        ]))
        .err()
        .unwrap();
        assert_eq!(error.code, ErrorCode::Unsupported);
    }
    #[test]
    fn product_addresses_are_typed_data_not_permission() {
        let parsed = input(&args(&[
            "--serve-jobs",
            "--job-root",
            "C:/jobs",
            "--vad-root",
            "C:/release",
            "--vad-workspace",
            "C:/media",
        ]))
        .unwrap();
        let plan = parsed.vad.unwrap().plan().unwrap();
        assert_eq!(plan.audio_root, abs("media/audio"));
    }
    #[test]
    fn partial_duplicate_and_relative_addresses_are_rejected() {
        assert!(
            input(&args(&[
                "--serve-jobs",
                "--job-root",
                "C:/jobs",
                "--vad-root",
                "C:/release"
            ]))
            .is_err()
        );
        assert!(
            input(&args(&[
                "--serve-jobs",
                "--job-root",
                "C:/jobs",
                "--vad-root",
                "C:/release",
                "--vad-root",
                "C:/other"
            ]))
            .is_err()
        );
        let parsed = input(&args(&[
            "--serve-jobs",
            "--job-root",
            "C:/jobs",
            "--vad-root",
            "relative",
            "--vad-workspace",
            "C:/data",
        ]))
        .unwrap();
        assert!(parsed.vad.unwrap().plan().is_err());
    }
}
