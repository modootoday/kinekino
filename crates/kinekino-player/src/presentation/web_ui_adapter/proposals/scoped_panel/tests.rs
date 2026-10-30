use super::*;
use crate::application::api::ProposalClient;
#[path = "tests/current_inspection.rs"]
mod current_inspection;
use kinekino_player::{
    application::{
        api::PlayerSdk,
        proposals::{CreateProposal, InspectProposal},
    },
    assembly::meari::proposal_registration::ProposalSdkRegistration,
    domain::{api_values::DecimalU64, proposals::*},
};
use meari::{
    application::{Language, state_program::StateProgram},
    domain::{
        build::{ContentHash, TargetKind},
        host_callbacks::{CallbackArguments, CallbackReference, CallbackSignature},
    },
};
use std::{cell::Cell, sync::Arc};

struct Lease {
    sdk: ProposalSdkRegistration,
    broken_handle: bool,
    busy_close: Rc<Cell<bool>>,
}
impl ParentProposalLease for Lease {
    fn registration(&self) -> Result<&RegistrationHandle, PluginError> {
        if self.broken_handle {
            return Err(PluginError::new(
                PluginErrorCode::Busy,
                "Injected handle fault",
            ));
        }
        self.sdk.handle()
    }
    fn close(&mut self) -> Result<(), PluginError> {
        if self.busy_close.get() {
            return Err(PluginError::new(
                PluginErrorCode::Busy,
                "Injected unbind fault",
            ));
        }
        self.sdk.close()
    }
}
pub(in super::super) fn fixture() -> (StateProgram, ScopedCardInput, StoredProposal) {
    let component = meari::parse(
        "type Props={};export function Frame(props:Props){return <main/>;}",
        Language::Tsx,
    )
    .unwrap();
    let program = StateProgram::mount(component, "scoped-card-frame".into(), 1).unwrap();
    let scope = PluginScope {
        instance: program.snapshot().unwrap().instance,
        source_generation: 1,
    };
    let store_path = std::env::current_dir()
        .unwrap()
        .join("runtime/quality-review/ui-scoped-card-store")
        .join(format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&store_path).unwrap();
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default().with_proposal_store(
        kinekino_player::infrastructure::proposals::ProposalFiles::open(&store_path).unwrap(),
    ));
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
    let document = client
        .create_proposal(
            RequestContext::default(),
            CreateProposal {
                proposal_id: "scoped-card".into(),
                source: SourceSnapshot::Cue(CueSourceSnapshot {
                    source_id: "caption-source".into(),
                    source_revision: DecimalU64(3),
                    source_sha256: "a".repeat(64),
                    cues: vec![SourceCue {
                        cue_id: "cue-a".into(),
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
    let stored = client
        .inspect_proposal(
            RequestContext::default(),
            InspectProposal {
                proposal_id: document.proposal_id.clone(),
            },
        )
        .unwrap();
    let registry = Rc::new(RefCell::new(
        PluginRegistry::new(
            ContentHash::of(b"scoped-card-fixture"),
            TargetKind::Native,
            scope.clone(),
            8,
        )
        .unwrap(),
    ));
    let input = ScopedCardInput {
        client,
        context: RequestContext::default(),
        scope,
        registry,
        selection: BootstrapSelection {
            proposal_id: document.proposal_id,
            document_revision: document.draft_revision,
            source_id: document.source.source_id().into(),
            source_revision: document.source.source_revision(),
            source_sha256: document.source.source_sha256().into(),
            target: BootstrapTarget::Card { index: 1 },
        },
        labels: BootstrapLabels {
            panel: super::super::panel_model::PanelLabels {
                feedback: crate::presentation::web_ui_adapter::proposals::review_feedback::FeedbackLabels::english(),
                revision: "revision".into(),
                source_identity: "source".into(),
                quality_unknown: "unreviewed".into(),
                selected_item: "second".into(),
                range: "range".into(),
                result: "result".into(),
            },
            source_count_label: "source".into(),
            unselected_source_text: "choose".into(),
        },
        budget: PanelBudget {
            entries: 8,
            text_bytes: 8192,
        },
        pins: super::super::product_window::product_intent_pins(),
        namespace: "scoped-card".into(),
        pending_limit: 8,
        local_signatures: BTreeMap::new(),
    };
    (program, input, stored)
}
type RegisterParent = dyn FnOnce(
    ApplicationClientHandle,
    Rc<RefCell<PluginRegistry>>,
    &PluginScope,
    Rc<RefCell<RequestContext>>,
) -> Result<Lease, PluginError>;
fn register(broken_handle: bool, busy_close: Rc<Cell<bool>>) -> Box<RegisterParent> {
    Box::new(move |client, registry, scope, context| {
        Ok(Lease {
            sdk: ProposalSdkRegistration::register(client, registry, scope, context)?,
            broken_handle,
            busy_close,
        })
    })
}

#[test]
fn stale_identity_or_foreign_registry_never_enters_parent_registration() {
    for variant in 0..4 {
        let (_frame, mut input, stored) = fixture();
        match variant {
            0 => input.selection.source_sha256 = "b".repeat(64),
            1 => input.selection.document_revision.0 += 1,
            2 => input.selection.target = BootstrapTarget::Card { index: 2 },
            _ => input.scope.source_generation += 1,
        }
        let result = bind_card_panel::<Lease>(input, stored, |_, _, _, _| {
            panic!("Invalid selection must precede SDK registration")
        });
        assert!(result.is_err());
    }
}

#[test]
fn registration_failure_retains_original_parent_until_explicit_unbind() {
    let (_frame, input, stored) = fixture();
    let busy = Rc::new(Cell::new(true));
    let mut failure = match bind_card_panel(input, stored, register(true, busy.clone())) {
        Err(failure) => failure,
        Ok(_) => panic!("Injected registration handle fault must be returned"),
    };
    assert!(failure.parent.is_some());
    assert!(failure.session.is_none());
    assert!(failure.retry_unbind().is_err());
    assert!(failure.parent.is_some());
    busy.set(false);
    failure.retry_unbind().unwrap();
    assert!(failure.parent.as_ref().unwrap().sdk.handle().is_err());
}

#[test]
fn original_scope_second_card_edit_save_accept_and_reopen_use_existing_sdk() {
    let (frame, input, stored) = fixture();
    let original_instance = frame.snapshot().unwrap().instance;
    let client = input.client.clone();
    let mut panel = bind_card_panel(input, stored, register(false, Rc::new(Cell::new(false))))
        .unwrap_or_else(|_| panic!("Canonical selected card must bind"));
    assert_eq!(
        panel.session.review().scope().view.instance,
        original_instance
    );
    assert_eq!(
        panel.initial_props()["front"],
        Value::String("second".into())
    );
    let export = panel.session.export_callbacks().unwrap();
    for (name, arguments, signature) in [
        (
            "onFrontChange",
            CallbackArguments::Text("修正した表面".into()),
            CallbackSignature::Text,
        ),
        (
            "onSave",
            CallbackArguments::Action,
            CallbackSignature::Action,
        ),
        (
            "onAccept",
            CallbackArguments::Action,
            CallbackSignature::Action,
        ),
    ] {
        panel
            .session
            .dispatch_after_renderer_guard(
                &export,
                &CallbackReference {
                    binding: name.into(),
                    signature,
                },
                arguments,
            )
            .unwrap();
        let report = panel.session.drain_after_dispatch().unwrap();
        assert!(!report.sdk_error);
        assert!(report.view_error.is_none());
        assert!(report.transport_error.is_none());
        if name == "onSave" {
            let reopened = client
                .inspect_proposal(
                    RequestContext::default(),
                    InspectProposal {
                        proposal_id: "scoped-card".into(),
                    },
                )
                .unwrap();
            assert!(reopened.acceptance.is_none(), "Save must not accept");
        }
    }
    let reopened = client
        .inspect_proposal(
            RequestContext::default(),
            InspectProposal {
                proposal_id: "scoped-card".into(),
            },
        )
        .unwrap();
    assert!(reopened.acceptance.is_some());
    let Some(ProposalPayload::Cards { cards }) = reopened.document.draft else {
        panic!("Typed cards must survive reopen")
    };
    assert_eq!(cards[0].front, "first");
    assert_eq!(cards[1].front, "修正した表面");
    panel.unbind().unwrap();
    assert!(
        export
            .resolve(&CallbackReference {
                binding: "onSave".into(),
                signature: CallbackSignature::Action
            })
            .is_err()
    );
    assert!(panel.parent.sdk.handle().is_err());
}

#[test]
fn async_output_review_panel_unbinds_without_closing_original_execution() {
    use kinekino_player::{
        application::sdk_execution::{
            CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner,
        },
        assembly::meari::scoped_card_session::bind_scoped_card,
    };
    use std::time::{Duration, Instant};
    struct Notify;
    impl CompletionNotifier for Notify {
        fn notify(&self) -> Result<(), kinekino_player::application::api::ApiError> {
            Ok(())
        }
    }
    let (_frame, input, stored) = fixture();
    let client = input.client.clone();
    let execution = ExecutionOwner::start(
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
    let handle = execution.handle();
    let mut panel = bind_scoped_card(
        input,
        stored,
        handle,
        super::super::product_window::product_codec_budget(),
    )
    .unwrap_or_else(|_| panic!("Original SDK execution must attach to the scoped editor"));
    assert!(panel.session.output_review().is_some());
    assert!(execution.handle().attach().is_err());
    let export = panel.session.export_callbacks().unwrap();
    for (prop, args, signature) in [
        (
            "onFrontChange",
            CallbackArguments::Text("async second".into()),
            CallbackSignature::Text,
        ),
        (
            "onSave",
            CallbackArguments::Action,
            CallbackSignature::Action,
        ),
        (
            "onAccept",
            CallbackArguments::Action,
            CallbackSignature::Action,
        ),
    ] {
        panel
            .session
            .dispatch_after_renderer_guard(
                &export,
                &CallbackReference {
                    binding: prop.into(),
                    signature,
                },
                args,
            )
            .unwrap();
        let report = panel.session.drain_after_dispatch().unwrap();
        assert!(report.view_error.is_none());
        assert!(report.transport_error.is_none());
        assert!(!report.sdk_error);
        let deadline = Instant::now() + Duration::from_secs(2);
        while panel.session.operation_pending() {
            panel.session.pump_after_dispatch().unwrap();
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(panel.session.review().error().is_none());
        if prop == "onSave" {
            assert!(panel.session.review().stored().acceptance.is_none());
        }
    }
    let reopened = client
        .inspect_proposal(
            RequestContext::default(),
            InspectProposal {
                proposal_id: "scoped-card".into(),
            },
        )
        .unwrap();
    assert!(reopened.acceptance.is_some());
    let Some(ProposalPayload::Cards { cards }) = reopened.document.draft else {
        panic!("Cards must remain typed")
    };
    assert_eq!(cards[1].front, "async second");
    panel.unbind().unwrap();
    let next = execution.handle().attach().unwrap();
    next.release().unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let ExecutionCloseProgress::Closed(receipt) = execution.poll_close().unwrap() {
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
