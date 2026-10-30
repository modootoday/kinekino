use kinekino_player::{application, infrastructure};

/// What the `player` process should do after the host body returns.
#[derive(Debug, PartialEq, Eq)]
pub enum Desktop {
    /// The host ran a native proposal/product window to completion.
    Done,
    /// Plain viewing: run the Meari player window (`kinekino watch`).
    Watch { smoke: bool },
}

/// Desktop player process body (was `kinekino-player/src/main.rs`); entry: `kinekino_app::player`.
pub fn main() -> anyhow::Result<Desktop> {
    #[cfg(windows)]
    {
        let args: Vec<_> = std::env::args().collect();
        infrastructure::diagnostics::configure(args.iter().any(|arg| arg == "--record-validation"));
        let review_root = application::launch::review_root(&args)?;
        let proposal_root = application::launch::proposal_root(&args)?;
        let project_root = application::launch::project_root(&args)?;
        let job_root = application::launch::job_root(&args)?;
        let historical = application::launch::historical_verifier(&args)?;
        let launch = application::launch::desktop_launch_plan(&args)
            .map_err(|error| anyhow::anyhow!("{:?}: {}", error.code, error.message))?;
        let mode = launch.mode;
        #[cfg(feature = "native-proposal-window")]
        let playback_configuration =
            application::launch::native_playback::desktop_configuration(&args)
                .map_err(|error| anyhow::anyhow!("{:?}: {}", error.code, error.message))?;
        #[cfg(feature = "native-proposal-window")]
        let provider_configuration = application::launch::browser_provider::desktop_configuration(
            &args,
            job_root.as_deref(),
            playback_configuration.as_ref().map(|c| c.catalog.as_path()),
        )
        .map_err(|error| anyhow::anyhow!("{:?}: {}", error.code, error.message))?;
        if mode == application::launch::DesktopMode::DesignPreview {
            anyhow::bail!("the egui design preview was removed; the player UI is Meari (`kinekino watch`)");
        }
        #[cfg(feature = "native-proposal-window")]
        let viewer_initialization_started = std::time::Instant::now();
        #[cfg(feature = "native-proposal-window")]
        let native_input = if mode == application::launch::DesktopMode::NativeProposal {
            Some(
                kinekino_player::assembly::meari::product_native_launch::prepare_native_launch(
                    &args,
                )
                .map_err(|error| anyhow::anyhow!("{:?}: {}", error.code, error.message))?,
            )
        } else {
            None
        };
        let start_url = application::launch::start_url(&args);
        let (status_publisher, status_reader) =
            application::playback_status::native_status_channel();
        let (control_binder, control_port) =
            infrastructure::native_playback_control::native_control_channel();
        #[cfg(feature = "native-proposal-window")]
        let job_read_root = if provider_configuration.is_some() {
            None
        } else {
            job_root.as_deref()
        };
        #[cfg(not(feature = "native-proposal-window"))]
        let job_read_root = job_root.as_deref();
        let sdk = with_job_reads(
            application::api::PlayerSdk::new(infrastructure::subtitle_files::SubtitleFiles),
            job_read_root,
        )
        .map_err(|error| anyhow::anyhow!("Job history initialization failed: {error:?}"))?
        .with_frame_capture(infrastructure::frame_capture::FfmpegFrameCapture::discover())
        .with_playback_status(status_reader)
        .with_playback_control(control_port);
        let sdk = if let Some(root) = proposal_root {
            sdk.with_proposal_store(
                infrastructure::proposals::open_configured(&root, historical.as_ref()).map_err(
                    |error| anyhow::anyhow!("Proposal storage initialization failed: {error:?}"),
                )?,
            )
        } else {
            sdk
        };
        let sdk = if let Some(root) = project_root.as_ref() {
            sdk.with_project_store(infrastructure::projects::ProjectFiles::open(root).map_err(
                |error| anyhow::anyhow!("Project storage initialization failed: {error:?}"),
            )?)
        } else {
            sdk
        };
        #[cfg(feature = "native-proposal-window")]
        let configured = if let Some(root) = review_root.as_ref() {
            let store = infrastructure::model_review::ReviewFiles::open(root).map_err(|error| {
                anyhow::anyhow!("Review storage initialization failed: {error:?}")
            })?;
            kinekino_player::assembly::meari::product_viewer_launch::routed::configured_client(
                sdk.with_review_store(store),
                playback_configuration.as_ref(),
                provider_configuration.as_ref(),
            )
        } else {
            kinekino_player::assembly::meari::product_viewer_launch::routed::configured_client(
                sdk,
                playback_configuration.as_ref(),
                provider_configuration.as_ref(),
            )
        };
        #[cfg(feature = "native-proposal-window")]
        let (client, mut native_playback) = match configured {
            Ok(configured) => configured,
            Err(failure) => fatal_native_setup(failure),
        };
        #[cfg(not(feature = "native-proposal-window"))]
        let client: application::api::ApplicationClientHandle = if let Some(root) = review_root {
            let store =
                infrastructure::model_review::ReviewFiles::open(&root).map_err(|error| {
                    anyhow::anyhow!("Review storage initialization failed: {error:?}")
                })?;
            std::sync::Arc::new(sdk.with_review_store(store))
        } else {
            std::sync::Arc::new(sdk)
        };
        let paths: application::subtitles::path_picker::SubtitlePathPickerHandle =
            std::rc::Rc::new(infrastructure::subtitle_path_picker::NativeSubtitlePathPicker);
        #[cfg(feature = "native-proposal-window")]
        let viewer_project =
            kinekino_player::assembly::meari::product_viewer_launch::prepare_project(
                launch.project,
                client.as_ref(),
                project_root.as_deref(),
            )
            .map_err(|error| abort_native_setup(&mut native_playback, error))?;
        let local_host = infrastructure::local_host::LocalHost::start(
            client.clone(),
            infrastructure::local_host::TransportOptions::default(),
        )
        .map_err(|error| {
            #[cfg(feature = "native-proposal-window")]
            {
                abort_native_setup(&mut native_playback, error)
            }
            #[cfg(not(feature = "native-proposal-window"))]
            {
                anyhow::anyhow!("Local host initialization failed: {error:?}")
            }
        })?;
        let local_host_event = serde_json::json!({"endpoint":local_host.endpoint(),"is_running":local_host.is_running(),"transport":"named_pipe","scope":"current_user_same_privilege"});
        #[cfg(feature = "native-proposal-window")]
        if mode == application::launch::DesktopMode::NativeViewer {
            let _diagnostic_failure =
                kinekino_player::assembly::meari::product_viewer_launch::write_viewer_event(
                    "local-host-ready",
                    &local_host_event,
                    infrastructure::diagnostics::enabled(),
                    viewer_initialization_started.elapsed().as_millis(),
                    "native-viewer-initialization",
                    &mut std::io::stdout().lock(),
                );
        } else {
            infrastructure::diagnostics::record("local-host-ready", &local_host_event);
        }
        #[cfg(not(feature = "native-proposal-window"))]
        infrastructure::diagnostics::record("local-host-ready", &local_host_event);
        #[cfg(feature = "native-proposal-window")]
        if let Some(input) = native_input {
            let outcome = input.run(client);
            let report = outcome.report();
            infrastructure::diagnostics::record("native-proposal-outcome", &report);
            if !outcome.succeeded() {
                eprintln!("{report}");
                // Explicit abnormal process boundary: no Drop/anyhow conversion certifies cleanup.
                // The owned outcome remains alive until termination; its report states incompleteness.
                std::process::exit(1);
            }
            drop(outcome);
            drop(local_host);
            return Ok(Desktop::Done);
        }
        #[cfg(feature = "native-proposal-window")]
        if mode == application::launch::DesktopMode::NativeViewer {
            let viewer_started = std::time::Instant::now();
            if let Some(playback) = native_playback {
                let outcome = playback.run(local_host, viewer_project);
                let succeeded = outcome.succeeded();
                let _diagnostic_failures = outcome.write_report(
                    infrastructure::diagnostics::enabled(),
                    viewer_started.elapsed().as_millis(),
                    &mut std::io::stdout().lock(),
                    &mut std::io::stderr().lock(),
                );
                if !succeeded {
                    std::process::exit(1);
                }
                if let Err(retained) = outcome.end_review() {
                    eprintln!("{}", retained.report());
                    std::process::exit(1);
                }
                return Ok(Desktop::Done);
            }
            let outcome =
                kinekino_player::assembly::meari::product_viewer_launch::run_product_viewer_with_project(
                    client, local_host, viewer_project,
                );
            let succeeded = outcome.succeeded();
            let report = outcome.report();
            let _diagnostic_failures =
                kinekino_player::assembly::meari::product_viewer_launch::write_viewer_report(
                    &report,
                    succeeded,
                    infrastructure::diagnostics::enabled(),
                    viewer_started.elapsed().as_millis(),
                    &mut std::io::stdout().lock(),
                    &mut std::io::stderr().lock(),
                );
            if !succeeded {
                // Original endpoint/native owners remain in outcome at this abnormal boundary.
                std::process::exit(1);
            }
            return Ok(Desktop::Done);
        }
        // The egui/WebView player screen is gone: plain viewing is the Meari player window.
        let _ = (start_url, client, paths, status_publisher, control_binder);
        drop(local_host);
        return Ok(Desktop::Watch {
            smoke: args.iter().any(|arg| arg == "--smoke-test"),
        });
    }
    #[cfg(not(windows))]
    {
        return Ok(Desktop::Watch {
            smoke: std::env::args().any(|arg| arg == "--smoke-test"),
        });
    }
}

#[cfg(all(windows, feature = "native-proposal-window"))]
fn fatal_native_setup(
    failure: Box<
        kinekino_player::assembly::meari::product_viewer_launch::routed::ProductPlaybackSetupFailure,
    >,
) -> ! {
    eprintln!("{}", failure.report());
    // The original failure carrier remains alive at this explicit abnormal process boundary.
    std::process::exit(1)
}
#[cfg(all(windows, feature = "native-proposal-window"))]
fn abort_native_setup(
    playback: &mut Option<
        kinekino_player::assembly::meari::product_viewer_launch::routed::ProductPlaybackBootstrap,
    >,
    error: application::api::ApiError,
) -> anyhow::Error {
    if let Some(playback) = playback.take() {
        fatal_native_setup(playback.abort_setup(error));
    }
    anyhow::anyhow!("{:?}: {}", error.code, error.message)
}

/// Trusted launch assembly only; does not create a task worker or default root.
#[cfg(windows)]
fn with_job_reads(
    sdk: application::api::PlayerSdk<infrastructure::subtitle_files::SubtitleFiles>,
    root: Option<&std::path::Path>,
) -> Result<
    application::api::PlayerSdk<infrastructure::subtitle_files::SubtitleFiles>,
    application::api::ApiError,
> {
    match root {
        Some(root) => Ok(sdk.with_jobs(infrastructure::jobs::read_only(root)?)),
        None => Ok(sdk),
    }
}

#[cfg(all(test, windows))]
mod job_read_tests {
    use super::*;
    use application::{
        api::{
            ApiError, ApplicationClient, ApplicationClientHandle, ErrorCode, JobClient, Request,
            RequestContext,
        },
        jobs::{GetJob, ListJobs},
        sdk_execution::{
            CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner,
            ExecutionSnapshot,
        },
    };
    use serde_json::json;
    use std::{
        fs,
        panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
        path::PathBuf,
        sync::{
            Arc,
            atomic::{AtomicU64, Ordering},
        },
        thread,
        time::{Duration, Instant},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    fn root() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "gui-job-read-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        // Preserve the original filesystem fixture on failure or success.
        path
    }
    fn sdk() -> application::api::PlayerSdk<infrastructure::subtitle_files::SubtitleFiles> {
        application::api::PlayerSdk::new(infrastructure::subtitle_files::SubtitleFiles)
    }
    fn list() -> ListJobs {
        ListJobs {
            cursor: None,
            limit: Some(1),
        }
    }
    fn capabilities(client: &dyn ApplicationClient) -> serde_json::Value {
        let response = client.execute(Request::new("capabilities"));
        assert!(response.ok);
        response.data.unwrap()["jobs"].clone()
    }
    #[test]
    fn absent_job_root_preserves_unconfigured_sdk() {
        let client = with_job_reads(sdk(), None).unwrap();
        let cap = capabilities(&client);
        assert_eq!(cap["read_backend_available"], false);
        assert_eq!(cap["execution_host_available"], false);
        assert_eq!(
            client
                .list_jobs(RequestContext::default(), list())
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    #[test]
    fn configured_root_reads_original_store_without_job_worker() {
        let root = root();
        let client = with_job_reads(sdk(), Some(&root)).unwrap();
        assert!(root.join("job-events").is_dir());
        assert!(root.join("job-sources").is_dir());
        assert!(!root.join("jobs-owner.lock").exists());
        let cap = capabilities(&client);
        assert_eq!(cap["read_backend_available"], true);
        assert_eq!(cap["execution_host_available"], false);
        assert_eq!(cap["operations"], json!([]));
        assert_eq!(cap["automatic_restart_execution"], false);
        let page = client.list_jobs(RequestContext::default(), list()).unwrap();
        assert!(page.jobs.is_empty());
        assert!(page.next_cursor.is_none());
        let missing = client
            .get_job(
                RequestContext::default(),
                GetJob {
                    job_id: "job-00000000000000000000000000000001"
                        .to_owned()
                        .try_into()
                        .unwrap(),
                },
            )
            .unwrap_err();
        assert_eq!(missing.code, ErrorCode::NotFound);
        let submit = client.execute(Request {
            payload: json!({}),
            ..Request::new("jobs.submit")
        });
        assert_eq!(submit.error.unwrap().code, ErrorCode::Unsupported);
        assert!(!root.join("jobs-owner.lock").exists());
    }
    #[test]
    fn invalid_job_root_preserves_original_guard_error_without_creating_root() {
        let root = root();
        let missing = root.join("absent");
        let expected = match infrastructure::jobs::read_only(&missing) {
            Ok(_) => panic!("Missing original root must fail"),
            Err(error) => error,
        };
        let actual = match with_job_reads(sdk(), Some(&missing)) {
            Ok(_) => panic!("Assembly must preserve factory failure"),
            Err(error) => error,
        };
        assert_eq!(actual.code, expected.code);
        assert_eq!(actual.message, expected.message);
        assert!(!missing.exists());
        let ordinary_file = root.join("not-a-directory");
        fs::write(&ordinary_file, b"preserve").unwrap();
        assert!(with_job_reads(sdk(), Some(&ordinary_file)).is_err());
        assert_eq!(fs::read(ordinary_file).unwrap(), b"preserve");
    }
    struct Notify;
    impl CompletionNotifier for Notify {
        fn notify(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    #[test]
    fn same_configured_client_flows_through_original_execution_owner() {
        let root = root();
        let client: ApplicationClientHandle = Arc::new(with_job_reads(sdk(), Some(&root)).unwrap());
        let weak = Arc::downgrade(&client);
        let owner = ExecutionOwner::start(
            client.clone(),
            ExecutionBudget {
                max_operations: 2,
                max_request_bytes: 4096,
                max_total_request_bytes: 8192,
                max_terminal_bytes: 4096,
                max_total_terminal_bytes: 8192,
            },
            Arc::new(Notify),
        )
        .unwrap();
        drop(client);
        let handle = owner.handle();
        let action = catch_unwind(AssertUnwindSafe(|| {
            assert!(weak.upgrade().is_some());
            let operation = handle
                .submit(Request {
                    payload: json!({"cursor":null,"limit":1}),
                    ..Request::new("jobs.list")
                })
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            let terminal = loop {
                if let ExecutionSnapshot::Terminal(terminal) = handle.peek(&operation).unwrap() {
                    break terminal;
                }
                assert!(
                    Instant::now() < deadline,
                    "Original query did not reach terminal"
                );
                thread::sleep(Duration::from_millis(2));
            };
            terminal.validate_response().unwrap();
            assert!(terminal.response.ok);
            assert_eq!(terminal.response.data.as_ref().unwrap()["jobs"], json!([]));
            handle.acknowledge(&operation).unwrap();
            assert!(!root.join("jobs-owner.lock").exists());
        }));
        // Drain/ACK and close the same SDK owner even if an assertion unwound.
        let deadline = Instant::now() + Duration::from_secs(3);
        let closed = loop {
            if let Ok(operations) = handle.retained_operations() {
                for operation in operations {
                    if matches!(handle.peek(&operation), Ok(ExecutionSnapshot::Terminal(_))) {
                        let _ack = handle.acknowledge(&operation);
                    }
                }
            }
            if matches!(owner.poll_close(), Ok(ExecutionCloseProgress::Closed(_))) {
                break true;
            }
            if Instant::now() >= deadline {
                break false;
            }
            thread::sleep(Duration::from_millis(2));
        };
        if !closed {
            // Explicit incomplete test-process boundary; never a cleanup receipt.
            let _original_owner = Box::leak(Box::new(owner));
        }
        if let Err(payload) = action {
            resume_unwind(payload);
        }
        assert!(closed, "Original SDK owner closeout remains incomplete");
    }
}
