//! Actual read-only SDK/plugin/document connection; no job worker, CLI, OS window or model.
use super::document::{JobDetailsMountFailure, MountedJobDetails};
use super::*;
use crate::{
    application::{
        api::{ApplicationClient, ApplicationClientHandle, PlayerSdk, Request, Response},
        sdk_execution::{
            CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner,
        },
    },
    domain::api_values::DecimalU64,
};
use kinekino_jobs::{
    application::{JobService, JobStore, SourceBlobStore},
    domain::PreparedJobInput,
    infrastructure::{JobFiles, RootedJobPaths},
};
use meari_native::domain::{Viewport, widgets::Action};
use std::{
    any::Any,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

// Reuse the private adapter's original source without widening product visibility.
mod immutable_storage_paths {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/infrastructure/immutable_storage_paths.rs"
    ));
}

struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct RecordingClient {
    actual: ApplicationClientHandle,
    calls: Arc<Mutex<Vec<Request>>>,
}
impl ApplicationClient for RecordingClient {
    fn execute(&self, request: Request) -> Response {
        self.calls.lock().unwrap().push(request.clone());
        self.actual.execute(request)
    }
}
struct Paths;
impl RootedJobPaths for Paths {
    fn root(&self, path: &Path) -> Result<(PathBuf, Vec<File>), ApiError> {
        immutable_storage_paths::root(path)
    }
    fn directory(&self, path: &Path) -> Result<File, ApiError> {
        immutable_storage_paths::directory(path)
    }
    fn file(&self, path: &Path) -> Result<File, ApiError> {
        immutable_storage_paths::file(path)
    }
    fn lock(&self, path: &Path) -> Result<File, ApiError> {
        immutable_storage_paths::lock(path)
    }
}
fn target() -> GetJob {
    GetJob {
        job_id: format!("job-{}", "a".repeat(32)).try_into().unwrap(),
    }
}
fn readonly_client() -> (ApplicationClientHandle, JobRecord) {
    let root = std::env::current_dir()
        .unwrap()
        .join("runtime/quality-review/ui-job-details-stores")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&root).unwrap();
    let files = Arc::new(JobFiles::open(&root, Arc::new(Paths)).unwrap());
    let source = files
        .put_utf8("1\n00:00:00,000 --> 00:00:01,000\nfixture\n")
        .unwrap();
    let job = JobRecord::queued(
        target().job_id,
        PreparedJobInput {
            operation: "subtitle.convert".into(),
            parameters: serde_json::json!({"fixture":"readonly detail"}),
            source,
            execution_timeout_ms: DecimalU64(30000),
        },
    )
    .unwrap();
    files.admit(&job).unwrap();
    let sdk: ApplicationClientHandle =
        Arc::new(PlayerSdk::default().with_jobs(Arc::new(JobService::read_only(files))));
    (sdk, job)
}

enum RetainedSurface {
    Mounted(Box<MountedJobDetails>),
    MountFailed(Box<JobDetailsMountFailure>),
}
struct RetainedFailure {
    primary: Option<Box<dyn Any + Send>>,
    cleanup: Option<String>,
    diagnostic_error: Option<std::io::Error>,
    surface: RetainedSurface,
    _execution: ExecutionOwner,
}
impl RetainedFailure {
    fn report(&mut self, complete: bool) {
        let stderr = std::io::stderr();
        let mut output = stderr.lock();
        self.diagnostic_error = writeln!(
            output,
            "job_details_case_closeout complete={complete} primary_panic={} cleanup_error={:?}",
            self.primary.is_some(),
            self.cleanup
        )
        .and_then(|()| output.flush())
        .err();
    }
}
impl std::fmt::Debug for RetainedFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let primary = self.primary.as_ref().map(|payload| {
            payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("non-string panic")
        });
        let mut debug = f.debug_struct("RetainedJobDetailsCase");
        debug
            .field("primary", &primary)
            .field("cleanup", &self.cleanup)
            .field("diagnostic_error", &self.diagnostic_error);
        match &self.surface {
            RetainedSurface::Mounted(panel) => {
                debug.field("original_document", &panel.document.stamp());
            }
            RetainedSurface::MountFailed(failure) => {
                debug.field("mount_failure", &failure.diagnostic);
            }
        }
        debug.finish()
    }
}
fn with_panel(
    actual: ApplicationClientHandle,
    body: impl FnOnce(&MountedJobDetails, &Arc<Mutex<Vec<Request>>>),
) -> Result<(), Box<RetainedFailure>> {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let execution = ExecutionOwner::start(
        Arc::new(RecordingClient {
            actual,
            calls: calls.clone(),
        }),
        ExecutionBudget {
            max_operations: 2,
            max_request_bytes: 262144,
            max_total_request_bytes: 524288,
            max_terminal_bytes: 262144,
            max_total_terminal_bytes: 524288,
        },
        Arc::new(Notify),
    )
    .unwrap();
    let attachment = execution.handle().attach().unwrap();
    let panel = match MountedJobDetails::mount(
        attachment,
        target(),
        RequestContext::default(),
        crate::assembly::meari::player_shell_document::DocumentOptions {
            namespace: "job-details-test".into(),
            epoch: 1,
            source_generation: 1,
            viewport: Viewport {
                width: 300,
                height: 720,
                dark: true,
            },
        },
        "",
        None,
    ) {
        Ok(panel) => panel,
        Err(failure) => {
            let mut retained = Box::new(RetainedFailure {
                primary: None,
                cleanup: Some(
                    "Mount failed; original owner/attachment cleanup is unverified".into(),
                ),
                diagnostic_error: None,
                surface: RetainedSurface::MountFailed(Box::new(failure)),
                _execution: execution,
            });
            retained.report(false);
            return Err(retained);
        }
    };
    let primary =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(&panel, &calls))).err();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut cleanup = None;
    let detached = loop {
        match panel.close_query_owner() {
            Ok(true) => break true,
            Ok(false) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(1)),
            Ok(false) => {
                cleanup = Some("Original query cleanup remains".into());
                break false;
            }
            Err(error) => {
                cleanup = Some(error.message);
                break false;
            }
        }
    };
    let mut complete = false;
    if detached {
        match execution.handle().attach() {
            Ok(next) => {
                if let Err(error) = next.release() {
                    cleanup = Some(error.message);
                }
            }
            Err(error) => cleanup = Some(error.message),
        }
        while cleanup.is_none() {
            match execution.poll_close() {
                Ok(ExecutionCloseProgress::Closed(receipt)) => {
                    complete = receipt.worker_stopped && receipt.unacknowledged == 0;
                    if !complete {
                        cleanup = Some(format!("Incomplete original execution: {receipt:?}"));
                    }
                    break;
                }
                Ok(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(1)),
                Ok(_) => {
                    cleanup = Some("Original SDK worker close remains".into());
                    break;
                }
                Err(error) => {
                    cleanup = Some(error.message);
                    break;
                }
            }
        }
    }
    let mut retained = Box::new(RetainedFailure {
        primary,
        cleanup,
        diagnostic_error: None,
        surface: RetainedSurface::Mounted(Box::new(panel)),
        _execution: execution,
    });
    retained.report(complete);
    if retained.primary.is_some()
        || retained.cleanup.is_some()
        || retained.diagnostic_error.is_some()
        || !complete
    {
        return Err(retained);
    }
    Ok(())
}
fn event(panel: &MountedJobDetails, binding: &str) -> PendingCallback {
    let path = panel
        .document
        .try_with(|document| {
            document
                .module_projection()
                .unwrap()
                .callback_bindings()
                .into_iter()
                .find(|(_, callback)| callback.reference.binding == binding)
                .unwrap()
                .0
        })
        .unwrap();
    assert!(
        panel
            .document
            .dispatch_action(Action::Activate { path })
            .unwrap()
    );
    let (mut pending, _) = panel
        .document
        .try_reproject(&panel.document.stamp().unwrap(), |document| {
            Ok::<_, String>(document.drain_callbacks())
        })
        .unwrap();
    assert_eq!(pending.len(), 1);
    pending.pop().unwrap()
}
fn settle(panel: &MountedJobDetails) {
    let end = Instant::now() + Duration::from_secs(3);
    while panel.owner.borrow().as_ref().unwrap().pending().is_some() {
        panel.after_dispatch(0).unwrap();
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn actual_readonly_sdk_record_mounts_through_fixed_panel_and_retires_old_refresh_callback()
-> Result<(), Box<RetainedFailure>> {
    let (client, expected) = readonly_client();
    with_panel(client, |panel, calls| {
        assert!(calls.lock().unwrap().is_empty());
        let pending = event(panel, "onRefreshJob");
        panel
            .admit(&panel.document.stamp().unwrap(), &pending)
            .unwrap();
        assert!(panel.owner.borrow().as_ref().unwrap().view.loading);
        assert!(
            panel
                .admit(&panel.document.stamp().unwrap(), &pending)
                .is_err()
        );
        assert!(!panel.after_dispatch(1).unwrap());
        settle(panel);
        let owner = panel.owner.borrow();
        let record = owner.as_ref().unwrap().view.record().unwrap();
        assert_eq!(record.job_id, expected.job_id);
        assert_eq!(record.revision, expected.revision);
        assert_eq!(record.input_sha256, expected.input_sha256);
        assert_eq!(calls.lock().unwrap().len(), 1);
        assert_eq!(calls.lock().unwrap()[0].command, "jobs.get");
        drop(owner);
        assert!(
            panel
                .admit(&panel.document.stamp().unwrap(), &pending)
                .is_err()
        );
        let next = event(panel, "onRefreshJob");
        panel
            .admit(&panel.document.stamp().unwrap(), &next)
            .unwrap();
        settle(panel);
        assert_eq!(calls.lock().unwrap().len(), 2);
        assert!(
            calls
                .lock()
                .unwrap()
                .iter()
                .all(|request| request.command == "jobs.get")
        );
    })
}
#[test]
fn default_sdk_unsupported_is_displayed_without_inventing_a_job_state()
-> Result<(), Box<RetainedFailure>> {
    with_panel(Arc::new(PlayerSdk::default()), |panel, calls| {
        let pending = event(panel, "onRefreshJob");
        panel
            .admit(&panel.document.stamp().unwrap(), &pending)
            .unwrap();
        settle(panel);
        let owner = panel.owner.borrow();
        let owner = owner.as_ref().unwrap();
        assert!(owner.view.record().is_none());
        assert!(owner.view.diagnostic.contains("Unsupported"));
        assert!(matches!(
            owner.retained().unwrap().event.event,
            PluginEventKind::Failed(_)
        ));
        assert_eq!(calls.lock().unwrap().len(), 1);
    })
}
#[test]
fn close_drains_only_original_query_and_rejects_old_callback_without_cancelling_the_job()
-> Result<(), Box<RetainedFailure>> {
    with_panel(Arc::new(PlayerSdk::default()), |panel, calls| {
        let pending = event(panel, "onRefreshJob");
        panel
            .admit(&panel.document.stamp().unwrap(), &pending)
            .unwrap();
        assert!(panel.owner.borrow().as_ref().unwrap().pending().is_some());
        let deadline = Instant::now() + Duration::from_secs(3);
        while !panel.close_query_owner().unwrap() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        panel
            .document
            .try_with(|document| {
                let buttons = document
                    .elements()
                    .iter()
                    .filter(|element| element.dom_tag == "button")
                    .collect::<Vec<_>>();
                assert_eq!(buttons.len(), 2);
                assert!(
                    buttons
                        .iter()
                        .all(|element| element.attributes.contains_key("disabled"))
                );
            })
            .unwrap();
        assert!(
            panel
                .admit(&panel.document.stamp().unwrap(), &pending)
                .is_err()
        );
        assert!(
            calls
                .lock()
                .unwrap()
                .iter()
                .all(|request| request.command == "jobs.get")
        );
    })
}
