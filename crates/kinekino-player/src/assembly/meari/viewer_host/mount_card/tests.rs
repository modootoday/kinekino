//! Real fixed product graph/SDK/store. Text input is a technical setter, not OS keyboard proof.
use super::*;
use crate::application::api::ProposalClient;
#[path = "tests/capture_readonly.rs"]
mod capture_readonly;
use crate::{
    application::{
        api::PlayerSdk,
        proposals::{CreateProposal, InspectProposal},
        sdk_execution::{
            CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner,
        },
    },
    assembly::meari::{
        card_current_inspection::{CardCurrentInspection, CardInspectionInput},
        player_shell_startup::{ShellInputs, prepare_shell},
        viewer_callbacks::ViewerCallbacks,
    },
    domain::{api_values::DecimalU64, proposals::*},
    infrastructure::proposals::ProposalFiles,
    presentation::web_ui_adapter::player_shell::document::{
        StartLabels, enable_read_panel, read_props,
    },
};
use meari_native::{
    domain::{Viewport, widgets::Action},
    infrastructure::document::InteractiveDocument,
};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Default)]
struct InspectGate {
    entered: AtomicBool,
    released: AtomicBool,
    calls: AtomicUsize,
    all_calls: std::sync::Mutex<Vec<String>>,
}
struct RecordedSdk {
    inner: ApplicationClientHandle,
    gate: Arc<InspectGate>,
}
impl crate::application::api::ApplicationClient for RecordedSdk {
    fn execute(
        &self,
        request: crate::application::api::Request,
    ) -> crate::application::api::Response {
        self.gate
            .all_calls
            .lock()
            .unwrap()
            .push(request.command.clone());
        if request.command == "proposals.inspect_output_review_target" {
            self.gate.calls.fetch_add(1, Ordering::Relaxed);
            self.gate.entered.store(true, Ordering::Release);
            while !self.gate.released.load(Ordering::Acquire) {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        self.inner.execute(request)
    }
}
struct ReleaseOnDrop(Arc<InspectGate>);
impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        self.0.released.store(true, Ordering::Release);
    }
}

struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), crate::application::api::ApiError> {
        Ok(())
    }
}
type MountFixture = (ProductCardMountInput, ExecutionOwner, Arc<InspectGate>);
fn fixture() -> MountFixture {
    fixture_with_read_owner().0
}
fn fixture_with_read_owner() -> (MountFixture, Rc<RefCell<ViewerCallbacks>>) {
    fixture_with_capture_choices(false)
}
fn fixture_with_capture_choices(
    capture_choices: bool,
) -> (MountFixture, Rc<RefCell<ViewerCallbacks>>) {
    let directory = std::env::current_dir()
        .unwrap()
        .join("runtime/quality-review/ui-current-card-mount-store")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&directory).unwrap();
    let inner: ApplicationClientHandle = Arc::new(
        PlayerSdk::default().with_proposal_store(ProposalFiles::open(&directory).unwrap()),
    );
    let gate = Arc::new(InspectGate::default());
    let client: ApplicationClientHandle = Arc::new(RecordedSdk {
        inner,
        gate: gate.clone(),
    });
    let payload = ProposalPayload::Cards {
        cards: vec![
            Card {
                front: "first".into(),
                back: "first back".into(),
                source_refs: vec!["cue-a".into()],
            },
            Card {
                front: "second".into(),
                back: "second back".into(),
                source_refs: vec!["cue-a".into()],
            },
        ],
    };
    let seed = client
        .create_proposal(
            RequestContext::default(),
            CreateProposal {
                proposal_id: "selected-card".into(),
                source: SourceSnapshot::Cue(CueSourceSnapshot {
                    source_id: "selected-source".into(),
                    source_revision: DecimalU64(3),
                    source_sha256: "a".repeat(64),
                    cues: vec![SourceCue {
                        cue_id: "cue-a".into(),
                        text: "Source caption".into(),
                        start_us: None,
                        end_us: None,
                    }],
                }),
                generated: GeneratedResult {
                    task: ProposalTask::Cards,
                    raw_text: serde_json::to_string(&payload).unwrap(),
                },
            },
        )
        .unwrap();
    let selected = NativeProposalSelection {
        proposal_id: seed.proposal_id.clone(),
        document_revision: seed.draft_revision,
        source_id: seed.source.source_id().into(),
        source_revision: seed.source.source_revision(),
        source_sha256: seed.source.source_sha256().into(),
        target: crate::application::launch::native_proposal::NativeProposalTarget::Card {
            index: 1,
        },
    };
    let owner = ExecutionOwner::start(
        client.clone(),
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 262144,
            max_total_request_bytes: 8 * 262144,
            max_terminal_bytes: 262144,
            max_total_terminal_bytes: 8 * 262144,
        },
        Arc::new(Notify),
    )
    .unwrap();
    let labels = StartLabels {
        connection: "Not observed".into(),
        time: "Not observed".into(),
    };
    let props = read_props(&labels);
    let mut prepared = prepare_shell(
        ShellInputs {
            client: client.clone(),
            host_owner: (),
        },
        labels,
    )
    .unwrap_or_else(|_| panic!("Fixed Viewing source must prepare"));
    enable_read_panel(&mut prepared.request.document, props.clone())
        .unwrap_or_else(|_| panic!("Fixed Viewing read source must prepare"));
    let (entry, mut sources) = product_modules_for_panel(BootstrapPanel::Card);
    let provider = meari::parse_module_graph(&entry, &sources).unwrap();
    let (studio, _) = FrameComposition::new(
        crate::domain::product::PRODUCT.display_name.into(),
        String::new(),
    )
    .prepare(provider, &mut sources)
    .unwrap();
    let frame = ProductFrameBlueprint::prepare(
        prepared.request.document.graph,
        prepared.request.document.inventory,
        studio,
        sources,
    )
    .unwrap();
    let program = frame
        .mount_viewing(
            props,
            "current-card-mount".into(),
            1,
            NonZeroU64::new(1).unwrap(),
        )
        .unwrap();
    let callbacks = Rc::new(RefCell::new(ViewerCallbacks::new(
        owner.handle(),
        RequestContext::default(),
    )));
    if capture_choices {
        use crate::domain::projects::{
            ProjectProposalSnapshot, ProjectRevision, ProjectWorkState, WorkingState,
            canonical_sha256,
        };
        let captured = client
            .inspect_proposal(
                RequestContext::default(),
                InspectProposal {
                    proposal_id: seed.proposal_id.clone(),
                },
            )
            .unwrap();
        let mut project = ProjectRevision {
            schema_version: "1".into(),
            project_id: "readonly-capture".into(),
            revision: DecimalU64(1),
            parent_revision: None,
            title: "Saved capture".into(),
            watching: None,
            proposals: vec![ProjectProposalSnapshot {
                capture_sha256: canonical_sha256(&captured).unwrap(),
                captured,
                working_draft: None,
                working_state: WorkingState::Absent,
                proposal_origin: None,
            }],
            work_state: ProjectWorkState::default(),
        };
        project.work_state = project.derived_work_state();
        callbacks
            .borrow_mut()
            .configure_card_choices(
                project,
                NonZeroU64::new(1).unwrap(),
                std::num::NonZeroUsize::new(1).unwrap(),
            )
            .unwrap();
    }
    let exporting = callbacks.clone();
    let document = InteractiveDocument::from_module_program_with_callbacks(
        program,
        "",
        Viewport {
            width: 320,
            height: 200,
            dark: true,
        },
        None,
        1,
        move |scope| {
            exporting
                .borrow_mut()
                .export(scope)
                .map_err(|error| error.message)
        },
    )
    .unwrap();
    let document = NativeDocumentHandle::new(document, 1).unwrap();
    let stamp = document.stamp().unwrap();
    assert!(
        callbacks
            .borrow_mut()
            .prepare_route_release(&stamp.owner)
            .unwrap()
            .is_none()
    );
    let registry = Rc::new(RefCell::new(
        PluginRegistry::new(
            frame.graph.fingerprint().unwrap(),
            TargetKind::Native,
            stamp.owner.clone(),
            8,
        )
        .unwrap(),
    ));
    let mut inspection = CardCurrentInspection::new(CardInspectionInput {
        execution: owner.handle(),
        scope: stamp.owner.clone(),
        registry,
        context: Rc::new(RefCell::new(RequestContext::default())),
        namespace: "before-card-mount".into(),
        limit: 8,
        codec: product_codec_budget(),
    })
    .unwrap_or_else(|_| panic!("Original execution must inspect"));
    inspection
        .begin(BootstrapSelection::try_from(selected.clone()).unwrap())
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !inspection.poll().unwrap() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    let Ok(crate::presentation::web_ui_adapter::proposals::pending_host::PendingOutput::Inspected(
        stored,
    )) = &inspection.terminal().unwrap().outcome
    else {
        panic!("Typed current inspect required")
    };
    let stored = stored.clone();
    inspection.release().unwrap();
    (
        (
            ProductCardMountInput {
                document,
                turn: HostTurn {
                    stamp,
                    reason: HostTurnReason::Input,
                    accepted_callbacks: 0,
                    rejected_callbacks: 0,
                    remaining_callbacks: 0,
                },
                frame,
                selected,
                stored,
                client,
                execution: owner.handle(),
                context: RequestContext::default(),
                namespace: "current-card-editor".into(),
                generation: NonZeroU64::new(2).unwrap(),
                labels: ProductWindowLabels::korean(),
                queue: None,
                navigation: None,
            },
            owner,
            gate,
        ),
        callbacks,
    )
}
fn callback_path(handle: &NativeDocumentHandle, name: &str) -> String {
    handle
        .try_with(|document| {
            document
                .module_projection()
                .unwrap()
                .callback_bindings()
                .into_iter()
                .find(|(_, callback)| callback.reference.binding == name)
                .unwrap()
                .0
        })
        .unwrap()
}
fn settle(runtime: &mut NativeHostRuntime<ActiveCardBridge>) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.host().operation_pending() {
        runtime.after_dispatch(HostTurnReason::Completion).unwrap();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn join(owner: ExecutionOwner) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let ExecutionCloseProgress::Closed(receipt) = owner.poll_close().unwrap() {
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn actual_product_card_provider_edits_saves_explicitly_accepts_and_reopens_on_original_execution() {
    let (input, owner, gate) = fixture();
    let _release = ReleaseOnDrop(gate.clone());
    let handle = input.document.clone();
    let before = handle.stamp().unwrap();
    let client = input.client.clone();
    let (bridge, receipt) =
        mount_current_card(input).unwrap_or_else(|failure| panic!("{}", failure.diagnostic));
    assert_eq!(receipt.before(), &before);
    assert_eq!(receipt.after().host_id, before.host_id);
    assert_eq!(
        receipt.after().document_revision,
        before.document_revision + 1
    );
    assert_eq!(
        bridge.review().stored().document.proposal_id,
        "selected-card"
    );
    let mut runtime = NativeHostRuntime::new(handle.clone(), bridge);
    let inspect = callback_path(&handle, "outputReviewOnInspect");
    assert!(
        !handle
            .try_with(|document| document
                .elements()
                .iter()
                .find(|element| element.path == inspect)
                .unwrap()
                .attributes
                .contains_key("disabled"))
            .unwrap()
    );
    let front = callback_path(&handle, "onFrontChange");
    // The library's technical input API emits the actual source onChange queue. This is
    // not OS keyboard/IME input, pointer hit testing, CSS/font fidelity or a visual check.
    handle
        .try_reproject(&handle.stamp().unwrap(), |document| {
            document.set_input_value(&front, "edited second")
        })
        .unwrap();
    runtime.after_dispatch(HostTurnReason::Input).unwrap();
    assert!(runtime.host().review().dirty());
    let edited_draft =
        serde_json::to_value(runtime.host().review().display().unwrap().working_draft).unwrap();
    assert!(runtime.host().navigation_guards().dirty);
    assert_eq!(
        runtime.host_mut().select_item(0),
        Err(crate::presentation::web_ui_adapter::proposals::ViewError::SaveRequired)
    );
    assert_eq!(
        serde_json::to_value(runtime.host().review().display().unwrap().working_draft).unwrap(),
        edited_draft
    );
    for binding in ["onSave", "onAccept"] {
        let path = callback_path(&handle, binding);
        handle.dispatch_action(Action::Activate { path }).unwrap();
        runtime.after_dispatch(HostTurnReason::Input).unwrap();
        settle(&mut runtime);
        assert!(runtime.host().review().error().is_none());
        if binding == "onSave" {
            assert!(runtime.host().review().stored().acceptance.is_none());
        }
    }
    let path = callback_path(&handle, "outputReviewOnInspect");
    handle.dispatch_action(Action::Activate { path }).unwrap();
    runtime.after_dispatch(HostTurnReason::Input).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !gate.entered.load(Ordering::Acquire) {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(runtime.host().output_review_state().unwrap().busy());
    assert!(runtime.host().navigation_guards().pending);
    assert_eq!(
        runtime.host_mut().select_item(0),
        Err(crate::presentation::web_ui_adapter::proposals::ViewError::Busy)
    );
    let operations = owner.handle().retained_operations().unwrap();
    assert_eq!(operations.len(), 1);
    let original_operation = operations[0].clone();
    assert!(matches!(
        owner.handle().peek(&original_operation).unwrap(),
        crate::application::sdk_execution::ExecutionSnapshot::Running { .. }
    ));
    gate.released.store(true, Ordering::Release);
    settle(&mut runtime);
    let reviewed = runtime.host().output_review_state().unwrap();
    assert!(!reviewed.busy());
    assert!(reviewed.inspection().is_some() || reviewed.error().is_some());
    assert!(matches!(
        owner.handle().peek(&original_operation).unwrap(),
        crate::application::sdk_execution::ExecutionSnapshot::Acknowledged
    ));
    runtime.after_dispatch(HostTurnReason::Completion).unwrap();
    assert_eq!(gate.calls.load(Ordering::Acquire), 1);
    let reopened = client
        .inspect_proposal(
            RequestContext::default(),
            InspectProposal {
                proposal_id: "selected-card".into(),
            },
        )
        .unwrap();
    assert!(reopened.acceptance.is_some());
    let Some(ProposalPayload::Cards { cards }) = reopened.document.draft else {
        panic!("Typed Card draft must persist")
    };
    assert_eq!(cards[0].front, "first");
    assert_eq!(cards[1].front, "edited second");
    runtime.host_mut().unbind_scope().unwrap();
    let next = owner.handle().attach().unwrap();
    next.release().unwrap();
    join(owner);
}

#[test]
fn actual_product_card_unbind_and_viewing_read_reexport_use_original_document_and_worker() {
    let ((input, owner, _gate), callbacks) = fixture_with_read_owner();
    let frame = input.frame.clone();
    let handle = input.document.clone();
    let (mut bridge, _) =
        mount_current_card(input).unwrap_or_else(|failure| panic!("{}", failure.diagnostic));
    let card_stamp = handle.stamp().unwrap();
    assert!(owner.handle().attach().is_err());
    bridge.unbind_scope().unwrap();
    let bindings = callbacks
        .borrow_mut()
        .resume_released_route(&card_stamp.owner)
        .unwrap();
    assert!(owner.handle().attach().is_err());
    let (_, receipt) = handle
        .try_reproject(&card_stamp, |document| {
            let props = read_props(&StartLabels {
                connection: "Not observed".into(),
                time: "Not observed".into(),
            });
            let props = frame.workspace_props(
                document,
                WorkspaceRoute::Viewing,
                NonZeroU64::new(3).unwrap(),
                props,
            )?;
            document.update_module_props_with_callbacks(props, bindings)
        })
        .unwrap();
    assert_eq!(receipt.before(), &card_stamp);
    assert_eq!(receipt.after().owner, card_stamp.owner);
    assert_eq!(receipt.after().host_id, card_stamp.host_id);
    assert_eq!(
        receipt.after().document_revision,
        card_stamp.document_revision + 1
    );
    let path = callback_path(&handle, "onReadCapabilities");
    handle.dispatch_action(Action::Activate { path }).unwrap();
    // Technical fixture drain; this is not delivery through the original Viewer runtime.
    let (pending, _) = handle
        .try_reproject(&handle.stamp().unwrap(), |document| {
            Ok::<_, String>(document.drain_callbacks())
        })
        .unwrap();
    assert_eq!(pending.len(), 1);
    let submitted = callbacks
        .borrow()
        .dispatch(&card_stamp.owner, &pending[0])
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(terminal) = callbacks
            .borrow_mut()
            .accept_terminal(&submitted.ticket)
            .unwrap()
        {
            terminal.owner_ack.unwrap();
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    callbacks
        .borrow_mut()
        .prepare_route_release(&card_stamp.owner)
        .unwrap();
    join(owner);
}

struct OriginalViewingReturnFailure {
    primary: Option<Box<dyn std::any::Any + Send>>,
    cleanup_error: Option<HostAccessError>,
    cleanup_panic: Option<Box<dyn std::any::Any + Send>>,
    competing_error: Option<crate::application::api::ApiError>,
    diagnostic: Option<std::io::Error>,
    cleanup_pending: bool,
    // A failed cleanup must retain the original owners, never silently invoke Drop.
    runtime: std::mem::ManuallyDrop<NativeHostRuntime<ViewerHost>>,
    _competing:
        std::mem::ManuallyDrop<Option<crate::application::sdk_execution::ExecutionAttachment>>,
}
impl std::fmt::Debug for OriginalViewingReturnFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginalViewingReturnFailure")
            .field("primary_panic", &self.primary.is_some())
            .field("cleanup_error", &self.cleanup_error)
            .field("cleanup_panic", &self.cleanup_panic.is_some())
            .field("competing_error", &self.competing_error)
            .field("cleanup_pending", &self.cleanup_pending)
            .field("last_host_error", &self.runtime.last_host_error())
            .field("close_progress", &self.runtime.host().last_close_progress())
            .field("return_error", &self.runtime.host().viewing_return_error())
            .field("diagnostic", &self.diagnostic)
            .finish()
    }
}
#[test]
#[ignore = "Explicit central LocalHost/named-pipe slot; no GUI. Synthetic route observations, not rendered frames."]
fn original_viewer_return_retains_failed_read_export_retries_and_closes_actual_local_host()
-> Result<(), Box<OriginalViewingReturnFailure>> {
    use crate::assembly::meari::{
        player_shell_host::StartRouteObservation,
        viewer_host::{
            ViewerHost,
            close::{LocalEndpoint, OrderedClose, ViewerExecution},
        },
    };
    use crate::presentation::web_ui_adapter::player_shell::{
        NavigationError, NavigationGuards, RouteCatalog, RouteHostObservation, ShellState,
    };
    let ((input, owner, _gate), callbacks) = fixture_with_read_owner();
    let client = input.client.clone();
    let blueprint = input.frame.clone();
    let handle = input.document.clone();
    let execution = owner.handle();
    let (bridge, _) =
        mount_current_card(input).unwrap_or_else(|failure| panic!("{}", failure.diagnostic));
    let catalog = RouteCatalog::bundled().unwrap();
    let mut state = ShellState::new();
    let start = state
        .request_route(&catalog, "view.start", NavigationGuards::default())
        .unwrap();
    state
        .observe_route(
            &start,
            RouteHostObservation::Reported {
                route: catalog.get("view.start").unwrap().id().clone(),
            },
        )
        .unwrap();
    let card = state
        .request_route(&catalog, "studio.cards", NavigationGuards::default())
        .unwrap();
    // Deliberately synthetic route state: the test has no renderer/window context. The
    // return's actual commit must retain its receipt and report failed frame arming.
    state
        .observe_route(
            &card,
            RouteHostObservation::Reported {
                route: catalog.get("studio.cards").unwrap().id().clone(),
            },
        )
        .unwrap();
    let observer = Rc::new(RefCell::new(StartRouteObservation::new(state, start)));
    let endpoint =
        crate::infrastructure::local_host::LocalHost::start(client.clone(), Default::default())
            .unwrap();
    let cards = Rc::new(RefCell::new(super::super::cards::CardRoute::default()));
    let jobs = Rc::new(RefCell::new(super::super::jobs::JobsLedger::default()));
    let mut host = ViewerHost {
        native_route: {
            let mut gate =
                crate::assembly::meari::viewer_host::route_media::NativeRouteGate::default();
            gate.declare_headless().unwrap();
            gate
        },
        window_ui: Default::default(),
        automatic: Default::default(),
        registration_redraw: Default::default(),
        transition_slot: None,
        frame: Default::default(),
        document: Some(handle.clone()),
        base_props: read_props(&StartLabels {
            connection: "Not observed".into(),
            time: "Not observed".into(),
        }),
        product_frame: Some(blueprint),
        read_retries: 3,
        read_admitted: false,
        viewing_chooser: Default::default(),
        viewing_generation: NonZeroU64::new(1).unwrap(),
        shutdown_started: false,
        completion: None,
        client,
        closing: OrderedClose::new(
            LocalEndpoint::new(endpoint),
            ViewerExecution {
                owner,
                callbacks: callbacks.clone(),
                cards: cards.clone(),
                jobs: jobs.clone(),
            },
        ),
        callbacks: callbacks.clone(),
        cards,
        jobs,
        route_observation: Some(observer),
        identity: Rc::new(()),
        last_error: None,
    };
    host.bind_active_card(bridge)
        .unwrap_or_else(|_| panic!("Original Card must bind"));
    let mut runtime = NativeHostRuntime::new(handle.clone(), host);
    let mut competing: Option<crate::application::sdk_execution::ExecutionAttachment> = None;
    let primary = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let front = callback_path(&handle, "onFrontChange");
        handle
            .try_reproject(&handle.stamp().unwrap(), |document| {
                document.set_input_value(&front, "whole viewer edit")
            })
            .unwrap();
        runtime.after_dispatch(HostTurnReason::Input).unwrap();
        let turn = || HostTurn {
            stamp: handle.stamp().unwrap(),
            reason: HostTurnReason::Input,
            accepted_callbacks: 0,
            rejected_callbacks: 0,
            remaining_callbacks: 0,
        };
        assert!(matches!(
            runtime.host_mut().request_viewing_return(&turn()),
            Err(super::super::ViewerHostError::Navigation(
                NavigationError::UnsavedEdits
            ))
        ));
        assert!(runtime.host().cards.borrow().viewing_return.is_none());
        let save = callback_path(&handle, "onSave");
        handle
            .dispatch_action(Action::Activate { path: save })
            .unwrap();
        runtime.after_dispatch(HostTurnReason::Input).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while runtime
            .host()
            .cards
            .borrow()
            .active
            .as_ref()
            .unwrap()
            .operation_pending()
        {
            runtime.after_dispatch(HostTurnReason::Completion).unwrap();
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        let mut suffix = turn();
        suffix.remaining_callbacks = 1;
        assert!(matches!(
            runtime.host_mut().request_viewing_return(&suffix),
            Err(super::super::ViewerHostError::Navigation(
                NavigationError::Busy
            ))
        ));
        runtime.host_mut().request_viewing_return(&turn()).unwrap();
        let borrowed = callbacks.borrow();
        // Technical transaction seam: the full runtime's earlier Jobs borrow guard is not
        // evidence that this later return/export boundary was reached.
        assert!(matches!(
            runtime.host_mut().advance_viewing_return(&turn()),
            Err(super::super::ViewerHostError::UnsupportedCallback)
        ));
        assert!(matches!(
            runtime.host().viewing_return_error(),
            Some(super::super::ViewerHostError::UnsupportedCallback)
        ));
        assert!(runtime.host().viewing_return_receipt().is_none());
        assert!(matches!(
            runtime.host_mut().cancel_viewing_return_request(),
            Err(super::super::ViewerHostError::Navigation(
                NavigationError::RecoveryBlocked
            ))
        ));
        drop(borrowed);
        // Card scope released, but no read owner could be borrowed. A real competing
        // attachment now forces an actual SDK attach failure on the explicit retry.
        competing = Some(execution.attach().unwrap());
        runtime.host_mut().retry_viewing_return().unwrap();
        assert!(runtime.after_dispatch(HostTurnReason::Input).is_err());
        assert!(!callbacks.borrow().is_released());
        assert_eq!(
            callbacks.borrow().scope(),
            Some(&handle.stamp().unwrap().owner)
        );
        competing.as_ref().unwrap().release().unwrap();
        competing = None;
        runtime.host_mut().retry_viewing_return().unwrap();
        runtime.after_dispatch(HostTurnReason::Input).unwrap();
        let committed = runtime.host().viewing_return_receipt().unwrap();
        assert_eq!(committed.after(), &handle.stamp().unwrap());
        assert!(matches!(
            runtime.host().viewing_return_error(),
            Some(super::super::ViewerHostError::Navigation(
                NavigationError::RecoveryBlocked
            ))
        ));
        assert!(runtime.host().cards.borrow().active.is_none());
        let read = callback_path(&handle, "onReadCapabilities");
        handle
            .dispatch_action(Action::Activate { path: read })
            .unwrap();
        runtime.after_dispatch(HostTurnReason::Input).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while !callbacks.borrow().tickets().unwrap().is_empty() {
            runtime.after_dispatch(HostTurnReason::Completion).unwrap();
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
    }))
    .err();
    let mut competing_error = None;
    let cleanup = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Some(attachment) = &competing {
            match attachment.release() {
                Ok(()) => competing = None,
                Err(error) => competing_error = Some(error),
            }
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut closed = runtime.request_close(CloseCause::User)?;
        while closed != HostCloseStatus::HostCleanupComplete {
            if Instant::now() >= deadline {
                return Ok::<_, HostAccessError>(false);
            }
            std::thread::sleep(Duration::from_millis(1));
            closed = runtime.observe_close(CloseCause::User)?;
        }
        assert!(matches!(runtime.host().last_close_progress(),
            Some(super::super::ViewerCloseProgress::Complete { endpoint, execution })
            if endpoint.is_quiescent() && execution.worker_stopped && execution.unacknowledged == 0));
        runtime.finish_headless_close()?;
        assert!(execution.attach().is_err());
        assert!(runtime.host().shutdown_started);
        Ok(true)
    }));
    let (complete, cleanup_error, cleanup_panic) = match cleanup {
        Ok(Ok(complete)) => (complete, None, None),
        Ok(Err(error)) => (false, Some(error), None),
        Err(panic) => (false, None, Some(panic)),
    };
    let complete = complete && competing_error.is_none();
    use std::io::Write;
    let mut output = std::io::stderr().lock();
    let diagnostic = writeln!(
        output,
        "original_viewing_return_closeout complete={complete} primary_panic={} cleanup_panic={}",
        primary.is_some(),
        cleanup_panic.is_some()
    )
    .and_then(|()| output.flush())
    .err();
    if primary.is_some() || !complete || diagnostic.is_some() {
        return Err(Box::new(OriginalViewingReturnFailure {
            primary,
            cleanup_error,
            cleanup_panic,
            competing_error,
            diagnostic,
            cleanup_pending: !complete,
            runtime: std::mem::ManuallyDrop::new(runtime),
            _competing: std::mem::ManuallyDrop::new(competing),
        }));
    }
    Ok(())
}
#[test]
fn stale_selected_snapshot_retains_original_input_and_document_before_editor_registration() {
    let (mut input, owner, _) = fixture();
    input.selected.document_revision.0 += 1;
    let handle = input.document.clone();
    let before = handle.stamp().unwrap();
    let mut failure = match mount_current_card(input) {
        Err(failure) => failure,
        Ok(_) => panic!("Stale capture cannot open current editor"),
    };
    assert_eq!(handle.stamp().unwrap(), before);
    assert_eq!(failure.input.selected.document_revision.0, 2);
    assert!(
        matches!(&failure.partial, Some(ProductCardMountPartial::Session(failure)) if failure.parent.is_none())
    );
    assert!(failure.poll_release().unwrap());
    let next = owner.handle().attach().unwrap();
    next.release().unwrap();
    join(owner);
}
