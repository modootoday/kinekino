use super::*;
use kinekino_player::{
    application::{
        api::RequestContext,
        proposals::{AcceptProposal, CreateProposal, EditProposal},
    },
    domain::{api_values::DecimalU64, proposals::*},
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::*;
use meari::{
    application::{Language, plugins::*, state_program::StateProgram},
    domain::build::{ContentHash, TargetKind},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
    rc::Rc,
    sync::Arc,
};

#[path = "raw_reviews.rs"]
mod raw_reviews;

#[derive(Default)]
struct Probe {
    calls: Vec<String>,
    replies: BTreeMap<String, serde_json::Value>,
    fail_ack: bool,
    acks: usize,
    shutdowns: usize,
}
struct Port<I, O> {
    probe: Rc<RefCell<Probe>>,
    command: &'static str,
    cache: Option<Arc<Result<O, ApiError>>>,
    types: PhantomData<I>,
}
impl<I, O: DeserializeOwned> AsyncSdkPort<I, O> for Port<I, O> {
    fn start(&mut self, _: &PluginTicket, _: I) -> Result<(), PluginError> {
        self.cache = None;
        self.probe.borrow_mut().calls.push(self.command.into());
        Ok(())
    }
    fn peek_terminal(
        &mut self,
        _: &PluginTicket,
    ) -> Result<Option<Arc<Result<O, ApiError>>>, PluginError> {
        if self.cache.is_none()
            && let Some(value) = self.probe.borrow_mut().replies.remove(self.command)
        {
            self.cache = Some(Arc::new(serde_json::from_value(value).unwrap()));
        }
        Ok(self.cache.clone())
    }
    fn acknowledge(&mut self, _: &PluginTicket) -> Result<(), PluginError> {
        if self.probe.borrow().fail_ack {
            return Err(PluginError::new(PluginErrorCode::Busy, "ACK retry"));
        }
        self.probe.borrow_mut().acks += 1;
        Ok(())
    }
    fn cancel(&mut self, _: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        Ok(CancelReceipt::Requested)
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        self.probe.borrow_mut().shutdowns += 1;
        Ok(())
    }
}
fn hash(value: &str) -> ContentHash {
    ContentHash::of(value.as_bytes())
}
fn descriptor(command: &str) -> CapabilityDescriptor {
    CapabilityDescriptor {
        targets: BTreeSet::from([TargetKind::Native]),
        request_schema: hash(&format!("{command}:request")),
        reply_schema: hash(&format!("{command}:reply")),
        error_schema: Some(hash("error")),
        event_schema: None,
    }
}
fn binding<I: DeserializeOwned + 'static, O: Serialize + DeserializeOwned + 'static>(
    probe: &Rc<RefCell<Probe>>,
    command: &'static str,
) -> (String, Box<dyn AsyncSdkBinding>) {
    (
        command.into(),
        async_binding(
            descriptor(command),
            Port::<I, O> {
                probe: probe.clone(),
                command,
                cache: None,
                types: PhantomData::<I>,
            },
        ),
    )
}
fn seed() -> StoredProposal {
    serde_json::from_value(serde_json::json!({"document":{"schema_version":"1","proposal_id":"proposal-a",
        "source":{"source_id":"source-a","source_revision":"7","source_sha256":"a".repeat(64),"cues":[{"cue_id":"cue-a","text":"원문","start_us":null,"end_us":null}]},
        "generated":{"task":"cards","raw_text":"preserved original"},"draft_revision":"2","state":"editable",
        "draft":{"task":"cards","cards":[{"front":"front","back":"back","source_refs":["cue-a"]}]},"import_error":null},"acceptance":null})).unwrap()
}
fn fixture() -> (PendingReviewController, Rc<RefCell<Probe>>, StoredProposal) {
    fixture_profile(false)
}
fn fixture_profile(raw: bool) -> (PendingReviewController, Rc<RefCell<Probe>>, StoredProposal) {
    let seed = seed();
    let probe = Rc::new(RefCell::new(Probe::default()));
    let program = StateProgram::mount(
        meari::parse(
            "type Props={};export function Scope(props:Props){return <main/>;}",
            Language::Tsx,
        )
        .unwrap(),
        "pending-controller".into(),
        1,
    )
    .unwrap();
    let scope = PluginScope {
        instance: program.snapshot().unwrap().instance,
        source_generation: 1,
    };
    let mut registry = PluginRegistry::new(
        hash("pending-controller"),
        TargetKind::Native,
        scope.clone(),
        8,
    )
    .unwrap();
    let mut bindings = BTreeMap::from([
        binding::<CreateProposal, ProposalDocument>(&probe, "proposals.create"),
        binding::<InspectProposal, StoredProposal>(&probe, "proposals.inspect"),
        binding::<EditProposal, ProposalDocument>(&probe, "proposals.edit"),
        binding::<AcceptProposal, AcceptanceReceipt>(&probe, "proposals.accept"),
        binding::<
            kinekino_player::application::ai_provenance::ImportHistoricalProposal,
            StoredProposal,
        >(&probe, "proposals.import_historical"),
    ]);
    if raw {
        use kinekino_player::application::model_review::*;
        bindings.extend([
            binding::<InspectReviewPackage, ReviewPackageView>(&probe, "reviews.inspect"),
            binding::<ReadReviewTextAsset, ReviewTextAsset>(&probe, "reviews.read_text_asset"),
            binding::<kinekino_player::domain::model_review::RecordReviewDecision, StoredDecision>(
                &probe,
                "reviews.record_decision",
            ),
            binding::<ListReviewDecisions, Vec<StoredDecision>>(&probe, "reviews.decisions"),
        ]);
    }
    let manifest = PluginManifest {
        api_version: PLUGIN_API_VERSION,
        id: "test.pending-proposals".into(),
        version: "1".into(),
        contract: hash("contract"),
        provider: hash("provider"),
        capabilities: bindings
            .iter()
            .map(|(name, binding)| (name.clone(), binding.descriptor().clone()))
            .collect(),
        assets: BTreeMap::new(),
    };
    let grants = manifest.capabilities.keys().cloned().collect();
    let (adapter, pump) = TypedAsyncSdkPlugin::new(
        manifest,
        bindings,
        AsyncCodecBudget {
            max_pending: 8,
            max_terminal_wire_bytes: 1024 * 1024,
            max_total_codec_wire_bytes: 8 * 1024 * 1024 * 4,
        },
    )
    .unwrap();
    let registration = registry.register(Box::new(adapter), grants).unwrap();
    let host = PendingRegistryProposalHost::new(super::super::pending_host::PendingHostInput {
        registry: Rc::new(RefCell::new(registry)),
        registration,
        scope: scope.clone(),
        context: Rc::new(RefCell::new(RequestContext::default())),
        namespace: "pending-controller".into(),
        pump,
        limit: 8,
    })
    .unwrap();
    let session = ReviewSession::new(scope, seed.clone()).unwrap();
    (
        PendingReviewController::new(host, session).unwrap(),
        probe,
        seed,
    )
}
fn reply<T: Serialize>(probe: &Rc<RefCell<Probe>>, command: &str, result: Result<T, ApiError>) {
    probe
        .borrow_mut()
        .replies
        .insert(command.into(), serde_json::to_value(result).unwrap());
}
struct HistoryChurn(PluginManifest);
impl PluginAdapter for HistoryChurn {
    fn manifest(&self) -> &PluginManifest {
        &self.0
    }
    fn validate_payload(
        &self,
        _: &str,
        _: PayloadDirection,
        _: &PluginPayload,
    ) -> Result<(), PluginError> {
        Ok(())
    }
    fn dispatch(&mut self, invocation: &PluginInvocation) -> Result<PluginDispatch, PluginError> {
        Ok(PluginDispatch::Immediate(Ok(PluginPayload {
            schema: descriptor("churn").reply_schema,
            value: invocation.payload.value.clone(),
        })))
    }
    fn cancel(&mut self, _: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        Ok(CancelReceipt::Completed)
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}
#[test]
fn completion_registry_guard_is_distinct_from_provider_busy_and_never_replays_save() {
    let (mut controller, probe, mut seed) = fixture();
    let scope = controller.session.scope().clone();
    let field = controller.session.text_fields().remove(0).input;
    controller
        .session
        .change_text(&scope, &field, "retained".into())
        .unwrap();
    controller.save(&scope).unwrap();
    seed.document.draft_revision = DecimalU64(3);
    reply(&probe, "proposals.edit", Ok(seed.document.clone()));
    let registry = controller.host.registry_handle();
    let guard = registry.borrow_mut();
    assert!(controller.poll().is_err());
    assert!(controller.completion_guard_blocked());
    assert_eq!(probe.borrow().acks, 0);
    drop(guard);
    probe.borrow_mut().fail_ack = true;
    assert!(controller.poll().is_err());
    assert!(!controller.completion_guard_blocked());
    assert!(controller.quiesce_for_parent_close().is_err());
    assert!(
        !controller.close_waiting(),
        "provider Busy is not a guard/terminal wait"
    );
    probe.borrow_mut().fail_ack = false;
    controller.quiesce_for_parent_close().unwrap();
    assert_eq!(probe.borrow().acks, 1);
    assert_eq!(probe.borrow().calls, ["proposals.edit"]);
}
#[test]
fn close_retries_cached_ack_after_core_terminal_history_eviction() {
    let (mut controller, probe, mut seed) = fixture();
    let scope = controller.session.scope().clone();
    let field = controller.session.text_fields().remove(0).input;
    controller
        .session
        .change_text(&scope, &field, "retained edit".into())
        .unwrap();
    controller.save(&scope).unwrap();
    let ticket = controller.active_ticket().unwrap().clone();
    seed.document.draft_revision = DecimalU64(3);
    reply(&probe, "proposals.edit", Ok(seed.document.clone()));
    probe.borrow_mut().fail_ack = true;
    assert!(controller.poll().is_err());
    let registry = controller.host.registry_handle();
    let manifest = PluginManifest {
        api_version: PLUGIN_API_VERSION,
        id: "test.history-churn".into(),
        version: "1".into(),
        contract: hash("churn-contract"),
        provider: hash("churn-provider"),
        capabilities: BTreeMap::from([("churn".into(), descriptor("churn"))]),
        assets: BTreeMap::new(),
    };
    let churn = registry
        .borrow_mut()
        .register(
            Box::new(HistoryChurn(manifest)),
            BTreeSet::from(["churn".into()]),
        )
        .unwrap();
    for _ in 0..17 {
        registry
            .borrow_mut()
            .dispatch(
                &churn,
                "churn",
                PluginPayload {
                    schema: descriptor("churn").request_schema,
                    value: serde_json::json!({}),
                },
            )
            .unwrap();
    }
    assert_eq!(
        registry.borrow_mut().cancel(&ticket).unwrap_err().code,
        PluginErrorCode::Stale
    );
    probe.borrow_mut().fail_ack = false;
    controller.quiesce_for_parent_close().unwrap();
    assert_eq!(probe.borrow().acks, 1);
    assert_eq!(probe.borrow().calls, ["proposals.edit"]);
    let Some(MutationReceipt::Saved(saved)) = controller.session.confirmed_mutation() else {
        panic!("original mutation receipt retained")
    };
    assert_eq!(
        serde_json::to_value(saved).unwrap(),
        serde_json::to_value(&seed.document).unwrap()
    );
    assert!(controller.active_ticket().is_none());
    controller.quiesce_for_parent_close().unwrap();
    controller.close_sdk().unwrap();
    assert_eq!(probe.borrow().acks, 1);
    assert_eq!(probe.borrow().shutdowns, 5);
}
#[test]
fn save_waits_for_ack_then_issues_one_async_inspect_without_repeating_mutation() {
    let (mut controller, probe, mut seed) = fixture();
    let scope = controller.session.scope().clone();
    let super::super::ReviewItem::Card { front, .. } = controller.session.review_items().remove(0)
    else {
        panic!("card")
    };
    controller
        .session
        .change_text(&scope, &front.input, "수정".into())
        .unwrap();
    controller.save(&scope).unwrap();
    assert!(!controller.poll().unwrap());
    let ProposalPayload::Cards { cards } = seed.document.draft.as_mut().unwrap() else {
        unreachable!()
    };
    cards[0].front = "수정".into();
    seed.document.draft_revision = DecimalU64(3);
    probe.borrow_mut().fail_ack = true;
    reply(&probe, "proposals.edit", Ok(seed.document.clone()));
    assert!(controller.poll().is_err());
    assert_eq!(probe.borrow().calls, vec!["proposals.edit"]);
    probe.borrow_mut().fail_ack = false;
    assert!(controller.poll().unwrap());
    assert_eq!(
        probe.borrow().calls,
        vec!["proposals.edit", "proposals.inspect"]
    );
    assert!(controller.session.confirmed_mutation().is_some());
    reply(&probe, "proposals.inspect", Ok(seed));
    assert!(controller.poll().unwrap());
    assert!(!controller.session.dirty());
    assert!(controller.session.pending().is_none());
    assert!(controller.projection_needed());
    controller.projection_committed();
    assert!(!controller.projection_needed());
}
#[test]
fn accept_uses_real_receipt_then_async_inspect_and_never_reissues_accept() {
    let (mut controller, probe, mut seed) = fixture();
    let scope = controller.session.scope().clone();
    controller.accept(&scope).unwrap();
    let receipt = AcceptanceReceipt {
        proposal_id: seed.document.proposal_id.clone(),
        draft_revision: DecimalU64(2),
        source_id: scope.source_id.clone(),
        source_revision: scope.source_revision,
        source_sha256: scope.source_sha256.clone(),
        output_revision: DecimalU64(3),
        output: seed.document.draft.clone().unwrap(),
        automatic_application_enabled: false,
    };
    seed.document.draft_revision = DecimalU64(3);
    seed.acceptance = Some(receipt.clone());
    reply(&probe, "proposals.accept", Ok(receipt));
    controller.poll().unwrap();
    reply::<StoredProposal>(
        &probe,
        "proposals.inspect",
        Err(ApiError::new(ErrorCode::Unsupported, "reader unavailable")),
    );
    controller.poll().unwrap();
    assert!(controller.session.confirmed_mutation().is_some());
    controller.refresh().unwrap();
    reply(&probe, "proposals.inspect", Ok(seed));
    controller.poll().unwrap();
    assert_eq!(
        probe.borrow().calls,
        vec!["proposals.accept", "proposals.inspect", "proposals.inspect"]
    );
}
#[test]
fn canonical_cancelled_is_visible_as_user_cancel_and_close_retries() {
    let (mut controller, probe, _) = fixture();
    let scope = controller.session.scope().clone();
    controller.accept(&scope).unwrap();
    assert!(controller.close_sdk().is_err());
    reply::<AcceptanceReceipt>(
        &probe,
        "proposals.accept",
        Err(ApiError::new(ErrorCode::Cancelled, "queued cancellation")),
    );
    controller.poll().unwrap();
    assert!(controller.last_terminal_was_cancelled());
    assert_eq!(
        controller.session.error().unwrap().code,
        ErrorCode::Cancelled
    );
    controller.close_sdk().unwrap();
    assert!(probe.borrow().shutdowns > 0);
}

#[test]
fn all_five_capabilities_use_explicit_pending_admission_and_canonical_outputs() {
    use kinekino_player::application::ai_provenance::ImportHistoricalProposal;
    let (mut controller, probe, seed) = fixture();
    let receipt = AcceptanceReceipt {
        proposal_id: seed.document.proposal_id.clone(),
        draft_revision: DecimalU64(2),
        source_id: "source-a".into(),
        source_revision: DecimalU64(7),
        source_sha256: "a".repeat(64),
        output_revision: DecimalU64(3),
        output: seed.document.draft.clone().unwrap(),
        automatic_application_enabled: false,
    };
    let requests = [
        PendingRequest::Create(CreateProposal {
            proposal_id: "proposal-a".into(),
            source: seed.document.source.clone(),
            generated: seed.document.generated.clone(),
        }),
        PendingRequest::Inspect(InspectProposal {
            proposal_id: "proposal-a".into(),
        }),
        PendingRequest::Edit {
            revision: DecimalU64(2),
            input: EditProposal {
                proposal_id: "proposal-a".into(),
                draft: seed.document.draft.clone().unwrap(),
            },
        },
        PendingRequest::Accept {
            revision: DecimalU64(2),
            input: AcceptProposal {
                proposal_id: "proposal-a".into(),
                target_source_revision: DecimalU64(7),
            },
        },
        PendingRequest::Import(ImportHistoricalProposal {
            proposal_id: "proposal-a".into(),
            source_id: "source-a".into(),
            source_revision: DecimalU64(7),
            reference: CompanionReference {
                sha256: "c".repeat(64),
                bytes: 42,
                item_id: "original-item".into(),
            },
        }),
    ];
    let mut tickets = Vec::new();
    for request in requests {
        let ProposalAdmission::Pending(ticket) = controller.host.submit(request).unwrap() else {
            panic!("explicit pending")
        };
        tickets.push(ticket);
    }
    reply(&probe, "proposals.create", Ok(seed.document.clone()));
    reply(&probe, "proposals.inspect", Ok(seed.clone()));
    reply(&probe, "proposals.edit", Ok(seed.document.clone()));
    reply(&probe, "proposals.accept", Ok(receipt));
    reply(&probe, "proposals.import_historical", Ok(seed));
    for (index, ticket) in tickets.iter().enumerate() {
        let terminal = controller.host.poll(ticket).unwrap().unwrap();
        assert!(matches!(
            (index, terminal.outcome),
            (0, Ok(PendingOutput::Created(_)))
                | (1, Ok(PendingOutput::Inspected(_)))
                | (2, Ok(PendingOutput::Edited(_)))
                | (3, Ok(PendingOutput::Accepted(_)))
                | (4, Ok(PendingOutput::Imported(_)))
        ));
    }
    assert_eq!(probe.borrow().calls.len(), 5);
    assert_eq!(probe.borrow().acks, 5);
    controller.close_sdk().unwrap();
}

#[test]
fn another_owner_ticket_cannot_complete_the_current_review() {
    let (mut first, _, _) = fixture();
    let first_scope = first.session.scope().clone();
    first.accept(&first_scope).unwrap();
    let (mut second, _, _) = fixture();
    let second_scope = second.session.scope().clone();
    second.accept(&second_scope).unwrap();
    let foreign = second.active_ticket().unwrap().clone();
    assert!(first.host.poll(&foreign).is_err());
    assert!(first.session.pending().is_some());
}

fn document_owner(
    controller: PendingReviewController,
) -> (
    super::super::ProposalDocumentSession,
    Rc<RefCell<super::super::ReviewIntents>>,
    super::super::NodeLease,
    super::super::TextInput,
) {
    use super::super::panel_model::*;
    let registry = controller.host.registry_handle();
    let selected =
        PanelSelection::at(controller.session(), controller.session().scope(), 0).unwrap();
    let prepared = prepare_panel(
        controller.session(),
        controller.session().scope(),
        &selected,
        PanelLabels {
            feedback: crate::presentation::web_ui_adapter::proposals::review_feedback::FeedbackLabels::korean(),
            revision: "문서 2".into(),
            source_identity: "명시 원본".into(),
            quality_unknown: "품질 미판정".into(),
            selected_item: "명시 항목".into(),
            range: String::new(),
            result: String::new(),
        },
        PanelBudget {
            entries: 8,
            text_bytes: 8192,
        },
        super::super::IntentPins {
            contract: hash("contract"),
            provider: hash("provider"),
            request: hash("request"),
            reply: hash("reply"),
        },
    )
    .unwrap();
    let queue = prepared.bindings.queue.clone();
    let node = queue.borrow_mut().bind_node().unwrap();
    let field = controller.session().text_fields().remove(0).input;
    let owner = super::super::ProposalDocumentSession::new_pending_retaining(
        registry,
        controller,
        prepared.bindings,
    )
    .unwrap_or_else(|failure| panic!("{:?}", failure.error));
    (owner, queue, node, field)
}
#[test]
fn document_queue_admits_without_sdk_and_drains_through_shared_interaction_path() {
    let (controller, probe, _) = fixture();
    let (mut owner, queue, node, field) = document_owner(controller);
    queue
        .borrow_mut()
        .enqueue_text(&node, field, "수정 문구".into())
        .unwrap();
    queue
        .borrow_mut()
        .enqueue_action(&node, super::super::ReviewAction::Save)
        .unwrap();
    assert!(probe.borrow().calls.is_empty());
    let report = owner.drain_after_dispatch().unwrap();
    assert_eq!(report.processed, 2);
    assert!(report.transport_error.is_none());
    assert!(owner.operation_pending());
    assert_eq!(probe.borrow().calls, vec!["proposals.edit"]);
    reply::<ProposalDocument>(
        &probe,
        "proposals.edit",
        Err(ApiError::new(ErrorCode::Cancelled, "queued")),
    );
    owner.close().unwrap();
    assert_eq!(probe.borrow().shutdowns, 0);
}
#[test]
fn close_revokes_callbacks_but_keeps_pending_owner_until_terminal_ack() {
    let (controller, probe, _) = fixture();
    let (mut owner, queue, node, _) = document_owner(controller);
    let export = owner.export_callbacks().unwrap();
    let reference = meari::domain::host_callbacks::CallbackReference {
        binding: "onAccept".into(),
        signature: meari::domain::host_callbacks::CallbackSignature::Action,
    };
    queue
        .borrow_mut()
        .enqueue_action(&node, super::super::ReviewAction::Accept)
        .unwrap();
    owner.drain_after_dispatch().unwrap();
    assert!(owner.close().is_err());
    assert!(owner.operation_pending());
    assert!(export.resolve(&reference).is_err());
    reply::<AcceptanceReceipt>(
        &probe,
        "proposals.accept",
        Err(ApiError::new(ErrorCode::Cancelled, "queued")),
    );
    owner.close().unwrap();
    assert!(!owner.operation_pending());
    assert_eq!(probe.borrow().acks, 1);
    assert_eq!(probe.borrow().shutdowns, 0);
}
