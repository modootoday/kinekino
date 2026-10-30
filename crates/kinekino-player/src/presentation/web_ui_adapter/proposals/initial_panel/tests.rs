use super::super::IntentPins;
use super::super::document_factory::{ExplicitPanelSelection, PanelModuleInput};
use super::super::panel_model::{PanelBudget, PanelLabels};
use super::*;
use kinekino_player::{
    application::{
        api::{
            API_VERSION, ApplicationClient, ApplicationClientHandle, ErrorCode, Request, Response,
        },
        sdk_execution::{CompletionNotifier, ExecutionBudget},
    },
    domain::api_values::DecimalU64,
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::AsyncCodecBudget;
use meari::domain::build::ContentHash;
use meari_native::domain::Viewport;
use std::{
    cell::Cell,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

struct Signals(mpsc::Sender<()>);
impl CompletionNotifier for Signals {
    fn notify(&self) -> Result<(), ApiError> {
        let _ = self.0.send(());
        Ok(())
    }
}
struct Client {
    calls: AtomicUsize,
    entered: mpsc::Sender<()>,
    gate: (Mutex<bool>, Condvar),
    fail: bool,
}
impl ApplicationClient for Client {
    fn execute(&self, request: Request) -> Response {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let _ = self.entered.send(());
        let mut ready = self.gate.0.lock().unwrap();
        while !*ready {
            ready = self.gate.1.wait(ready).unwrap();
        }
        Response {
            api_version: API_VERSION,
            request_id: request.context.request_id,
            ok: !self.fail,
            data: (!self.fail).then(|| serde_json::to_value(seed()).unwrap()),
            error: self
                .fail
                .then(|| ApiError::new(ErrorCode::NotFound, "canonical initial failure")),
        }
    }
}
fn seed() -> StoredProposal {
    serde_json::from_value(serde_json::json!({"document":{"schema_version":"1","proposal_id":"initial-a","source":{"source_id":"source-a","source_revision":"7","source_sha256":"a".repeat(64),"cues":[{"cue_id":"a","text":"원문","start_us":null,"end_us":null}]},"generated":{"task":"cards","raw_text":"preserved raw"},"draft_revision":"2","state":"editable","draft":{"task":"cards","cards":[{"front":"first","back":"back","source_refs":["a"]},{"front":"second","back":"second back","source_refs":["a"]}]},"import_error":null},"acceptance":null})).unwrap()
}
type Setup = (
    ExecutionOwner,
    Arc<Client>,
    mpsc::Receiver<()>,
    mpsc::Receiver<()>,
);
fn setup(block: bool, fail: bool) -> Setup {
    let (entered, rx) = mpsc::channel();
    let (notify, signals) = mpsc::channel();
    let client = Arc::new(Client {
        calls: AtomicUsize::new(0),
        entered,
        gate: (Mutex::new(!block), Condvar::new()),
        fail,
    });
    let owner = ExecutionOwner::start(
        client.clone(),
        ExecutionBudget {
            max_operations: 4,
            max_request_bytes: 8192,
            max_total_request_bytes: 32768,
            max_terminal_bytes: 65536,
            max_total_terminal_bytes: 262144,
        },
        Arc::new(Signals(notify)),
    )
    .unwrap();
    (owner, client, rx, signals)
}
fn release(client: &Client) {
    *client.gate.0.lock().unwrap() = true;
    client.gate.1.notify_all();
}
fn selection() -> BootstrapSelection {
    BootstrapSelection {
        proposal_id: "initial-a".into(),
        document_revision: DecimalU64(2),
        source_id: "source-a".into(),
        source_revision: DecimalU64(7),
        source_sha256: "a".repeat(64),
        target: BootstrapTarget::Card { index: 1 },
    }
}
fn labels() -> PanelLabels {
    PanelLabels {
        feedback:
            crate::presentation::web_ui_adapter::proposals::review_feedback::FeedbackLabels::korean(
            ),
        revision: "문서 2".into(),
        source_identity: "원문".into(),
        quality_unknown: "품질 미판정".into(),
        selected_item: "두 번째 항목".into(),
        range: String::new(),
        result: String::new(),
    }
}
fn bootstrap_labels() -> BootstrapLabels {
    BootstrapLabels {
        panel: labels(),
        source_count_label: String::new(),
        unselected_source_text: String::new(),
    }
}
fn start(owner: ExecutionOwner) -> InitialPanelStartup {
    InitialPanelStartup::start(owner, RequestContext::default(), selection(), 65536)
        .unwrap_or_else(|failure| panic!("{}", failure.error.message))
}
fn await_preparation<I: InitialInspectionPort>(
    startup: &mut InitialPanelStartup<I>,
    signals: &mpsc::Receiver<()>,
) -> InitialPreparationState {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let state = startup
            .prepare(bootstrap_labels(), None)
            .unwrap_or_else(|_| panic!("canonical preparation"));
        if !matches!(state, InitialPreparationState::Loading) {
            return state;
        }
        signals
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap();
    }
}
fn close<I: InitialInspectionPort>(startup: &mut InitialPanelStartup<I>) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while startup.close().is_err() {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
}

struct RetryAck {
    inner: InitialProposalInspection,
    fail_once: bool,
    attempts: Cell<usize>,
    successful: Cell<usize>,
}
impl InitialInspectionPort for RetryAck {
    fn peek_terminal(&mut self) -> Result<Option<Arc<Result<StoredProposal, ApiError>>>, ApiError> {
        self.inner.peek_terminal()
    }
    fn retained_terminal(&self) -> Result<Option<Arc<ExecutionTerminal>>, ApiError> {
        self.inner.retained_terminal()
    }
    fn acknowledge(&mut self) -> Result<(), ApiError> {
        self.attempts.set(self.attempts.get() + 1);
        if self.fail_once {
            self.fail_once = false;
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "technical ACK transport failure",
            ));
        }
        self.inner.acknowledge()?;
        self.successful.set(self.successful.get() + 1);
        Ok(())
    }
    fn cancel(&self) -> Result<(), ApiError> {
        self.inner.cancel()?;
        Ok(())
    }
}
fn retry_ack(startup: InitialPanelStartup) -> InitialPanelStartup<RetryAck> {
    let InitialPanelStartup {
        initial,
        owner,
        selection,
        prepared,
        accepted_failure,
        accepted_shutdown,
        acknowledged,
        closing,
    } = startup;
    InitialPanelStartup {
        initial: RetryAck {
            inner: initial,
            fail_once: true,
            attempts: Cell::new(0),
            successful: Cell::new(0),
        },
        owner,
        selection,
        prepared,
        accepted_failure,
        accepted_shutdown,
        acknowledged,
        closing,
    }
}
#[test]
fn selection_mismatch_shutdown_acceptance_keeps_arcs_and_never_authorizes_live_mount() {
    let (owner, client, _, signals) = setup(false, false);
    let mut startup = retry_ack(start(owner));
    let mut bad = selection();
    bad.target = BootstrapTarget::Card { index: 99 };
    startup.select_before_preparation(bad).unwrap();
    signals.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(matches!(
        startup.prepare(bootstrap_labels(), None),
        Err(InitialPrepareError::Selection(
            BootstrapError::ItemOutOfRange
        ))
    ));
    let typed = startup.initial.peek_terminal().unwrap().unwrap();
    let raw = startup.retained_terminal().unwrap().unwrap();
    assert!(matches!(
        startup.poll_close().unwrap(),
        ExecutionCloseProgress::TerminalAckPending {
            unacknowledged: 1,
            ..
        }
    ));
    assert_eq!(startup.initial.attempts.get(), 0);
    assert!(startup.acknowledge_for_shutdown().is_err());
    let retained = startup.accepted_shutdown.as_ref().unwrap().clone();
    assert!(Arc::ptr_eq(retained.snapshot(), &typed));
    assert!(Arc::ptr_eq(retained.raw_terminal(), &raw));
    let accepted = startup.acknowledge_for_shutdown().unwrap();
    assert!(Arc::ptr_eq(&accepted, &retained));
    let again = startup.acknowledge_for_shutdown().unwrap();
    assert!(Arc::ptr_eq(&accepted, &again));
    assert_eq!(startup.initial.attempts.get(), 2);
    assert_eq!(startup.initial.successful.get(), 1);
    let mut startup = startup
        .into_handoff()
        .err()
        .expect("shutdown acceptance cannot mount");
    close(&mut startup);
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
}
#[test]
fn ack_transport_failure_retains_preparation_and_retries_only_ack_before_mount() {
    let (owner, client, _, signals) = setup(false, false);
    let mut startup = retry_ack(start(owner));
    signals.recv_timeout(Duration::from_secs(3)).unwrap();
    startup
        .prepare(bootstrap_labels(), None)
        .unwrap_or_else(|_| panic!("prepare"));
    let prepared = startup.prepared().unwrap().clone();
    assert!(startup.acknowledge_prepared().is_err());
    assert!(Arc::ptr_eq(startup.prepared().unwrap(), &prepared));
    let mut startup = startup
        .into_handoff()
        .err()
        .expect("mount forbidden while ACK failed");
    startup.acknowledge_prepared().unwrap();
    assert!(Arc::ptr_eq(startup.prepared().unwrap(), &prepared));
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    close(&mut startup);
}
#[test]
fn canonical_failure_ack_retry_retains_same_raw_and_typed_acceptance() {
    let (owner, client, _, signals) = setup(false, true);
    let mut startup = retry_ack(start(owner));
    signals.recv_timeout(Duration::from_secs(3)).unwrap();
    let InitialPreparationState::Failed(snapshot) = startup
        .prepare(bootstrap_labels(), None)
        .unwrap_or_else(|_| panic!("failure"))
    else {
        panic!("failed")
    };
    assert!(startup.acknowledge_failure().is_err());
    let accepted = startup.accepted_failure().unwrap().clone();
    assert!(Arc::ptr_eq(accepted.snapshot(), &snapshot));
    let result = startup.acknowledge_failure().unwrap();
    assert!(Arc::ptr_eq(&result, &accepted));
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    close(&mut startup);
}
fn input(client: ApplicationClientHandle) -> PanelFactoryInput {
    let (entry, inventory) = super::super::product_window::product_modules_for_panel(
        super::super::bootstrap::BootstrapPanel::Card,
    );
    PanelFactoryInput {
        client,
        context: RequestContext::default(),
        selection: ExplicitPanelSelection {
            proposal: InspectProposal {
                proposal_id: "initial-a".into(),
            },
            source_id: "source-a".into(),
            source_revision: DecimalU64(7),
            source_sha256: "a".repeat(64),
            item_index: Some(1),
            mode: super::super::document_factory::PanelSelectionMode::Item,
        },
        modules: PanelModuleInput {
            entry,
            inventory,
            initial_props: BTreeMap::new(),
            instance_namespace: "initial-mount-retry".into(),
            epoch: 1,
            source_generation: 1,
            css: String::new(),
            viewport: Viewport {
                width: 1180,
                height: 760,
                dark: true,
            },
            assets: None,
        },
        labels: labels(),
        budget: PanelBudget {
            entries: 8,
            text_bytes: 8192,
        },
        pins: IntentPins {
            contract: ContentHash::of(b"contract"),
            provider: ContentHash::of(b"provider"),
            request: ContentHash::of(b"request"),
            reply: ContentHash::of(b"reply"),
        },
        registry_pending: 8,
        request_namespace: "initial-mount-retry".into(),
    }
}
fn register(
    execution: ExecutionHandle,
    registry: Rc<RefCell<PluginRegistry>>,
    scope: &PluginScope,
    context: Rc<RefCell<RequestContext>>,
) -> Result<AsyncProposalRegistration, PluginError> {
    AsyncProposalRegistration::register(
        registry,
        scope,
        context,
        execution,
        AsyncCodecBudget {
            max_pending: 8,
            max_terminal_wire_bytes: 65536,
            max_total_codec_wire_bytes: 8 * 65536 * 4,
        },
    )
}
#[test]
fn unacknowledged_success_keeps_same_startup_and_never_mounts() {
    let (owner, client, _, signals) = setup(false, false);
    let mut startup = start(owner);
    signals.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(startup.acknowledge_prepared().is_err());
    assert!(matches!(
        startup
            .prepare(bootstrap_labels(), None)
            .unwrap_or_else(|_| panic!("prepare")),
        InitialPreparationState::Prepared
    ));
    let raw = startup.prepared().unwrap().raw_terminal().clone();
    let mut startup = startup.into_handoff().err().expect("ACK still required");
    assert!(Arc::ptr_eq(
        startup.prepared().unwrap().raw_terminal(),
        &raw
    ));
    startup.acknowledge_prepared().unwrap();
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    close(&mut startup);
}
#[test]
fn late_canonical_failure_is_explicitly_accepted_before_worker_close() {
    let (owner, client, entered, signals) = setup(true, true);
    let mut startup = start(owner);
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    startup.cancel().unwrap();
    assert!(startup.close().is_err());
    release(&client);
    signals.recv_timeout(Duration::from_secs(3)).unwrap();
    let InitialPreparationState::Failed(observed) = await_preparation(&mut startup, &signals)
    else {
        panic!("failed")
    };
    assert_eq!(
        observed.as_ref().as_ref().unwrap_err().code,
        ErrorCode::NotFound
    );
    let accepted = startup.acknowledge_failure().unwrap();
    assert!(Arc::ptr_eq(accepted.snapshot(), &observed));
    assert_eq!(
        accepted
            .raw_terminal()
            .response
            .error
            .as_ref()
            .unwrap()
            .code,
        ErrorCode::NotFound
    );
    close(&mut startup);
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
}
#[test]
fn queued_cancel_never_executes_initial_inspect_and_can_be_acknowledged() {
    let (owner, client, entered, signals) = setup(true, false);
    let execution = owner.handle();
    let blocker = execution.submit(Request::new("proposals.inspect")).unwrap();
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    let mut startup = start(owner);
    startup.cancel().unwrap();
    signals.recv_timeout(Duration::from_secs(3)).unwrap();
    let InitialPreparationState::Failed(observed) = startup
        .prepare(bootstrap_labels(), None)
        .unwrap_or_else(|_| panic!("queued cancelled"))
    else {
        panic!("failed")
    };
    assert_eq!(
        observed.as_ref().as_ref().unwrap_err().code,
        ErrorCode::Cancelled
    );
    startup.acknowledge_failure().unwrap();
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    release(&client);
    signals.recv_timeout(Duration::from_secs(3)).unwrap();
    execution.acknowledge(&blocker).unwrap();
    close(&mut startup);
}
#[test]
fn canonical_success_with_wrong_selection_cannot_be_acknowledged_as_failure() {
    let (owner, client, _, signals) = setup(false, false);
    let mut selected = selection();
    selected.source_revision = DecimalU64(8);
    let mut startup = InitialPanelStartup::start(owner, RequestContext::default(), selected, 65536)
        .unwrap_or_else(|_| panic!("start"));
    signals.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(matches!(
        startup.prepare(bootstrap_labels(), None),
        Err(InitialPrepareError::Selection(
            BootstrapError::StaleSelection
        ))
    ));
    assert!(startup.acknowledge_failure().is_err());
    assert!(startup.retained_terminal().unwrap().is_some());
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    // Fault remains explicitly owned. Repairing selection is not a new SDK execution.
    startup.select_before_preparation(selection()).unwrap();
    startup
        .prepare(bootstrap_labels(), None)
        .unwrap_or_else(|_| panic!("explicit corrected selection"));
    startup.acknowledge_prepared().unwrap();
    close(&mut startup);
}
#[test]
fn repeated_module_and_parent_failure_reuses_same_preparation_without_reinspect() {
    let (owner, client, _, signals) = setup(false, false);
    let mut startup = start(owner);
    signals.recv_timeout(Duration::from_secs(3)).unwrap();
    startup
        .prepare(bootstrap_labels(), None)
        .unwrap_or_else(|_| panic!("prepare"));
    startup.acknowledge_prepared().unwrap();
    let handoff = startup.into_handoff().unwrap_or_else(|_| panic!("handoff"));
    let raw = handoff.prepared().raw_terminal().clone();
    let mut bad = input(client.clone());
    bad.modules.inventory.clear();
    let failure = handoff
        .mount(bad, None, register)
        .err()
        .expect("module failure");
    assert!(matches!(
        &failure.factory.cause,
        super::super::document_factory::FactoryCause::Module(_)
    ));
    let handoff = failure
        .recover_for_retry()
        .unwrap_or_else(|_| panic!("module recovery"));
    let post_register_entered = Cell::new(false);
    let failure = handoff
        .mount(
            input(client.clone()),
            None,
            |execution, registry, scope, context| {
                let mut registered = register(execution, registry, scope, context)?;
                post_register_entered.set(true);
                registered.close()?;
                Ok(registered)
            },
        )
        .err()
        .expect("explicit post-register handle fault");
    assert!(
        post_register_entered.get(),
        "must reach the actual parent registration failure"
    );
    let handoff = failure
        .recover_for_retry()
        .unwrap_or_else(|_| panic!("partial unbind recovery"));
    assert!(Arc::ptr_eq(handoff.prepared().raw_terminal(), &raw));
    let mut mounted = handoff
        .mount(input(client.clone()), None, register)
        .unwrap_or_else(|failure| panic!("{:?}", failure.factory.cause));
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    let deadline = Instant::now() + Duration::from_secs(3);
    while mounted.panel.close().is_err() {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
}

#[test]
fn handoff_shutdown_keeps_prepared_raw_and_joins_same_worker_without_mount() {
    let (owner, client, _, signals) = setup(false, false);
    let mut startup = start(owner);
    signals.recv_timeout(Duration::from_secs(3)).unwrap();
    startup
        .prepare(bootstrap_labels(), None)
        .unwrap_or_else(|_| panic!("prepare"));
    startup.acknowledge_prepared().unwrap();
    let handoff = startup.into_handoff().unwrap_or_else(|_| panic!("handoff"));
    let prepared = handoff.prepared().clone();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match handoff.poll_shutdown().unwrap() {
            ExecutionCloseProgress::Closed(receipt) => {
                assert!(receipt.worker_stopped);
                assert_eq!(receipt.unacknowledged, 0);
                break;
            }
            ExecutionCloseProgress::WorkerExitPending { .. } => assert!(Instant::now() < deadline),
            other => panic!("{other:?}"),
        }
        std::thread::yield_now();
    }
    assert!(Arc::ptr_eq(&prepared, handoff.prepared()));
    assert!(Arc::ptr_eq(
        prepared.raw_terminal(),
        handoff.prepared().raw_terminal()
    ));
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn final_mount_cleanup_joins_partial_or_unregistered_owner_and_disables_retry() {
    for registered in [false, true] {
        let (owner, client, _, signals) = setup(false, false);
        let mut startup = start(owner);
        signals.recv_timeout(Duration::from_secs(3)).unwrap();
        startup
            .prepare(bootstrap_labels(), None)
            .unwrap_or_else(|_| panic!("prepare"));
        startup.acknowledge_prepared().unwrap();
        let handoff = startup.into_handoff().unwrap_or_else(|_| panic!("handoff"));
        let raw = handoff.prepared().raw_terminal().clone();
        let mut request = input(client.clone());
        if !registered {
            request.modules.inventory.clear();
        }
        let mut failure = handoff
            .mount(request, None, move |execution, registry, scope, context| {
                let mut parent = register(execution, registry, scope, context)?;
                if registered {
                    parent.close()?;
                }
                Ok(parent)
            })
            .err()
            .expect("explicit failure before or after registration");
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match failure.poll_cleanup().unwrap() {
                super::super::document_factory::ParentCloseProgress::Complete => break,
                super::super::document_factory::ParentCloseProgress::WorkerExitPending => {
                    assert!(Instant::now() < deadline)
                }
                other => panic!("{other:?}"),
            }
            std::thread::yield_now();
        }
        assert!(Arc::ptr_eq(&raw, failure.prepared.raw_terminal()));
        let progress = failure
            .close_progress
            .as_ref()
            .or_else(|| failure.factory.parent_close_progress());
        assert!(
            matches!(progress, Some(ExecutionCloseProgress::Closed(receipt)) if receipt.worker_stopped && receipt.unacknowledged == 0),
            "registered={registered}"
        );
        assert!(
            failure.recover_for_retry().is_err(),
            "final cleanup cannot remount a joined worker"
        );
        assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    }
}
