//! R1 policy/SDK handoff fixture. No production chooser, editor, window or checked frame.
use super::*;
use crate::application::api::ProposalClient;
use crate::{
    application::{
        api::{
            ApiError, ApplicationClient, ApplicationClientHandle, PlayerSdk, Request,
            RequestContext, Response,
        },
        proposals::{CreateProposal, list::ListProposals},
        sdk_execution::{CompletionNotifier, ExecutionBudget, ExecutionHandle, ExecutionOwner},
    },
    assembly::meari::player_shell_host::StartRouteObservation,
    domain::{api_values::DecimalU64, proposals::*},
    infrastructure::local_host::{EndpointClient, LocalHost, PendingLocalHost, TransportOptions},
    presentation::web_ui_adapter::{
        player_shell::document::product_frame::ProductFrameBlueprint,
        player_shell::{RouteHostObservation, ShellState},
        proposals::{
            bootstrap::BootstrapPanel, frame_composition::FrameComposition,
            product_window::product_modules_for_panel,
        },
    },
};
use meari::{
    application::{
        host_callbacks::{CallbackBindings, HostCallbacks},
        source_modules::ModuleProgram,
    },
    domain::{
        Span,
        build::BuildPath,
        host_callbacks::{
            CallbackArguments, CallbackInvocation, CallbackReference, CallbackSignature,
        },
    },
};
use meari_native::{domain::Viewport, infrastructure::document::InteractiveDocument};
use std::{
    any::Any,
    collections::BTreeMap,
    io::Write,
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn selection() -> NativeProposalSelection {
    NativeProposalSelection {
        proposal_id: "live-proposal".into(),
        document_revision: DecimalU64(1),
        source_id: "live-source".into(),
        source_revision: DecimalU64(9_007_199_254_740_993),
        source_sha256: "a".repeat(64),
        target: NativeProposalTarget::Highlight { index: 1 },
    }
}
#[test]
fn trusted_table_preserves_explicit_identity_and_rejects_empty_overflow_and_repair() {
    let expected = selection();
    let choices = TrustedLiveChoices::from_trusted_local(
        vec![expected.clone()],
        NonZeroU64::new(7).unwrap(),
        NonZeroUsize::new(2).unwrap(),
    )
    .unwrap();
    assert_eq!(choices.rows[0].0, "live-proposal-7-0");
    assert_eq!(
        serde_json::to_value(choices.rows[0].1.as_ref()).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    assert!(
        TrustedLiveChoices::from_trusted_local(
            Vec::new(),
            NonZeroU64::new(1).unwrap(),
            NonZeroUsize::new(1).unwrap()
        )
        .is_err()
    );
    assert!(
        TrustedLiveChoices::from_trusted_local(
            vec![expected.clone(), expected],
            NonZeroU64::new(1).unwrap(),
            NonZeroUsize::new(1).unwrap()
        )
        .is_err()
    );
    let mut repair = selection();
    repair.target = NativeProposalTarget::DocumentRepair {
        selection:
            crate::application::launch::native_proposal::DocumentRepairSelection::Unselected {},
    };
    assert!(
        TrustedLiveChoices::from_trusted_local(
            vec![repair],
            NonZeroU64::new(1).unwrap(),
            NonZeroUsize::new(1).unwrap()
        )
        .is_err()
    );
}

#[test]
#[ignore = "Assigned original LocalHost/SDK headless slot; synthetic repair only, no actual frame acceptance"]
fn prepared_manual_repair_mounts_all_tasks_with_exact_props_and_original_inspection()
-> Result<(), Box<Failure>> {
    for (variant, task) in [
        (3, ProposalTask::Correction),
        (4, ProposalTask::Cards),
        (5, ProposalTask::Highlights),
    ] {
        run_rig(variant, true, |rig| {
            let (key, prepared) =
                rig.prepare_chooser_target(NativeProposalTarget::ManualRepair { task });
            let input = rig.input(LIVE_ENTRY_BINDING, CallbackArguments::Text(key));
            assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
            rig.settle();
            assert_eq!(
                rig.host().live_entry_phase(),
                Some(LiveEntryPhase::Inspected)
            );
            let outcome = rig
                .runtime
                .as_mut()
                .unwrap()
                .after_dispatch(HostTurnReason::Completion);
            let effect = match outcome {
                Ok(effect) => effect,
                Err(error) => {
                    rig.runtime_errors.push(error);
                    if let Some(error) = rig.runtime.as_ref().unwrap().last_host_error().cloned() {
                        rig.typed_host_errors.push(error);
                    }
                    panic!(
                        "Synthetic ManualRepair strict mount failed; original runtime and typed errors retained"
                    );
                }
            };
            let receipt = effect
                .committed
                .as_ref()
                .expect("The original mount must synchronously return its receipt");
            let cards = rig.host().cards.borrow();
            let entry = cards.live_opening.as_ref().unwrap();
            assert!(Arc::ptr_eq(&entry.request.selected, &prepared.selection));
            assert_eq!(
                entry.request.selected.target,
                NativeProposalTarget::ManualRepair { task }
            );
            assert_eq!(entry.receipt.as_ref(), Some(receipt));
            assert_eq!(entry.proof.as_ref().unwrap().receipt(), receipt);
            assert!(entry.mount_failure.is_none());
            assert!(entry.retained_inspection.as_ref().unwrap().is_released());
            assert!(cards.active.is_some());
            assert!(!entry.observed && !entry.gate.get());
            assert_eq!(
                rig.calls(),
                2,
                "One preparation inspect and one current inspect"
            );
        })?;
    }
    Ok(())
}

struct Notify;
#[test]
#[ignore = "Assigned original SDK headless one-shot relay slot; no actual native frame proof"]
fn pending_viewing_registration_relays_only_original_queued_redraw_once() -> Result<(), Box<Failure>>
{
    with_rig(0, |rig| {
        let stamp = rig.host().document.as_ref().unwrap().stamp().unwrap();
        let turn = HostTurn {
            stamp: stamp.clone(),
            reason: HostTurnReason::Completion,
            accepted_callbacks: 0,
            rejected_callbacks: 0,
            remaining_callbacks: 0,
        };
        assert!(!rig.host().take_pending_route_registration_redraw(&turn));
        rig.host().registration_redraw.queue(&stamp).unwrap();
        assert!(rig.host().take_pending_route_registration_redraw(&turn));
        assert!(!rig.host().take_pending_route_registration_redraw(&turn));
        assert_eq!(
            rig.calls(),
            0,
            "A redraw request is not SDK work or frame proof"
        );
        assert_eq!(
            rig.host().registration_redraw.snapshot(),
            (true, true, false)
        );
    })
}
#[test]
#[ignore = "Assigned original SDK headless close slot; model port does not establish native cleanup"]
fn close_during_native_retirement_keeps_original_ticket_without_starting_inspection()
-> Result<(), Box<Failure>> {
    with_rig(6, |rig| {
        let scope = rig.host().document.as_ref().unwrap().stamp().unwrap().owner;
        let port = Rc::new(super::super::route_media::tests::TestPort::new(scope));
        rig.host_mut()
            .install_route_media_port(port.clone())
            .unwrap();
        let input = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        let end = Instant::now() + Duration::from_secs(3);
        while port.begins.get() == 0 {
            rig.tick();
            assert!(Instant::now() < end);
        }
        let generation = rig.host().native_route.retirement_generation().unwrap();
        let progress = rig.host_mut().poll_unrun_shutdown().unwrap();
        rig.close_progress.push(progress);
        assert!(rig.host().shutdown_started);
        assert!(matches!(rig.admit(&input), IntentAdmission::Rejected(_)));
        assert_eq!(
            rig.host().native_route.retirement_generation(),
            Some(generation)
        );
        assert_eq!(port.begins.get(), 1);
        assert_eq!(rig.calls(), 0);
        rig.assert_native_retirement_retries(generation);
    })
}
#[test]
#[ignore = "Assigned original SDK headless retirement ordering slot; model port is not native cleanup proof"]
fn native_retirement_pending_blocks_original_current_inspection_until_same_ticket_completes()
-> Result<(), Box<Failure>> {
    with_rig(6, |rig| {
        let scope = rig.host().document.as_ref().unwrap().stamp().unwrap().owner;
        let port = Rc::new(super::super::route_media::tests::TestPort::new(scope));
        rig.host_mut()
            .install_route_media_port(port.clone())
            .unwrap();
        let input = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        let end = Instant::now() + Duration::from_secs(3);
        while port.begins.get() == 0 {
            rig.tick();
            assert!(
                Instant::now() < end,
                "Original callback release did not reach retirement"
            );
        }
        for _ in 0..3 {
            rig.tick();
        }
        assert_eq!(
            rig.host().live_entry_phase(),
            Some(LiveEntryPhase::Draining)
        );
        assert_eq!(
            rig.calls(),
            0,
            "Pending native cleanup cannot start SDK current inspection"
        );
        assert_eq!(
            port.begins.get(),
            1,
            "The original retirement must not be reissued"
        );
        assert!(rig.host().cards.borrow().active.is_none());
        let generation = rig.host().native_route.retirement_generation().unwrap();
        rig.assert_native_retirement_retries(generation);
        let retries_before_completion = rig.native_retirement_retries.len();
        port.retire_ready.set(true);
        rig.settle();
        assert_eq!(
            rig.host().live_entry_phase(),
            Some(LiveEntryPhase::Inspected)
        );
        assert_eq!(rig.calls(), 1);
        assert_eq!(port.begins.get(), 1);
        assert_eq!(
            rig.host().native_route.retirement_generation(),
            Some(generation)
        );
        assert_eq!(
            rig.native_retirement_retries.len(),
            retries_before_completion
        );
        rig.assert_native_retirement_retries(generation);
    })
}
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
#[derive(Default)]
struct Probe {
    calls: Mutex<Vec<Request>>,
    hold: AtomicBool,
    entered: AtomicBool,
}
struct Observed {
    actual: ApplicationClientHandle,
    probe: Arc<Probe>,
}
impl ApplicationClient for Observed {
    fn execute(&self, request: Request) -> Response {
        self.probe.calls.lock().unwrap().push(request.clone());
        if request.command == "proposals.inspect" {
            self.probe.entered.store(true, Ordering::Release);
            let end = Instant::now() + Duration::from_secs(5);
            while self.probe.hold.load(Ordering::Acquire) && Instant::now() < end {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        self.actual.execute(request)
    }
}
#[derive(Default)]
struct Rig {
    chooser: bool,
    expected_failure: Option<u8>,
    native_retirement_fixture: bool,
    native_retirement_retries: Vec<(u64, Duration)>,
    runtime_errors: Vec<HostAccessError>,
    typed_host_errors: Vec<ViewerHostError>,
    // Manual host-admission exports, not the Native document's renderer export.
    // Keep their original revocable lease alive through owner cleanup.
    input_exports: RefCell<Vec<CallbackBindings>>,
    admission_errors: Vec<ViewerHostError>,
    close_progress: Vec<ViewerCloseProgress>,

    setup_carrier: Option<Box<dyn Any>>,
    cleanup_carrier: Option<Box<dyn Any>>,
    expected_carrier: Option<Box<dyn Any>>,
    host: Option<ViewerHost>,
    runtime: Option<NativeHostRuntime<ViewerHost>>,
    endpoint: Option<LocalHost>,
    pending_endpoint: Option<PendingLocalHost>,
    execution: Option<ExecutionOwner>,
    handle: Option<ExecutionHandle>,
    probe: Arc<Probe>,
    root: Option<PathBuf>,
}
impl Rig {
    fn assert_native_retirement_retries(&self, generation: u64) {
        assert!(!self.native_retirement_retries.is_empty());
        assert!(
            self.native_retirement_retries
                .iter()
                .all(|retry| *retry == (generation, Duration::from_millis(5)))
        );
        assert_eq!(
            self.runtime_errors.len(),
            self.native_retirement_retries.len()
        );
        assert_eq!(
            self.typed_host_errors.len(),
            self.native_retirement_retries.len()
        );
        assert!(self.typed_host_errors.iter().all(|error|
            matches!(error, ViewerHostError::ReadConsumerRetry(delay) if *delay == Duration::from_millis(5))));
        assert!(self.expected_failure.is_none());
    }
    fn setup_failure<T: std::fmt::Debug + 'static>(&mut self, error: T) -> String {
        let diagnostic = format!("{error:?}");
        self.setup_carrier.get_or_insert_with(|| Box::new(error));
        diagnostic
    }
    fn setup(&mut self, variant: u8) -> Result<(), String> {
        self.expected_failure = matches!(variant, 1 | 2).then_some(variant);
        self.native_retirement_fixture = variant == 6;
        let root = std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join("runtime/quality-review/ui-live-entry")
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
        let actual: ApplicationClientHandle = Arc::new(
            PlayerSdk::default().with_proposal_store(
                crate::infrastructure::proposals::ProposalFiles::open(&root)
                    .map_err(|e| self.setup_failure(e))?,
            ),
        );
        let payload = ProposalPayload::Highlights {
            highlights: vec![
                Highlight {
                    title: "first".into(),
                    source_refs: vec!["cue-a".into()],
                },
                Highlight {
                    title: "explicit second".into(),
                    source_refs: vec!["cue-a".into()],
                },
            ],
        };
        let document = actual
            .create_proposal(
                RequestContext::default(),
                CreateProposal {
                    proposal_id: "live-proposal".into(),
                    source: SourceSnapshot::Cue(CueSourceSnapshot {
                        source_id: "live-source".into(),
                        source_revision: DecimalU64(9_007_199_254_740_993),
                        source_sha256: "a".repeat(64),
                        cues: vec![SourceCue {
                            cue_id: "cue-a".into(),
                            text: "원문 日本語".into(),
                            start_us: None,
                            end_us: None,
                        }],
                    }),
                    generated: GeneratedResult {
                        task: match variant {
                            3 => ProposalTask::Correction,
                            4 => ProposalTask::Cards,
                            _ => ProposalTask::Highlights,
                        },
                        raw_text: if variant >= 3 {
                            "synthetic non-schema model response".into()
                        } else {
                            serde_json::to_string(&payload).unwrap()
                        },
                    },
                },
            )
            .map_err(|e| self.setup_failure(e))?;
        let mut selected = selection();
        selected.document_revision = document.draft_revision;
        if variant >= 3 {
            assert_eq!(document.state, DraftState::NeedsRepair);
            assert!(document.draft.is_none());
            selected.target = NativeProposalTarget::ManualRepair {
                task: document.generated.task,
            };
        }
        if variant == 1 {
            selected.source_sha256 = "b".repeat(64);
        }
        if variant == 2 {
            selected.proposal_id = "missing-proposal".into();
        }
        self.endpoint = Some(
            LocalHost::start(actual, TransportOptions::default())
                .map_err(|e| self.setup_failure(e))?,
        );
        let endpoint: ApplicationClientHandle = Arc::new(
            EndpointClient::new(
                self.endpoint.as_ref().unwrap().endpoint(),
                TransportOptions::default(),
            )
            .map_err(|e| self.setup_failure(e))?,
        );
        let client: ApplicationClientHandle = Arc::new(Observed {
            actual: endpoint,
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
                Arc::new(Notify),
            )
            .map_err(|e| self.setup_failure(e))?,
        );
        self.handle = Some(self.execution.as_ref().unwrap().handle());
        self.host = Some(ViewerHost::new(
            client,
            self.execution.take().unwrap(),
            self.endpoint.take().unwrap(),
        ));
        if variant != 6 {
            self.host
                .as_mut()
                .unwrap()
                .declare_headless_route_media()
                .map_err(|error| self.setup_failure(error))?;
        }
        let choices = TrustedLiveChoices::from_trusted_local(
            vec![selected.clone(), selected],
            NonZeroU64::new(7).unwrap(),
            NonZeroUsize::new(2).unwrap(),
        )
        .map_err(|e| self.setup_failure(e))?;
        self.host
            .as_mut()
            .unwrap()
            .configure_live_choices(choices)
            .map_err(|e| e.to_string())?;
        if self.chooser {
            self.host
                .as_ref()
                .unwrap()
                .callbacks()
                .borrow_mut()
                .configure_proposal_chooser(ListProposals::default())
                .map_err(|e| e.message)?;
        }
        let entry: BuildPath = "live-seam.tsx"
            .to_owned()
            .try_into()
            .map_err(|e| format!("{e:?}"))?;
        // Deliberately no live button: production user entry is R3, not this contract fixture.
        let inventory = BTreeMap::from([(
            entry.clone(),
            "type Props={};export function Seam(props:Props){return <main/>;}".into(),
        )]);
        let viewing =
            meari::parse_module_graph(&entry, &inventory).map_err(|e| format!("{e:?}"))?;
        let panel = if variant >= 3 {
            BootstrapPanel::ManualRepair
        } else {
            BootstrapPanel::Card
        };
        let (studio_entry, mut studio_inventory) = product_modules_for_panel(panel);
        let studio = meari::parse_module_graph(&studio_entry, &studio_inventory)
            .map_err(|e| format!("{e:?}"))?;
        let (studio, _) = FrameComposition::new(
            crate::domain::product::PRODUCT.display_name.into(),
            String::new(),
        )
        .prepare(studio, &mut studio_inventory)
        .map_err(|e| format!("{e:?}"))?;
        let frame = ProductFrameBlueprint::prepare(viewing, inventory, studio, studio_inventory)?;
        let props = BTreeMap::new();
        let program: ModuleProgram = frame.mount_viewing(
            props.clone(),
            "live-seam".into(),
            1,
            NonZeroU64::new(1).unwrap(),
        )?;
        let callbacks = self.host.as_ref().unwrap().callbacks();
        let document = InteractiveDocument::from_module_program_with_callbacks(
            program,
            "",
            Viewport {
                width: 640,
                height: 480,
                dark: true,
            },
            None,
            1,
            move |scope| callbacks.borrow_mut().export(scope).map_err(|e| e.message),
        )?;
        let document = NativeDocumentHandle::new(document, 1).map_err(|e| e.to_string())?;
        self.host
            .as_mut()
            .unwrap()
            .bind_product_document(document.clone(), props, frame);
        let mut state = ShellState::new();
        let ticket = state
            .request_route(
                &RouteCatalog::bundled().map_err(|e| format!("{e:?}"))?,
                "view.start",
                NavigationGuards::default(),
            )
            .map_err(|e| format!("{e:?}"))?;
        let mut observer = StartRouteObservation::new(state, ticket.clone());
        let viewing_route = observer
            .state
            .route_target(&ticket)
            .map_err(|e| format!("{e:?}"))?
            .clone();
        // Route model setup only; never called checked-window observation or actual frame proof.
        observer
            .state
            .observe_route(
                &ticket,
                RouteHostObservation::Reported {
                    route: viewing_route,
                },
            )
            .map_err(|e| format!("{e:?}"))?;
        self.host
            .as_mut()
            .unwrap()
            .bind_route_observation(Rc::new(RefCell::new(observer)));
        self.runtime = Some(NativeHostRuntime::new(document, self.host.take().unwrap()));
        Ok(())
    }
    fn host(&self) -> &ViewerHost {
        self.runtime
            .as_ref()
            .map(NativeHostRuntime::host)
            .or(self.host.as_ref())
            .unwrap()
    }
    fn host_mut(&mut self) -> &mut ViewerHost {
        if let Some(runtime) = &mut self.runtime {
            runtime.host_mut()
        } else {
            self.host.as_mut().unwrap()
        }
    }
    fn input(&self, prop: &str, args: CallbackArguments) -> PendingCallback {
        let bindings = self.host().callbacks().borrow().bindings().unwrap();
        let signature = if matches!(
            prop,
            LIVE_ENTRY_BINDING | "onChooserDocument" | "onChooserItem"
        ) {
            CallbackSignature::Text
        } else {
            CallbackSignature::Action
        };
        let pending = PendingCallback {
            handle: bindings
                .resolve(&CallbackReference {
                    binding: prop.into(),
                    signature,
                })
                .unwrap(),
            invocation: CallbackInvocation {
                prop: prop.into(),
                arguments: args,
                span: Span::default(),
            },
        };
        self.input_exports.borrow_mut().push(bindings);
        pending
    }
    fn admit(&mut self, pending: &PendingCallback) -> IntentAdmission<ViewerHostError> {
        let stamp = self.host().document.as_ref().unwrap().stamp().unwrap();
        let admission = self.host_mut().admit_callback(&stamp, pending);
        if let IntentAdmission::Rejected(error) = &admission {
            self.admission_errors.push(error.clone());
        }
        admission
    }
    fn tick(&mut self) {
        let phase = self.host().live_entry_phase();
        let outcome = self
            .runtime
            .as_mut()
            .unwrap()
            .after_dispatch(HostTurnReason::Completion);
        match outcome {
            Ok(receipt) => {
                if !matches!(
                    phase,
                    Some(
                        LiveEntryPhase::Inspected
                            | LiveEntryPhase::AwaitingArm
                            | LiveEntryPhase::AwaitingFrame
                            | LiveEntryPhase::Activating
                            | LiveEntryPhase::Active
                    )
                ) {
                    assert!(
                        receipt.committed.is_none(),
                        "Inspect handoff cannot commit a panel"
                    );
                }
            }
            Err(error) => {
                self.runtime_errors.push(error);
                assert!(
                    matches!(
                        self.runtime_errors.last().unwrap(),
                        HostAccessError::Invalid(_)
                    ),
                    "Only the runtime's typed-consumer failure wrapper is expected"
                );
                let typed = self
                    .runtime
                    .as_ref()
                    .unwrap()
                    .last_host_error()
                    .cloned()
                    .expect("Original typed host error must be retained");
                self.typed_host_errors.push(typed.clone());
                if self.native_retirement_fixture
                    && self.host().live_entry_phase() == Some(LiveEntryPhase::Draining)
                    && matches!(&typed, ViewerHostError::ReadConsumerRetry(delay) if *delay == Duration::from_millis(5))
                {
                    assert!(self.host().callbacks.borrow().is_released());
                    let generation = self
                        .host()
                        .native_route
                        .retirement_generation()
                        .expect("Native retry requires the original retirement ticket");
                    {
                        let cards = self.host().cards.borrow();
                        assert!(cards.inspection.is_none());
                        assert!(cards.active.is_none());
                        assert!(cards.live_opening.as_ref().unwrap().receipt.is_none());
                    }
                    assert_eq!(self.calls(), 0);
                    let Some(HostRetry::After(delay)) = self.host().retry_after_dispatch(&typed)
                    else {
                        panic!("Original native scheduler must retain its timed retry")
                    };
                    assert_eq!(delay, Duration::from_millis(5));
                    self.native_retirement_retries.push((generation, delay));
                    // Model WindowPort's retained retry and Application.schedule.defer;
                    // retain both original errors and resume this same runtime only after due.
                    std::thread::sleep(delay);
                    return;
                }
                let variant = self
                    .expected_failure
                    .expect("A positive fixture must not suppress a runtime error");
                assert_eq!(self.host().live_entry_phase(), Some(LiveEntryPhase::Failed));
                let cards = self.host().cards.borrow();
                let entry = cards.live_opening.as_ref().unwrap();
                let ViewerHostError::Callback(actual) = &typed else {
                    panic!("Unexpected typed host failure: {typed:?}")
                };
                let Some(ViewerHostError::Callback(retained)) = &entry.error else {
                    panic!("Original entry error is missing")
                };
                assert!(
                    Arc::ptr_eq(actual, retained),
                    "Runtime and original entry must own the same error"
                );
                let terminal = cards.inspection.as_ref().unwrap().terminal().unwrap();
                match variant {
                    1 => {
                        assert_eq!(actual.code, PluginErrorCode::Stale);
                        assert_eq!(entry.validation_error, Some(BootstrapError::StaleSelection));
                        assert!(matches!(&terminal.outcome, Ok(PendingOutput::Inspected(_))));
                    }
                    2 => {
                        assert_eq!(actual.code, PluginErrorCode::Closing);
                        assert!(entry.validation_error.is_none());
                        assert!(
                            matches!(&terminal.outcome, Err(error) if error.code == crate::application::api::ErrorCode::NotFound)
                        );
                    }
                    _ => unreachable!(
                        "Only the two explicit negative fixtures allow an original host error"
                    ),
                }
            }
        }
    }
    fn settle(&mut self) {
        let end = Instant::now() + Duration::from_secs(3);
        while !matches!(
            self.host().live_entry_phase(),
            Some(LiveEntryPhase::Inspected | LiveEntryPhase::Failed)
        ) {
            self.tick();
            assert!(
                Instant::now() < end,
                "Original live inspection did not settle"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    fn calls(&self) -> usize {
        self.probe
            .calls
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.command == "proposals.inspect")
            .count()
    }
    fn chooser_action(&mut self, prop: &str, args: CallbackArguments) {
        let scope = self
            .host()
            .document
            .as_ref()
            .unwrap()
            .stamp()
            .unwrap()
            .owner;
        let pending = self.input(prop, args);
        let callbacks = self.host().callbacks();
        {
            let mut owner = callbacks.borrow_mut();
            owner.queue_chooser(&scope, &pending).unwrap();
            owner.drain_chooser(&scope).unwrap();
        }
        let end = Instant::now() + Duration::from_secs(3);
        loop {
            let mut owner = callbacks.borrow_mut();
            owner.consume_reads().unwrap();
            if owner.tickets().unwrap().is_empty() {
                assert!(owner.chooser_failure().is_none());
                break;
            }
            assert!(
                Instant::now() < end,
                "Original chooser request did not settle"
            );
            drop(owner);
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    fn prepare_chooser(
        &mut self,
    ) -> (
        String,
        crate::assembly::meari::viewer_callbacks::chooser::PreparedSelection,
    ) {
        self.prepare_chooser_target(NativeProposalTarget::Highlight { index: 1 })
    }
    fn prepare_chooser_target(
        &mut self,
        target: NativeProposalTarget,
    ) -> (
        String,
        crate::assembly::meari::viewer_callbacks::chooser::PreparedSelection,
    ) {
        self.chooser_action("onChooserList", CallbackArguments::Action);
        let key = self
            .host()
            .callbacks()
            .borrow()
            .chooser_documents()
            .iter()
            .find(|row| row.metadata.proposal_id == "live-proposal")
            .unwrap()
            .key
            .clone();
        self.chooser_action("onChooserDocument", CallbackArguments::Text(key));
        let item = self
            .host()
            .callbacks()
            .borrow()
            .chooser_items()
            .iter()
            .find(|row| row.target == target)
            .unwrap()
            .key
            .clone();
        self.chooser_action("onChooserItem", CallbackArguments::Text(item));
        let scope = self
            .host()
            .document
            .as_ref()
            .unwrap()
            .stamp()
            .unwrap()
            .owner;
        let callbacks = self.host().callbacks();
        let mut owner = callbacks.borrow_mut();
        let key = owner.register_prepared_open(&scope).unwrap();
        (key, owner.chooser_prepared().unwrap().clone())
    }
    fn cleanup(&mut self) -> Result<(), String> {
        self.probe.hold.store(false, Ordering::Release);
        let end = Instant::now() + Duration::from_secs(5);
        if self.runtime.is_some() || self.host.is_some() {
            loop {
                let progress = match self.host_mut().poll_unrun_shutdown() {
                    Ok(progress) => progress,
                    Err(error) => {
                        let diagnostic = format!("{error:?}");
                        self.cleanup_carrier = Some(Box::new(error));
                        return Err(diagnostic);
                    }
                };
                match progress {
                    ViewerCloseProgress::Complete { execution, .. }
                        if execution.worker_stopped && execution.unacknowledged == 0 =>
                    {
                        break;
                    }
                    _ if Instant::now() < end => std::thread::sleep(Duration::from_millis(1)),
                    _ => return Err("Original Viewer close pending".into()),
                }
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
                let progress = match owner.poll_close() {
                    Ok(progress) => progress,
                    Err(error) => {
                        let diagnostic = format!("{error:?}");
                        self.cleanup_carrier = Some(Box::new(error));
                        return Err(diagnostic);
                    }
                };
                if let crate::application::sdk_execution::ExecutionCloseProgress::Closed(receipt) =
                    progress
                {
                    if receipt.worker_stopped && receipt.unacknowledged == 0 {
                        break;
                    }
                    return Err("Original SDK incomplete".into());
                }
                if Instant::now() >= end {
                    return Err("Original SDK close pending".into());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        self.input_exports.borrow_mut().clear();
        Ok(())
    }
}
struct Failure {
    setup: Option<String>,
    setup_panic: Option<Box<dyn Any + Send>>,
    primary: Option<Box<dyn Any + Send>>,
    cleanup: Option<String>,
    cleanup_panic: Option<Box<dyn Any + Send>>,
    diagnostic: Option<std::io::Error>,
    _rig: Rig,
}
impl std::fmt::Debug for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveEntryOriginalOwnerFailure")
            .field("setup", &self.setup)
            .field("setup_carrier", &self._rig.setup_carrier.is_some())
            .field("cleanup_carrier", &self._rig.cleanup_carrier.is_some())
            .field("admission_errors", &self._rig.admission_errors)
            .field("close_progress", &self._rig.close_progress)
            .field("runtime_errors", &self._rig.runtime_errors)
            .field("typed_host_errors", &self._rig.typed_host_errors)
            .field("setup_panic", &self.setup_panic.is_some())
            .field("primary_panic", &self.primary.is_some())
            .field("cleanup", &self.cleanup)
            .field("cleanup_panic", &self.cleanup_panic.is_some())
            .field("diagnostic", &self.diagnostic)
            .finish()
    }
}
fn with_rig(variant: u8, body: impl FnOnce(&mut Rig)) -> Result<(), Box<Failure>> {
    run_rig(variant, false, body)
}
fn with_chooser_rig(body: impl FnOnce(&mut Rig)) -> Result<(), Box<Failure>> {
    run_rig(0, true, body)
}
fn run_rig(variant: u8, chooser: bool, body: impl FnOnce(&mut Rig)) -> Result<(), Box<Failure>> {
    let mut rig = Rig {
        chooser,
        ..Rig::default()
    };
    let (setup, setup_panic) =
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| rig.setup(variant))) {
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
    let mut failure = Box::new(Failure {
        setup,
        setup_panic,
        primary,
        cleanup,
        cleanup_panic,
        diagnostic: None,
        _rig: rig,
    });
    let mut output = std::io::stderr().lock();
    failure.diagnostic = writeln!(
        output,
        "live_entry_closeout complete={} primary_panic={} setup_panic={} cleanup_panic={}",
        failure.cleanup.is_none() && failure.cleanup_panic.is_none(),
        failure.primary.is_some(),
        failure.setup_panic.is_some(),
        failure.cleanup_panic.is_some()
    )
    .and_then(|()| output.flush())
    .err();
    if failure.setup.is_some()
        || failure.setup_panic.is_some()
        || failure.primary.is_some()
        || failure.cleanup.is_some()
        || failure.cleanup_panic.is_some()
        || failure.diagnostic.is_some()
    {
        Err(failure)
    } else {
        Ok(())
    }
}

#[test]
#[ignore = "Assigned original LocalHost/SDK headless slot; no user entry or frame acceptance"]
fn current_duplicate_is_coalesced_and_unknown_capture_or_foreign_handle_is_rejected()
-> Result<(), Box<Failure>> {
    with_rig(0, |rig| {
        let input = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        for key in [
            "live-proposal-7-1",
            "live-proposal-6-0",
            "project-task-7-0-0",
            "",
        ] {
            let pending = rig.input(LIVE_ENTRY_BINDING, CallbackArguments::Text(key.into()));
            assert!(matches!(rig.admit(&pending), IntentAdmission::Rejected(_)));
        }
        let scope = rig.host().document.as_ref().unwrap().stamp().unwrap().owner;
        let mut foreign = HostCallbacks::new(scope);
        foreign
            .bind_local(LIVE_ENTRY_BINDING.into(), CallbackSignature::Text)
            .unwrap();
        let export = foreign.export_bindings().unwrap();
        let mut pending = input;
        pending.handle = export
            .resolve(&CallbackReference {
                binding: LIVE_ENTRY_BINDING.into(),
                signature: CallbackSignature::Text,
            })
            .unwrap();
        assert!(matches!(rig.admit(&pending), IntentAdmission::Rejected(_)));
        assert_eq!(rig.calls(), 0);
    })
}

#[test]
#[ignore = "Assigned original LocalHost/SDK headless slot"]
fn jobs_and_live_intents_are_mutually_exclusive_in_both_orders() -> Result<(), Box<Failure>> {
    for live_first in [false, true] {
        with_rig(0, |rig| {
            let live = rig.input(
                LIVE_ENTRY_BINDING,
                CallbackArguments::Text("live-proposal-7-0".into()),
            );
            let jobs = rig.input("onOpenJobs", CallbackArguments::Action);
            let (first, second) = if live_first {
                (&live, &jobs)
            } else {
                (&jobs, &live)
            };
            assert!(matches!(rig.admit(first), IntentAdmission::Accepted));
            assert!(matches!(rig.admit(second), IntentAdmission::Rejected(_)));
            assert_eq!(rig.calls(), 0);
        })?;
    }
    Ok(())
}

#[test]
#[ignore = "Assigned original LocalHost/SDK headless slot"]
fn callback_suffix_keeps_authenticated_intent_without_inspect_or_route_commit()
-> Result<(), Box<Failure>> {
    with_rig(0, |rig| {
        let input = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        let stamp = rig.host().document.as_ref().unwrap().stamp().unwrap();
        assert!(
            rig.host_mut()
                .advance_live_entry(&HostTurn {
                    stamp,
                    reason: HostTurnReason::Input,
                    accepted_callbacks: 1,
                    rejected_callbacks: 0,
                    remaining_callbacks: 1
                })
                .unwrap()
                .is_none()
        );
        assert!(rig.host().callbacks().borrow().has_live_entry());
        assert!(rig.host().live_entry_phase().is_none());
        assert_eq!(rig.calls(), 0);
    })
}

#[test]
#[ignore = "Assigned original LocalHost/SDK headless slot; model observation only"]
fn one_original_sdk_inspection_is_acked_released_and_retained_without_editor_or_observed_route()
-> Result<(), Box<Failure>> {
    with_rig(0, |rig| {
        assert!(rig.handle.as_ref().unwrap().attach().is_err());
        let input = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        rig.settle();
        assert_eq!(
            rig.host().live_entry_phase(),
            Some(LiveEntryPhase::Inspected)
        );
        assert_eq!(rig.calls(), 1);
        let cards = rig.host().cards.borrow();
        assert!(cards.active.is_none() && cards.mount_failure.is_none());
        assert!(cards.inspection.as_ref().unwrap().is_released());
        assert_eq!(
            cards
                .inspection
                .as_ref()
                .unwrap()
                .selection()
                .unwrap()
                .source_revision
                .0,
            9_007_199_254_740_993
        );
        drop(cards);
        assert_eq!(
            rig.host()
                .route_observation
                .as_ref()
                .unwrap()
                .borrow()
                .state
                .observed_route()
                .unwrap()
                .as_str(),
            "view.start"
        );
        let claim = rig.handle.as_ref().unwrap().attach().unwrap();
        claim.release().unwrap();
        rig.tick();
        rig.tick();
        assert_eq!(rig.calls(), 1);
    })
}

#[test]
#[ignore = "Assigned original LocalHost/SDK headless slot"]
fn stale_live_identity_keeps_successful_raw_terminal_and_never_reinspects()
-> Result<(), Box<Failure>> {
    with_rig(1, |rig| {
        let input = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        rig.settle();
        assert_eq!(rig.host().live_entry_phase(), Some(LiveEntryPhase::Failed));
        let cards = rig.host().cards.borrow();
        assert_eq!(
            cards.live_opening.as_ref().unwrap().validation_error,
            Some(BootstrapError::StaleSelection)
        );
        assert!(
            cards
                .inspection
                .as_ref()
                .unwrap()
                .terminal()
                .unwrap()
                .outcome
                .is_ok()
        );
        assert!(cards.active.is_none());
        drop(cards);
        rig.tick();
        assert_eq!(rig.calls(), 1);
    })
}

#[test]
#[ignore = "Assigned original LocalHost/SDK headless slot"]
fn failed_original_inspection_keeps_canonical_error_and_original_close_owner()
-> Result<(), Box<Failure>> {
    with_rig(2, |rig| {
        let input = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        rig.settle();
        assert_eq!(rig.host().live_entry_phase(), Some(LiveEntryPhase::Failed));
        let cards = rig.host().cards.borrow();
        let terminal = cards.inspection.as_ref().unwrap().terminal().unwrap();
        assert!(
            matches!(&terminal.outcome,Err(error) if error.code==crate::application::api::ErrorCode::NotFound)
        );
        assert!(cards.live_opening.as_ref().unwrap().first_error.is_some());
        drop(cards);
        rig.tick();
        assert_eq!(rig.calls(), 1);
    })
}

#[test]
#[ignore = "Assigned original LocalHost/SDK headless slot"]
fn close_during_pending_inspect_uses_same_worker_ticket_and_preserves_entry()
-> Result<(), Box<Failure>> {
    with_rig(0, |rig| {
        rig.probe.hold.store(true, Ordering::Release);
        let input = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        rig.tick();
        let end = Instant::now() + Duration::from_secs(3);
        while !rig.probe.entered.load(Ordering::Acquire) {
            rig.tick();
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            rig.host().live_entry_phase(),
            Some(LiveEntryPhase::Inspecting)
        );
        let first = rig.host_mut().poll_unrun_shutdown().unwrap();
        rig.close_progress.push(first.clone());
        assert!(!matches!(first, ViewerCloseProgress::Complete { .. }));
        assert!(rig.host().shutdown_started);
        assert!(matches!(rig.admit(&input), IntentAdmission::Rejected(_)));
        assert!(rig.host().cards.borrow().inspection.is_some());
        // Endpoint joins precede callback/Jobs/Card release. The first poll may
        // still own a real pending endpoint; retain it and use the same deadline.
        while rig.host().live_entry_phase() != Some(LiveEntryPhase::Closing) {
            assert!(
                Instant::now() < end,
                "Original endpoint-first close did not reach the retained live owner"
            );
            let progress = rig.host_mut().poll_unrun_shutdown().unwrap();
            rig.close_progress.push(progress.clone());
            assert!(!matches!(progress, ViewerCloseProgress::Complete { .. }));
            assert!(rig.host().cards.borrow().inspection.is_some());
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(rig.host().live_entry_phase(), Some(LiveEntryPhase::Closing));
        assert!(matches!(rig.admit(&input), IntentAdmission::Rejected(_)));
        rig.probe.hold.store(false, Ordering::Release);
        assert_eq!(rig.calls(), 1);
    })
}

#[test]
#[ignore = "Assigned R2 original LocalHost/SDK slot; no actual checked-frame acceptance"]
fn committed_live_mount_returns_exact_first_effect_even_when_observer_borrow_is_busy()
-> Result<(), Box<Failure>> {
    with_rig(0, |rig| {
        let input = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        rig.settle();
        let observer = rig.host().route_observation.as_ref().unwrap().clone();
        let held = observer.borrow_mut();
        let outcome = rig
            .runtime
            .as_mut()
            .unwrap()
            .after_dispatch(HostTurnReason::Completion);
        let effect = match outcome {
            Ok(effect) => effect,
            Err(error) => {
                rig.runtime_errors.push(error);
                if let Some(error) = rig.runtime.as_ref().unwrap().last_host_error().cloned() {
                    rig.typed_host_errors.push(error);
                }
                panic!("Positive mount fixture must not suppress a runtime error");
            }
        };
        let receipt = effect
            .committed
            .as_ref()
            .expect("Actual original props commit must be returned synchronously");
        {
            let cards = rig.host().cards.borrow();
            let entry = cards.live_opening.as_ref().unwrap();
            assert_eq!(entry.receipt.as_ref(), Some(receipt));
            assert_eq!(entry.proof.as_ref().unwrap().receipt(), receipt);
            assert_eq!(entry.phase, LiveEntryPhase::AwaitingArm);
            assert!(!entry.observed && !entry.gate.get());
            assert!(entry.first_error.is_some() && entry.error.is_some());
            assert!(cards.active.is_some());
            assert!(cards.inspection.is_none());
            assert!(entry.retained_inspection.as_ref().unwrap().is_released());
        }
        let retry = rig
            .runtime
            .as_mut()
            .unwrap()
            .after_dispatch(HostTurnReason::Completion)
            .unwrap();
        assert!(
            retry.committed.is_none(),
            "Rearm cannot replay the old synchronous receipt"
        );
        drop(held);
        rig.tick(); // Model-only initial observer has no actual frame context; remains unobserved.
        let cards = rig.host().cards.borrow();
        let entry = cards.live_opening.as_ref().unwrap();
        assert_eq!(entry.receipt.as_ref(), Some(receipt));
        rig.host()
            .document
            .as_ref()
            .unwrap()
            .validate_retained_commit(entry.proof.as_ref().unwrap())
            .unwrap();
        assert!(!entry.observed && !entry.gate.get());
        assert_eq!(rig.calls(), 1);
    })
}

#[test]
#[ignore = "Assigned R2 original LocalHost/SDK slot; no actual checked-frame acceptance"]
fn live_input_and_return_are_blocked_until_original_entry_observation() -> Result<(), Box<Failure>>
{
    with_rig(0, |rig| {
        let pending = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&pending), IntentAdmission::Accepted));
        rig.settle();
        rig.tick();
        assert!(matches!(rig.admit(&pending), IntentAdmission::Rejected(_)));
        let handle = rig.host().document.as_ref().unwrap().clone();
        let turn = HostTurn {
            stamp: handle.stamp().unwrap(),
            reason: HostTurnReason::Input,
            accepted_callbacks: 0,
            rejected_callbacks: 0,
            remaining_callbacks: 0,
        };
        assert!(rig.host_mut().request_viewing_return(&turn).is_err());
        assert!(rig.host().cards.borrow().viewing_return.is_none());
        assert!(
            !rig.host()
                .cards
                .borrow()
                .live_opening
                .as_ref()
                .unwrap()
                .gate
                .get()
        );
        assert_eq!(rig.calls(), 1);
    })
}

#[test]
#[ignore = "Assigned R2 original LocalHost/SDK slot"]
fn closed_inventory_failure_keeps_exact_released_inspection_and_original_mount_input()
-> Result<(), Box<Failure>> {
    with_rig(0, |rig| {
        let pending = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&pending), IntentAdmission::Accepted));
        rig.settle();
        let path: BuildPath = "studio/clips/highlight-proposal-panel.tsx"
            .to_owned()
            .try_into()
            .unwrap();
        rig.host_mut()
            .product_frame
            .as_mut()
            .unwrap()
            .inventory
            .remove(&path);
        let before = rig.host().document.as_ref().unwrap().stamp().unwrap();
        // Host error is deliberately retained by runtime; do not unwrap it into a fake success.
        match rig
            .runtime
            .as_mut()
            .unwrap()
            .after_dispatch(HostTurnReason::Completion)
        {
            Err(error) => rig.expected_carrier = Some(Box::new(error)),
            Ok(_) => panic!("Original missing inventory must fail without a commit"),
        }
        let cards = rig.host().cards.borrow();
        let entry = cards.live_opening.as_ref().unwrap();
        assert_eq!(entry.phase, LiveEntryPhase::Failed);
        assert!(entry.receipt.is_none() && entry.proof.is_none());
        assert!(entry.mount_failure.is_some());
        assert!(entry.retained_inspection.as_ref().unwrap().is_released());
        let input = &entry.mount_failure.as_ref().unwrap().input;
        let current = rig.host().document.as_ref().unwrap().stamp().unwrap();
        assert_eq!(input.document.stamp().unwrap(), current);
        assert_eq!(input.turn.stamp, current);
        assert_eq!(current.host_id, before.host_id);
        assert_eq!(current.owner, before.owner);
        assert_eq!(current.definition_revision, before.definition_revision);
        assert_eq!(current.document_revision, before.document_revision);
        assert_eq!(
            current.turn_sequence,
            before.turn_sequence.checked_add(1).unwrap()
        );
        assert!(cards.active.is_none());
        assert_eq!(rig.calls(), 1);
    })
}

#[test]
#[ignore = "Assigned R2 original LocalHost/SDK slot"]
fn close_keeps_the_committed_unobserved_live_owner_on_the_original_cleanup_ledger()
-> Result<(), Box<Failure>> {
    with_rig(0, |rig| {
        let pending = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&pending), IntentAdmission::Accepted));
        rig.settle();
        rig.tick();
        assert!(rig.host().cards.borrow().active.is_some());
        let end = Instant::now() + Duration::from_secs(3);
        let progress = rig.host_mut().poll_unrun_shutdown().unwrap();
        rig.close_progress.push(progress);
        assert!(rig.host().shutdown_started);
        // Inherit endpoint-first observation for this R2 owner as well. Unlike
        // the held R1 worker, this already-idle owner may finish in the same poll.
        while rig.host().live_entry_phase() != Some(LiveEntryPhase::Closing) {
            assert!(
                Instant::now() < end,
                "Original committed-owner close did not reach its live phase"
            );
            let progress = rig.host_mut().poll_unrun_shutdown().unwrap();
            rig.close_progress.push(progress);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(rig.host().live_entry_phase(), Some(LiveEntryPhase::Closing));
        assert!(
            rig.host()
                .cards
                .borrow()
                .live_opening
                .as_ref()
                .unwrap()
                .receipt
                .is_some()
        );
        assert!(matches!(rig.admit(&pending), IntentAdmission::Rejected(_)));
    })
}

#[test]
fn fixture_notifier_failure_does_not_count_as_an_original_wake() {
    use crate::assembly::meari::viewer_native_probe::{
        Probe as NativeProbe,
        live_retained::{Case, Mode},
    };
    struct Original(std::sync::atomic::AtomicUsize);
    impl CompletionNotifier for Original {
        fn notify(&self) -> Result<(), ApiError> {
            self.0.fetch_add(1, Ordering::AcqRel);
            Ok(())
        }
    }
    let original = Arc::new(Original(std::sync::atomic::AtomicUsize::new(0)));
    let probe = NativeProbe::for_live(Case::Highlight, Mode::WakeFailure);
    let decorated = probe.live_notifier(original.clone());
    let error = decorated.notify().unwrap_err();
    assert_eq!(error.code, crate::application::api::ErrorCode::Internal);
    assert_eq!(original.0.load(Ordering::Acquire), 0);
    decorated.notify().unwrap();
    assert_eq!(original.0.load(Ordering::Acquire), 1);
    // A delegate contract only. Actual startup wake progression belongs to the checked child.
}

#[test]
#[ignore = "Original SDK owner slot; injected proof failure is a model, not a Meari or frame fault"]
fn model_entry_proof_failure_keeps_first_effect_and_original_close_owner()
-> Result<(), Box<Failure>> {
    use crate::assembly::meari::viewer_native_probe::live_retained::{ModelProofScope, ProofStage};
    with_rig(0, |rig| {
        let pending = rig.input(
            LIVE_ENTRY_BINDING,
            CallbackArguments::Text("live-proposal-7-0".into()),
        );
        assert!(matches!(rig.admit(&pending), IntentAdmission::Accepted));
        rig.settle();
        let _fault = ModelProofScope::enter(Some(ProofStage::Entry));
        let effect = rig
            .runtime
            .as_mut()
            .unwrap()
            .after_dispatch(HostTurnReason::Completion)
            .unwrap();
        let receipt = effect
            .committed
            .expect("Real first commit remains synchronous on model proof error");
        {
            let cards = rig.host().cards.borrow();
            let entry = cards.live_opening.as_ref().unwrap();
            assert_eq!(entry.receipt.as_ref(), Some(&receipt));
            assert!(entry.proof.is_none());
            assert!(matches!(
                entry.proof_error.as_deref(),
                Some(HostAccessError::Busy)
            ));
            assert_eq!(entry.phase, LiveEntryPhase::Failed);
            assert!(cards.active.is_some());
            assert!(!entry.observed && !entry.gate.get());
        }
        let next = rig
            .runtime
            .as_mut()
            .unwrap()
            .after_dispatch(HostTurnReason::Completion)
            .unwrap();
        assert!(
            next.committed.is_none(),
            "No historical proof retry or recommit"
        );
        let cards = rig.host().cards.borrow();
        let entry = cards.live_opening.as_ref().unwrap();
        assert_eq!(entry.receipt.as_ref(), Some(&receipt));
        assert!(entry.proof.is_none());
        assert!(matches!(
            entry.proof_error.as_deref(),
            Some(HostAccessError::Busy)
        ));
        assert_eq!(entry.phase, LiveEntryPhase::Failed);
        assert!(!entry.observed && !entry.gate.get());
        assert_eq!(rig.calls(), 1);
    })
}

#[test]
#[ignore = "Assigned prepared Open LocalHost/SDK headless slot; no source button or frame acceptance"]
fn prepared_open_moves_exact_carrier_then_runs_one_target_bound_current_inspection()
-> Result<(), Box<Failure>> {
    with_chooser_rig(|rig| {
        let (key, prepared) = rig.prepare_chooser();
        assert_eq!(rig.calls(), 1, "Preparation inspect is distinct from R1");
        assert!(rig.handle.as_ref().unwrap().attach().is_err());
        let input = rig.input(LIVE_ENTRY_BINDING, CallbackArguments::Text(key));
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        rig.settle();
        assert_eq!(
            rig.host().live_entry_phase(),
            Some(LiveEntryPhase::Inspected)
        );
        assert_eq!(rig.calls(), 2, "Exactly one R1 inspect after preparation");
        let cards = rig.host().cards.borrow();
        let opening = cards.live_opening.as_ref().unwrap();
        assert!(Arc::ptr_eq(&opening.request.selected, &prepared.selection));
        let retained = opening.prepared.as_ref().unwrap();
        assert_eq!(retained.scope, prepared.scope);
        assert_eq!(retained.generation, prepared.generation);
        assert!(Arc::ptr_eq(&retained.selection, &prepared.selection));
        assert_eq!(
            opening.request.selected.target,
            NativeProposalTarget::Highlight { index: 1 }
        );
        assert!(cards.inspection.as_ref().unwrap().is_released());
        assert!(cards.inspection.as_ref().unwrap().terminal().is_some());
        assert!(cards.active.is_none());
        drop(cards);
        assert!(!rig.host().callbacks().borrow().has_live_entry());
        assert!(rig.host().callbacks().borrow().chooser_prepared().is_none());
        assert!(matches!(rig.admit(&input), IntentAdmission::Rejected(_)));
        assert_eq!(rig.calls(), 2);
        let next = rig.handle.as_ref().unwrap().attach().unwrap();
        next.release().unwrap();
    })
}

#[test]
#[ignore = "Assigned prepared Open LocalHost/SDK headless slot; route model guard only"]
fn prepared_open_suffix_and_route_error_preserve_original_intent_before_transfer()
-> Result<(), Box<Failure>> {
    with_chooser_rig(|rig| {
        let (key, prepared) = rig.prepare_chooser();
        let input = rig.input(LIVE_ENTRY_BINDING, CallbackArguments::Text(key));
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        let stamp = rig.host().document.as_ref().unwrap().stamp().unwrap();
        assert!(
            rig.host_mut()
                .advance_live_entry(&HostTurn {
                    stamp: stamp.clone(),
                    reason: HostTurnReason::Input,
                    accepted_callbacks: 1,
                    rejected_callbacks: 0,
                    remaining_callbacks: 1,
                })
                .unwrap()
                .is_none()
        );
        assert!(rig.host().live_entry_phase().is_none());
        assert_eq!(rig.calls(), 1);
        let observer = rig.host().route_observation.as_ref().unwrap().clone();
        let blocker = observer
            .borrow_mut()
            .state
            .request_route(
                &RouteCatalog::bundled().unwrap(),
                "view.start",
                NavigationGuards::default(),
            )
            .unwrap();
        let error = rig
            .host_mut()
            .advance_live_entry(&HostTurn {
                stamp: stamp.clone(),
                reason: HostTurnReason::Completion,
                accepted_callbacks: 0,
                rejected_callbacks: 0,
                remaining_callbacks: 0,
            })
            .err()
            .unwrap();
        assert!(matches!(
            &error,
            ViewerHostError::Navigation(
                crate::presentation::web_ui_adapter::player_shell::NavigationError::Busy
            )
        ));
        rig.expected_carrier = Some(Box::new(error));
        assert!(rig.host().live_entry_phase().is_none());
        let (request, carrier) = rig
            .host()
            .callbacks()
            .borrow()
            .live_entry_handoff(&stamp.owner)
            .unwrap()
            .unwrap();
        assert!(Arc::ptr_eq(&request.selected, &prepared.selection));
        assert!(Arc::ptr_eq(
            &carrier.unwrap().selection,
            &prepared.selection
        ));
        assert_eq!(rig.calls(), 1);
        observer.borrow_mut().state.cancel_route(&blocker).unwrap();
        rig.settle();
        assert_eq!(rig.calls(), 2);
        assert_eq!(
            rig.host().live_entry_phase(),
            Some(LiveEntryPhase::Inspected)
        );
    })
}

#[test]
#[ignore = "Assigned prepared Open LocalHost/SDK headless slot; real fixture-store stale revision"]
fn prepared_open_stale_current_document_retains_raw_terminal_and_carrier_without_reinspect()
-> Result<(), Box<Failure>> {
    with_chooser_rig(|rig| {
        let (key, prepared) = rig.prepare_chooser();
        let changed = rig
            .host()
            .client
            .edit_proposal(
                RequestContext {
                    expected_revision: Some(prepared.selection.document_revision.0.to_string()),
                    ..RequestContext::default()
                },
                crate::application::proposals::EditProposal {
                    proposal_id: "live-proposal".into(),
                    draft: ProposalPayload::Highlights {
                        highlights: vec![
                            Highlight {
                                title: "changed first".into(),
                                source_refs: vec!["cue-a".into()],
                            },
                            Highlight {
                                title: "changed second".into(),
                                source_refs: vec!["cue-a".into()],
                            },
                        ],
                    },
                },
            )
            .unwrap();
        assert_ne!(changed.draft_revision, prepared.selection.document_revision);
        rig.expected_failure = Some(1);
        let input = rig.input(LIVE_ENTRY_BINDING, CallbackArguments::Text(key));
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        rig.settle();
        assert_eq!(rig.host().live_entry_phase(), Some(LiveEntryPhase::Failed));
        let cards = rig.host().cards.borrow();
        let opening = cards.live_opening.as_ref().unwrap();
        assert_eq!(
            opening.validation_error,
            Some(BootstrapError::StaleSelection)
        );
        assert!(opening.first_error.is_some());
        assert!(Arc::ptr_eq(
            &opening.prepared.as_ref().unwrap().selection,
            &prepared.selection
        ));
        assert!(
            cards
                .inspection
                .as_ref()
                .unwrap()
                .terminal()
                .unwrap()
                .outcome
                .is_ok()
        );
        assert!(cards.active.is_none());
        drop(cards);
        rig.tick();
        assert_eq!(rig.calls(), 2);
    })
}

#[test]
#[ignore = "Assigned prepared Open LocalHost/SDK headless slot; original pending-read close owner"]
fn prepared_open_close_during_current_inspection_keeps_same_carrier_and_original_worker()
-> Result<(), Box<Failure>> {
    with_chooser_rig(|rig| {
        let (key, prepared) = rig.prepare_chooser();
        rig.probe.entered.store(false, Ordering::Release);
        rig.probe.hold.store(true, Ordering::Release);
        let input = rig.input(LIVE_ENTRY_BINDING, CallbackArguments::Text(key));
        assert!(matches!(rig.admit(&input), IntentAdmission::Accepted));
        let end = Instant::now() + Duration::from_secs(3);
        while !rig.probe.entered.load(Ordering::Acquire) {
            rig.tick();
            assert!(Instant::now() < end, "Original R1 inspect did not start");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            rig.host().live_entry_phase(),
            Some(LiveEntryPhase::Inspecting)
        );
        assert_eq!(rig.calls(), 2);
        while rig.host().live_entry_phase() != Some(LiveEntryPhase::Closing) {
            let progress = rig.host_mut().poll_unrun_shutdown().unwrap();
            assert!(!matches!(progress, ViewerCloseProgress::Complete { .. }));
            rig.close_progress.push(progress);
            assert!(
                Instant::now() < end,
                "Original close did not retain pending inspect"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        let cards = rig.host().cards.borrow();
        let opening = cards.live_opening.as_ref().unwrap();
        assert!(Arc::ptr_eq(&opening.request.selected, &prepared.selection));
        assert!(Arc::ptr_eq(
            &opening.prepared.as_ref().unwrap().selection,
            &prepared.selection
        ));
        assert!(cards.inspection.is_some());
        drop(cards);
        assert!(matches!(rig.admit(&input), IntentAdmission::Rejected(_)));
        rig.probe.hold.store(false, Ordering::Release);
        assert_eq!(rig.calls(), 2);
    })
}
