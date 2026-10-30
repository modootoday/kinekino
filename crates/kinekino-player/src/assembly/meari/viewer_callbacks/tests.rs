use super::*;
#[path = "tests/card_selection.rs"]
mod card_selection;
#[path = "tests/native_playback.rs"]
mod native_playback;
#[path = "tests/read_resume.rs"]
mod read_resume;
#[path = "tests/window_ui.rs"]
mod window_ui;
use crate::application::{
    api::{ApplicationClientHandle, PlayerSdk, RequestContext},
    sdk_execution::{CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner},
};
use meari::{
    application::{host_callbacks::PendingCallback, plugins::*, source_modules::ModuleProgram},
    domain::{
        Span, Value,
        build::BuildPath,
        host_callbacks::{
            CallbackArguments, CallbackInvocation, CallbackReference, CallbackSignature,
        },
    },
};
use meari_native::{domain::Viewport, infrastructure::document::InteractiveDocument};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};
struct Notify;

#[test]
fn automatic_bootstrap_revalidates_operation_and_retains_original_ticket_until_ack() {
    let fixture = route_fixture();
    assert!(
        fixture
            .owner
            .borrow()
            .next_automatic_step(false)
            .unwrap()
            .is_none()
    );
    let plan = fixture
        .owner
        .borrow()
        .next_automatic_step(true)
        .unwrap()
        .unwrap();
    assert_eq!(plan.action, super::progression::Action::ReadCapabilities);
    let mut foreign = plan.clone();
    foreign.operation = "foreign-operation".into();
    assert_eq!(
        fixture
            .owner
            .borrow()
            .dispatch_automatic_step(&fixture.scope, &foreign, true)
            .err()
            .unwrap()
            .code,
        PluginErrorCode::Stale
    );
    assert!(fixture.owner.borrow().tickets().unwrap().is_empty());
    let receipt = fixture
        .owner
        .borrow()
        .dispatch_automatic_step(&fixture.scope, &plan, true)
        .unwrap();
    assert!(
        fixture
            .owner
            .borrow()
            .next_automatic_step(true)
            .unwrap()
            .is_none()
    );
    let deadline = Instant::now() + Duration::from_secs(2);
    while fixture
        .owner
        .borrow()
        .tickets()
        .unwrap()
        .contains(&receipt.ticket)
    {
        fixture.owner.borrow_mut().consume_reads().unwrap();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        fixture.owner.borrow().projection.borrow().panel.rows[0].phase,
        crate::presentation::web_ui_adapter::player_shell::read_observation::ReadPhase::Received
    );
    assert!(
        fixture
            .owner
            .borrow()
            .next_automatic_step(true)
            .unwrap()
            .is_none()
    );
    assert!(
        fixture
            .owner
            .borrow_mut()
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    close(&fixture.execution);
}

struct RouteFixture {
    execution: ExecutionOwner,
    owner: Rc<RefCell<ViewerCallbacks>>,
    _document: InteractiveDocument,
    scope: PluginScope,
}
fn route_fixture() -> RouteFixture {
    let execution = execution();
    let owner = Rc::new(RefCell::new(ViewerCallbacks::new(
        execution.handle(),
        RequestContext::default(),
    )));
    let exporting = owner.clone();
    let document = InteractiveDocument::from_module_program_with_callbacks(
        program("onReadCapabilities"),
        "",
        Viewport {
            width: 160,
            height: 80,
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
    let scope = document.callback_scope().unwrap().clone();
    RouteFixture {
        execution,
        owner,
        _document: document,
        scope,
    }
}

#[test]
fn route_release_rejects_foreign_scope_and_keeps_original_worker_open() {
    let fixture = route_fixture();
    let mut foreign = fixture.scope.clone();
    foreign.source_generation += 1;
    assert_eq!(
        fixture
            .owner
            .borrow_mut()
            .prepare_route_release(&foreign)
            .unwrap_err()
            .code,
        PluginErrorCode::Stale
    );
    assert!(!fixture.owner.borrow().is_released());
    assert!(fixture.execution.handle().attach().is_err());
    assert!(
        fixture
            .owner
            .borrow_mut()
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    assert!(fixture.owner.borrow().is_released());
    let next = fixture.execution.handle().attach().unwrap();
    assert!(
        fixture
            .owner
            .borrow_mut()
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    next.release().unwrap();
    close(&fixture.execution);
}

#[test]
fn route_release_keeps_original_ticket_and_attachment_until_terminal_ack() {
    let fixture = route_fixture();
    let (_bindings, pending) = local_input(&fixture.owner.borrow(), "onReadCapabilities");
    let receipt = fixture
        .owner
        .borrow()
        .dispatch(&fixture.scope, &pending)
        .unwrap();
    let tickets = fixture
        .owner
        .borrow_mut()
        .prepare_route_release(&fixture.scope)
        .unwrap()
        .unwrap();
    assert_eq!(tickets.as_slice(), std::slice::from_ref(&receipt.ticket));
    assert!(!fixture.owner.borrow().is_released());
    assert!(
        fixture
            .owner
            .borrow()
            .dispatch(&fixture.scope, &pending)
            .is_err()
    );
    assert!(fixture.execution.handle().attach().is_err());
    let deadline = Instant::now() + Duration::from_secs(2);
    let terminal = loop {
        if let Some(terminal) = fixture
            .owner
            .borrow_mut()
            .accept_terminal(&receipt.ticket)
            .unwrap()
        {
            break terminal;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    };
    assert!(terminal.owner_ack.is_ok());
    terminal.owner_ack.unwrap();
    assert!(
        fixture
            .owner
            .borrow_mut()
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    let next = fixture.execution.handle().attach().unwrap();
    next.release().unwrap();
    close(&fixture.execution);
}

fn local_input(
    owner: &ViewerCallbacks,
    prop: &str,
) -> (
    meari::application::host_callbacks::CallbackBindings,
    PendingCallback,
) {
    let bindings = owner.bindings().unwrap();
    let input = PendingCallback {
        handle: bindings
            .resolve(&CallbackReference {
                binding: prop.into(),
                signature: CallbackSignature::Action,
            })
            .unwrap(),
        invocation: CallbackInvocation {
            prop: prop.into(),
            arguments: CallbackArguments::Action,
            span: Span::default(),
        },
    };
    (bindings, input)
}
#[test]
fn double_click_local_retry_consumes_one_exhaustion_generation_without_resubmit() {
    let AckFixture {
        execution,
        owner,
        document: _document,
        scope,
        attempts,
        fail,
        ready,
        starts,
    } = ack_fixture();
    let (_read_export, read) = local_input(&owner.borrow(), "onReadCapabilities");
    assert!(matches!(
        owner.borrow_mut().queue_local(&scope, &read),
        Err(meari::application::host_callbacks::LocalDispatchError::NotLocal)
    ));
    assert_eq!(starts.get(), 0);
    owner.borrow().dispatch(&scope, &read).unwrap();
    ready.set(true);
    let now = Instant::now();
    for ms in [0, 25, 50] {
        owner
            .borrow_mut()
            .consume_reads_at(now + Duration::from_millis(ms))
            .unwrap();
    }
    let (_export, retry) = local_input(&owner.borrow(), "onRetryCapabilitiesAck");
    owner.borrow_mut().queue_local(&scope, &retry).unwrap();
    owner.borrow_mut().queue_local(&scope, &retry).unwrap();
    assert_eq!(attempts.borrow().get("capabilities"), Some(&3));
    let drained = owner.borrow_mut().drain_local(&scope).unwrap();
    assert!(drained.resumed);
    assert!(!drained.refreshed);
    assert_eq!(
        owner.borrow().local_error().unwrap().code,
        PluginErrorCode::Stale
    );
    assert!(owner.borrow().local_queue_empty());
    for ms in [75, 100, 125, 1000] {
        owner
            .borrow_mut()
            .consume_reads_at(now + Duration::from_millis(ms))
            .unwrap();
    }
    assert_eq!(attempts.borrow().get("capabilities"), Some(&6));
    assert_eq!(starts.get(), 1);
    assert!(!owner.borrow_mut().drain_local(&scope).unwrap().resumed); // no replay of the second click
    assert_eq!(attempts.borrow().get("capabilities"), Some(&6));
    fail.set(false);
    owner.borrow_mut().queue_local(&scope, &retry).unwrap(); // new admission after new exhaustion
    owner.borrow_mut().drain_local(&scope).unwrap();
    owner
        .borrow_mut()
        .consume_reads_at(now + Duration::from_secs(2))
        .unwrap();
    assert_eq!(attempts.borrow().get("capabilities"), Some(&7));
    assert_eq!(starts.get(), 1);
    assert!(owner.borrow().tickets().unwrap().is_empty());
    assert!(owner.borrow_mut().prepare_release().unwrap().is_none());
    close(&execution);
}
#[test]
fn local_refresh_is_consumed_once_and_stale_or_closing_inputs_do_no_sdk_work() {
    let AckFixture {
        execution,
        owner,
        document: _document,
        scope,
        attempts,
        starts,
        ..
    } = ack_fixture();
    let (export, input) = local_input(&owner.borrow(), "onRefreshReadProjection");
    owner.borrow_mut().queue_local(&scope, &input).unwrap();
    assert!(owner.borrow_mut().drain_local(&scope).unwrap().refreshed);
    owner
        .borrow()
        .projection
        .borrow_mut()
        .projection_failed("retained preflight error".into());
    assert!(!owner.borrow_mut().drain_local(&scope).unwrap().refreshed);
    assert!(owner.borrow().projection.borrow().dirty);
    export.invalidate();
    assert!(matches!(
        owner.borrow_mut().queue_local(&scope, &input),
        Err(meari::application::host_callbacks::LocalDispatchError::Invalid(_))
    ));
    let (_fresh_export, fresh) = local_input(&owner.borrow(), "onRefreshReadProjection");
    owner.borrow_mut().queue_local(&scope, &fresh).unwrap();
    owner.borrow_mut().stop_admission();
    assert!(owner.borrow().local_queue_empty());
    assert!(owner.borrow_mut().queue_local(&scope, &fresh).is_err());
    assert_eq!(starts.get(), 0);
    assert!(attempts.borrow().is_empty());
    assert!(owner.borrow_mut().prepare_release().unwrap().is_none());
    close(&execution);
}

struct AckFixture {
    starts: Rc<std::cell::Cell<usize>>,
    execution: ExecutionOwner,
    owner: Rc<RefCell<ViewerCallbacks>>,
    document: InteractiveDocument,
    scope: PluginScope,
    attempts: Rc<RefCell<BTreeMap<String, usize>>>,
    fail: Rc<std::cell::Cell<bool>>,
    ready: Rc<std::cell::Cell<bool>>,
}
fn ack_fixture() -> AckFixture {
    use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::{
        AsyncCodecBudget, AsyncSdkPort, TypedAsyncSdkPlugin, async_binding,
    };
    use std::cell::Cell;
    struct FaultPort {
        starts: Rc<Cell<usize>>,
        attempts: Rc<RefCell<BTreeMap<String, usize>>>,
        fail: Rc<Cell<bool>>,
        ready: Rc<Cell<bool>>,
    }
    impl AsyncSdkPort<codec::ReadInput, serde_json::Value> for FaultPort {
        fn start(&mut self, _: &PluginTicket, _: codec::ReadInput) -> Result<(), PluginError> {
            self.starts.set(self.starts.get() + 1);
            Ok(())
        }
        fn peek_terminal(
            &mut self,
            _: &PluginTicket,
        ) -> Result<
            Option<Arc<Result<serde_json::Value, crate::application::api::ApiError>>>,
            PluginError,
        > {
            Ok(self
                .ready
                .get()
                .then(|| Arc::new(Ok(serde_json::json!({"commands":[]})))))
        }
        fn acknowledge(&mut self, ticket: &PluginTicket) -> Result<(), PluginError> {
            *self
                .attempts
                .borrow_mut()
                .entry(ticket.capability().into())
                .or_default() += 1;
            if self.fail.get() && ticket.capability() == "capabilities" {
                Err(PluginError::new(PluginErrorCode::Busy, "ACK fault"))
            } else {
                Ok(())
            }
        }
        fn cancel(&mut self, _: &PluginTicket) -> Result<CancelReceipt, PluginError> {
            Ok(CancelReceipt::Requested)
        }
        fn shutdown(&mut self) -> Result<(), PluginError> {
            Ok(())
        }
    }
    let execution = execution();
    let owner = Rc::new(RefCell::new(ViewerCallbacks::new(
        execution.handle(),
        RequestContext::default(),
    )));
    let exporting = owner.clone();
    let document = InteractiveDocument::from_module_program_with_callbacks(
        program("onReadCapabilities"),
        "",
        Viewport {
            width: 160,
            height: 80,
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
    let scope = document.callback_scope().unwrap().clone();
    let attempts = Rc::new(RefCell::new(BTreeMap::new()));
    let fail = Rc::new(Cell::new(true));
    let ready = Rc::new(Cell::new(false));
    let starts = Rc::new(Cell::new(0));
    let manifest = codec::manifest();
    let bindings = codec::READS
        .into_iter()
        .map(|(_, command)| {
            (
                command.into(),
                async_binding(
                    manifest.capabilities[command].clone(),
                    FaultPort {
                        starts: starts.clone(),
                        attempts: attempts.clone(),
                        fail: fail.clone(),
                        ready: ready.clone(),
                    },
                ),
            )
        })
        .collect();
    let (plugin, pump) = TypedAsyncSdkPlugin::new(
        manifest,
        bindings,
        AsyncCodecBudget {
            max_pending: 8,
            max_terminal_wire_bytes: 262144,
            max_total_codec_wire_bytes: 8 * 262144 * 4,
        },
    )
    .unwrap();
    owner
        .borrow_mut()
        .install_fault_adapter(plugin, pump)
        .unwrap();
    AckFixture {
        execution,
        owner,
        document,
        scope,
        attempts,
        fail,
        ready,
        starts,
    }
}

#[test]
fn ack_fault_budget_is_per_ticket_and_shared_across_consumer_turns() {
    let AckFixture {
        execution,
        owner,
        document: _document,
        scope,
        attempts,
        fail,
        ready,
        starts: _starts,
    } = ack_fixture();
    let bindings = owner.borrow().bindings().unwrap();
    for prop in ["onReadCapabilities", "onReadStatus"] {
        let pending = PendingCallback {
            handle: bindings
                .resolve(&CallbackReference {
                    binding: prop.into(),
                    signature: CallbackSignature::Action,
                })
                .unwrap(),
            invocation: CallbackInvocation {
                prop: prop.into(),
                arguments: CallbackArguments::Action,
                span: Span::default(),
            },
        };
        owner.borrow().dispatch(&scope, &pending).unwrap();
    }
    let now = Instant::now();
    owner.borrow_mut().consume_reads_at(now).unwrap();
    assert!(attempts.borrow().is_empty()); // terminal-not-ready consumes no ACK budget
    ready.set(true);
    owner.borrow_mut().consume_reads_at(now).unwrap();
    assert_eq!(attempts.borrow().get("capabilities"), Some(&1));
    assert_eq!(attempts.borrow().get("playback.status"), Some(&1));
    owner.borrow_mut().stop_admission();
    for ms in [5, 10, 24] {
        owner
            .borrow_mut()
            .consume_reads_at(now + Duration::from_millis(ms))
            .unwrap();
    }
    assert_eq!(attempts.borrow().get("capabilities"), Some(&1));
    owner
        .borrow_mut()
        .consume_reads_at(now + Duration::from_millis(25))
        .unwrap();
    owner
        .borrow_mut()
        .consume_reads_at(now + Duration::from_millis(50))
        .unwrap();
    owner
        .borrow_mut()
        .consume_reads_at(now + Duration::from_secs(60))
        .unwrap();
    assert_eq!(attempts.borrow().get("capabilities"), Some(&3));
    assert_eq!(owner.borrow().tickets().unwrap().len(), 1);
    assert!(owner.borrow().next_consumer_retry().is_none());
    fail.set(false);
    let ticket = owner.borrow().tickets().unwrap().remove(0);
    owner.borrow_mut().retry_read_acceptance(&ticket).unwrap();
    owner
        .borrow_mut()
        .consume_reads_at(now + Duration::from_secs(61))
        .unwrap();
    assert_eq!(attempts.borrow().get("capabilities"), Some(&4));
    assert!(owner.borrow().tickets().unwrap().is_empty());
    assert!(owner.borrow_mut().prepare_release().unwrap().is_none());
    close(&execution);
}

#[test]
fn actual_read_graph_reprojects_with_original_scope_and_commit_receipt() {
    use crate::presentation::web_ui_adapter::player_shell::{
        RouteCatalog,
        document::{StartLabels, enable_read_panel, prepare_start_document, read_props},
    };
    use meari_native::infrastructure::native_host::NativeDocumentHandle;
    let execution = execution();
    let owner = Rc::new(RefCell::new(ViewerCallbacks::new(
        execution.handle(),
        RequestContext::default(),
    )));
    let labels = StartLabels {
        connection: "연결 미관측".into(),
        time: "미관측".into(),
    };
    let mut prepared = prepare_start_document(
        &RouteCatalog::bundled().unwrap(),
        "view.start",
        StartLabels {
            connection: labels.connection.clone(),
            time: labels.time.clone(),
        },
    )
    .ok()
    .unwrap();
    let props = read_props(&labels);
    enable_read_panel(&mut prepared, props.clone())
        .ok()
        .unwrap();
    let program =
        ModuleProgram::mount(prepared.graph, prepared.props, "read-projection".into(), 1).unwrap();
    let exporting = owner.clone();
    let captured = Rc::new(RefCell::new(None));
    let save = captured.clone();
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
            let bindings = exporting
                .borrow_mut()
                .export(scope)
                .map_err(|error| error.message)?;
            *save.borrow_mut() = Some(
                bindings
                    .resolve(&CallbackReference {
                        binding: "onReadCapabilities".into(),
                        signature: CallbackSignature::Action,
                    })
                    .unwrap(),
            );
            Ok(bindings)
        },
    )
    .unwrap();
    let scope = document.callback_scope().unwrap().clone();
    let handle = NativeDocumentHandle::new(document, 1).unwrap();
    let before = handle.stamp().unwrap();
    let (_refresh_export, refresh) = local_input(&owner.borrow(), "onRefreshReadProjection");
    owner.borrow_mut().queue_local(&scope, &refresh).unwrap();
    assert!(owner.borrow_mut().drain_local(&scope).unwrap().refreshed);
    let invalid_bindings = owner.borrow().bindings().unwrap();
    assert!(
        handle
            .try_reproject(&before, |document| document
                .update_module_props_with_callbacks(BTreeMap::new(), invalid_bindings))
            .is_err()
    );
    assert_eq!(handle.stamp().unwrap(), before);
    assert!(!owner.borrow_mut().drain_local(&scope).unwrap().refreshed);
    assert!(owner.borrow().local_queue_empty()); // actual preflight failure cannot replay the intent
    let pending = PendingCallback {
        handle: captured.borrow_mut().take().unwrap(),
        invocation: CallbackInvocation {
            prop: "onReadCapabilities".into(),
            arguments: CallbackArguments::Action,
            span: Span::default(),
        },
    };
    let operation = owner.borrow().dispatch(&scope, &pending).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !owner.borrow().tickets().unwrap().is_empty() {
        owner.borrow_mut().consume_reads().unwrap();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    let event = owner
        .borrow()
        .projection
        .borrow()
        .retained_event(&operation.ticket)
        .unwrap()
        .clone();
    // Display-consumer ACK fault seam; the actual SDK operation above was ACKed successfully.
    {
        use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::TerminalAcceptance;
        let owner = owner.borrow();
        let mut projection = owner.projection.borrow_mut();
        projection.retain(TerminalAcceptance {
            event: event.clone(),
            owner_ack: Err(PluginError::new(
                PluginErrorCode::Busy,
                "internal detail must remain owned",
            )),
        });
        projection.retry_decode();
        assert_eq!(projection.panel.rows[0].phase, crate::presentation::web_ui_adapter::player_shell::read_observation::ReadPhase::AckFailed);
        assert_eq!(
            projection.panel.rows[0].error_key,
            Some("viewer.read.ack_failed")
        );
        assert!(projection.retained_event(&operation.ticket).is_some());
        projection.retain(TerminalAcceptance {
            event,
            owner_ack: Ok(()),
        });
    }
    let bindings = owner.borrow().bindings().unwrap();
    let (_, receipt) = handle
        .try_reproject(&before, |document| {
            document.update_module_props_with_callbacks(props, bindings)
        })
        .unwrap();
    assert_eq!(receipt.before(), &before);
    assert_eq!(receipt.after().owner, scope);
    assert!(receipt.after().document_revision > before.document_revision);
    assert!(owner.borrow().tickets().unwrap().is_empty());
    assert!(owner.borrow_mut().prepare_release().unwrap().is_none());
    close(&execution);
}

#[test]
fn read_consumer_retains_real_terminal_without_human_acceptance() {
    let execution = execution();
    let owner = Rc::new(RefCell::new(ViewerCallbacks::new(
        execution.handle(),
        RequestContext::default(),
    )));
    let captured = Rc::new(RefCell::new(None));
    let save = captured.clone();
    let export_owner = owner.clone();
    let document = InteractiveDocument::from_module_program_with_callbacks(
        program("onReadCapabilities"),
        "",
        Viewport {
            width: 160,
            height: 80,
            dark: true,
        },
        None,
        1,
        move |scope| {
            let bindings = export_owner
                .borrow_mut()
                .export(scope)
                .map_err(|error| error.message)?;
            *save.borrow_mut() = Some(
                bindings
                    .resolve(&CallbackReference {
                        binding: "onReadCapabilities".into(),
                        signature: CallbackSignature::Action,
                    })
                    .unwrap(),
            );
            Ok(bindings)
        },
    )
    .unwrap();
    let scope = document.callback_scope().unwrap().clone();
    let pending = PendingCallback {
        handle: captured.borrow_mut().take().unwrap(),
        invocation: CallbackInvocation {
            prop: "onReadCapabilities".into(),
            arguments: CallbackArguments::Action,
            span: Span::default(),
        },
    };
    owner.borrow().dispatch(&scope, &pending).unwrap();
    assert!(owner.borrow().dispatch(&scope, &pending).is_err());
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        owner.borrow_mut().consume_reads().unwrap();
        if owner.borrow().tickets().unwrap().is_empty() {
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    {
        let owner = owner.borrow();
        let projection = owner.projection.borrow();
        assert_eq!(projection.panel.rows[0].phase, crate::presentation::web_ui_adapter::player_shell::read_observation::ReadPhase::Received);
        assert!(
            projection.panel.rows[0]
                .value
                .as_ref()
                .unwrap()
                .contains("commands")
        );
    }
    owner.borrow_mut().consume_reads().unwrap();
    owner.borrow().projection.borrow_mut().retry_decode();
    assert!(owner.borrow().tickets().unwrap().is_empty());
    assert!(owner.borrow_mut().prepare_release().unwrap().is_none());
    close(&execution);
}
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), crate::application::api::ApiError> {
        Ok(())
    }
}
fn execution() -> ExecutionOwner {
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    ExecutionOwner::start(
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
    .unwrap()
}
fn program(binding: &str) -> ModuleProgram {
    let entry: BuildPath = "viewer-callback-test.tsx".to_owned().try_into().unwrap();
    let graph = meari::parse_module_graph(&entry, &BTreeMap::from([(entry.clone(),
        "type Props={action:()=>void};export function Probe(props:Props){return <button onClick={() => props.action()}>Read</button>;}".into())])).unwrap();
    ModuleProgram::mount(
        graph,
        BTreeMap::from([(
            "action".into(),
            Value::CallbackReference(CallbackReference {
                binding: binding.into(),
                signature: CallbackSignature::Action,
            }),
        )]),
        "viewer-callback-test".into(),
        1,
    )
    .unwrap()
}
fn close(owner: &ExecutionOwner) {
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
fn manifest_separates_native_instance_controls_from_empty_read_inputs() {
    let manifest = codec::manifest();
    manifest.validate().unwrap();
    assert_eq!(
        manifest
            .capabilities
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "capabilities",
            "playback.engine",
            "playback.pause",
            "playback.play",
            "playback.seek",
            "playback.set_volume",
            "playback.status"
        ]
    );
    assert_eq!(codec::manifest().provider, manifest.provider);
    assert!(
        serde_json::from_value::<codec::ReadInput>(serde_json::json!({"command":"playback.play"}))
            .is_err()
    );
    let reply: codec::CapabilitiesReply = serde_json::from_value(serde_json::json!({"commands":[{"command":"future.command","status":"unknown"}],"future_section":{"raw":true}})).unwrap();
    assert_eq!(reply.commands[0].status, "unknown");
    assert!(reply.sections.contains_key("future_section"));
}

#[test]
fn actual_mounted_scope_keeps_terminal_until_explicit_acceptance_and_release() {
    let execution = execution();
    let owner = Rc::new(RefCell::new(ViewerCallbacks::new(
        execution.handle(),
        RequestContext::default(),
    )));
    let captured = Rc::new(RefCell::new(None));
    let save = captured.clone();
    let export_owner = owner.clone();
    let document = InteractiveDocument::from_module_program_with_callbacks(
        program("onReadCapabilities"),
        "",
        Viewport {
            width: 160,
            height: 80,
            dark: true,
        },
        None,
        1,
        move |scope| {
            let bindings = export_owner
                .borrow_mut()
                .export(scope)
                .map_err(|error| error.message)?;
            *save.borrow_mut() = Some(
                bindings
                    .resolve(&CallbackReference {
                        binding: "onReadCapabilities".into(),
                        signature: CallbackSignature::Action,
                    })
                    .unwrap(),
            );
            Ok(bindings)
        },
    )
    .unwrap();
    let scope = document.callback_scope().unwrap().clone();
    assert_eq!(owner.borrow().scope(), Some(&scope));
    let pending = PendingCallback {
        handle: captured.borrow_mut().take().unwrap(),
        invocation: CallbackInvocation {
            prop: "onReadCapabilities".into(),
            arguments: CallbackArguments::Action,
            span: Span::default(),
        },
    };
    let mut foreign = scope.clone();
    foreign.source_generation += 1;
    assert!(owner.borrow().dispatch(&foreign, &pending).is_err());
    assert!(owner.borrow().tickets().unwrap().is_empty());
    let receipt = owner.borrow().dispatch(&scope, &pending).unwrap();
    assert!(matches!(receipt.outcome, PluginDispatch::Pending));
    let retained = owner.borrow_mut().prepare_release().unwrap().unwrap();
    assert_eq!(retained.as_slice(), std::slice::from_ref(&receipt.ticket));
    assert!(!owner.borrow().is_released());
    assert!(owner.borrow().dispatch(&scope, &pending).is_err());
    let deadline = Instant::now() + Duration::from_secs(2);
    let accepted = loop {
        if let Some(accepted) = owner.borrow_mut().accept_terminal(&receipt.ticket).unwrap() {
            break accepted;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    };
    accepted.owner_ack.unwrap();
    assert!(matches!(
        accepted.event.event,
        PluginEventKind::Completed(_)
    ));
    assert!(owner.borrow_mut().prepare_release().unwrap().is_none());
    assert!(owner.borrow().is_released());
    close(&execution);
}

#[test]
fn late_document_binding_failure_retains_real_scope_and_exclusive_attachment() {
    let execution = execution();
    let owner = Rc::new(RefCell::new(ViewerCallbacks::new(
        execution.handle(),
        RequestContext::default(),
    )));
    let export_owner = owner.clone();
    let result = InteractiveDocument::from_module_program_with_callbacks(
        program("notExported"),
        "",
        Viewport {
            width: 160,
            height: 80,
            dark: true,
        },
        None,
        1,
        move |scope| {
            let bindings = export_owner
                .borrow_mut()
                .export(scope.clone())
                .map_err(|error| error.message)?;
            // Fault injection: retain one real operation when a later source binding fails.
            let handle = bindings
                .resolve(&CallbackReference {
                    binding: "onReadCapabilities".into(),
                    signature: CallbackSignature::Action,
                })
                .unwrap();
            export_owner
                .borrow()
                .dispatch(
                    &scope,
                    &PendingCallback {
                        handle,
                        invocation: CallbackInvocation {
                            prop: "onReadCapabilities".into(),
                            arguments: CallbackArguments::Action,
                            span: Span::default(),
                        },
                    },
                )
                .unwrap();
            Ok(bindings)
        },
    );
    assert!(result.is_err());
    assert!(owner.borrow().scope().is_some());
    assert!(!owner.borrow().is_released());
    assert!(execution.handle().attach().is_err());
    let tickets = owner.borrow_mut().prepare_release().unwrap().unwrap();
    assert_eq!(tickets.len(), 1);
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(accepted) = owner.borrow_mut().accept_terminal(&tickets[0]).unwrap() {
            accepted.owner_ack.unwrap();
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(owner.borrow_mut().prepare_release().unwrap().is_none());
    close(&execution);
}

#[test]
#[ignore = "Requires an explicit LocalHost pipe and Native-loop startup slot; no OS view is run"]
fn real_viewer_launch_consumes_the_root_export_and_releases_original_owners() {
    use crate::{
        assembly::meari::{
            native_execution_startup::prepare_native_execution,
            player_shell_document::DocumentOptions,
            player_shell_host::ShellHostConfig,
            viewer_cleanup::{
                ViewerCleanupLimits, ViewerCleanupStatus, ViewerNativeState, ViewerOwners,
            },
            viewer_native_launch::{ViewerLaunchInput, finish_viewer, prepare_viewer_launch},
        },
        infrastructure::local_host::{LocalHost, TransportOptions},
        presentation::web_ui_adapter::player_shell::document::StartLabels,
    };
    use meari_native::{
        application::{WindowOptions, window_lifecycle::LifecycleLimits},
        infrastructure::native_host::HostScheduleLimits,
    };
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let endpoint = LocalHost::start(client.clone(), TransportOptions::default()).unwrap();
    let schedule =
        HostScheduleLimits::new(Duration::from_millis(2), Duration::from_millis(250), 100).unwrap();
    let execution = prepare_native_execution(
        client.clone(),
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 262144,
            max_total_request_bytes: 8 * 262144,
            max_terminal_bytes: 262144,
            max_total_terminal_bytes: 8 * 262144,
        },
        schedule,
    )
    .unwrap();
    let launch = match prepare_viewer_launch(ViewerLaunchInput {
        client: client.clone(),
        endpoint,
        execution,
        labels: StartLabels {
            connection: "관측 없음".into(),
            time: "관측 없음".into(),
        },
        document: DocumentOptions {
            namespace: "viewer-root-export-test".into(),
            epoch: 1,
            source_generation: 1,
            viewport: Viewport {
                width: 320,
                height: 180,
                dark: true,
            },
        },
        host: ShellHostConfig {
            options: WindowOptions::default(),
            children: LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
            schedule,
        },
    }) {
        Ok(launch) => launch,
        Err(_) => panic!("production launch must consume callback mount"),
    };
    let owners = launch.into_unrun();
    let ViewerOwners::Unrun { controller, .. } = &owners else {
        panic!("unrun authority retained")
    };
    let host = controller.pending_owner().unwrap();
    assert!(Arc::ptr_eq(&client, host.client()));
    let callbacks = host.callbacks();
    assert!(callbacks.borrow().scope().is_some());
    assert!(!callbacks.borrow().is_released());
    let result = finish_viewer(
        owners,
        ViewerCleanupLimits::new(Duration::from_secs(2), 500, Duration::from_millis(2)).unwrap(),
    );
    assert_eq!(
        result.status,
        ViewerCleanupStatus::WorkersQuiescent(ViewerNativeState::NotRun)
    );
    assert!(callbacks.borrow().is_released());
}
