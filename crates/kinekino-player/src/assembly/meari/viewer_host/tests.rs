use super::close::*;
use super::*;

struct RegistrationReceiptHost {
    document: NativeDocumentHandle,
    redraw: Rc<super::super::player_shell_host::registration_redraw::RegistrationRedraw>,
}
impl NativeWindowHost for RegistrationReceiptHost {
    type Error = std::io::Error;
    type CloseCheckpoint = ();
    fn admit_callback(
        &mut self,
        _: &HostStamp,
        _: &PendingCallback,
    ) -> IntentAdmission<Self::Error> {
        panic!("registration redraw fixture exports no callbacks")
    }
    fn after_dispatch(&mut self, turn: HostTurn) -> Result<HostEffects, Self::Error> {
        Ok(HostEffects {
            committed: None,
            redraw: self
                .redraw
                .take_for(&turn, self.document.lifecycle() == HostLifecycle::Open),
        })
    }
    fn close(&mut self, _: CloseRequest<()>) -> CloseDecision<(), Self::Error> {
        self.redraw.cancel();
        CloseDecision::HostCleanupComplete { checkpoint: () }
    }
}

#[test]
fn registration_requests_one_original_completion_redraw_without_a_fake_frame() {
    let document = frame_document();
    let redraw = Rc::new(
        super::super::player_shell_host::registration_redraw::RegistrationRedraw::default(),
    );
    redraw.queue(&document.stamp().unwrap()).unwrap();
    let mut runtime = NativeHostRuntime::new(
        document.clone(),
        RegistrationReceiptHost {
            document: document.clone(),
            redraw: redraw.clone(),
        },
    );
    assert!(
        !runtime
            .after_dispatch(HostTurnReason::Input)
            .unwrap()
            .redraw
    );
    let first = runtime.after_dispatch(HostTurnReason::Completion).unwrap();
    assert!(first.redraw);
    assert!(first.committed.is_none());
    assert!(
        !runtime
            .after_dispatch(HostTurnReason::Completion)
            .unwrap()
            .redraw
    );
    redraw.queue(&document.stamp().unwrap()).unwrap();
    assert!(
        !runtime
            .after_dispatch(HostTurnReason::Completion)
            .unwrap()
            .redraw
    );
    assert_eq!(redraw.snapshot(), (true, true, false));
}

#[test]
fn registration_redraw_is_rejected_after_original_close_or_foreign_scope() {
    let document = frame_document();
    let original = document.stamp().unwrap();
    let redraw = Rc::new(
        super::super::player_shell_host::registration_redraw::RegistrationRedraw::default(),
    );
    redraw.queue(&document.stamp().unwrap()).unwrap();
    let mut runtime = NativeHostRuntime::new(
        document.clone(),
        RegistrationReceiptHost {
            document: document.clone(),
            redraw: redraw.clone(),
        },
    );
    runtime.request_close(CloseCause::User).unwrap();
    assert_ne!(document.lifecycle(), HostLifecycle::Open);
    assert!(redraw.snapshot().2);
    let foreign = frame_document();
    let mut foreign_runtime = NativeHostRuntime::new(
        foreign.clone(),
        RegistrationReceiptHost {
            document: foreign,
            redraw: redraw.clone(),
        },
    );
    assert!(
        !foreign_runtime
            .after_dispatch(HostTurnReason::Completion)
            .unwrap()
            .redraw
    );
    let other = Rc::new(
        super::super::player_shell_host::registration_redraw::RegistrationRedraw::default(),
    );
    other.queue(&original).unwrap();
    let another = frame_document();
    let mut runtime = NativeHostRuntime::new(
        another.clone(),
        RegistrationReceiptHost {
            document: another,
            redraw: other.clone(),
        },
    );
    assert!(
        !runtime
            .after_dispatch(HostTurnReason::Completion)
            .unwrap()
            .redraw
    );
    assert_eq!(other.snapshot(), (true, false, true));
}

#[test]
fn retained_routed_trace_is_bounded_and_keeps_functional_errors_separate() {
    use super::super::player_shell_host::registration_redraw::{
        CheckpointTrace, RoutedFrameTrace, bounded_error,
    };
    let mut trace = RoutedFrameTrace {
        bound: true,
        registration_attempted: true,
        registered: true,
        ..Default::default()
    };
    trace.error(Some(&"\n".repeat(4096)), Some(&"한".repeat(4096)));
    trace.redraw_cancelled = true;
    let checkpoint_error = bounded_error(&"\n".repeat(4096));
    assert!(checkpoint_error.truncated);
    assert_eq!(checkpoint_error.message.chars().count(), 128);
    trace.last_checkpoint = Some(CheckpointTrace {
        surface: "prepared_surface",
        original_engine_retained: false,
        retained_route_generation: None,
        admission_revoked: true,
        state_revoked: true,
        cleanup_retained: true,
        first_error: Some(checkpoint_error.clone()),
        revocation_error: Some(checkpoint_error.clone()),
    });
    trace.checkpoint_read_error = Some(checkpoint_error);
    assert!(trace.registered && trace.bound && trace.registration_attempted);
    assert!(!trace.activated && !trace.capture_attempted && !trace.ready_captured);
    assert!(trace.error_text_truncated);
    assert_eq!(
        trace
            .first_functional_error
            .as_ref()
            .unwrap()
            .chars()
            .count(),
        512
    );
    assert_eq!(
        trace.revocation_error.as_ref().unwrap().chars().count(),
        512
    );
    assert!(serde_json::to_vec(&trace).unwrap().len() <= 16384);
}

fn frame_document() -> NativeDocumentHandle {
    use meari_native::{domain::Viewport, infrastructure::document::InteractiveDocument};
    let source = meari::parse(
        "type Props={};export function App(props:Props){return <main/>;}",
        meari::application::Language::Tsx,
    )
    .unwrap();
    NativeDocumentHandle::new(
        InteractiveDocument::new(
            source,
            "",
            Viewport {
                width: 100,
                height: 100,
                dark: false,
            },
        )
        .unwrap(),
        1,
    )
    .unwrap()
}
fn pending_frame(stamp: &HostStamp) -> FrameOwnedResult {
    use crate::assembly::meari::quarantine_surface::frame_delivery::FrameResultState;
    FrameOwnedResult {
        request: std::num::NonZeroU64::new(9).unwrap(),
        observed: stamp.clone(),
        window: None,
        state: FrameResultState::Pending,
    }
}
struct FrameReceiptHost {
    document: NativeDocumentHandle,
    reflection: FrameReflection,
    turns: usize,
}
impl NativeWindowHost for FrameReceiptHost {
    type Error = std::io::Error;
    type CloseCheckpoint = ();
    fn admit_callback(
        &mut self,
        _: &HostStamp,
        _: &meari::application::host_callbacks::PendingCallback,
    ) -> IntentAdmission<Self::Error> {
        panic!("frame receipt fixture exports no callbacks")
    }
    fn after_dispatch(&mut self, turn: HostTurn) -> Result<HostEffects, Self::Error> {
        self.turns += 1;
        let (_, receipt) = self
            .document
            .try_reproject(&turn.stamp, |_| Ok::<_, std::io::Error>(()))
            .map_err(|error| std::io::Error::other(format!("{error:?}")))?;
        let mut effects = HostEffects {
            committed: Some(receipt),
            redraw: false,
        };
        self.reflection.apply(&turn, &mut effects);
        Ok(effects)
    }
    fn close(&mut self, _: CloseRequest<()>) -> CloseDecision<(), Self::Error> {
        CloseDecision::HostCleanupComplete { checkpoint: () }
    }
}
#[test]
fn frame_reflection_keeps_original_native_commit_and_has_no_default_effect() {
    let document = frame_document();
    let mut runtime = NativeHostRuntime::new(
        document.clone(),
        FrameReceiptHost {
            document,
            reflection: FrameReflection::default(),
            turns: 0,
        },
    );
    let receipt = runtime.after_dispatch(HostTurnReason::Completion).unwrap();
    assert!(receipt.committed.is_some());
    assert!(!receipt.redraw);
    assert_eq!(runtime.host().turns, 1);
    assert!(runtime.host().reflection.result.is_none());
}
#[test]
fn opted_in_completion_reflects_owned_data_without_changing_original_commit_receipt() {
    use crate::assembly::meari::quarantine_surface::frame_delivery::reflection_channel_for_test;
    let document = frame_document();
    let observed = document.stamp().unwrap();
    let (receiver, publish) = reflection_channel_for_test(
        std::num::NonZeroU64::new(9).unwrap(),
        &document.stamp().unwrap(),
        std::time::Instant::now() + Duration::from_secs(30),
    );
    publish(pending_frame(&observed), true);
    let mut reflection = FrameReflection {
        receiver: Some(receiver),
        ..Default::default()
    };
    let mut input = HostTurn {
        stamp: observed.clone(),
        reason: HostTurnReason::Input,
        accepted_callbacks: 0,
        rejected_callbacks: 0,
        remaining_callbacks: 0,
    };
    input.stamp.turn_sequence += 1;
    let mut effects = HostEffects::default();
    reflection.apply(&input, &mut effects);
    assert!(reflection.result.is_none());
    assert!(!effects.redraw);
    let mut runtime = NativeHostRuntime::new(
        document.clone(),
        FrameReceiptHost {
            document,
            reflection,
            turns: 0,
        },
    );
    let receipt = runtime.after_dispatch(HostTurnReason::Completion).unwrap();
    assert!(receipt.redraw);
    let commit = receipt.committed.unwrap();
    assert_eq!(
        commit.before().document_revision,
        observed.document_revision
    );
    assert_eq!(
        commit.after().document_revision,
        observed.document_revision + 1
    );
    let reflected = runtime.host().reflection.result.as_ref().unwrap();
    assert_eq!(reflected.observed, observed);
    assert_eq!(reflected.request.get(), 9);
    assert!(runtime.host().reflection.error.is_none());
    assert_eq!(runtime.host().turns, 1);
}
#[test]
fn foreign_frame_receiver_error_does_not_replace_sdk_native_commit_protocol() {
    use crate::assembly::meari::quarantine_surface::frame_delivery::reflection_channel_for_test;
    let document = frame_document();
    let mut foreign = document.stamp().unwrap();
    foreign.owner.source_generation += 1;
    let (receiver, publish) = reflection_channel_for_test(
        std::num::NonZeroU64::new(9).unwrap(),
        &document.stamp().unwrap(),
        std::time::Instant::now() + Duration::from_secs(30),
    );
    publish(pending_frame(&foreign), true);
    let mut runtime = NativeHostRuntime::new(
        document.clone(),
        FrameReceiptHost {
            document,
            reflection: FrameReflection {
                receiver: Some(receiver),
                ..Default::default()
            },
            turns: 0,
        },
    );
    let receipt = runtime.after_dispatch(HostTurnReason::Completion).unwrap();
    assert!(receipt.committed.is_some());
    assert!(!receipt.redraw);
    assert!(runtime.host().reflection.result.is_none());
    assert_eq!(
        runtime.host().reflection.error,
        Some(FrameReceiveError::ForeignScope)
    );
    assert_eq!(runtime.host().turns, 1);
}

#[test]
fn real_runtime_capacity_suffix_preserves_mailbox_then_same_turn_completion_reflects_data() {
    use crate::assembly::meari::quarantine_surface::frame_delivery::reflection_channel_for_test;
    use meari::{
        application::{
            host_callbacks::{CallbackHandle, HostCallbacks, PendingCallback},
            source_modules::ModuleProgram,
        },
        domain::{
            Value,
            build::BuildPath,
            host_callbacks::{CallbackReference, CallbackSignature},
        },
    };
    use meari_native::{
        domain::{Viewport, widgets::Action},
        infrastructure::document::InteractiveDocument,
    };
    use std::collections::BTreeMap;
    let entry: BuildPath = "frame-suffix.tsx".to_owned().try_into().unwrap();
    let graph=meari::parse_module_graph(&entry,&BTreeMap::from([(entry.clone(),"type Props={onIntent:()=>void;};export function App(props:Props){return <button onClick={()=>props.onIntent()}>Intent</button>;}".into())])).unwrap();
    let values = BTreeMap::from([(
        "onIntent".into(),
        Value::CallbackReference(CallbackReference {
            binding: "onIntent".into(),
            signature: CallbackSignature::Action,
        }),
    )]);
    let program = ModuleProgram::mount(graph, values, "frame-suffix".into(), 1).unwrap();
    let mut callback_owner = None;
    let document = InteractiveDocument::from_module_program_with_callbacks(
        program,
        "button{width:100px;height:30px}",
        Viewport {
            width: 100,
            height: 100,
            dark: false,
        },
        None,
        1,
        |scope| {
            let mut callbacks = HostCallbacks::new(scope);
            callbacks
                .bind_local("onIntent".into(), CallbackSignature::Action)
                .map_err(|error| error.to_string())?;
            let bindings = callbacks
                .export_bindings()
                .map_err(|error| error.to_string())?;
            callback_owner = Some(callbacks);
            Ok(bindings)
        },
    )
    .unwrap();
    let _callback_owner = callback_owner.unwrap();
    let action = document
        .module_projection()
        .unwrap()
        .callback_bindings()
        .into_iter()
        .find(|(_, handler)| handler.reference.binding == "onIntent")
        .unwrap()
        .0;
    let handle = NativeDocumentHandle::new(document, 1).unwrap();
    for _ in 0..2 {
        handle
            .dispatch_action(Action::Activate {
                path: action.clone(),
            })
            .unwrap();
    }
    struct SuffixHost {
        reflection: FrameReflection,
        tail: bool,
        accepted: Vec<CallbackHandle>,
    }
    impl NativeWindowHost for SuffixHost {
        type Error = std::io::Error;
        type CloseCheckpoint = ();
        fn admit_callback(
            &mut self,
            _: &HostStamp,
            callback: &PendingCallback,
        ) -> IntentAdmission<Self::Error> {
            if !self.accepted.is_empty() && !self.tail {
                return IntentAdmission::Capacity;
            }
            self.accepted.push(callback.handle.clone());
            IntentAdmission::Accepted
        }
        fn after_dispatch(&mut self, turn: HostTurn) -> Result<HostEffects, Self::Error> {
            let mut effects = HostEffects::default();
            self.reflection.apply(&turn, &mut effects);
            Ok(effects)
        }
        fn close(&mut self, _: CloseRequest<()>) -> CloseDecision<(), Self::Error> {
            CloseDecision::HostCleanupComplete { checkpoint: () }
        }
    }
    let (receiver, publish) = reflection_channel_for_test(
        std::num::NonZeroU64::new(9).unwrap(),
        &handle.stamp().unwrap(),
        std::time::Instant::now() + Duration::from_secs(30),
    );
    let mut runtime = NativeHostRuntime::new(
        handle.clone(),
        SuffixHost {
            reflection: FrameReflection {
                receiver: Some(receiver),
                ..Default::default()
            },
            tail: false,
            accepted: Vec::new(),
        },
    );
    let first = runtime.after_dispatch(HostTurnReason::Input).unwrap();
    assert_eq!(first.remaining_callbacks, 1);
    assert_eq!(runtime.host().accepted.len(), 1);
    let observed = handle.stamp().unwrap();
    publish(pending_frame(&observed), true);
    let blocked = runtime.after_dispatch(HostTurnReason::Completion).unwrap();
    assert_eq!(blocked.remaining_callbacks, 1);
    assert_eq!(blocked.stamp.turn_sequence, observed.turn_sequence);
    assert!(!blocked.redraw);
    assert!(runtime.host().reflection.result.is_none());
    runtime.host_mut().tail = true;
    let resumed = runtime.after_dispatch(HostTurnReason::Completion).unwrap();
    assert_eq!(resumed.remaining_callbacks, 0);
    assert_eq!(resumed.stamp, observed);
    assert!(resumed.redraw);
    assert!(resumed.committed.is_none());
    assert_eq!(runtime.host().accepted.len(), 2);
    assert_eq!(
        runtime.host().reflection.result.as_ref().unwrap().observed,
        observed
    );
    assert!(runtime.host().reflection.error.is_none());
}

#[test]
#[ignore = "Requires an explicit LocalHost pipe slot; CPU runtime only, no Native loop or OS view"]
fn real_host_completion_and_closing_preserve_read_consumer_and_worker_joins() {
    use crate::{
        application::{
            api::{
                ApplicationClient, ApplicationClientHandle, PlayerSdk, Request, RequestContext,
                Response,
            },
            sdk_execution::{CompletionNotifier, ExecutionBudget, ExecutionOwner},
        },
        infrastructure::local_host::{EndpointClient, LocalHost, TransportOptions},
        presentation::web_ui_adapter::player_shell::{
            RouteCatalog,
            document::{StartLabels, enable_read_panel, prepare_start_document, read_props},
        },
    };
    use meari::{
        application::source_modules::ModuleProgram,
        domain::{
            Span,
            host_callbacks::{
                CallbackArguments, CallbackInvocation, CallbackReference, CallbackSignature,
            },
        },
    };
    use meari_native::{
        domain::Viewport,
        infrastructure::{document::InteractiveDocument, native_host::*},
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::time::Instant;
    struct Notifier(Arc<AtomicUsize>);
    impl CompletionNotifier for Notifier {
        fn notify(&self) -> Result<(), ApiError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    let notified = Arc::new(AtomicUsize::new(0));
    use std::sync::{Mutex, mpsc};
    struct Gate {
        entered: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
        sdk: PlayerSdk,
    }
    impl ApplicationClient for Gate {
        fn execute(&self, request: Request) -> Response {
            if request.context.request_id == "c-endpoint-gate" {
                let _ = self.entered.send(());
                let _ = self
                    .release
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(10));
            }
            self.sdk.execute(request)
        }
    }
    struct Release(Option<mpsc::Sender<()>>);
    impl Drop for Release {
        fn drop(&mut self) {
            if let Some(release) = self.0.take() {
                let _ = release.send(());
            }
        }
    }
    let (entered, observed) = mpsc::channel();
    let (release, waiting) = mpsc::channel();
    let mut release = Release(Some(release));
    let client: ApplicationClientHandle = Arc::new(Gate {
        entered,
        release: Mutex::new(waiting),
        sdk: PlayerSdk::default(),
    });
    let execution = ExecutionOwner::start(
        client.clone(),
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 262144,
            max_total_request_bytes: 8 * 262144,
            max_terminal_bytes: 262144,
            max_total_terminal_bytes: 8 * 262144,
        },
        Arc::new(Notifier(notified.clone())),
    )
    .unwrap();
    let endpoint = LocalHost::start(client.clone(), TransportOptions::default()).unwrap();
    let endpoint_name = endpoint.endpoint().to_owned();
    let mut host = ViewerHost::new(client, execution, endpoint);
    let callbacks = host.callbacks();
    let labels = StartLabels {
        connection: "미관측".into(),
        time: "미관측".into(),
    };
    let props = read_props(&labels);
    let mut prepared =
        prepare_start_document(&RouteCatalog::bundled().unwrap(), "view.start", labels)
            .ok()
            .unwrap();
    enable_read_panel(&mut prepared, props.clone())
        .ok()
        .unwrap();
    let program =
        ModuleProgram::mount(prepared.graph, prepared.props, "read-host-turn".into(), 1).unwrap();
    let exporting = callbacks.clone();
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
            exporting
                .borrow_mut()
                .export(scope)
                .map_err(|error| error.message)
        },
    )
    .unwrap();
    let handle = NativeDocumentHandle::new(document, 1).unwrap();
    host.bind_document(handle.clone(), props);
    let mut runtime = NativeHostRuntime::new(handle.clone(), host);
    let submit = |runtime: &mut NativeHostRuntime<ViewerHost>| {
        let bindings = callbacks.borrow().bindings().unwrap();
        let pending = meari::application::host_callbacks::PendingCallback {
            handle: bindings
                .resolve(&CallbackReference {
                    binding: "onReadCapabilities".into(),
                    signature: CallbackSignature::Action,
                })
                .unwrap(),
            invocation: CallbackInvocation {
                prop: "onReadCapabilities".into(),
                arguments: CallbackArguments::Action,
                span: Span::default(),
            },
        };
        assert!(matches!(
            runtime
                .host_mut()
                .admit_callback(&handle.stamp().unwrap(), &pending),
            IntentAdmission::Accepted
        ));
        // bindings lease remains owned by this test until admission has completed.
    };
    submit(&mut runtime);
    let deadline = Instant::now() + Duration::from_secs(5);
    while notified.load(Ordering::SeqCst) == 0 {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    let receipt = runtime.after_dispatch(HostTurnReason::Completion).unwrap();
    assert!(receipt.committed.is_some());
    assert!(callbacks.borrow().tickets().unwrap().is_empty());
    assert!(
        callbacks.borrow().projection.borrow().panel.rows[0]
            .value
            .is_some()
    );
    let local_bindings = callbacks.borrow().bindings().unwrap();
    let refresh = meari::application::host_callbacks::PendingCallback {
        handle: local_bindings
            .resolve(&CallbackReference {
                binding: "onRefreshReadProjection".into(),
                signature: CallbackSignature::Action,
            })
            .unwrap(),
        invocation: CallbackInvocation {
            prop: "onRefreshReadProjection".into(),
            arguments: CallbackArguments::Action,
            span: Span::default(),
        },
    };
    assert!(matches!(
        runtime
            .host_mut()
            .admit_callback(&handle.stamp().unwrap(), &refresh),
        IntentAdmission::Accepted
    ));
    assert!(
        runtime
            .after_dispatch(HostTurnReason::Completion)
            .unwrap()
            .committed
            .is_some()
    );
    assert!(callbacks.borrow().local_queue_empty());
    assert!(callbacks.borrow().tickets().unwrap().is_empty());
    // Simulate the document replacing the exact export that supplied this queued handle.
    local_bindings.invalidate();
    assert!(matches!(
        runtime
            .host_mut()
            .admit_callback(&handle.stamp().unwrap(), &refresh),
        IntentAdmission::Rejected(_)
    ));
    use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::{
        AsyncCodecBudget, AsyncSdkPort, TypedAsyncSdkPlugin, async_binding,
    };
    use meari::application::plugins::*;
    use std::cell::Cell;
    struct Fault(Rc<Cell<usize>>, Rc<Cell<bool>>);
    impl AsyncSdkPort<super::super::viewer_callbacks::codec::ReadInput, serde_json::Value> for Fault {
        fn start(
            &mut self,
            _: &PluginTicket,
            _: super::super::viewer_callbacks::codec::ReadInput,
        ) -> Result<(), PluginError> {
            Ok(())
        }
        fn peek_terminal(
            &mut self,
            _: &PluginTicket,
        ) -> Result<Option<Arc<Result<serde_json::Value, ApiError>>>, PluginError> {
            Ok(Some(Arc::new(Ok(serde_json::json!({"commands":[]})))))
        }
        fn acknowledge(&mut self, _: &PluginTicket) -> Result<(), PluginError> {
            self.0.set(self.0.get() + 1);
            if self.1.get() {
                Err(PluginError::new(
                    PluginErrorCode::Busy,
                    "controlled ACK failure",
                ))
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
    let attempts = Rc::new(Cell::new(0));
    let fail_ack = Rc::new(Cell::new(true));
    let manifest = super::super::viewer_callbacks::codec::manifest();
    let bindings = super::super::viewer_callbacks::codec::READS
        .into_iter()
        .map(|(_, command)| {
            (
                command.into(),
                async_binding(
                    manifest.capabilities[command].clone(),
                    Fault(attempts.clone(), fail_ack.clone()),
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
    callbacks
        .borrow_mut()
        .install_fault_adapter(plugin, pump)
        .unwrap();
    let requester = std::thread::spawn(move || {
        EndpointClient::new(&endpoint_name, TransportOptions::default())
            .unwrap()
            .execute(Request {
                context: RequestContext {
                    request_id: "c-endpoint-gate".into(),
                    ..Default::default()
                },
                command: "capabilities".into(),
                payload: serde_json::json!({}),
            })
    });
    observed.recv_timeout(Duration::from_secs(2)).unwrap();
    submit(&mut runtime); // fake ACK-fault adapter; same production consumer/actual endpoint gate
    let mut progress = runtime.request_close(CloseCause::User).unwrap();
    assert!(matches!(
        runtime
            .host_mut()
            .admit_callback(&handle.stamp().unwrap(), &refresh),
        IntentAdmission::Rejected(_)
    ));
    let gate_until = Instant::now() + Duration::from_millis(100);
    while Instant::now() < gate_until {
        assert_ne!(progress, HostCloseStatus::HostCleanupComplete);
        std::thread::sleep(Duration::from_millis(5));
        progress = runtime.observe_close(CloseCause::User).unwrap();
    }
    assert_eq!(attempts.get(), 3);
    assert!(matches!(
        runtime.host().last_close_progress(),
        Some(ViewerCloseProgress::EndpointPending(_))
    ));
    assert_eq!(callbacks.borrow().tickets().unwrap().len(), 1);
    fail_ack.set(false);
    let ticket = callbacks.borrow().tickets().unwrap().remove(0);
    callbacks
        .borrow_mut()
        .retry_read_acceptance(&ticket)
        .unwrap();
    let _ = release.0.take().unwrap().send(());
    let _response = requester.join().unwrap();
    while progress != HostCloseStatus::HostCleanupComplete {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
        progress = runtime.observe_close(CloseCause::User).unwrap();
    }
    assert!(callbacks.borrow().tickets().unwrap().is_empty());
    assert_eq!(attempts.get(), 4);
    assert!(callbacks.borrow().is_released());
    assert!(
        matches!(runtime.host().last_close_progress(), Some(ViewerCloseProgress::Complete { endpoint, execution }) if endpoint.is_quiescent() && execution.worker_stopped && execution.unacknowledged == 0)
    );
    runtime.finish_headless_close().unwrap();
}
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        sdk_execution::{ExecutionCloseProgress, ExecutionCloseReceipt, WorkerCloseState},
    },
    infrastructure::local_host::{DispatchState, LocalHostShutdownStatus, WorkerJoin},
};
use std::{cell::RefCell, collections::VecDeque};

struct Endpoint {
    states: VecDeque<LocalHostShutdownStatus>,
    calls: CallLog,
    _retained: Rc<()>,
}
struct Execution {
    states: VecDeque<Result<ExecutionCloseProgress, ApiError>>,
    calls: CallLog,
    _retained: Rc<()>,
}
type CallLog = Rc<RefCell<Vec<&'static str>>>;
struct CardExecution {
    pending: Rc<std::cell::Cell<bool>>,
    fail_once: bool,
    calls: CallLog,
}
impl ExecutionPort for CardExecution {
    fn prepare_card_shutdown(&mut self) -> Result<bool, ViewerCloseError> {
        self.calls.borrow_mut().push("card");
        if self.fail_once {
            self.fail_once = false;
            return Err(ViewerCloseError::CardScope(Arc::new(
                meari::application::plugins::PluginError::new(
                    meari::application::plugins::PluginErrorCode::Busy,
                    "Card scope retained",
                ),
            )));
        }
        Ok(!self.pending.get())
    }
    fn poll_shutdown(&mut self) -> Result<ExecutionCloseProgress, ApiError> {
        self.calls.borrow_mut().push("execution");
        Ok(complete())
    }
}

#[test]
fn active_card_pending_does_not_close_original_execution_or_repeat_endpoint_join() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let endpoint = Endpoint {
        states: vec![status(WorkerJoin::Joined, WorkerJoin::Joined)].into(),
        calls: calls.clone(),
        _retained: Rc::new(()),
    };
    let pending = Rc::new(std::cell::Cell::new(true));
    let mut close = OrderedClose::new(
        endpoint,
        CardExecution {
            pending: pending.clone(),
            fail_once: false,
            calls: calls.clone(),
        },
    );
    assert!(matches!(
        close.poll(),
        Ok(ViewerCloseProgress::CardScopePending)
    ));
    assert_eq!(calls.borrow().as_slice(), &["endpoint", "card"]);
    pending.set(false);
    assert!(matches!(
        close.poll(),
        Ok(ViewerCloseProgress::Complete { .. })
    ));
    assert_eq!(
        calls.borrow().as_slice(),
        &["endpoint", "card", "card", "execution"]
    );
    close.poll().unwrap();
    assert_eq!(calls.borrow().len(), 4);
}

#[test]
fn active_card_release_error_retains_owners_and_join_receipt_for_retry() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let endpoint = Endpoint {
        states: vec![status(WorkerJoin::Joined, WorkerJoin::Joined)].into(),
        calls: calls.clone(),
        _retained: Rc::new(()),
    };
    let mut close = OrderedClose::new(
        endpoint,
        CardExecution {
            pending: Rc::new(std::cell::Cell::new(false)),
            fail_once: true,
            calls: calls.clone(),
        },
    );
    assert!(matches!(close.poll(), Err(ViewerCloseError::CardScope(_))));
    assert_eq!(calls.borrow().as_slice(), &["endpoint", "card"]);
    assert!(matches!(
        close.poll(),
        Ok(ViewerCloseProgress::Complete { .. })
    ));
    assert_eq!(
        calls.borrow().as_slice(),
        &["endpoint", "card", "card", "execution"]
    );
}
type TestClose = OrderedClose<Endpoint, Execution>;
impl EndpointPort for Endpoint {
    fn poll_shutdown(&mut self) -> LocalHostShutdownStatus {
        self.calls.borrow_mut().push("endpoint");
        self.states.pop_front().expect("extra endpoint poll")
    }
}
impl ExecutionPort for Execution {
    fn poll_shutdown(&mut self) -> Result<ExecutionCloseProgress, ApiError> {
        self.calls.borrow_mut().push("execution");
        self.states.pop_front().expect("extra execution poll")
    }
}
fn status(pipe: WorkerJoin, sdk: WorkerJoin) -> LocalHostShutdownStatus {
    LocalHostShutdownStatus {
        pipe,
        sdk,
        dispatch: Some(DispatchState::Running),
    }
}
fn complete() -> ExecutionCloseProgress {
    ExecutionCloseProgress::Closed(ExecutionCloseReceipt {
        worker_stopped: true,
        unacknowledged: 0,
    })
}
fn ordered(
    endpoint: Vec<LocalHostShutdownStatus>,
    execution: Vec<Result<ExecutionCloseProgress, ApiError>>,
) -> (TestClose, CallLog, Rc<()>) {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let retained = Rc::new(());
    (
        OrderedClose::new(
            Endpoint {
                states: endpoint.into(),
                calls: calls.clone(),
                _retained: retained.clone(),
            },
            Execution {
                states: execution.into(),
                calls: calls.clone(),
                _retained: retained.clone(),
            },
        ),
        calls,
        retained,
    )
}

#[test]
fn accepted_endpoint_work_drains_and_both_joins_precede_execution_close() {
    let (mut owner, calls, retained) = ordered(
        vec![
            status(WorkerJoin::Pending, WorkerJoin::Pending),
            status(WorkerJoin::Joined, WorkerJoin::Pending),
            status(WorkerJoin::Joined, WorkerJoin::Joined),
        ],
        vec![Ok(complete())],
    );
    assert!(matches!(
        owner.poll(),
        Ok(ViewerCloseProgress::EndpointPending(_))
    ));
    assert!(matches!(
        owner.poll(),
        Ok(ViewerCloseProgress::EndpointPending(_))
    ));
    assert_eq!(*calls.borrow(), ["endpoint", "endpoint"]);
    assert_eq!(Rc::strong_count(&retained), 3);
    assert!(matches!(
        owner.poll(),
        Ok(ViewerCloseProgress::Complete { .. })
    ));
    assert_eq!(
        *calls.borrow(),
        ["endpoint", "endpoint", "endpoint", "execution"]
    );
    assert!(matches!(
        owner.poll(),
        Ok(ViewerCloseProgress::Complete { .. })
    ));
    assert_eq!(calls.borrow().len(), 4);
    // Both original endpoint/execution ports remain owned even after receipt.
    assert_eq!(Rc::strong_count(&retained), 3);
}

#[test]
fn join_panic_does_not_start_execution_shutdown_or_discard_owners() {
    let (mut owner, calls, retained) = ordered(
        vec![status(WorkerJoin::Panicked, WorkerJoin::Joined)],
        vec![],
    );
    assert!(matches!(
        owner.poll(),
        Err(ViewerCloseError::EndpointJoin(_))
    ));
    assert_eq!(*calls.borrow(), ["endpoint"]);
    assert_eq!(Rc::strong_count(&retained), 3);
}

#[test]
fn terminal_ack_and_worker_exit_are_pending_without_auto_acceptance() {
    let (mut owner, calls, _) = ordered(
        vec![status(WorkerJoin::Joined, WorkerJoin::Joined)],
        vec![
            Ok(ExecutionCloseProgress::TerminalAckPending {
                unacknowledged: 1,
                worker: WorkerCloseState::Joined,
            }),
            Ok(ExecutionCloseProgress::WorkerExitPending {
                loop_finished: true,
            }),
            Ok(complete()),
        ],
    );
    assert!(matches!(
        owner.poll(),
        Ok(ViewerCloseProgress::ExecutionPending(
            ExecutionCloseProgress::TerminalAckPending { .. }
        ))
    ));
    assert!(matches!(
        owner.poll(),
        Ok(ViewerCloseProgress::ExecutionPending(
            ExecutionCloseProgress::WorkerExitPending { .. }
        ))
    ));
    assert!(matches!(
        owner.poll(),
        Ok(ViewerCloseProgress::Complete { .. })
    ));
    assert_eq!(
        *calls.borrow(),
        ["endpoint", "execution", "execution", "execution"]
    );
}

#[test]
fn poisoned_dispatch_is_not_fabricated_as_idle_and_invalid_worker_receipt_is_rejected() {
    let mut endpoint = status(WorkerJoin::Joined, WorkerJoin::Joined);
    endpoint.dispatch = None;
    let (mut owner, _, _) = ordered(
        vec![endpoint],
        vec![Ok(ExecutionCloseProgress::Closed(ExecutionCloseReceipt {
            worker_stopped: false,
            unacknowledged: 1,
        }))],
    );
    assert!(matches!(
        owner.poll(),
        Err(ViewerCloseError::InvalidExecutionReceipt(_))
    ));
    let (mut owner, _, _) = ordered(vec![endpoint], vec![Ok(complete())]);
    match owner.poll().unwrap() {
        ViewerCloseProgress::Complete { endpoint, .. } => assert!(endpoint.dispatch.is_none()),
        _ => panic!("two actual joins allow quiescence, without inventing dispatch state"),
    }
}

#[test]
fn sdk_diagnostic_and_endpoint_join_receipt_remain_owned_on_error() {
    let (mut owner, calls, retained) = ordered(
        vec![status(WorkerJoin::Joined, WorkerJoin::Joined)],
        vec![Err(ApiError::new(
            ErrorCode::NeedsInput,
            "original SDK diagnostic",
        ))],
    );
    match owner.poll() {
        Err(ViewerCloseError::Sdk(error)) => assert_eq!(error.message, "original SDK diagnostic"),
        _ => panic!("original error required"),
    }
    assert_eq!(*calls.borrow(), ["endpoint", "execution"]);
    assert_eq!(Rc::strong_count(&retained), 3);
}

#[test]
fn endpoint_joins_precede_retained_callback_ack_release_and_original_worker_join() {
    use crate::application::{
        api::{ApplicationClientHandle, PlayerSdk, RequestContext},
        sdk_execution::{CompletionNotifier, ExecutionBudget, ExecutionOwner},
    };
    use meari::application::plugins::PluginDispatch;
    use meari::{
        application::{host_callbacks::PendingCallback, state_program::StateProgram},
        domain::{
            Span,
            host_callbacks::{
                CallbackArguments, CallbackInvocation, CallbackReference, CallbackSignature,
            },
        },
    };
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };
    struct Notify;
    impl CompletionNotifier for Notify {
        fn notify(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let owner = ExecutionOwner::start(
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
    let callbacks = Rc::new(RefCell::new(ViewerCallbacks::new(
        owner.handle(),
        RequestContext::default(),
    )));
    let component = meari::parse(
        "type Props={};export function Root(props:Props){return <main/>;}",
        meari::application::Language::Tsx,
    )
    .unwrap();
    let program = StateProgram::mount(component, "viewer-close-order".into(), 1).unwrap();
    let scope = meari::application::plugins::PluginScope {
        instance: program.snapshot().unwrap().instance,
        source_generation: 1,
    };
    let bindings = callbacks.borrow_mut().export(scope.clone()).unwrap();
    let handle = bindings
        .resolve(&CallbackReference {
            binding: "onReadCapabilities".into(),
            signature: CallbackSignature::Action,
        })
        .unwrap();
    let pending = PendingCallback {
        handle,
        invocation: CallbackInvocation {
            prop: "onReadCapabilities".into(),
            arguments: CallbackArguments::Action,
            span: Span::default(),
        },
    };
    let receipt = callbacks.borrow().dispatch(&scope, &pending).unwrap();
    assert!(matches!(receipt.outcome, PluginDispatch::Pending));
    callbacks.borrow_mut().stop_admission();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let endpoint = Endpoint {
        states: vec![
            status(WorkerJoin::Joined, WorkerJoin::Pending),
            status(WorkerJoin::Joined, WorkerJoin::Joined),
        ]
        .into(),
        calls: calls.clone(),
        _retained: Rc::new(()),
    };
    let mut close = OrderedClose::new(
        endpoint,
        ViewerExecution {
            owner,
            callbacks: callbacks.clone(),
            cards: Rc::new(RefCell::new(super::cards::CardRoute::default())),
            jobs: Rc::new(RefCell::new(super::jobs::JobsLedger::default())),
        },
    );
    assert!(matches!(
        close.poll(),
        Ok(ViewerCloseProgress::EndpointPending(_))
    ));
    assert!(!callbacks.borrow().is_released());
    match close.poll().unwrap() {
        ViewerCloseProgress::CallbackPending { tickets } => {
            assert_eq!(tickets.as_slice(), std::slice::from_ref(&receipt.ticket))
        }
        _ => panic!("terminal ACK must precede release and owner polling"),
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(accepted) = callbacks
            .borrow_mut()
            .accept_terminal(&receipt.ticket)
            .unwrap()
        {
            accepted.owner_ack.unwrap();
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    loop {
        if matches!(close.poll(), Ok(ViewerCloseProgress::Complete { .. })) {
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(callbacks.borrow().is_released());
    assert_eq!(*calls.borrow(), ["endpoint", "endpoint"]);
}
