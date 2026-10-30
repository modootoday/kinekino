//! Original LocalHost/SDK/product graph fixture. Synthetic notices prove policy only.
use super::*;
use crate::{
    application::{
        api::{
            ApiError, ApplicationClient, ApplicationClientHandle, ErrorCode, PlayerSdk, Request,
            Response,
        },
        sdk_execution::{CompletionNotifier, ExecutionBudget, ExecutionHandle, ExecutionOwner},
    },
    assembly::meari::{
        player_shell_host::StartRouteObservation,
        player_shell_startup::{ShellInputs, prepare_shell},
        viewer_cleanup::ViewerNoChild,
    },
    domain::api_values::DecimalU64,
    infrastructure::local_host::{EndpointClient, LocalHost, PendingLocalHost, TransportOptions},
    presentation::web_ui_adapter::{
        player_shell::{
            ShellState,
            document::{
                StartLabels, enable_read_panel, product_frame::ProductFrameBlueprint, read_props,
            },
        },
        proposals::{
            bootstrap::BootstrapPanel, frame_composition::FrameComposition,
            product_window::product_modules_for_panel,
        },
    },
};
use kinekino_jobs::{
    application::{JobStore, SourceBlobStore},
    domain::{JobRecord, PreparedJobInput},
    infrastructure::{JobFiles, RootedJobPaths},
};
use meari_native::{
    application::window_lifecycle::{
        LifecycleLimits, MonotonicClock, ReadyKind, WindowLifecycle, WindowStamp,
    },
    domain::{Viewport, graphics::FrameTicket, widgets::Action},
    infrastructure::{
        document::InteractiveDocument,
        window::{
            NativeFrameObservation, NativeFrameSubmission, NativeWindowNotice,
            NativeWindowNoticeKind,
        },
    },
};
use std::{
    any::Any,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::Waker,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

mod rooted_paths {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/infrastructure/immutable_storage_paths.rs"
    ));
}
struct Paths;
impl RootedJobPaths for Paths {
    fn root(&self, path: &Path) -> Result<(PathBuf, Vec<File>), ApiError> {
        rooted_paths::root(path)
    }
    fn directory(&self, path: &Path) -> Result<File, ApiError> {
        rooted_paths::directory(path)
    }
    fn file(&self, path: &Path) -> Result<File, ApiError> {
        rooted_paths::file(path)
    }
    fn lock(&self, path: &Path) -> Result<File, ApiError> {
        rooted_paths::lock(path)
    }
}
#[derive(Default)]
struct Notify {
    pending: AtomicBool,
    requests: AtomicUsize,
    fail: AtomicBool,
}
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), ApiError> {
        self.requests.fetch_add(1, Ordering::AcqRel);
        if self.fail.load(Ordering::Acquire) {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original continuation wake delivery fault",
            ));
        }
        self.pending.store(true, Ordering::Release);
        Ok(())
    }
}
struct Clock;
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}
#[derive(Default)]
pub(super) struct Probe {
    pub calls: Mutex<Vec<Request>>,
    pub responses: Mutex<Vec<serde_json::Value>>,
    pub hold_get: AtomicBool,
    pub get_entered: AtomicBool,
    pub fail_get: AtomicBool,
}
struct ObservedClient {
    actual: ApplicationClientHandle,
    probe: Arc<Probe>,
}
/// Retained-callback cases tap the real native drain without applying the event twice.
/// Normal click/tick paths delegate admission and effects to the original ViewerHost.
struct ObservedHost {
    inner: ViewerHost,
    capture_only: bool,
    captured: Vec<PendingCallback>,
}
impl NativeWindowHost for ObservedHost {
    type Error = ViewerHostError;
    type CloseCheckpoint = crate::assembly::meari::viewer_host::ViewerCloseCheckpoint;
    fn admit_callback(
        &mut self,
        stamp: &HostStamp,
        pending: &PendingCallback,
    ) -> IntentAdmission<Self::Error> {
        if self.capture_only {
            self.captured.push(PendingCallback {
                handle: pending.handle.clone(),
                invocation: pending.invocation.clone(),
            });
            IntentAdmission::Accepted
        } else {
            self.inner.admit_callback(stamp, pending)
        }
    }
    fn after_dispatch(&mut self, turn: HostTurn) -> Result<HostEffects, Self::Error> {
        if self.capture_only {
            Ok(HostEffects::default())
        } else {
            self.inner.after_dispatch(turn)
        }
    }
    fn retry_after_dispatch(&self, error: &Self::Error) -> Option<HostRetry> {
        self.inner.retry_after_dispatch(error)
    }
    fn can_close(&mut self, stamp: &HostStamp, cause: CloseCause) -> Result<bool, Self::Error> {
        self.inner.can_close(stamp, cause)
    }
    fn close(
        &mut self,
        request: CloseRequest<Self::CloseCheckpoint>,
    ) -> CloseDecision<Self::CloseCheckpoint, Self::Error> {
        self.inner.close(request)
    }
    fn observe_close(
        &mut self,
        request: CloseRequest<Self::CloseCheckpoint>,
    ) -> CloseDecision<Self::CloseCheckpoint, Self::Error> {
        self.inner.observe_close(request)
    }
    fn next_close_observation(&self, checkpoint: &Self::CloseCheckpoint) -> HostRetry {
        self.inner.next_close_observation(checkpoint)
    }
}
impl ApplicationClient for ObservedClient {
    fn execute(&self, request: Request) -> Response {
        self.probe.calls.lock().unwrap().push(request.clone());
        if request.command == "jobs.get" {
            self.probe.get_entered.store(true, Ordering::Release);
            let deadline = Instant::now() + Duration::from_secs(5);
            while self.probe.hold_get.load(Ordering::Acquire) && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        let request_id = request.context.request_id.clone();
        let is_get = request.command == "jobs.get";
        let response = self.actual.execute(request);
        self.probe
            .responses
            .lock()
            .unwrap()
            .push(serde_json::to_value(&response).unwrap());
        if is_get && self.probe.fail_get.swap(false, Ordering::AcqRel) {
            return Response::failure(
                request_id,
                ApiError::new(
                    ErrorCode::EngineFailed,
                    "Original get response delivery fault",
                ),
            );
        }
        response
    }
}
pub(super) struct Rig {
    pub host: Option<ViewerHost>,
    runtime: Option<NativeHostRuntime<ObservedHost>>,
    endpoint: Option<LocalHost>,
    pending_endpoint: Option<PendingLocalHost>,
    execution: Option<ExecutionOwner>,
    pub handle: Option<ExecutionHandle>,
    pub probe: Arc<Probe>,
    completion: Arc<Notify>,
    pub records: Vec<JobRecord>,
    pub observer: Option<Rc<RefCell<StartRouteObservation>>>,
    window: Option<WindowStamp>,
    sequence: u64,
    pub root: Option<PathBuf>,
}
impl Rig {
    fn empty() -> Self {
        Self {
            host: None,
            runtime: None,
            endpoint: None,
            pending_endpoint: None,
            execution: None,
            handle: None,
            probe: Arc::new(Probe::default()),
            completion: Arc::new(Notify::default()),
            records: Vec::new(),
            observer: None,
            window: None,
            sequence: 0,
            root: None,
        }
    }
    fn setup(&mut self, supported: bool, seed: bool) -> Result<(), String> {
        let root = std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join("runtime/quality-review/ui-jobs-product-route")
            .join(format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_nanos()
            ));
        self.root = Some(root.clone());
        std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let files = JobFiles::open(&root, Arc::new(Paths)).map_err(|e| e.message)?;
        let source = files
            .put_utf8("1\n00:00:00,000 --> 00:00:01,000\nOriginal fixture\n")
            .map_err(|e| e.message)?;
        for digit in ['a', 'b'].into_iter().take(if seed { 2 } else { 0 }) {
            let job = JobRecord::queued(
                format!("job-{}", digit.to_string().repeat(32))
                    .try_into()
                    .map_err(str::to_owned)?,
                PreparedJobInput {
                    operation: "subtitle.convert".into(),
                    parameters: serde_json::json!({"fixture":digit.to_string()}),
                    source: source.clone(),
                    execution_timeout_ms: DecimalU64(30000),
                },
            )
            .map_err(str::to_owned)?;
            files.admit(&job).map_err(|e| e.message)?;
            self.records.push(job);
        }
        drop(files);
        let actual: ApplicationClientHandle =
            if supported {
                Arc::new(PlayerSdk::default().with_jobs(
                    crate::infrastructure::jobs::read_only(&root).map_err(|e| e.message)?,
                ))
            } else {
                Arc::new(PlayerSdk::default())
            };
        self.endpoint =
            Some(LocalHost::start(actual, TransportOptions::default()).map_err(|e| e.message)?);
        let endpoint = EndpointClient::new(
            self.endpoint.as_ref().unwrap().endpoint(),
            TransportOptions::default(),
        )
        .map_err(|e| e.message)?;
        let client: ApplicationClientHandle = Arc::new(ObservedClient {
            actual: Arc::new(endpoint),
            probe: self.probe.clone(),
        });
        self.execution = Some(
            ExecutionOwner::start(
                client.clone(),
                ExecutionBudget {
                    max_operations: 8,
                    max_request_bytes: 262144,
                    max_total_request_bytes: 8 * 262144,
                    max_terminal_bytes: 262144,
                    max_total_terminal_bytes: 8 * 262144,
                },
                self.completion.clone(),
            )
            .map_err(|e| e.message)?,
        );
        self.handle = Some(self.execution.as_ref().unwrap().handle());
        self.host = Some(ViewerHost::new_with_completion(
            client.clone(),
            self.execution.take().unwrap(),
            self.endpoint.take().unwrap(),
            self.completion.clone(),
        ));
        // All fallible graph/DOM work follows the original host's retained ownership.
        let host = self.host.as_mut().unwrap();
        let labels = StartLabels {
            connection: "Not observed".into(),
            time: "Not observed".into(),
        };
        let props = read_props(&labels);
        let mut prepared = prepare_shell(
            ShellInputs {
                client,
                host_owner: (),
            },
            labels,
        )
        .map_err(|_| "Product preparation".to_owned())?;
        enable_read_panel(&mut prepared.request.document, props.clone())
            .map_err(|_| "Read panel source".to_owned())?;
        let (entry, mut inventory) = product_modules_for_panel(BootstrapPanel::Card);
        let graph = meari::parse_module_graph(&entry, &inventory).map_err(|e| format!("{e:?}"))?;
        let (studio, _) = FrameComposition::new(
            crate::domain::product::PRODUCT.display_name.into(),
            String::new(),
        )
        .prepare(graph, &mut inventory)
        .map_err(|e| format!("{e:?}"))?;
        let frame = ProductFrameBlueprint::prepare(
            prepared.request.document.graph,
            prepared.request.document.inventory,
            studio,
            inventory,
        )?;
        let program = frame.mount_viewing(
            props.clone(),
            "jobs-product-route".into(),
            1,
            NonZeroU64::new(1).unwrap(),
        )?;
        let callbacks = host.callbacks();
        let document = InteractiveDocument::from_module_program_with_callbacks(
            program,
            "",
            Viewport {
                width: 1280,
                height: 720,
                dark: true,
            },
            None,
            1,
            move |scope| callbacks.borrow_mut().export(scope).map_err(|e| e.message),
        )
        .map_err(|e| format!("{e:?}"))?;
        host.bind_product_document(
            NativeDocumentHandle::new(document, 1).map_err(|e| format!("{e:?}"))?,
            props,
            frame,
        );
        let mut state = ShellState::new();
        let ticket = state
            .request_route(
                &RouteCatalog::bundled().map_err(|e| format!("{e:?}"))?,
                "view.start",
                NavigationGuards::default(),
            )
            .map_err(|e| format!("{e:?}"))?;
        let observer = Rc::new(RefCell::new(StartRouteObservation::new(state, ticket)));
        host.bind_route_observation(observer.clone());
        self.observer = Some(observer);
        self.window = Some(
            WindowLifecycle::<ViewerNoChild>::new(
                LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
                Clock,
                Waker::noop().clone(),
            )
            .map_err(|e| format!("{e:?}"))?
            .ready(ReadyKind::NewWindow)
            .map_err(|e| format!("{e:?}"))?,
        );
        self.frame();
        self.runtime = Some(NativeHostRuntime::new(
            self.document(),
            ObservedHost {
                inner: self.host.take().unwrap(),
                capture_only: false,
                captured: Vec::new(),
            },
        ));
        Ok(())
    }
    pub fn host(&self) -> &ViewerHost {
        self.runtime
            .as_ref()
            .map(|runtime| &runtime.host().inner)
            .or(self.host.as_ref())
            .unwrap()
    }
    pub fn host_mut(&mut self) -> &mut ViewerHost {
        if let Some(runtime) = &mut self.runtime {
            &mut runtime.host_mut().inner
        } else {
            self.host.as_mut().unwrap()
        }
    }
    pub fn document(&self) -> NativeDocumentHandle {
        self.host().document.as_ref().unwrap().clone()
    }
    pub fn turn(&self) -> HostTurn {
        HostTurn {
            stamp: self.document().stamp().unwrap(),
            reason: HostTurnReason::Completion,
            accepted_callbacks: 0,
            rejected_callbacks: 0,
            remaining_callbacks: 0,
        }
    }
    pub fn tick(&mut self) {
        self.runtime
            .as_mut()
            .unwrap()
            .after_dispatch(HostTurnReason::Completion)
            .unwrap();
    }
    pub fn wake_requests(&self) -> usize {
        self.completion.requests.load(Ordering::Acquire)
    }
    pub fn fail_wake(&self) {
        self.completion.fail.store(true, Ordering::Release);
    }
    pub fn drive_wake(&mut self) -> bool {
        if !self.completion.pending.swap(false, Ordering::AcqRel) {
            return false;
        }
        self.tick();
        true
    }
    pub fn wait_wakes(&mut self, predicate: impl Fn(&Self) -> bool) {
        let end = Instant::now() + Duration::from_secs(3);
        while !predicate(self) {
            self.drive_wake();
            assert!(
                Instant::now() < end,
                "Original completion did not advance route"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    pub fn drain_wakes(&mut self) {
        for _ in 0..32 {
            if !self.drive_wake() {
                return;
            }
        }
        panic!("Normal continuation did not stop within the fixture bound");
    }
    pub fn detail_ready(&self, index: usize) -> bool {
        let text = self.text();
        self.host()
            .jobs
            .borrow()
            .document
            .as_ref()
            .is_some_and(|document| document.detail_result_ready(&self.records[index].job_id))
            && text.contains(self.records[index].job_id.as_str())
            && text.contains(&self.records[index].input.source.sha256)
            && self.enabled("onRefreshJob")
    }
    pub fn wait(&mut self, predicate: impl Fn(&Self) -> bool) {
        let end = Instant::now() + Duration::from_secs(3);
        while !predicate(self) {
            self.tick();
            assert!(Instant::now() < end, "Original route did not settle");
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    pub fn calls(&self, command: &str) -> usize {
        self.probe
            .calls
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.command == command)
            .count()
    }
    pub fn observed(&self) -> String {
        self.observer
            .as_ref()
            .unwrap()
            .borrow()
            .state
            .observed_route()
            .map(|r| r.as_str().to_owned())
            .unwrap_or_default()
    }
    pub fn frame(&mut self) {
        self.sequence += 1;
        let current = self.document().stamp().unwrap();
        let window = self.window.unwrap();
        let notice = NativeWindowNotice {
            revision: self.sequence,
            host_id: current.host_id,
            kind: NativeWindowNoticeKind::FrameSubmitted,
            window: Some(window),
            geometry: None,
            frame: Some(NativeFrameObservation {
                retained: None,
                document: current.clone(),
                submission: NativeFrameSubmission {
                    ticket: FrameTicket {
                        context_id: "synthetic-jobs-route-policy".into(),
                        context_generation: 1,
                        scene_generation: self.sequence,
                    },
                    width: 1280,
                    height: 720,
                    commands: 0,
                },
            }),
        };
        self.observer
            .as_ref()
            .unwrap()
            .borrow_mut()
            .observe_synthetic_notice_for_contract_test(&notice, &current, window);
    }
    pub fn text(&self) -> String {
        self.document()
            .try_with(|d| {
                d.elements()
                    .iter()
                    .flat_map(|e| {
                        e.text_runs
                            .iter()
                            .map(|run| run.text.clone())
                            .chain(e.text_override.clone())
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap()
    }
    fn path(&self, binding: &str, label: Option<&str>) -> String {
        let handle = self.document();
        handle
            .try_with(|d| {
                d.module_projection()
                    .unwrap()
                    .callback_bindings()
                    .into_iter()
                    .find(|(path, callback)| {
                        callback.reference.binding == binding
                            && label.is_none_or(|label| {
                                d.elements().iter().any(|e| {
                                    e.owns_path(path)
                                        && e.attributes
                                            .get("aria-label")
                                            .is_some_and(|value| value == label)
                                })
                            })
                    })
                    .expect("Real enabled callback path")
                    .0
            })
            .unwrap()
    }
    pub fn pending(&mut self, binding: &str, label: Option<&str>) -> PendingCallback {
        let handle = self.document();
        let path = self.path(binding, label);
        assert!(handle.dispatch_action(Action::Activate { path }).unwrap());
        let runtime = self.runtime.as_mut().unwrap();
        runtime.host_mut().capture_only = true;
        let result = runtime.after_dispatch(HostTurnReason::Input);
        runtime.host_mut().capture_only = false;
        result.unwrap();
        let mut pending = std::mem::take(&mut runtime.host_mut().captured);
        assert_eq!(pending.len(), 1);
        pending.remove(0)
    }
    pub fn admit(&mut self, pending: &PendingCallback) {
        let stamp = self.document().stamp().unwrap();
        assert!(matches!(
            self.host_mut().admit_callback(&stamp, pending),
            IntentAdmission::Accepted
        ));
    }
    pub fn click(&mut self, binding: &str, label: Option<&str>) {
        let path = self.path(binding, label);
        assert!(
            self.document()
                .dispatch_action(Action::Activate { path })
                .unwrap()
        );
        self.runtime
            .as_mut()
            .unwrap()
            .after_dispatch(HostTurnReason::Input)
            .unwrap();
    }
    pub fn enabled(&self, binding: &str) -> bool {
        self.document()
            .try_with(|d| {
                d.module_projection()
                    .unwrap()
                    .callback_bindings()
                    .into_iter()
                    .any(|(path, callback)| {
                        callback.reference.binding == binding
                            && d.elements().iter().any(|e| {
                                e.owns_path(&path) && !e.attributes.contains_key("disabled")
                            })
                    })
            })
            .unwrap()
    }
    pub fn enter(&mut self) {
        self.click("onOpenJobs", None);
        self.wait(|r| {
            r.host()
                .jobs
                .borrow()
                .entry
                .as_ref()
                .is_some_and(|e| e.phase == EntryPhase::Armed)
        });
        assert_eq!(self.observed(), "view.start");
        assert_eq!(self.calls("jobs.list"), 0);
        self.frame();
        self.tick();
        self.wait(|r| r.calls("jobs.list") == 1 && r.enabled("onRefreshJobs"));
    }
    pub fn select(&mut self, index: usize) {
        let id = self.records[index].job_id.as_str().to_owned();
        self.click("onSelectJob", Some(&id));
    }
    pub fn begin_return(&mut self) {
        let pending = self.pending("onNavigate", Some("시청으로 돌아가기"));
        self.admit(&pending);
        let cell = self.host().jobs.clone();
        let mut jobs = cell.borrow_mut();
        jobs.document.as_mut().unwrap().advance(0).unwrap();
        self.host_mut().request_jobs_return(&mut jobs).unwrap();
        drop(jobs);
        self.tick();
    }
    fn cleanup(&mut self) -> Result<(), String> {
        self.probe.hold_get.store(false, Ordering::Release);
        let end = Instant::now() + Duration::from_secs(5);
        if self.host.is_some() || self.runtime.is_some() {
            let host = self.host_mut();
            loop {
                match host.poll_unrun_shutdown() {
                    Ok(crate::assembly::meari::viewer_host::ViewerCloseProgress::Complete {
                        execution,
                        ..
                    }) if execution.worker_stopped && execution.unacknowledged == 0 => break,
                    Ok(_) => (),
                    Err(error) => return Err(format!("{error:?}")),
                }
                if Instant::now() >= end {
                    return Err("Original host close pending".into());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        if self.pending_endpoint.is_none() {
            self.pending_endpoint = self.endpoint.take().map(LocalHost::begin_shutdown);
        }
        if let Some(endpoint) = &mut self.pending_endpoint {
            while !endpoint.poll().is_quiescent() {
                if Instant::now() >= end {
                    return Err("Original endpoint close pending".into());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        if let Some(owner) = &mut self.execution {
            loop {
                if let crate::application::sdk_execution::ExecutionCloseProgress::Closed(receipt) =
                    owner.poll_close().map_err(|e| e.message)?
                {
                    if receipt.worker_stopped && receipt.unacknowledged == 0 {
                        break;
                    }
                    return Err("Original owner incomplete".into());
                }
                if Instant::now() >= end {
                    return Err("Original owner close pending".into());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        Ok(())
    }
}
pub(super) struct CaseFailure {
    setup: Option<String>,
    setup_panic: Option<Box<dyn Any + Send>>,
    primary: Option<Box<dyn Any + Send>>,
    cleanup: Option<String>,
    cleanup_panic: Option<Box<dyn Any + Send>>,
    diagnostic: Option<std::io::Error>,
    _rig: Rig,
}
impl std::fmt::Debug for CaseFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginalJobsProductRouteFailure")
            .field("setup", &self.setup)
            .field("setup_panic", &self.setup_panic.is_some())
            .field("primary_panic", &self.primary.is_some())
            .field("cleanup", &self.cleanup)
            .field("cleanup_panic", &self.cleanup_panic.is_some())
            .field("diagnostic", &self.diagnostic)
            .finish()
    }
}
pub(super) fn with_rig(
    supported: bool,
    body: impl FnOnce(&mut Rig),
) -> Result<(), Box<CaseFailure>> {
    run_rig(supported, true, body)
}
pub(super) fn with_empty_rig(body: impl FnOnce(&mut Rig)) -> Result<(), Box<CaseFailure>> {
    run_rig(true, false, body)
}
fn run_rig(
    supported: bool,
    seed: bool,
    body: impl FnOnce(&mut Rig),
) -> Result<(), Box<CaseFailure>> {
    let mut rig = Rig::empty();
    let (setup, setup_panic) =
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| rig.setup(supported, seed)))
        {
            Ok(result) => (result.err(), None),
            Err(panic) => (None, Some(panic)),
        };
    let primary = if setup.is_none() && setup_panic.is_none() {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(&mut rig))).err()
    } else {
        None
    };
    let (cleanup, cleanup_panic) =
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| rig.cleanup())) {
            Ok(result) => (result.err(), None),
            Err(panic) => (None, Some(panic)),
        };
    let mut retained = Box::new(CaseFailure {
        setup,
        setup_panic,
        primary,
        cleanup,
        cleanup_panic,
        diagnostic: None,
        _rig: rig,
    });
    let mut output = std::io::stderr().lock();
    retained.diagnostic = writeln!(
        output,
        "jobs_product_route_closeout complete={} primary_panic={} setup_panic={} cleanup_panic={}",
        retained.cleanup.is_none() && retained.cleanup_panic.is_none(),
        retained.primary.is_some(),
        retained.setup_panic.is_some(),
        retained.cleanup_panic.is_some()
    )
    .and_then(|()| output.flush())
    .err();
    if retained.setup.is_some()
        || retained.setup_panic.is_some()
        || retained.primary.is_some()
        || retained.cleanup.is_some()
        || retained.cleanup_panic.is_some()
        || retained.diagnostic.is_some()
    {
        return Err(retained);
    }
    Ok(())
}
