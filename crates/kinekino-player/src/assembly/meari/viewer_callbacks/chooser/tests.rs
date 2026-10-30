use super::*;
use crate::{
    application::{
        api::{ApplicationClientHandle, PlayerSdk, RequestContext},
        proposals::list::{ListEvidence, SelectionAuthority},
        sdk_execution::{
            CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner,
        },
    },
    assembly::meari::viewer_callbacks::{ViewerCallbacks, codec},
    domain::{api_values::DecimalU64, proposals::*},
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::{
    AsyncCodecBudget, AsyncSdkPort, AsyncSdkPump, TypedAsyncSdkPlugin, async_binding,
};
use meari::{
    application::{Language, state_program::StateProgram},
    domain::{
        Span,
        host_callbacks::{
            CallbackArguments, CallbackInvocation, CallbackReference, CallbackSignature,
        },
    },
};
use meari_native::infrastructure::native_host::IntentAdmission;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Default)]
struct Probe {
    starts: Vec<(String, Value)>,
    tickets: Vec<PluginTicket>,
    acks: usize,
    cancels: usize,
    fail_ack: bool,
    ready: bool,
    error: Option<ErrorCode>,
    replies: BTreeMap<String, Value>,
}
struct Port {
    probe: Rc<RefCell<Probe>>,
    command: &'static str,
}
impl<I: Serialize> AsyncSdkPort<I, Value> for Port {
    fn start(&mut self, ticket: &PluginTicket, input: I) -> Result<(), PluginError> {
        let mut probe = self.probe.borrow_mut();
        probe
            .starts
            .push((self.command.into(), serde_json::to_value(input).unwrap()));
        probe.tickets.push(ticket.clone());
        Ok(())
    }
    fn peek_terminal(
        &mut self,
        _: &PluginTicket,
    ) -> Result<Option<Arc<Result<Value, ApiError>>>, PluginError> {
        let probe = self.probe.borrow();
        Ok(probe.ready.then(|| {
            Arc::new(match probe.error {
                Some(code) => Err(ApiError::new(code, "Injected SDK failure")),
                None => Ok(probe.replies[self.command].clone()),
            })
        }))
    }
    fn acknowledge(&mut self, ticket: &PluginTicket) -> Result<(), PluginError> {
        let mut probe = self.probe.borrow_mut();
        if probe.fail_ack {
            return Err(busy());
        }
        probe.acks += 1;
        probe.tickets.retain(|retained| retained != ticket);
        Ok(())
    }
    fn cancel(&mut self, _: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        self.probe.borrow_mut().cancels += 1;
        Ok(CancelReceipt::Requested)
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        if self.probe.borrow().tickets.is_empty() {
            Ok(())
        } else {
            Err(busy())
        }
    }
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
    probe: Rc<RefCell<Probe>>,
    bindings: meari::application::host_callbacks::CallbackBindings,
}
impl Fixture {
    fn new() -> Self {
        Self::build(true)
    }
    fn actual() -> Self {
        Self::build(false)
    }
    fn build(injected: bool) -> Self {
        let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
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
        let program = StateProgram::mount(
            meari::parse(
                "type Props={};export function Scope(props:Props){return <main/>;}",
                Language::Tsx,
            )
            .unwrap(),
            "chooser-test".into(),
            1,
        )
        .unwrap();
        let scope = PluginScope {
            instance: program.snapshot().unwrap().instance,
            source_generation: 1,
        };
        let mut owner = ViewerCallbacks::new(execution.handle(), RequestContext::default());
        owner
            .configure_proposal_chooser(ListProposals::default())
            .unwrap();
        owner.export(scope.clone()).unwrap();
        let probe = Rc::new(RefCell::new(Probe {
            ready: true,
            replies: BTreeMap::from([
                (
                    "proposals.list".into(),
                    serde_json::to_value(page()).unwrap(),
                ),
                (
                    "proposals.inspect".into(),
                    serde_json::to_value(document()).unwrap(),
                ),
                ("capabilities".into(), json!({"commands":[]})),
                ("playback.engine".into(), Value::Null),
                ("playback.status".into(), Value::Null),
            ]),
            ..Probe::default()
        }));
        if injected {
            let (plugin, pump) = plugin(probe.clone());
            owner.install_fault_adapter(plugin, pump).unwrap();
        }
        let bindings = owner.bindings().unwrap();
        Self {
            execution,
            owner,
            scope,
            probe,
            bindings,
        }
    }
    fn pending(&self, prop: &str, arguments: CallbackArguments) -> PendingCallback {
        let signature = match &arguments {
            CallbackArguments::Text(_) => CallbackSignature::Text,
            _ => CallbackSignature::Action,
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
        let pending = self.pending(prop, arguments);
        self.owner.queue_chooser(&self.scope, &pending).unwrap();
        self.owner.drain_chooser(&self.scope).unwrap();
        self.owner.consume_reads().unwrap();
    }
    fn list(&mut self) {
        self.act("onChooserList", CallbackArguments::Action);
    }
    fn inspect(&mut self) {
        let key = self.owner.chooser_documents()[0].key.clone();
        self.act("onChooserDocument", CallbackArguments::Text(key));
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.probe.borrow_mut().fail_ack = false;
        self.probe.borrow_mut().ready = true;
        self.owner.stop_admission();
        for step in 1..=4 {
            let _ = self
                .owner
                .consume_reads_at(Instant::now() + Duration::from_secs(step));
            if self.owner.prepare_route_release(&self.scope).ok() == Some(None) {
                break;
            }
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        let closed = loop {
            if let Ok(ExecutionCloseProgress::Closed(receipt)) = self.execution.poll_close() {
                break Some(receipt);
            }
            if Instant::now() >= deadline {
                break None;
            }
            std::thread::yield_now();
        };
        if !std::thread::panicking() {
            let receipt = closed.expect("original fixture worker must join");
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            assert!(self.owner.is_released());
        }
    }
}
fn plugin(probe: Rc<RefCell<Probe>>) -> (TypedAsyncSdkPlugin, AsyncSdkPump) {
    let manifest = manifest_for(ReadProfile::ProposalChooser);
    let mut bindings = BTreeMap::new();
    macro_rules! bind {
        ($command:literal, $input:ty) => {
            bindings.insert(
                $command.into(),
                async_binding::<$input, Value, _>(
                    manifest.capabilities[$command].clone(),
                    Port {
                        probe: probe.clone(),
                        command: $command,
                    },
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
fn document() -> StoredProposal {
    StoredProposal {
        document: ProposalDocument {
            schema_version: "1".into(),
            proposal_id: "proposal-1".into(),
            source: SourceSnapshot::Cue(CueSourceSnapshot {
                source_id: "source-1".into(),
                source_revision: DecimalU64(1),
                source_sha256: "a".repeat(64),
                cues: vec![SourceCue {
                    cue_id: "cue-1".into(),
                    text: "Original".into(),
                    start_us: None,
                    end_us: None,
                }],
            }),
            generated: GeneratedResult {
                task: ProposalTask::Cards,
                raw_text: "{}".into(),
            },
            draft_revision: DecimalU64(1),
            state: DraftState::Editable,
            draft: Some(ProposalPayload::Cards {
                cards: vec![Card {
                    front: "Front".into(),
                    back: "Back".into(),
                    source_refs: vec!["cue-1".into()],
                }],
            }),
            import_error: None,
        },
        acceptance: None,
        historical: None,
    }
}
fn page() -> ProposalListPage {
    ProposalListPage {
        schema_version: "proposal-list/1".into(),
        rows: vec![ProposalListRow {
            proposal_id: "proposal-1".into(),
            task: ProposalTask::Cards,
            draft_revision: DecimalU64(1),
            draft_state: DraftState::Editable,
            source_id: "source-1".into(),
            source_revision: DecimalU64(1),
            source_sha256: "a".repeat(64),
            typed_item_count: 1,
        }],
        next_cursor: None,
        dataset_witness: "b".repeat(64),
        snapshot_witness: "c".repeat(64),
        current_evidence: ListEvidence::NotChecked,
        selection_authority: SelectionAuthority::None,
    }
}

#[test]
fn chooser_profile_preserves_reads3_and_adds_only_typed_reads() {
    let default = codec::manifest();
    assert_eq!(default, manifest_for(ReadProfile::Default));
    let union = manifest_for(ReadProfile::ProposalChooser);
    assert_eq!(default.capabilities.len(), 3);
    assert_eq!(union.capabilities.len(), 5);
    for (_, command) in codec::READS {
        assert_eq!(default.capabilities[command], union.capabilities[command]);
    }
    assert_ne!(default.contract, union.contract);
    assert!(
        union
            .capabilities
            .keys()
            .all(|key| !key.contains("edit") && !key.contains("accept"))
    );
}
#[test]
fn explicit_document_then_item_retains_full_result_and_exact_target() {
    let mut fixture = Fixture::new();
    fixture.list();
    assert!(fixture.owner.chooser_items().is_empty());
    assert!(fixture.owner.chooser_prepared().is_none());
    fixture.inspect();
    assert!(fixture.owner.chooser_document().is_some());
    assert!(fixture.owner.chooser_prepared().is_none());
    let key = fixture.owner.chooser_items()[0].key.clone();
    fixture.act("onChooserItem", CallbackArguments::Text(key));
    let prepared = fixture.owner.chooser_prepared().unwrap();
    assert_eq!(
        prepared.selection.target,
        NativeProposalTarget::Card { index: 0 }
    );
    assert_eq!(prepared.selection.source_sha256, "a".repeat(64));
    assert_eq!(fixture.probe.borrow().starts.len(), 2);
    assert_eq!(fixture.probe.borrow().acks, 2);
    assert!(fixture.owner.tickets().unwrap().is_empty());
}
#[test]
fn old_and_renderer_fabricated_keys_cannot_select_after_refresh() {
    let mut fixture = Fixture::new();
    fixture.list();
    let old = fixture.owner.chooser_documents()[0].key.clone();
    fixture.inspect();
    let old_item = fixture.owner.chooser_items()[0].key.clone();
    fixture.list();
    assert_ne!(old, fixture.owner.chooser_documents()[0].key);
    for (prop, key) in [
        ("onChooserDocument", old),
        ("onChooserItem", old_item),
        ("onChooserDocument", "proposal-1".into()),
    ] {
        let pending = fixture.pending(prop, CallbackArguments::Text(key));
        assert!(
            fixture
                .owner
                .queue_chooser(&fixture.scope, &pending)
                .is_err()
        );
    }
    assert!(fixture.owner.chooser_prepared().is_none());
    assert_eq!(fixture.probe.borrow().starts.len(), 3);
}
#[test]
fn original_ack_failure_retries_without_resubmit_or_publishing_keys() {
    let mut fixture = Fixture::new();
    fixture.probe.borrow_mut().fail_ack = true;
    fixture.list();
    let ticket = fixture.owner.tickets().unwrap()[0].clone();
    assert!(fixture.owner.chooser_documents().is_empty());
    assert!(fixture.owner.chooser_terminal().unwrap().owner_ack.is_err());
    let now = Instant::now();
    fixture
        .owner
        .consume_reads_at(now + Duration::from_secs(1))
        .unwrap();
    fixture
        .owner
        .consume_reads_at(now + Duration::from_secs(2))
        .unwrap();
    fixture.owner.retry_read_acceptance(&ticket).unwrap();
    fixture.probe.borrow_mut().fail_ack = false;
    fixture
        .owner
        .consume_reads_at(now + Duration::from_secs(3))
        .unwrap();
    assert_eq!(fixture.probe.borrow().starts.len(), 1);
    assert_eq!(fixture.owner.chooser_documents().len(), 1);
    assert_eq!(
        fixture.owner.chooser_terminal().unwrap().event.ticket,
        ticket
    );
}
#[test]
fn metadata_mismatch_is_stale_and_never_becomes_an_item_choice() {
    let mut fixture = Fixture::new();
    fixture.list();
    let mut changed = document();
    changed.document.draft_revision = DecimalU64(2);
    fixture.probe.borrow_mut().replies.insert(
        "proposals.inspect".into(),
        serde_json::to_value(changed).unwrap(),
    );
    fixture.inspect();
    assert_eq!(
        fixture.owner.chooser_failure().unwrap().code,
        PluginErrorCode::Stale
    );
    assert!(fixture.owner.chooser_items().is_empty());
    assert!(fixture.owner.chooser_prepared().is_none());
    assert!(fixture.owner.tickets().unwrap().is_empty());
}
#[test]
fn pending_intents_coalesce_and_other_intents_are_busy() {
    let mut fixture = Fixture::new();
    fixture.probe.borrow_mut().ready = false;
    let pending = fixture.pending("onChooserList", CallbackArguments::Action);
    fixture
        .owner
        .queue_chooser(&fixture.scope, &pending)
        .unwrap();
    fixture
        .owner
        .queue_chooser(&fixture.scope, &pending)
        .unwrap();
    fixture.owner.drain_chooser(&fixture.scope).unwrap();
    fixture
        .owner
        .queue_chooser(&fixture.scope, &pending)
        .unwrap();
    let next = fixture.pending("onChooserNext", CallbackArguments::Action);
    assert!(fixture.owner.queue_chooser(&fixture.scope, &next).is_err());
    assert_eq!(fixture.probe.borrow().starts.len(), 1);
}
#[test]
fn close_waits_for_mixed_reads_and_chooser_and_reexport_retires_keys() {
    let mut fixture = Fixture::new();
    fixture.list();
    let old = fixture.owner.chooser_documents()[0].key.clone();
    let old_pending = fixture.pending("onChooserDocument", CallbackArguments::Text(old));
    fixture.probe.borrow_mut().ready = false;
    fixture.act("onChooserList", CallbackArguments::Action);
    let capabilities = fixture.pending("onReadCapabilities", CallbackArguments::Action);
    fixture
        .owner
        .dispatch(&fixture.scope, &capabilities)
        .unwrap();
    assert_eq!(
        fixture
            .owner
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .unwrap()
            .len(),
        2
    );
    fixture.probe.borrow_mut().ready = true;
    fixture.owner.consume_reads().unwrap();
    assert!(fixture.owner.chooser_documents().is_empty());
    assert!(
        fixture
            .owner
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    let attachment = fixture.execution.handle().attach().unwrap();
    attachment.release().unwrap();
    fixture.bindings = fixture.owner.resume_released_route(&fixture.scope).unwrap();
    assert_eq!(fixture.owner.chooser_phase(), Some(ChooserPhase::Idle));
    assert!(
        fixture
            .owner
            .queue_chooser(&fixture.scope, &old_pending)
            .is_err()
    );
    assert!(
        fixture
            .owner
            .configure_proposal_chooser(ListProposals::default())
            .is_err()
    );
    assert_eq!(fixture.probe.borrow().starts.len(), 3);
}
#[test]
fn pagination_uses_host_cursor_and_retires_previous_page_keys() {
    let mut fixture = Fixture::new();
    let mut first = page();
    first.next_cursor = Some("host-cursor".into());
    fixture.probe.borrow_mut().replies.insert(
        "proposals.list".into(),
        serde_json::to_value(first).unwrap(),
    );
    fixture.list();
    let old = fixture.owner.chooser_documents()[0].key.clone();
    let mut last = page();
    last.rows[0].proposal_id = "proposal-2".into();
    fixture
        .probe
        .borrow_mut()
        .replies
        .insert("proposals.list".into(), serde_json::to_value(last).unwrap());
    fixture.act("onChooserNext", CallbackArguments::Action);
    assert_eq!(fixture.probe.borrow().starts[1].1["cursor"], "host-cursor");
    let pending = fixture.pending("onChooserDocument", CallbackArguments::Text(old));
    assert!(
        fixture
            .owner
            .queue_chooser(&fixture.scope, &pending)
            .is_err()
    );
    let next = fixture.pending("onChooserNext", CallbackArguments::Action);
    assert!(fixture.owner.queue_chooser(&fixture.scope, &next).is_err());
}
#[test]
fn sdk_errors_and_malformed_pages_do_not_publish_fallback_rows() {
    for code in [
        ErrorCode::SourceExpired,
        ErrorCode::RevisionConflict,
        ErrorCode::ResourceExhausted,
        ErrorCode::Unsupported,
    ] {
        let mut fixture = Fixture::new();
        fixture.list();
        fixture.probe.borrow_mut().error = Some(code);
        fixture.list();
        assert!(fixture.owner.chooser_documents().is_empty());
        assert!(fixture.owner.chooser_prepared().is_none());
        let details = fixture
            .owner
            .chooser_failure()
            .unwrap()
            .details
            .as_ref()
            .unwrap();
        assert_eq!(details.value["code"], serde_json::to_value(code).unwrap());
        assert!(fixture.owner.tickets().unwrap().is_empty());
    }
    let mut fixture = Fixture::new();
    fixture
        .probe
        .borrow_mut()
        .replies
        .insert("proposals.list".into(), json!({"rows":[]}));
    fixture.list();
    assert_eq!(
        fixture.owner.chooser_failure().unwrap().code,
        PluginErrorCode::InvalidPayload
    );
    assert!(fixture.owner.chooser_documents().is_empty());
    assert!(fixture.owner.tickets().unwrap().is_empty());
}
#[test]
fn foreign_scope_and_revoked_queued_callback_do_no_sdk_work() {
    let mut fixture = Fixture::new();
    let pending = fixture.pending("onChooserList", CallbackArguments::Action);
    let mut foreign = fixture.scope.clone();
    foreign.source_generation += 1;
    assert!(fixture.owner.queue_chooser(&foreign, &pending).is_err());
    fixture
        .owner
        .queue_chooser(&fixture.scope, &pending)
        .unwrap();
    fixture.bindings.invalidate();
    assert!(fixture.owner.drain_chooser(&fixture.scope).is_err());
    assert!(fixture.probe.borrow().starts.is_empty());
    fixture.bindings = fixture.owner.bindings().unwrap();
    fixture.list();
    assert_eq!(fixture.probe.borrow().starts.len(), 1);
}
#[test]
fn generation_exhaustion_and_repair_preserve_explicit_unselected_entry() {
    let mut chooser = Chooser::new(ListProposals::default()).unwrap();
    chooser.generation = u64::MAX;
    assert!(chooser.advance().is_err());
    chooser.counter = u64::MAX;
    assert!(chooser.key("item").is_err());
    let mut repair = document();
    repair.document.state = DraftState::NeedsRepair;
    repair.document.draft = None;
    repair.validate().unwrap();
    let before = serde_json::to_vec(&repair).unwrap();
    let selected = targets(&repair).unwrap();
    assert_eq!(
        selected,
        [NativeProposalTarget::ManualRepair {
            task: repair.document.generated.task
        }]
    );
    assert_eq!(selected[0].checked_item_index().unwrap(), None);
    assert_eq!(serde_json::to_vec(&repair).unwrap(), before);
    assert!(
        Chooser::new(ListProposals {
            cursor: Some("external".into()),
            ..ListProposals::default()
        })
        .is_err()
    );
}
#[test]
fn cancel_retains_original_ticket_and_discards_its_late_success() {
    let mut fixture = Fixture::new();
    fixture.probe.borrow_mut().ready = false;
    fixture.list();
    let original = fixture.owner.tickets().unwrap()[0].clone();
    assert_eq!(
        fixture.owner.cancel_chooser(&fixture.scope).unwrap(),
        Some(CancelReceipt::Requested)
    );
    assert_eq!(fixture.owner.tickets().unwrap(), vec![original]);
    assert_eq!(fixture.probe.borrow().cancels, 1);
    fixture.probe.borrow_mut().ready = true;
    fixture.owner.consume_reads().unwrap();
    assert!(fixture.owner.tickets().unwrap().is_empty());
    assert!(fixture.owner.chooser_documents().is_empty());
    assert_eq!(fixture.owner.chooser_phase(), Some(ChooserPhase::Idle));
    assert_eq!(fixture.probe.borrow().starts.len(), 1);
}
#[test]
fn typed_query_change_retires_keys_and_does_not_submit_implicitly() {
    let mut fixture = Fixture::new();
    fixture.list();
    let old = fixture.owner.chooser_documents()[0].key.clone();
    fixture
        .owner
        .set_chooser_query(
            &fixture.scope,
            ListProposals {
                task: Some(ProposalTask::Cards),
                limit: Some(1),
                cursor: None,
            },
        )
        .unwrap();
    let pending = fixture.pending("onChooserDocument", CallbackArguments::Text(old));
    assert!(
        fixture
            .owner
            .queue_chooser(&fixture.scope, &pending)
            .is_err()
    );
    assert!(fixture.owner.chooser_page().is_none());
    assert_eq!(fixture.probe.borrow().starts.len(), 1);
    fixture.list();
    assert_eq!(
        fixture.probe.borrow().starts[1].1,
        json!({"task":"cards","limit":1})
    );
}
#[test]
fn actual_sdk_owner_keeps_one_attachment_until_terminal_ack_and_release() {
    let mut fixture = Fixture::actual();
    assert!(fixture.execution.handle().attach().is_err());
    let input = fixture.pending("onChooserList", CallbackArguments::Action);
    fixture.owner.queue_chooser(&fixture.scope, &input).unwrap();
    fixture.owner.drain_chooser(&fixture.scope).unwrap();
    assert_eq!(fixture.owner.tickets().unwrap().len(), 1);
    assert!(fixture.execution.handle().attach().is_err());
    assert!(
        fixture
            .owner
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_some()
    );
    let deadline = Instant::now() + Duration::from_secs(2);
    while !fixture.owner.tickets().unwrap().is_empty() {
        fixture.owner.consume_reads().unwrap();
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert!(fixture.owner.chooser_documents().is_empty());
    assert!(
        fixture
            .owner
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    let attachment = fixture.execution.handle().attach().unwrap();
    attachment.release().unwrap();
}
#[test]
fn partial_export_failure_keeps_original_attachment_for_cleanup() {
    let mut fixture = Fixture::actual();
    assert!(
        fixture
            .owner
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    fixture.scope.source_generation = 0;
    let mut partial = ViewerCallbacks::new(fixture.execution.handle(), RequestContext::default());
    partial
        .configure_proposal_chooser(ListProposals::default())
        .unwrap();
    assert_eq!(
        partial.export(fixture.scope.clone()).err().unwrap().code,
        PluginErrorCode::InvalidManifest
    );
    assert!(fixture.execution.handle().attach().is_err());
    fixture.owner = partial;
    assert!(
        fixture
            .owner
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    let attachment = fixture.execution.handle().attach().unwrap();
    attachment.release().unwrap();
}
#[test]
fn prepared_carrier_is_retired_by_new_intent_and_cannot_be_reconstructed() {
    let mut fixture = Fixture::new();
    fixture.list();
    fixture.inspect();
    let key = fixture.owner.chooser_items()[0].key.clone();
    fixture.act("onChooserItem", CallbackArguments::Text(key));
    let original = fixture.owner.chooser_prepared().unwrap().clone();
    assert!(fixture.owner.chooser_matches_prepared(&original));
    let fabricated = PreparedSelection {
        scope: original.scope.clone(),
        generation: original.generation,
        selection: Arc::new((*original.selection).clone()),
    };
    assert!(!fixture.owner.chooser_matches_prepared(&fabricated));
    let pending = fixture.pending("onChooserList", CallbackArguments::Action);
    fixture
        .owner
        .queue_chooser(&fixture.scope, &pending)
        .unwrap();
    assert!(!fixture.owner.chooser_matches_prepared(&original));
    assert!(fixture.owner.chooser_prepared().is_none());
}
#[test]
fn typed_targets_preserve_cue_and_document_reference_membership() {
    let mut cue = document();
    cue.document.generated.task = ProposalTask::Correction;
    cue.document.draft = Some(ProposalPayload::Correction {
        edits: vec![Correction {
            cue_id: "cue-1".into(),
            text: "Corrected".into(),
        }],
    });
    cue.validate().unwrap();
    assert_eq!(
        targets(&cue).unwrap(),
        vec![NativeProposalTarget::CueCorrection {
            index: 0,
            cue_id: "cue-1".into()
        }]
    );
    use sha2::{Digest, Sha256};
    let asset = serde_json::to_string(&json!({"messages":[{"role":"user","content":concat!("Edit this ASR draft. Correct punctuation only. Do not guess unclear words. JSON only: {\"proposed\":string,\"uncertain_words\":[string]}. Under100words. /no_think\n", r#"["Actual draft."]"#)}]})).unwrap();
    let source = DocumentSourceSnapshot::from_asset(
        "document-1".into(),
        DecimalU64(1),
        format!("{:x}", Sha256::digest(asset.as_bytes())),
        asset,
    )
    .unwrap();
    let reference = source.text_reference("/0").unwrap();
    cue.document.schema_version = "2".into();
    cue.document.source = SourceSnapshot::Document(source);
    cue.document.draft = Some(ProposalPayload::DocumentCorrection {
        edits: vec![DocumentCorrection {
            source_ref: reference.clone(),
            text: "Corrected.".into(),
        }],
    });
    cue.validate().unwrap();
    assert_eq!(
        targets(&cue).unwrap(),
        vec![NativeProposalTarget::DocumentCorrection {
            index: 0,
            source_ref: reference
        }]
    );
    let mut highlight = document();
    highlight.document.generated.task = ProposalTask::Highlights;
    highlight.document.draft = Some(ProposalPayload::Highlights {
        highlights: vec![Highlight {
            title: "Title".into(),
            source_refs: vec!["cue-1".into()],
        }],
    });
    highlight.validate().unwrap();
    assert_eq!(
        targets(&highlight).unwrap(),
        vec![NativeProposalTarget::Highlight { index: 0 }]
    );
}

fn prepared_open(fixture: &mut Fixture) -> (String, PreparedSelection) {
    fixture.list();
    fixture.inspect();
    let item = fixture.owner.chooser_items()[0].key.clone();
    fixture.act("onChooserItem", CallbackArguments::Text(item));
    let prepared = fixture.owner.chooser_prepared().unwrap().clone();
    let key = fixture
        .owner
        .register_prepared_open(&fixture.scope)
        .unwrap();
    (key, prepared)
}
fn open_pending(fixture: &Fixture, key: &str) -> PendingCallback {
    fixture.pending("onOpenLiveProposal", CallbackArguments::Text(key.into()))
}
#[test]
fn prepared_open_registration_requires_ack_and_keeps_one_current_row() {
    let mut fixture = Fixture::new();
    assert!(
        fixture
            .owner
            .register_prepared_open(&fixture.scope)
            .is_err()
    );
    fixture.list();
    fixture.probe.borrow_mut().fail_ack = true;
    fixture.inspect();
    assert!(
        fixture
            .owner
            .register_prepared_open(&fixture.scope)
            .is_err()
    );
    fixture.probe.borrow_mut().fail_ack = false;
    fixture
        .owner
        .consume_reads_at(Instant::now() + Duration::from_secs(1))
        .unwrap();
    let item = fixture.owner.chooser_items()[0].key.clone();
    fixture.act("onChooserItem", CallbackArguments::Text(item));
    let key = fixture
        .owner
        .register_prepared_open(&fixture.scope)
        .unwrap();
    assert_ne!(key, "proposal-1");
    assert_eq!(
        key,
        fixture
            .owner
            .register_prepared_open(&fixture.scope)
            .unwrap()
    );
    assert_eq!(fixture.probe.borrow().starts.len(), 2);
}
#[test]
fn prepared_open_admission_checks_scope_key_and_original_pending_together() {
    let mut fixture = Fixture::new();
    let (key, prepared) = prepared_open(&mut fixture);
    let pending = open_pending(&fixture, &key);
    let mut foreign = fixture.scope.clone();
    foreign.source_generation += 1;
    assert!(matches!(
        fixture.owner.queue_live_entry(&foreign, &pending).unwrap(),
        IntentAdmission::Rejected(_)
    ));
    let forged = open_pending(&fixture, "proposal-1");
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &forged)
            .unwrap(),
        IntentAdmission::Rejected(_)
    ));
    let wrong_key = fixture.pending("onChooserDocument", CallbackArguments::Text(key.clone()));
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &wrong_key)
            .unwrap(),
        IntentAdmission::Rejected(_)
    ));
    assert!(!fixture.owner.has_live_entry());
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &pending)
            .unwrap(),
        IntentAdmission::Accepted
    ));
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &pending)
            .unwrap(),
        IntentAdmission::Accepted
    ));
    let (request, retained) = fixture
        .owner
        .live_entry_handoff(&fixture.scope)
        .unwrap()
        .unwrap();
    assert!(Arc::ptr_eq(&request.selected, &prepared.selection));
    assert!(Arc::ptr_eq(
        &retained.unwrap().selection,
        &prepared.selection
    ));
    assert_eq!(fixture.probe.borrow().starts.len(), 2);
}
#[test]
fn prepared_open_handoff_error_preserves_intent_and_blocks_competing_inputs() {
    let mut fixture = Fixture::new();
    let (key, prepared) = prepared_open(&mut fixture);
    let pending = open_pending(&fixture, &key);
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &pending)
            .unwrap(),
        IntentAdmission::Accepted
    ));
    let failed: Result<(), &'static str> = fixture
        .owner
        .commit_live_entry_handoff(&fixture.scope, |selected, original, carrier| {
            assert!(Arc::ptr_eq(&selected, &prepared.selection));
            assert_eq!(original.invocation.prop, "onOpenLiveProposal");
            assert!(Arc::ptr_eq(
                &carrier.unwrap().selection,
                &prepared.selection
            ));
            Err("route guard busy")
        })
        .unwrap();
    assert_eq!(failed, Err("route guard busy"));
    assert!(fixture.owner.has_live_entry());
    assert!(
        fixture
            .owner
            .live_entry_handoff(&fixture.scope)
            .unwrap()
            .is_some()
    );
    assert!(
        fixture
            .owner
            .register_prepared_open(&fixture.scope)
            .is_err()
    );
    assert!(
        fixture
            .owner
            .set_chooser_query(&fixture.scope, ListProposals::default())
            .is_err()
    );
    let refresh = fixture.pending("onChooserList", CallbackArguments::Action);
    assert!(
        fixture
            .owner
            .queue_chooser(&fixture.scope, &refresh)
            .is_err()
    );
    assert_eq!(fixture.probe.borrow().starts.len(), 2);
}
#[test]
fn prepared_open_success_consumes_only_after_recipient_stores_original_carrier() {
    let mut fixture = Fixture::new();
    let (key, prepared) = prepared_open(&mut fixture);
    let pending = open_pending(&fixture, &key);
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &pending)
            .unwrap(),
        IntentAdmission::Accepted
    ));
    let mut recipient = None;
    let committed: Result<(), ()> = fixture
        .owner
        .commit_live_entry_handoff(&fixture.scope, |selected, pending, carrier| {
            recipient = Some((selected, pending, carrier.unwrap()));
            Ok(())
        })
        .unwrap();
    assert_eq!(committed, Ok(()));
    let (selected, original, carrier) = recipient.unwrap();
    assert!(Arc::ptr_eq(&selected, &prepared.selection));
    assert!(Arc::ptr_eq(&carrier.selection, &prepared.selection));
    assert_eq!(original.invocation.prop, "onOpenLiveProposal");
    assert!(!fixture.owner.has_live_entry());
    assert_eq!(fixture.owner.chooser_phase(), Some(ChooserPhase::Closing));
    assert!(fixture.owner.chooser_prepared().is_none());
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &pending)
            .unwrap(),
        IntentAdmission::Rejected(_)
    ));
}
#[test]
fn revoked_open_lease_cannot_be_revived_by_a_new_export_or_enter_commit() {
    let mut fixture = Fixture::new();
    let (key, _) = prepared_open(&mut fixture);
    let pending = open_pending(&fixture, &key);
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &pending)
            .unwrap(),
        IntentAdmission::Accepted
    ));
    fixture.bindings.invalidate();
    let mut entered = false;
    assert!(
        fixture
            .owner
            .commit_live_entry_handoff(&fixture.scope, |_, _, _| {
                entered = true;
                Ok::<(), ()>(())
            })
            .is_err()
    );
    assert!(!entered);
    assert!(fixture.owner.has_live_entry());
    fixture.bindings = fixture.owner.bindings().unwrap();
    let fresh = open_pending(&fixture, &key);
    assert!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &fresh)
            .is_err()
    );
}
#[test]
fn query_cancel_and_reexport_retire_prepared_open_keys() {
    let mut fixture = Fixture::new();
    let (old, _) = prepared_open(&mut fixture);
    let pending = open_pending(&fixture, &old);
    fixture
        .owner
        .set_chooser_query(&fixture.scope, ListProposals::default())
        .unwrap();
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &pending)
            .unwrap(),
        IntentAdmission::Rejected(_)
    ));
    let (next, _) = prepared_open(&mut fixture);
    assert_ne!(old, next);
    let pending = open_pending(&fixture, &next);
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &pending)
            .unwrap(),
        IntentAdmission::Accepted
    ));
    fixture.owner.cancel_chooser(&fixture.scope).unwrap();
    assert!(!fixture.owner.has_live_entry());
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &pending)
            .unwrap(),
        IntentAdmission::Rejected(_)
    ));
    fixture.owner.prepare_route_release(&fixture.scope).unwrap();
    fixture.bindings = fixture.owner.resume_released_route(&fixture.scope).unwrap();
    assert!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &pending)
            .is_err()
    );
    let replay = open_pending(&fixture, &next);
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &replay)
            .unwrap(),
        IntentAdmission::Rejected(_)
    ));
}
#[test]
fn prepared_open_commit_keeps_other_reads_in_original_drain() {
    let mut fixture = Fixture::new();
    let (key, _) = prepared_open(&mut fixture);
    fixture.probe.borrow_mut().ready = false;
    let read = fixture.pending("onReadCapabilities", CallbackArguments::Action);
    let ticket = fixture
        .owner
        .dispatch(&fixture.scope, &read)
        .unwrap()
        .ticket;
    let pending = open_pending(&fixture, &key);
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &pending)
            .unwrap(),
        IntentAdmission::Accepted
    ));
    fixture
        .owner
        .commit_live_entry_handoff(&fixture.scope, |_, _, _| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(
        fixture.owner.prepare_route_release(&fixture.scope).unwrap(),
        Some(vec![ticket])
    );
    assert!(!fixture.owner.is_released());
    fixture.probe.borrow_mut().ready = true;
    fixture.owner.consume_reads().unwrap();
    assert!(
        fixture
            .owner
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
}
#[test]
fn partial_open_cannot_flow_through_legacy_route_request_without_carrier() {
    let mut fixture = Fixture::new();
    let (key, _) = prepared_open(&mut fixture);
    let pending = open_pending(&fixture, &key);
    assert!(matches!(
        fixture
            .owner
            .queue_live_entry(&fixture.scope, &pending)
            .unwrap(),
        IntentAdmission::Accepted
    ));
    assert_eq!(
        fixture
            .owner
            .live_entry_request(&fixture.scope)
            .err()
            .unwrap()
            .code,
        PluginErrorCode::MissingCapability
    );
    assert!(fixture.owner.has_live_entry());
    assert!(
        fixture
            .owner
            .live_entry_handoff(&fixture.scope)
            .unwrap()
            .unwrap()
            .1
            .is_some()
    );
}
#[test]
fn prepared_row_rejects_foreign_generation_scope_and_reconstructed_arc() {
    use crate::assembly::meari::viewer_callbacks::selection::LiveSelection;
    let mut fixture = Fixture::new();
    let (_, current) = prepared_open(&mut fixture);
    for variant in 0..3 {
        let mut fabricated = current.clone();
        match variant {
            0 => fabricated.generation += 1,
            1 => fabricated.scope.source_generation += 1,
            _ => fabricated.selection = Arc::new((*current.selection).clone()),
        }
        let mut choices = LiveSelection::for_chooser();
        let mut callbacks =
            meari::application::host_callbacks::HostCallbacks::new(fixture.scope.clone());
        choices.bind(&mut callbacks).unwrap();
        let key = choices.register_prepared(fabricated).unwrap();
        let bindings = callbacks.export_bindings().unwrap();
        let pending = PendingCallback {
            handle: bindings
                .resolve(&CallbackReference {
                    binding: "onOpenLiveProposal".into(),
                    signature: CallbackSignature::Text,
                })
                .unwrap(),
            invocation: CallbackInvocation {
                prop: "onOpenLiveProposal".into(),
                arguments: CallbackArguments::Text(key),
                span: Span::default(),
            },
        };
        let invocation = callbacks
            .dispatch_local(
                &pending.handle,
                pending.invocation.arguments.clone(),
                &fixture.scope,
            )
            .unwrap();
        assert!(matches!(
            choices.admit(&invocation, &pending, Some(&current)),
            IntentAdmission::Rejected(_)
        ));
        assert!(choices.request().is_none());
    }
}
#[test]
fn original_preconfigured_live_row_does_not_require_chooser_carrier() {
    use crate::assembly::meari::{
        viewer_callbacks::selection::LiveSelection, viewer_host::TrustedLiveChoices,
    };
    use std::num::{NonZeroU64, NonZeroUsize};
    let mut fixture = Fixture::new();
    let (_, current) = prepared_open(&mut fixture);
    let choices = TrustedLiveChoices::from_trusted_local(
        vec![(*current.selection).clone()],
        NonZeroU64::new(1).unwrap(),
        NonZeroUsize::new(1).unwrap(),
    )
    .unwrap();
    let mut choices = LiveSelection::new(choices);
    let mut callbacks =
        meari::application::host_callbacks::HostCallbacks::new(fixture.scope.clone());
    choices.bind(&mut callbacks).unwrap();
    let bindings = callbacks.export_bindings().unwrap();
    let pending = PendingCallback {
        handle: bindings
            .resolve(&CallbackReference {
                binding: "onOpenLiveProposal".into(),
                signature: CallbackSignature::Text,
            })
            .unwrap(),
        invocation: CallbackInvocation {
            prop: "onOpenLiveProposal".into(),
            arguments: CallbackArguments::Text("live-proposal-1-0".into()),
            span: Span::default(),
        },
    };
    let invocation = callbacks
        .dispatch_local(
            &pending.handle,
            pending.invocation.arguments.clone(),
            &fixture.scope,
        )
        .unwrap();
    assert!(matches!(
        choices.admit(&invocation, &pending, None),
        IntentAdmission::Accepted
    ));
    assert!(
        choices
            .validate_pending(&invocation, None)
            .unwrap()
            .is_none()
    );
}
