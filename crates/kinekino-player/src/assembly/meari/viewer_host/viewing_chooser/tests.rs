use super::*;
use crate::application::api::ProposalClient;
use crate::{
    application::{
        api::{ApiError, ApplicationClientHandle, PlayerSdk, RequestContext},
        proposals::CreateProposal,
        sdk_execution::{
            CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner,
        },
    },
    domain::{api_values::DecimalU64, proposals::*},
};
use meari::{
    application::{Language, host_callbacks::CallbackBindings, state_program::StateProgram},
    domain::{
        Span,
        host_callbacks::{
            CallbackArguments, CallbackInvocation, CallbackReference, CallbackSignature,
        },
    },
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn scope(epoch: u64) -> PluginScope {
    let component = meari::parse(
        "type Props={};export function Scope(props:Props){return <main/>;}",
        Language::Tsx,
    )
    .unwrap();
    let program = StateProgram::mount(component, "viewing-chooser-test".into(), epoch).unwrap();
    PluginScope {
        instance: program.snapshot().unwrap().instance,
        source_generation: 1,
    }
}

#[test]
fn committed_display_coalesces_only_identical_scope_and_values() {
    let scope = scope(1);
    let values = proposal_chooser::props(ChooserView::default(), ChooserLabels::korean()).unwrap();
    let mut projection = Projection::default();
    assert!(projection.changed(&scope, &values));
    projection.committed(scope.clone(), values.clone());
    assert!(!projection.changed(&scope, &values));
    let mut changed = values.clone();
    changed.insert("status".into(), Value::String("새 결과".into()));
    assert!(projection.changed(&scope, &changed));
    assert!(!projection.changed(&scope, &values));
}

#[test]
fn new_scope_cannot_reuse_the_previous_committed_display_snapshot() {
    let first = scope(1);
    let second = scope(2);
    let values = proposal_chooser::props(ChooserView::default(), ChooserLabels::korean()).unwrap();
    let mut projection = Projection::default();
    projection.committed(first.clone(), values.clone());
    assert!(projection.changed(&second, &values));
    projection.committed(second.clone(), values.clone());
    assert!(projection.changed(&first, &values));
    assert!(!projection.changed(&second, &values));
}

#[test]
fn display_number_does_not_wrap_or_change_the_supplied_target() {
    let target = NativeProposalTarget::Card { index: u32::MAX };
    assert_eq!(item_label(&target), "카드 4294967296");
    assert_eq!(target, NativeProposalTarget::Card { index: u32::MAX });
}

struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), ApiError> {
        Ok(())
    }
}

struct Fixture {
    execution: ExecutionOwner,
    owner: ViewerCallbacks,
    scope: PluginScope,
    bindings: CallbackBindings,
    initial_root: BTreeMap<String, Value>,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::current_dir()
            .unwrap()
            .join("runtime/quality-review/ui-viewing-chooser")
            .join(format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        std::fs::create_dir_all(&root).unwrap();
        println!("retained Viewing chooser fixture store: {}", root.display());
        let client: ApplicationClientHandle = Arc::new(PlayerSdk::default().with_proposal_store(
            crate::infrastructure::proposals::ProposalFiles::open(&root).unwrap(),
        ));
        let payload = ProposalPayload::Cards {
            cards: vec![
                Card {
                    front: "first".into(),
                    back: "first back".into(),
                    source_refs: vec!["cue-1".into()],
                },
                Card {
                    front: "explicit second".into(),
                    back: "second back".into(),
                    source_refs: vec!["cue-1".into()],
                },
            ],
        };
        client
            .create_proposal(
                RequestContext::default(),
                CreateProposal {
                    proposal_id: "chooser-ui-proposal".into(),
                    source: SourceSnapshot::Cue(CueSourceSnapshot {
                        source_id: "chooser-ui-source".into(),
                        source_revision: DecimalU64(1),
                        source_sha256: "a".repeat(64),
                        cues: vec![SourceCue {
                            cue_id: "cue-1".into(),
                            text: "원문 日本語".into(),
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
        let execution = ExecutionOwner::start(
            client,
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
        let mut owner = ViewerCallbacks::new(execution.handle(), RequestContext::default());
        let mut initial_root =
            crate::presentation::web_ui_adapter::player_shell::document::read_props(
                &crate::presentation::web_ui_adapter::player_shell::document::StartLabels {
                    connection: "연결".into(),
                    time: "00:00".into(),
                },
            );
        configure_initial(&mut owner, &mut initial_root).unwrap();
        let scope = scope(1);
        let bindings = owner.export(scope.clone()).unwrap();
        Self {
            execution,
            owner,
            scope,
            bindings,
            initial_root,
        }
    }
    fn pending(&self, prop: &str, arguments: CallbackArguments) -> PendingCallback {
        let signature = match &arguments {
            CallbackArguments::Action => CallbackSignature::Action,
            _ => CallbackSignature::Text,
        };
        PendingCallback {
            handle: self
                .bindings
                .resolve(&CallbackReference {
                    binding: prop.into(),
                    signature,
                })
                .unwrap(),
            invocation: CallbackInvocation {
                prop: prop.into(),
                arguments,
                span: Span::default(),
            },
        }
    }
    fn act(&mut self, prop: &str, arguments: CallbackArguments) {
        let callback = self.pending(prop, arguments);
        self.owner.queue_chooser(&self.scope, &callback).unwrap();
        self.owner.drain_chooser(&self.scope).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !self.owner.tickets().unwrap().is_empty() {
            self.owner.consume_reads().unwrap();
            assert!(
                Instant::now() < deadline,
                "original SDK terminal/ACK must settle"
            );
            std::thread::yield_now();
        }
    }
    fn list_and_select_document(&mut self) -> String {
        self.act("onChooserList", CallbackArguments::Action);
        let key = self.owner.chooser_documents()[0].key.clone();
        self.act("onChooserDocument", CallbackArguments::Text(key.clone()));
        key
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.owner.stop_admission();
        let deadline = Instant::now() + Duration::from_secs(2);
        let closed = loop {
            let _ = self.owner.consume_reads();
            if self.owner.prepare_route_release(&self.scope).ok() == Some(None)
                && let Ok(ExecutionCloseProgress::Closed(receipt)) = self.execution.poll_close()
            {
                break Some(receipt);
            }
            if Instant::now() >= deadline {
                break None;
            }
            std::thread::yield_now();
        };
        if !std::thread::panicking() {
            let receipt = closed.expect("original chooser execution must close");
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            assert!(self.owner.is_released());
        }
    }
}

fn assert_initial_product_source(root: BTreeMap<String, Value>) {
    use crate::presentation::web_ui_adapter::player_shell::{
        RouteCatalog,
        document::{self, StartLabels, product_frame::ProductFrameBlueprint},
    };
    let labels = StartLabels {
        connection: "연결".into(),
        time: "00:00".into(),
    };
    let mut viewing =
        document::prepare_start_document(&RouteCatalog::bundled().unwrap(), "view.start", labels)
            .ok()
            .unwrap();
    document::enable_read_panel(&mut viewing, root.clone())
        .ok()
        .unwrap();
    let path: meari::domain::build::BuildPath =
        "chooser-test-card.tsx".to_owned().try_into().unwrap();
    let sources = BTreeMap::from([(
        path.clone(),
        "export function Card(){return <article/>;}".into(),
    )]);
    let cards = meari::parse_module_graph(&path, &sources).unwrap();
    let frame =
        ProductFrameBlueprint::prepare(viewing.graph, viewing.inventory, cards, sources).unwrap();
    let program = frame
        .mount_viewing(
            root,
            "viewing-chooser-product-source".into(),
            1,
            std::num::NonZeroU64::new(1).unwrap(),
        )
        .unwrap();
    let projection = serde_json::to_string(&program.project().unwrap().component).unwrap();
    assert!(projection.contains("제안 목록 새로 조회"));
    assert!(projection.contains("선택한 제안 열기"));
}

#[test]
#[ignore = "Assigned headless original SDK/store and fixed ModuleProgram source only; no OS or visual acceptance"]
fn initial_source_and_explicit_document_item_project_one_current_open_key() {
    let mut fixture = Fixture::new();
    assert_initial_product_source(fixture.initial_root.clone());
    let initial = display(&mut fixture.owner, Some(&fixture.scope))
        .unwrap()
        .unwrap();
    assert_eq!(initial["listDisabled"], Value::Bool(false));
    assert_eq!(initial["openDisabled"], Value::Bool(true));
    let document_key = fixture.list_and_select_document();
    let unselected = display(&mut fixture.owner, Some(&fixture.scope))
        .unwrap()
        .unwrap();
    assert_eq!(
        unselected["selectedDocumentKey"],
        Value::String(document_key)
    );
    assert_eq!(unselected["selectedItemKey"], Value::String(String::new()));
    assert_eq!(unselected["openDisabled"], Value::Bool(true));
    let item_key = fixture.owner.chooser_items()[1].key.clone();
    fixture.act("onChooserItem", CallbackArguments::Text(item_key.clone()));
    let original = fixture.owner.chooser_prepared().unwrap().selection.clone();
    let selected = display(&mut fixture.owner, Some(&fixture.scope))
        .unwrap()
        .unwrap();
    assert_eq!(selected["selectedItemKey"], Value::String(item_key));
    assert_eq!(selected["openDisabled"], Value::Bool(false));
    assert!(matches!(&selected["openKey"], Value::String(key) if !key.is_empty()));
    assert_eq!(
        display(&mut fixture.owner, Some(&fixture.scope)).unwrap(),
        Some(selected)
    );
    assert!(Arc::ptr_eq(
        &original,
        &fixture.owner.chooser_prepared().unwrap().selection
    ));
}

#[test]
#[ignore = "Assigned headless original chooser export lifecycle; return frame and OS remain separate"]
fn reexport_retires_old_rows_and_requires_fresh_query_and_explicit_item_again() {
    let mut fixture = Fixture::new();
    let old_document = fixture.list_and_select_document();
    let item = fixture.owner.chooser_items()[1].key.clone();
    fixture.act("onChooserItem", CallbackArguments::Text(item));
    let selected = display(&mut fixture.owner, Some(&fixture.scope))
        .unwrap()
        .unwrap();
    let old_input = fixture.pending(
        "onChooserDocument",
        CallbackArguments::Text(old_document.clone()),
    );
    assert_eq!(
        fixture.owner.prepare_route_release(&fixture.scope).unwrap(),
        None
    );
    fixture.bindings = fixture.owner.resume_released_route(&fixture.scope).unwrap();
    assert!(
        fixture
            .owner
            .queue_chooser(&fixture.scope, &old_input)
            .is_err()
    );
    let returned = display(&mut fixture.owner, Some(&fixture.scope))
        .unwrap()
        .unwrap();
    assert_eq!(
        returned["selectedDocumentKey"],
        Value::String(String::new())
    );
    assert_eq!(returned["selectedItemKey"], Value::String(String::new()));
    assert_eq!(returned["openKey"], Value::String(String::new()));
    assert_eq!(returned["openDisabled"], Value::Bool(true));
    let fresh_document = fixture.list_and_select_document();
    assert_ne!(fresh_document, old_document);
    assert!(fixture.owner.chooser_prepared().is_none());
    let item = fixture.owner.chooser_items()[1].key.clone();
    fixture.act("onChooserItem", CallbackArguments::Text(item));
    let reopened = display(&mut fixture.owner, Some(&fixture.scope))
        .unwrap()
        .unwrap();
    assert_ne!(reopened["openKey"], selected["openKey"]);
}

// Same-runtime integration below is separate from the preceding standalone callback fixtures.
use crate::{
    application::{
        api::{ApplicationClient, ErrorCode, Request, Response},
        proposals::InspectProposal,
    },
    assembly::meari::player_shell_host::StartRouteObservation,
    infrastructure::local_host::{EndpointClient, LocalHost, PendingLocalHost, TransportOptions},
    presentation::web_ui_adapter::{
        player_shell::{
            NavigationGuards, RouteCatalog, RouteHostObservation, RouteTicket, ShellState,
            document::{self, StartLabels, product_frame::ProductFrameBlueprint},
        },
        proposals::{
            bootstrap::BootstrapPanel, frame_composition::FrameComposition,
            product_window::product_modules_for_panel,
        },
    },
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::{
    AsyncCodecBudget, AsyncSdkPort, AsyncSdkPump, TypedAsyncSdkPlugin, async_binding,
};
use meari::application::plugins::{CancelReceipt, PluginTicket};
use meari_native::{
    domain::{
        Viewport,
        widgets::{Action, Family},
    },
    infrastructure::{document::InteractiveDocument, native_host::*},
};
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    sync::Mutex,
};

#[derive(Default)]
struct RuntimeProbe {
    commands: Mutex<Vec<String>>,
}
struct RuntimeClient {
    actual: ApplicationClientHandle,
    probe: Arc<RuntimeProbe>,
}
impl ApplicationClient for RuntimeClient {
    fn execute(&self, request: Request) -> Response {
        self.probe
            .commands
            .lock()
            .unwrap()
            .push(request.command.clone());
        self.actual.execute(request)
    }
}

/// The only injected admission is one Capacity suffix; all successful admission/consumption
/// and all lifecycle policy remain the original ViewerHost on the original document.
struct AuditedHost {
    inner: ViewerHost,
    pause_second: bool,
    admission_count: usize,
    remaining_turns: Vec<usize>,
    accepted: Vec<bool>,
    last_accepted: Option<PendingCallback>,
}
impl NativeWindowHost for AuditedHost {
    type Error = ViewerHostError;
    type CloseCheckpoint = <ViewerHost as NativeWindowHost>::CloseCheckpoint;
    fn admit_callback(
        &mut self,
        stamp: &HostStamp,
        pending: &PendingCallback,
    ) -> IntentAdmission<Self::Error> {
        self.admission_count += 1;
        if self.pause_second && self.admission_count == 2 {
            return IntentAdmission::Capacity;
        }
        let admission = self.inner.admit_callback(stamp, pending);
        self.accepted
            .push(matches!(&admission, IntentAdmission::Accepted));
        if matches!(&admission, IntentAdmission::Accepted) {
            self.last_accepted = Some(PendingCallback {
                handle: pending.handle.clone(),
                invocation: pending.invocation.clone(),
            });
        }
        admission
    }
    fn after_dispatch(&mut self, turn: HostTurn) -> Result<HostEffects, Self::Error> {
        self.remaining_turns.push(turn.remaining_callbacks);
        self.inner.after_dispatch(turn)
    }
    fn retry_after_dispatch(&self, error: &Self::Error) -> Option<HostRetry> {
        self.inner.retry_after_dispatch(error)
    }
    fn next_close_observation(&self, checkpoint: &Self::CloseCheckpoint) -> HostRetry {
        self.inner.next_close_observation(checkpoint)
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
}

struct RuntimeFault {
    fail_once: bool,
    healthy: bool,
    cancelled: bool,
    starts: usize,
    tickets: Vec<PluginTicket>,
    page: serde_json::Value,
}
struct RuntimeFaultPort(Rc<RefCell<RuntimeFault>>);
impl<I: serde::Serialize> AsyncSdkPort<I, serde_json::Value> for RuntimeFaultPort {
    fn start(&mut self, ticket: &PluginTicket, _: I) -> Result<(), PluginError> {
        let mut fault = self.0.borrow_mut();
        fault.starts += 1;
        fault.tickets.push(ticket.clone());
        Ok(())
    }
    fn peek_terminal(
        &mut self,
        _: &PluginTicket,
    ) -> Result<Option<Arc<Result<serde_json::Value, ApiError>>>, PluginError> {
        let mut fault = self.0.borrow_mut();
        if fault.fail_once {
            fault.fail_once = false;
            return Err(PluginError::new(
                PluginErrorCode::Adapter,
                "x".repeat(1024 * 1024),
            ));
        }
        if fault.cancelled {
            return Ok(Some(Arc::new(Err(ApiError::new(
                ErrorCode::Cancelled,
                "explicit fixture cleanup",
            )))));
        }
        Ok(fault.healthy.then(|| Arc::new(Ok(fault.page.clone()))))
    }
    fn acknowledge(&mut self, ticket: &PluginTicket) -> Result<(), PluginError> {
        self.0
            .borrow_mut()
            .tickets
            .retain(|retained| retained != ticket);
        Ok(())
    }
    fn cancel(&mut self, _: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        self.0.borrow_mut().cancelled = true;
        Ok(CancelReceipt::Requested)
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        if self.0.borrow().tickets.is_empty() {
            Ok(())
        } else {
            Err(PluginError::new(
                PluginErrorCode::Busy,
                "original fixture ticket retained",
            ))
        }
    }
}
fn runtime_fault_adapter(fault: Rc<RefCell<RuntimeFault>>) -> (TypedAsyncSdkPlugin, AsyncSdkPump) {
    use crate::assembly::meari::viewer_callbacks::codec::{self, ReadProfile, manifest_for};
    let manifest = manifest_for(ReadProfile::ProposalChooser);
    let mut bindings = BTreeMap::new();
    macro_rules! bind {
        ($command:literal, $input:ty) => {
            bindings.insert(
                $command.into(),
                async_binding::<$input, serde_json::Value, _>(
                    manifest.capabilities[$command].clone(),
                    RuntimeFaultPort(fault.clone()),
                ),
            );
        };
    }
    bind!("capabilities", codec::ReadInput);
    bind!("playback.engine", codec::ReadInput);
    bind!("playback.status", codec::ReadInput);
    bind!("proposals.list", ListProposals);
    bind!("proposals.inspect", InspectProposal);
    TypedAsyncSdkPlugin::new(
        manifest,
        bindings,
        AsyncCodecBudget {
            max_pending: 8,
            max_terminal_wire_bytes: 262144,
            max_total_codec_wire_bytes: 8 * 262144 * 4,
        },
    )
    .unwrap()
}

#[derive(Default)]
struct RuntimeRig {
    runtime: Option<NativeHostRuntime<AuditedHost>>,
    host: Option<ViewerHost>,
    execution: Option<ExecutionOwner>,
    endpoint: Option<LocalHost>,
    pending_endpoint: Option<PendingLocalHost>,
    observer: Option<Rc<RefCell<StartRouteObservation>>>,
    initial_ticket: Option<RouteTicket>,
    fault: Option<Rc<RefCell<RuntimeFault>>>,
    root: Option<PathBuf>,
    probe: Arc<RuntimeProbe>,
    carriers: Vec<Box<dyn Any>>,
    export_errors: Rc<RefCell<Vec<PluginError>>>,
    runtime_errors: Vec<HostAccessError>,
    close_progress: Vec<ViewerCloseProgress>,
}
impl RuntimeRig {
    fn retain_error<T: std::fmt::Debug + 'static>(&mut self, error: T) -> String {
        let message = format!("{error:?}");
        self.carriers.push(Box::new(error));
        message
    }
    fn host(&self) -> &ViewerHost {
        self.runtime
            .as_ref()
            .map(|runtime| &runtime.host().inner)
            .or(self.host.as_ref())
            .unwrap()
    }
    fn host_mut(&mut self) -> &mut ViewerHost {
        if let Some(runtime) = &mut self.runtime {
            &mut runtime.host_mut().inner
        } else {
            self.host.as_mut().unwrap()
        }
    }
    fn handle(&self) -> NativeDocumentHandle {
        self.runtime.as_ref().unwrap().document()
    }
    fn setup(&mut self, observed_model: bool, fault_model: bool) -> Result<(), String> {
        let root = std::env::current_dir()
            .map_err(|error| self.retain_error(error))?
            .join("runtime/quality-review/ui-chooser-same-runtime")
            .join(format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|error| self.retain_error(error))?
                    .as_nanos()
            ));
        self.root = Some(root.clone());
        std::fs::create_dir_all(&root).map_err(|error| self.retain_error(error))?;
        let store = crate::infrastructure::proposals::ProposalFiles::open(&root)
            .map_err(|error| self.retain_error(error))?;
        let actual: ApplicationClientHandle =
            Arc::new(PlayerSdk::default().with_proposal_store(store));
        let payload = ProposalPayload::Cards {
            cards: vec![
                Card {
                    front: "first".into(),
                    back: "first back".into(),
                    source_refs: vec!["cue-1".into()],
                },
                Card {
                    front: "explicit second".into(),
                    back: "second back".into(),
                    source_refs: vec!["cue-1".into()],
                },
            ],
        };
        actual
            .create_proposal(
                RequestContext::default(),
                CreateProposal {
                    proposal_id: "runtime-chooser-proposal".into(),
                    source: SourceSnapshot::Cue(CueSourceSnapshot {
                        source_id: "runtime-chooser-source".into(),
                        source_revision: DecimalU64(1),
                        source_sha256: "a".repeat(64),
                        cues: vec![SourceCue {
                            cue_id: "cue-1".into(),
                            text: "원문 日本語".into(),
                            start_us: None,
                            end_us: None,
                        }],
                    }),
                    generated: GeneratedResult {
                        task: ProposalTask::Cards,
                        raw_text: serde_json::to_string(&payload)
                            .map_err(|error| self.retain_error(error))?,
                    },
                },
            )
            .map_err(|error| self.retain_error(error))?;
        if fault_model {
            let page = actual
                .list_proposals(RequestContext::default(), ListProposals::default())
                .map_err(|error| self.retain_error(error))?;
            let page = serde_json::to_value(page).map_err(|error| self.retain_error(error))?;
            self.fault = Some(Rc::new(RefCell::new(RuntimeFault {
                fail_once: true,
                healthy: false,
                cancelled: false,
                starts: 0,
                tickets: Vec::new(),
                page,
            })));
        }
        self.endpoint = Some(
            LocalHost::start(actual, TransportOptions::default())
                .map_err(|error| self.retain_error(error))?,
        );
        let endpoint = EndpointClient::new(
            self.endpoint.as_ref().unwrap().endpoint(),
            TransportOptions::default(),
        )
        .map_err(|error| self.retain_error(error))?;
        let client: ApplicationClientHandle = Arc::new(RuntimeClient {
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
                Arc::new(Notify),
            )
            .map_err(|error| self.retain_error(error))?,
        );
        self.host = Some(ViewerHost::new(
            client,
            self.execution.take().unwrap(),
            self.endpoint.take().unwrap(),
        ));
        let labels = StartLabels {
            connection: "연결".into(),
            time: "00:00".into(),
        };
        let mut props = document::read_props(&labels);
        self.host_mut()
            .configure_default_chooser(&mut props)
            .map_err(|error| self.retain_error(error))?;
        let mut viewing = document::prepare_start_document(
            &RouteCatalog::bundled().map_err(|error| self.retain_error(error))?,
            "view.start",
            labels,
        )
        .map_err(|error| {
            self.carriers.push(Box::new(error));
            "fixed Viewing preparation failed".to_string()
        })?;
        document::enable_read_panel(&mut viewing, props.clone()).map_err(|error| {
            self.carriers.push(Box::new(error));
            "fixed read-panel provider failed".to_string()
        })?;
        let (entry, mut sources) = product_modules_for_panel(BootstrapPanel::Card);
        let card = meari::parse_module_graph(&entry, &sources)
            .map_err(|error| self.retain_error(error))?;
        let (studio, _) = FrameComposition::new(
            crate::domain::product::PRODUCT.display_name.into(),
            String::new(),
        )
        .prepare(card, &mut sources)
        .map_err(|error| self.retain_error(error))?;
        let frame =
            ProductFrameBlueprint::prepare(viewing.graph, viewing.inventory, studio, sources)?;
        let program = frame.mount_viewing(
            props.clone(),
            "chooser-same-runtime".into(),
            1,
            std::num::NonZeroU64::new(1).unwrap(),
        )?;
        let callbacks = self.host().callbacks();
        let fault = self.fault.clone();
        let errors = self.export_errors.clone();
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
            move |scope| {
                let mut owner = callbacks.borrow_mut();
                let install = (|| {
                    owner.export(scope)?;
                    if let Some(fault) = fault {
                        let (plugin, pump) = runtime_fault_adapter(fault);
                        owner.install_fault_adapter(plugin, pump)?;
                    }
                    owner.bindings()
                })();
                install.map_err(|error: PluginError| {
                    let message = error.message.clone();
                    errors.borrow_mut().push(error);
                    message
                })
            },
        )?;
        let document =
            NativeDocumentHandle::new(document, 1).map_err(|error| self.retain_error(error))?;
        self.host_mut()
            .bind_product_document(document.clone(), props, frame);
        let mut state = ShellState::new();
        let ticket = state
            .request_route(
                &RouteCatalog::bundled().map_err(|error| self.retain_error(error))?,
                "view.start",
                NavigationGuards::default(),
            )
            .map_err(|error| self.retain_error(error))?;
        let observer = Rc::new(RefCell::new(StartRouteObservation::new(
            state,
            ticket.clone(),
        )));
        self.host_mut().bind_route_observation(observer.clone());
        self.observer = Some(observer);
        self.initial_ticket = Some(ticket);
        if observed_model {
            self.report_model_viewing()?;
        }
        self.runtime = Some(NativeHostRuntime::new(
            document,
            AuditedHost {
                inner: self.host.take().unwrap(),
                pause_second: false,
                admission_count: 0,
                remaining_turns: Vec::new(),
                accepted: Vec::new(),
                last_accepted: None,
            },
        ));
        Ok(())
    }
    fn report_model_viewing(&mut self) -> Result<(), String> {
        let ticket = self.initial_ticket.as_ref().unwrap();
        let mut observer = self.observer.as_ref().unwrap().borrow_mut();
        let route = observer
            .state
            .route_target(ticket)
            .map_err(|error| format!("{error:?}"))?
            .clone();
        // Original typed route-model setup; this is explicitly not retained/OS frame observation.
        observer
            .state
            .observe_route(ticket, RouteHostObservation::Reported { route })
            .map_err(|error| format!("{error:?}"))
    }
    fn count(&self, command: &str) -> usize {
        self.probe
            .commands
            .lock()
            .unwrap()
            .iter()
            .filter(|name| name.as_str() == command)
            .count()
    }
    fn activate_binding(&self, binding: &str) {
        let handle = self.handle();
        let path = handle
            .try_with(|document| {
                document
                    .module_projection()
                    .unwrap()
                    .callback_bindings()
                    .into_iter()
                    .find(|(_, callback)| callback.reference.binding == binding)
                    .expect("actual source binding")
                    .0
            })
            .unwrap();
        assert!(handle.dispatch_action(Action::Activate { path }).unwrap());
    }
    fn activate_row(&self, key: &str) {
        let handle = self.handle();
        let path = handle
            .try_with(|document| {
                document
                    .session
                    .widgets
                    .values()
                    .filter(|widget| widget.family == Family::RadioGroup)
                    .flat_map(|widget| widget.items.iter())
                    .find(|item| item.value == key)
                    .expect("actual source RadioGroup item")
                    .path
                    .clone()
            })
            .unwrap();
        assert!(handle.dispatch_action(Action::Activate { path }).unwrap());
    }
    fn tick(&mut self, reason: HostTurnReason) -> HostTurnReceipt {
        self.runtime
            .as_mut()
            .unwrap()
            .after_dispatch(reason)
            .unwrap()
    }
    fn settle_chooser(&mut self, phase: ChooserPhase) {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            self.tick(HostTurnReason::Completion);
            let callbacks = self.host().callbacks();
            let owner = callbacks.borrow();
            if owner.chooser_phase() == Some(phase) && owner.tickets().unwrap().is_empty() {
                assert!(owner.chooser_failure().is_none());
                break;
            }
            assert!(
                Instant::now() < deadline,
                "original UI chooser/ACK did not settle"
            );
            drop(owner);
            std::thread::yield_now();
        }
    }
    fn cleanup(&mut self) -> Result<(), String> {
        let end = Instant::now() + Duration::from_secs(5);
        if self.runtime.is_some() || self.host.is_some() {
            loop {
                let progress = self
                    .host_mut()
                    .poll_unrun_shutdown()
                    .map_err(|error| self.retain_error(error))?;
                let complete = matches!(&progress, ViewerCloseProgress::Complete { execution, .. } if execution.worker_stopped && execution.unacknowledged == 0);
                self.close_progress.push(progress);
                if complete {
                    break;
                }
                if Instant::now() >= end {
                    return Err("original Viewer endpoint/SDK cleanup pending".into());
                }
                std::thread::yield_now();
            }
        }
        if self.pending_endpoint.is_none() {
            self.pending_endpoint = self.endpoint.take().map(LocalHost::begin_shutdown);
        }
        if let Some(endpoint) = &mut self.pending_endpoint {
            while !endpoint.poll().is_quiescent() {
                if Instant::now() >= end {
                    return Err("original setup endpoint pending".into());
                }
                std::thread::yield_now();
            }
        }
        if let Some(owner) = &mut self.execution {
            loop {
                match owner.poll_close() {
                    Ok(ExecutionCloseProgress::Closed(receipt))
                        if receipt.worker_stopped && receipt.unacknowledged == 0 =>
                    {
                        break;
                    }
                    Ok(_) if Instant::now() < end => std::thread::yield_now(),
                    Ok(_) => return Err("original setup execution pending".into()),
                    Err(error) => {
                        let message = format!("{error:?}");
                        self.carriers.push(Box::new(error));
                        return Err(message);
                    }
                }
            }
        }
        Ok(())
    }
}
struct RuntimeFailure {
    setup: Option<String>,
    primary: Option<Box<dyn Any + Send>>,
    cleanup: Option<String>,
    cleanup_panic: Option<Box<dyn Any + Send>>,
    rig: RuntimeRig,
}
impl std::fmt::Debug for RuntimeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChooserSameRuntimeFailure")
            .field("setup", &self.setup)
            .field("primary_panic", &self.primary.is_some())
            .field("cleanup", &self.cleanup)
            .field("cleanup_panic", &self.cleanup_panic.is_some())
            .field("root", &self.rig.root)
            .field("export_errors", &self.rig.export_errors)
            .field("runtime_errors", &self.rig.runtime_errors)
            .field("close_progress", &self.rig.close_progress)
            .field("retained_carriers", &self.rig.carriers.len())
            .finish()
    }
}

// Deliberately process-lived: thread-local registry destruction drops references, never owners.
// A test's Termination::report must not implicitly retry/drop an incomplete cleanup family.
thread_local! {
    static INCOMPLETE_RUNTIME_FAILURES: RefCell<Vec<&'static RuntimeFailure>> = const { RefCell::new(Vec::new()) };
}
struct RuntimeFailureReport {
    diagnostic: String,
    retained_id: Option<usize>,
}
impl std::fmt::Debug for RuntimeFailureReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChooserRuntimeFailureReport")
            .field("diagnostic", &self.diagnostic)
            .field("retained_incomplete_owner", &self.retained_id)
            .finish()
    }
}
impl RuntimeFailure {
    fn report(self) -> RuntimeFailureReport {
        if self.cleanup.is_some() || self.cleanup_panic.is_some() {
            let retained: &'static RuntimeFailure = Box::leak(Box::new(self));
            INCOMPLETE_RUNTIME_FAILURES.with(|holder| holder.borrow_mut().push(retained));
            RuntimeFailureReport {
                diagnostic: format!("{retained:?}"),
                retained_id: Some(retained as *const RuntimeFailure as usize),
            }
        } else {
            // Original cleanup returned Ok only after endpoint/worker quiescence checks.
            RuntimeFailureReport {
                diagnostic: format!("{self:?}"),
                retained_id: None,
            }
        }
    }
}
fn with_runtime(
    observed: bool,
    fault: bool,
    body: impl FnOnce(&mut RuntimeRig),
) -> Result<(), RuntimeFailureReport> {
    let mut rig = RuntimeRig::default();
    let (setup, mut primary) = match catch_unwind(AssertUnwindSafe(|| rig.setup(observed, fault))) {
        Ok(result) => (result.err(), None),
        Err(panic) => (
            Some("setup panic; original payload retained".into()),
            Some(panic),
        ),
    };
    if setup.is_none() {
        primary = catch_unwind(AssertUnwindSafe(|| body(&mut rig))).err();
    }
    let (cleanup, cleanup_panic) = match catch_unwind(AssertUnwindSafe(|| rig.cleanup())) {
        Ok(result) => (result.err(), None),
        Err(panic) => (None, Some(panic)),
    };
    if setup.is_none() && primary.is_none() && cleanup.is_none() && cleanup_panic.is_none() {
        Ok(())
    } else {
        Err(RuntimeFailure {
            setup,
            primary,
            cleanup,
            cleanup_panic,
            rig,
        }
        .report())
    }
}

#[test]
#[ignore = "Assigned same NativeRuntime/fixed source/SDK headless integration; route observation is explicit model, no OS frame"]
fn same_runtime_source_chooser_callbacks_wait_for_zero_batch_before_original_open()
-> Result<(), RuntimeFailureReport> {
    with_runtime(true, false, |rig| {
        let identity = rig.host().identity.clone();
        let host_id = rig.handle().id();
        rig.runtime.as_mut().unwrap().host_mut().pause_second = true;
        rig.activate_binding("onChooserList");
        rig.activate_binding("onChooserList");
        let first = rig.tick(HostTurnReason::Input);
        assert_eq!(
            (first.accepted_callbacks, first.remaining_callbacks),
            (1, 1)
        );
        assert!(first.committed.is_none());
        assert_eq!(rig.count("proposals.list"), 0);
        assert!(
            rig.host()
                .callbacks()
                .borrow()
                .tickets()
                .unwrap()
                .is_empty()
        );
        let resumed = rig.tick(HostTurnReason::ResumePending);
        assert_eq!(resumed.remaining_callbacks, 0);
        rig.settle_chooser(ChooserPhase::PageReady);
        assert_eq!(rig.count("proposals.list"), 1);
        assert!(rig.tick(HostTurnReason::Completion).committed.is_none());
        let document = rig.host().callbacks().borrow().chooser_documents()[0]
            .key
            .clone();
        rig.activate_row(&document);
        rig.tick(HostTurnReason::Input);
        rig.settle_chooser(ChooserPhase::DocumentReady);
        let item = rig.host().callbacks().borrow().chooser_items()[1]
            .key
            .clone();
        rig.activate_row(&item);
        rig.tick(HostTurnReason::Input);
        assert_eq!(
            rig.host().callbacks().borrow().chooser_phase(),
            Some(ChooserPhase::Prepared)
        );
        let original = rig
            .host()
            .callbacks()
            .borrow()
            .chooser_prepared()
            .unwrap()
            .selection
            .clone();
        assert_eq!(original.target, NativeProposalTarget::Card { index: 1 });
        rig.activate_binding("onOpenLiveProposal");
        rig.tick(HostTurnReason::Input);
        let deadline = Instant::now() + Duration::from_secs(3);
        while rig.host().live_entry_phase() != Some(LiveEntryPhase::Inspected) {
            rig.tick(HostTurnReason::Completion);
            assert!(
                Instant::now() < deadline,
                "original target-bound inspection did not settle"
            );
            std::thread::yield_now();
        }
        assert_eq!(rig.count("proposals.inspect"), 2); // one preparation + the original R1 inspection
        let cards = rig.host().cards.borrow();
        let opening = cards.live_opening.as_ref().unwrap();
        assert!(Arc::ptr_eq(&original, &opening.request.selected));
        assert!(Arc::ptr_eq(
            &original,
            &opening.prepared.as_ref().unwrap().selection
        ));
        assert!(Rc::ptr_eq(&identity, &rig.host().identity));
        assert_eq!(rig.handle().id(), host_id);
        assert!(
            rig.runtime
                .as_ref()
                .unwrap()
                .host()
                .remaining_turns
                .contains(&1)
        );
    })
}

#[test]
#[ignore = "Assigned actual ViewerHost source admission on one NativeRuntime; original Reported route model only, no frame proof"]
fn actual_host_rejects_unobserved_or_old_export_without_queuing_chooser()
-> Result<(), RuntimeFailureReport> {
    with_runtime(false, false, |rig| {
        let snapshot = rig.host().viewing_chooser.committed.clone();
        rig.activate_binding("onChooserList");
        let rejected = rig.tick(HostTurnReason::Input);
        assert_eq!(
            (rejected.accepted_callbacks, rejected.rejected_callbacks),
            (0, 1)
        );
        assert!(rejected.committed.is_none());
        assert_eq!(rig.count("proposals.list"), 0);
        assert_eq!(rig.host().viewing_chooser.committed, snapshot);
        rig.report_model_viewing().unwrap();
        rig.activate_binding("onChooserList");
        rig.tick(HostTurnReason::Input);
        rig.settle_chooser(ChooserPhase::PageReady);
        let handle = rig.handle();
        let current = handle.stamp().unwrap();
        let callbacks = rig.host().callbacks();
        let original = rig
            .runtime
            .as_ref()
            .unwrap()
            .host()
            .last_accepted
            .as_ref()
            .unwrap();
        let pending = PendingCallback {
            handle: original.handle.clone(),
            invocation: original.invocation.clone(),
        };
        let old_path = handle
            .try_with(|document| {
                document
                    .module_projection()
                    .unwrap()
                    .callback_bindings()
                    .into_iter()
                    .find(|(_, callback)| callback.reference.binding == "onChooserList")
                    .unwrap()
                    .0
            })
            .unwrap();
        let old_source = handle
            .try_with(|document| serde_json::to_string(&document.source).unwrap())
            .unwrap();
        let old_snapshot = rig.host().viewing_chooser.committed.clone();
        let old_admissions = rig.runtime.as_ref().unwrap().host().admission_count;
        let mut foreign = current.clone();
        foreign.owner.source_generation += 1;
        assert!(matches!(
            rig.host_mut().admit_callback(&foreign, &pending),
            IntentAdmission::Rejected(ViewerHostError::ForeignCheckpoint)
        ));
        {
            let mut owner = callbacks.borrow_mut();
            assert_eq!(owner.prepare_route_release(&current.owner).unwrap(), None);
            owner.resume_released_route(&current.owner).unwrap();
        }
        // Revoked source exports reject before enqueue. Do not manufacture a callback past it.
        let error = handle
            .dispatch_action(Action::Activate { path: old_path })
            .unwrap_err();
        assert!(
            matches!(&error, HostAccessError::Invalid(message) if message == "document callback export is closed")
        );
        rig.runtime_errors.push(error);
        assert_eq!(
            rig.runtime.as_ref().unwrap().host().admission_count,
            old_admissions
        );
        assert_eq!(rig.runtime.as_ref().unwrap().pending_callbacks(), 0);
        assert_eq!(handle.stamp().unwrap(), current);
        assert_eq!(
            handle
                .try_with(|document| serde_json::to_string(&document.source).unwrap())
                .unwrap(),
            old_source
        );
        assert_eq!(rig.host().viewing_chooser.committed, old_snapshot);
        assert_eq!(rig.count("proposals.list"), 1);
        assert_eq!(rig.count("proposals.inspect"), 0);
        assert!(callbacks.borrow().tickets().unwrap().is_empty());
        assert!(rig.host().live_entry_phase().is_none());
        let admission = rig
            .host_mut()
            .admit_callback(&handle.stamp().unwrap(), &pending);
        assert!(
            matches!(&admission, IntentAdmission::Rejected(ViewerHostError::Callback(error)) if error.code == PluginErrorCode::Stale)
        );
        assert_eq!(rig.count("proposals.list"), 1);
        assert!(callbacks.borrow().tickets().unwrap().is_empty());
        assert_eq!(rig.host().viewing_chooser.committed, old_snapshot);
    })
}

#[test]
#[ignore = "Assigned real document/Runtime error boundaries with explicit adapter and callback faults; no swallowed failure or OS claim"]
fn actual_projection_error_keeps_original_document_export_and_committed_snapshot()
-> Result<(), RuntimeFailureReport> {
    with_runtime(true, true, |rig| {
        let handle = rig.handle();
        let before = rig.host().viewing_chooser.committed.clone();
        rig.activate_binding("onChooserList");
        let admitted = handle.stamp().unwrap();
        let source = handle
            .try_with(|document| serde_json::to_string(&document.source).unwrap())
            .unwrap();
        let error = rig
            .runtime
            .as_mut()
            .unwrap()
            .after_dispatch(HostTurnReason::Input)
            .unwrap_err();
        rig.runtime_errors.push(error);
        let first = match rig.host().last_error.as_ref().unwrap() {
            ViewerHostError::Callback(error) => error.clone(),
            error => panic!("original projection error: {error:?}"),
        };
        assert_eq!(first.code, PluginErrorCode::Adapter);
        assert_eq!(first.message, "prop exceeds total byte budget");
        assert_eq!(
            handle.stamp().unwrap().document_revision,
            admitted.document_revision
        );
        assert_eq!(rig.host().viewing_chooser.committed, before);
        assert_eq!(
            handle
                .try_with(|document| serde_json::to_string(&document.source).unwrap())
                .unwrap(),
            source
        );
        assert_eq!(rig.fault.as_ref().unwrap().borrow().starts, 1);
        // Finish that exact original adapter ticket; the second real click coalesces, not resubmits.
        rig.fault.as_ref().unwrap().borrow_mut().healthy = true;
        rig.host_mut().base_props.insert(
            "onReadCapabilities".into(),
            Value::CallbackReference(CallbackReference {
                binding: "missing-checked-document-binding".into(),
                signature: CallbackSignature::Action,
            }),
        );
        rig.activate_binding("onChooserList");
        let admitted = handle.stamp().unwrap();
        let source = handle
            .try_with(|document| serde_json::to_string(&document.source).unwrap())
            .unwrap();
        let error = rig
            .runtime
            .as_mut()
            .unwrap()
            .after_dispatch(HostTurnReason::Input)
            .unwrap_err();
        rig.runtime_errors.push(error);
        assert_eq!(
            handle.stamp().unwrap().document_revision,
            admitted.document_revision
        );
        assert_eq!(rig.host().viewing_chooser.committed, before);
        assert_eq!(
            handle
                .try_with(|document| serde_json::to_string(&document.source).unwrap())
                .unwrap(),
            source
        );
        assert_eq!(rig.fault.as_ref().unwrap().borrow().starts, 1);
        let retained = match rig.host().last_error.as_ref().unwrap() {
            ViewerHostError::Callback(error) => error,
            error => panic!("original first error lost: {error:?}"),
        };
        assert!(Arc::ptr_eq(&first, retained));
        // Authenticate a real old read-local source callback after the checked commit rejection.
        rig.activate_binding("onRefreshReadProjection");
        let admitted = handle.stamp().unwrap();
        let source = handle
            .try_with(|document| serde_json::to_string(&document.source).unwrap())
            .unwrap();
        let error = rig
            .runtime
            .as_mut()
            .unwrap()
            .after_dispatch(HostTurnReason::Input)
            .unwrap_err();
        rig.runtime_errors.push(error);
        assert_eq!(
            rig.runtime.as_ref().unwrap().host().accepted,
            vec![true, true, true]
        );
        assert_eq!(
            handle.stamp().unwrap().document_revision,
            admitted.document_revision
        );
        assert_eq!(rig.host().viewing_chooser.committed, before);
        assert_eq!(
            handle
                .try_with(|document| serde_json::to_string(&document.source).unwrap())
                .unwrap(),
            source
        );
        assert_eq!(rig.fault.as_ref().unwrap().borrow().starts, 1);
        assert!(
            matches!(rig.host().last_error.as_ref(), Some(ViewerHostError::Callback(retained)) if Arc::ptr_eq(&first, retained))
        );
        assert!(
            rig.host()
                .callbacks()
                .borrow()
                .projection
                .borrow()
                .panel
                .projection_error
                .is_some()
        );
        assert!(
            rig.host()
                .callbacks()
                .borrow()
                .tickets()
                .unwrap()
                .is_empty()
        );
        assert!(
            rig.probe.commands.lock().unwrap().is_empty(),
            "injected adapter is not claimed as real SDK execution"
        );
    })
}
