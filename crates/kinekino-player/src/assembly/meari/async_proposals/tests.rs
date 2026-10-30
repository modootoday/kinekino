use super::*;
mod policy;
use crate::{
    application::{
        api::{ApplicationClient, ApplicationClientHandle, PlayerSdk, Request, Response},
        proposals::InspectProposal,
        sdk_execution::{
            CompletionNotifier, ExecutionBudget, ExecutionOwner, ExecutionSnapshot,
            InitialProposalInspection,
        },
    },
    infrastructure::{proposals::ProposalFiles, subtitle_files::SubtitleFiles},
};
use meari::{
    application::{Language, state_program::StateProgram},
    domain::build::{ContentHash, TargetKind},
};
use serde_json::json;
use std::{
    cell::RefCell,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn register_review_probe(
    owner: &ExecutionOwner,
    registry: &Rc<RefCell<PluginRegistry>>,
    context: Rc<RefCell<RequestContext>>,
    grants: std::collections::BTreeSet<String>,
) -> (RegistrationHandle, AsyncSdkPump) {
    let (plugin, pump) = output_review_adapter(owner.handle(), codec(), move |ticket| {
        crate::assembly::meari::request_context::scoped_context(&context.borrow(), ticket)
    })
    .unwrap();
    let registration = registry
        .borrow_mut()
        .register(Box::new(plugin), grants)
        .unwrap();
    (registration, pump)
}
fn drain_review_probe(
    pump: &AsyncSdkPump,
    registry: &Rc<RefCell<PluginRegistry>>,
    ticket: &PluginTicket,
) -> PluginEvent {
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(accepted) = pump.accept(&mut registry.borrow_mut(), ticket).unwrap() {
            accepted.owner_ack.unwrap();
            return accepted.event;
        }
        assert!(Instant::now() < until);
        thread::sleep(Duration::from_millis(1));
    }
}
fn invoke_review_probe(
    pump: &AsyncSdkPump,
    registry: &Rc<RefCell<PluginRegistry>>,
    registration: &RegistrationHandle,
    command: &str,
    input: serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    let descriptor = registry
        .borrow()
        .manifest(registration)
        .unwrap()
        .capabilities[command]
        .clone();
    let receipt = registry
        .borrow_mut()
        .dispatch(
            registration,
            command,
            PluginPayload::encode(descriptor.request_schema, &input).unwrap(),
        )
        .unwrap();
    assert!(matches!(receipt.outcome, PluginDispatch::Pending));
    match drain_review_probe(pump, registry, &receipt.ticket).event {
        PluginEventKind::Completed(payload) => {
            Ok(payload.decode(&descriptor.reply_schema).unwrap())
        }
        PluginEventKind::Failed(error) => Err(error
            .details
            .unwrap()
            .decode(&descriptor.error_schema.unwrap())
            .unwrap()),
        _ => panic!("Canonical terminal must retain its actual outcome"),
    }
}
fn close_review_probe(
    owner: &ExecutionOwner,
    registry: &Rc<RefCell<PluginRegistry>>,
    registration: &RegistrationHandle,
    pump: AsyncSdkPump,
) {
    pump.begin_close().unwrap();
    assert!(pump.tickets().unwrap().is_empty());
    registry.borrow_mut().unregister(registration).unwrap();
    drop(pump);
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        if let Ok(receipt) = owner.close() {
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            break;
        }
        assert!(Instant::now() < until);
        thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn output_review_profile_is_explicit_and_preserves_legacy_contract_and_capabilities() {
    use crate::assembly::meari::proposal_manifest::*;
    let immediate = proposal_manifest();
    let legacy = async_proposal_manifest();
    let review = output_review_proposal_manifest();
    assert_eq!(immediate.capabilities.len(), 4);
    assert_eq!(legacy.capabilities.len(), 5);
    assert_eq!(
        legacy.contract,
        ContentHash::of(kinekino_media_contracts::plugin_contracts::proposal_v1::BYTES)
    );
    assert_eq!(immediate.contract, legacy.contract);
    assert_eq!(
        review.contract,
        ContentHash::of(
            kinekino_media_contracts::plugin_contracts::proposal_v1::OUTPUT_REVIEW_PROFILE_BYTES
        )
    );
    assert_ne!(review.contract, legacy.contract);
    assert_eq!(review.capabilities.len(), 8);
    for name in ASYNC_CAPABILITIES {
        assert_eq!(
            serde_json::to_value(&legacy.capabilities[name]).unwrap(),
            serde_json::to_value(&review.capabilities[name]).unwrap()
        );
    }
    let mut requests = std::collections::BTreeSet::new();
    for name in OUTPUT_REVIEW_CAPABILITIES {
        assert!(!legacy.capabilities.contains_key(name));
        let descriptor = &review.capabilities[name];
        assert_eq!(
            descriptor.targets,
            std::collections::BTreeSet::from([TargetKind::Native])
        );
        assert_ne!(descriptor.request_schema, descriptor.reply_schema);
        assert!(requests.insert(descriptor.request_schema.as_str().to_owned()));
        assert!(descriptor.error_schema.is_some());
    }
}
#[test]
fn output_review_three_typed_commands_and_legacy_inspect_share_one_owned_worker() {
    use crate::domain::proposals::output_review::{
        DeclaredReviewer, OutputDecisionInput, OutputVerdict,
    };
    let temp = Temp::new();
    let sdk: ApplicationClientHandle =
        Arc::new(PlayerSdk::default().with_proposal_store(ProposalFiles::open(&temp.0).unwrap()));
    let mut create = Request::new("proposals.create");
    create.payload = json!({"proposal_id":"synthetic-output-review","source":{"source_id":"synthetic-source","source_revision":"1","source_sha256":"a".repeat(64),"cues":[{"cue_id":"cue","text":"Synthetic original","start_us":null,"end_us":null}]},"generated":{"task":"correction","raw_text":"{\"task\":\"correction\",\"edits\":[{\"cue_id\":\"cue\",\"text\":\"Synthetic correction\"}]}"}});
    let document = sdk.execute(create);
    assert!(document.ok);
    let mut accept = Request::new("proposals.accept");
    accept.context.expected_revision = document.data.as_ref().unwrap()["draft_revision"]
        .as_str()
        .map(str::to_owned);
    accept.payload = json!({"proposal_id":"synthetic-output-review","target_source_revision":"1"});
    assert!(sdk.execute(accept).ok);
    let record = Arc::new(Record {
        inner: sdk,
        threads: Mutex::new(Vec::new()),
    });
    let owner = execution(record.clone());
    let (_program, _scope, registry) = scope();
    let context = Rc::new(RefCell::new(RequestContext::default()));
    let grants = output_review_proposal_manifest()
        .capabilities
        .keys()
        .cloned()
        .collect();
    let (registration, pump) = register_review_probe(&owner, &registry, context.clone(), grants);
    let inspection: OutputReviewInspection = serde_json::from_value(
        invoke_review_probe(
            &pump,
            &registry,
            &registration,
            "proposals.inspect_output_review_target",
            json!({"proposal_id":"synthetic-output-review"}),
        )
        .unwrap(),
    )
    .unwrap();
    context.borrow_mut().expected_revision = Some(inspection.target.output_revision.0.to_string());
    let input = OutputDecisionInput {
        decision_id: "synthetic-deferred".into(),
        target: inspection.target,
        criteria: inspection.criteria_identity,
        verdict: OutputVerdict::Deferred,
        note: "Synthetic transport fixture; no actual human quality approval".into(),
        declared_reviewer: DeclaredReviewer {
            label: "Synthetic declared reviewer".into(),
            reviewer_id: None,
        },
        claimed_review_unix_ms: None,
    };
    let saved: StoredOutputDecision = serde_json::from_value(
        invoke_review_probe(
            &pump,
            &registry,
            &registration,
            "proposals.record_output_decision",
            json!({"proposal_id":"synthetic-output-review","decision":input}),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!saved.decision.counts_as_model_sample);
    assert!(!saved.decision.automatic_application_enabled);
    context.borrow_mut().expected_revision = None;
    let history: OutputDecisionHistory = serde_json::from_value(
        invoke_review_probe(
            &pump,
            &registry,
            &registration,
            "proposals.output_decisions",
            json!({"proposal_id":"synthetic-output-review"}),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(history.records[0].decision_sha256, saved.decision_sha256);
    assert!(matches!(
        history.current_evidence,
        crate::application::proposals::output_review::CurrentEvidenceStatus::NotChecked
    ));
    assert!(
        invoke_review_probe(
            &pump,
            &registry,
            &registration,
            "proposals.inspect",
            json!({"proposal_id":"synthetic-output-review"})
        )
        .is_ok()
    );
    let threads = record.threads.lock().unwrap();
    assert_eq!(threads.len(), 4);
    assert!(
        threads
            .iter()
            .all(|id| *id == threads[0] && *id != thread::current().id())
    );
    drop(threads);
    close_review_probe(&owner, &registry, &registration, pump);
}
#[test]
fn output_review_capabilities_require_explicit_document_grants_before_execution() {
    let record = Arc::new(Record {
        inner: Arc::new(PlayerSdk::default()),
        threads: Mutex::new(Vec::new()),
    });
    let owner = execution(record.clone());
    let (_program, _scope, registry) = scope();
    let grants = crate::assembly::meari::proposal_manifest::ASYNC_CAPABILITIES
        .into_iter()
        .map(str::to_owned)
        .collect();
    let (registration, pump) = register_review_probe(
        &owner,
        &registry,
        Rc::new(RefCell::new(RequestContext::default())),
        grants,
    );
    for name in crate::assembly::meari::proposal_manifest::OUTPUT_REVIEW_CAPABILITIES {
        let descriptor = registry
            .borrow()
            .manifest(&registration)
            .unwrap()
            .capabilities[name]
            .clone();
        let error = registry
            .borrow_mut()
            .dispatch(
                &registration,
                name,
                PluginPayload::encode(
                    descriptor.request_schema,
                    &json!({"proposal_id":"not-authorized"}),
                )
                .unwrap(),
            )
            .err()
            .unwrap();
        assert_eq!(error.code, PluginErrorCode::PermissionDenied);
    }
    assert!(record.threads.lock().unwrap().is_empty());
    close_review_probe(&owner, &registry, &registration, pump);
}
#[test]
fn output_review_cancel_drains_original_ticket_before_scope_rebind_and_rejects_old_completion() {
    let owner = execution(Arc::new(PlayerSdk::default()));
    let (_program, scope, registry) = scope();
    let grants = output_review_proposal_manifest()
        .capabilities
        .keys()
        .cloned()
        .collect();
    let (registration, pump) = register_review_probe(
        &owner,
        &registry,
        Rc::new(RefCell::new(RequestContext::default())),
        grants,
    );
    let descriptor = registry
        .borrow()
        .manifest(&registration)
        .unwrap()
        .capabilities["proposals.output_decisions"]
        .clone();
    let receipt = registry
        .borrow_mut()
        .dispatch(
            &registration,
            "proposals.output_decisions",
            PluginPayload::encode(
                descriptor.request_schema,
                &ListOutputDecisions {
                    proposal_id: "synthetic-unavailable".into(),
                },
            )
            .unwrap(),
        )
        .unwrap();
    assert!(matches!(receipt.outcome, PluginDispatch::Pending));
    registry.borrow_mut().cancel(&receipt.ticket).unwrap();
    let next_scope = PluginScope {
        source_generation: scope.source_generation + 1,
        ..scope
    };
    assert_eq!(
        registry
            .borrow_mut()
            .rebind_scope(next_scope.clone())
            .unwrap_err()
            .code,
        PluginErrorCode::Busy
    );
    let terminal = drain_review_probe(&pump, &registry, &receipt.ticket);
    assert_eq!(registry.borrow().pending_count(), 0);
    registry.borrow_mut().rebind_scope(next_scope).unwrap();
    assert_eq!(
        registry
            .borrow_mut()
            .accept_event(&receipt.ticket, terminal.event)
            .unwrap_err()
            .code,
        PluginErrorCode::Stale
    );
    assert!(
        pump.accept(&mut registry.borrow_mut(), &receipt.ticket)
            .is_err()
    );
    close_review_probe(&owner, &registry, &registration, pump);
}
struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct Record {
    inner: ApplicationClientHandle,
    threads: Mutex<Vec<thread::ThreadId>>,
}
impl ApplicationClient for Record {
    fn execute(&self, request: Request) -> Response {
        self.threads.lock().unwrap().push(thread::current().id());
        self.inner.execute(request)
    }
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sdk-async-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn execution(client: ApplicationClientHandle) -> ExecutionOwner {
    ExecutionOwner::start(
        client,
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 65536,
            max_total_request_bytes: 1048576,
            max_terminal_bytes: 65536,
            max_total_terminal_bytes: 524288,
        },
        Arc::new(Notify),
    )
    .unwrap()
}
fn codec() -> AsyncCodecBudget {
    AsyncCodecBudget {
        max_pending: 8,
        max_terminal_wire_bytes: 65536,
        max_total_codec_wire_bytes: 2097152,
    }
}
fn scope() -> (StateProgram, PluginScope, Rc<RefCell<PluginRegistry>>) {
    let component = meari::parse(
        "type Props={};export function Owned(props:Props){return <main/>;}",
        Language::Tsx,
    )
    .unwrap();
    let program = StateProgram::mount(component, "owned-sdk-scope".into(), 1).unwrap();
    let scope = PluginScope {
        instance: program.snapshot().unwrap().instance,
        source_generation: 1,
    };
    let registry = Rc::new(RefCell::new(
        PluginRegistry::new(
            ContentHash::of(b"owned-sdk"),
            TargetKind::Native,
            scope.clone(),
            8,
        )
        .unwrap(),
    ));
    (program, scope, registry)
}
fn drain(
    lease: &AsyncProposalRegistration,
    registry: &Rc<RefCell<PluginRegistry>>,
    ticket: &PluginTicket,
) -> PluginEvent {
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(accepted) = lease
            .pump()
            .accept(&mut registry.borrow_mut(), ticket)
            .unwrap()
        {
            accepted.owner_ack.unwrap();
            return accepted.event;
        }
        assert!(Instant::now() < until);
        thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn initial_typed_acceptance_precedes_exclusive_document_attach_and_reuses_one_worker() {
    let temp = Temp::new();
    let sdk: ApplicationClientHandle = Arc::new(
        PlayerSdk::new(SubtitleFiles).with_proposal_store(ProposalFiles::open(&temp.0).unwrap()),
    );
    let mut create = Request::new("proposals.create");
    create.payload = json!({"proposal_id":"explicit-seed","source":{"source_id":"technical-source","source_revision":"1","source_sha256":"a".repeat(64),"cues":[{"cue_id":"explicit-cue","text":"technical fixture","start_us":null,"end_us":null}]},"generated":{"task":"correction","raw_text":"preserved malformed technical raw"}});
    assert!(sdk.execute(create).ok);
    let record = Arc::new(Record {
        inner: sdk,
        threads: Mutex::new(Vec::new()),
    });
    let owner = execution(record.clone());
    let handle = owner.handle();
    let mut initial = InitialProposalInspection::start(
        handle.clone(),
        RequestContext::default(),
        InspectProposal {
            proposal_id: "explicit-seed".into(),
        },
        65536,
    )
    .unwrap();
    let until = Instant::now() + Duration::from_secs(3);
    let prepared = loop {
        if let Some(ready) = initial.peek_terminal().unwrap() {
            break ready;
        }
        assert!(Instant::now() < until);
        thread::sleep(Duration::from_millis(1));
    };
    assert_eq!(
        prepared.as_ref().as_ref().unwrap().document.proposal_id,
        "explicit-seed"
    );
    assert!(handle.attach().is_err());
    let raw = initial.retained_terminal().unwrap().unwrap();
    initial.acknowledge().unwrap();
    assert!(Arc::ptr_eq(
        &raw,
        &initial.retained_terminal().unwrap().unwrap()
    ));
    // Only now create an actual document scope. Initial acceptance is no renderer ACK.
    let (_program, scope, registry) = scope();
    let context = Rc::new(RefCell::new(RequestContext::default()));
    let mut lease = AsyncProposalRegistration::register(
        registry.clone(),
        &scope,
        context.clone(),
        handle.clone(),
        codec(),
    )
    .unwrap();
    assert_eq!(
        handle
            .submit(Request::new("proposals.inspect"))
            .unwrap_err()
            .code,
        crate::application::api::ErrorCode::PermissionDenied
    );
    assert_eq!(
        handle.close().unwrap_err().code,
        crate::application::api::ErrorCode::PermissionDenied
    );
    let (_other_program, other_scope, other_registry) = self::scope();
    assert!(
        AsyncProposalRegistration::register(
            other_registry,
            &other_scope,
            context,
            handle.clone(),
            codec()
        )
        .is_err()
    );
    let registration = lease.handle().unwrap().clone();
    let descriptor = registry
        .borrow()
        .manifest(&registration)
        .unwrap()
        .capabilities["proposals.inspect"]
        .clone();
    let receipt = registry
        .borrow_mut()
        .dispatch(
            &registration,
            "proposals.inspect",
            PluginPayload::encode(
                descriptor.request_schema,
                &InspectProposal {
                    proposal_id: "explicit-seed".into(),
                },
            )
            .unwrap(),
        )
        .unwrap();
    assert!(matches!(receipt.outcome, PluginDispatch::Pending));
    let event = drain(&lease, &registry, &receipt.ticket);
    assert!(matches!(event.event, PluginEventKind::Completed(_)));
    assert!(
        lease
            .pump()
            .accept(&mut registry.borrow_mut(), &receipt.ticket)
            .is_err()
    );
    let threads = record.threads.lock().unwrap();
    assert_eq!(threads.len(), 2);
    assert_eq!(threads[0], threads[1]);
    assert_ne!(threads[0], thread::current().id());
    drop(threads);
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        if lease.unbind().is_ok() {
            break;
        }
        assert!(Instant::now() < until);
        thread::sleep(Duration::from_millis(1));
    }
    let (_next_program, next_scope, next_registry) = self::scope();
    let mut next = AsyncProposalRegistration::register(
        next_registry.clone(),
        &next_scope,
        Rc::new(RefCell::new(RequestContext::default())),
        handle.clone(),
        codec(),
    )
    .unwrap();
    // A retired document must not revoke the new attachment on Drop.
    drop(lease);
    let registration = next.handle().unwrap().clone();
    let schema = next_registry
        .borrow()
        .manifest(&registration)
        .unwrap()
        .capabilities["proposals.inspect"]
        .request_schema
        .clone();
    let receipt = next_registry
        .borrow_mut()
        .dispatch(
            &registration,
            "proposals.inspect",
            PluginPayload::encode(
                schema,
                &InspectProposal {
                    proposal_id: "explicit-seed".into(),
                },
            )
            .unwrap(),
        )
        .unwrap();
    assert!(matches!(receipt.outcome, PluginDispatch::Pending));
    assert!(matches!(
        drain(&next, &next_registry, &receipt.ticket).event,
        PluginEventKind::Completed(_)
    ));
    let threads = record.threads.lock().unwrap();
    assert_eq!(threads.len(), 3);
    assert!(threads.iter().all(|id| *id == threads[0]));
    drop(threads);
    next.unbind().unwrap();
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        if let Ok(receipt) = owner.close() {
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            break;
        }
        assert!(Instant::now() < until);
        thread::sleep(Duration::from_millis(1));
    }
    assert!(initial.acknowledge().unwrap().already_acknowledged);
}
struct InvalidClient {
    kind: u8,
    calls: std::sync::atomic::AtomicUsize,
}
impl ApplicationClient for InvalidClient {
    fn execute(&self, request: Request) -> Response {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut response = Response {
            api_version: crate::application::api::API_VERSION,
            request_id: request.context.request_id,
            ok: true,
            data: Some(json!({"not":"a stored proposal"})),
            error: None,
        };
        match self.kind {
            0 => response.api_version = "invalid",
            1 => response.request_id = "different-correlation".into(),
            2 => {
                response.error = Some(ApiError::new(
                    crate::application::api::ErrorCode::Internal,
                    "inconsistent envelope",
                ))
            }
            _ => {}
        };
        response
    }
}
#[test]
fn invalid_response_metadata_outcome_or_typed_reply_retains_original_without_ack() {
    for kind in 0..4 {
        let client = Arc::new(InvalidClient {
            kind,
            calls: std::sync::atomic::AtomicUsize::new(0),
        });
        let owner = execution(client.clone());
        let handle = owner.handle();
        let (_program, scope, registry) = scope();
        let mut lease = AsyncProposalRegistration::register(
            registry.clone(),
            &scope,
            Rc::new(RefCell::new(RequestContext::default())),
            handle.clone(),
            codec(),
        )
        .unwrap();
        let registration = lease.handle().unwrap().clone();
        let schema = registry
            .borrow()
            .manifest(&registration)
            .unwrap()
            .capabilities["proposals.inspect"]
            .request_schema
            .clone();
        let receipt = registry
            .borrow_mut()
            .dispatch(
                &registration,
                "proposals.inspect",
                PluginPayload::encode(
                    schema,
                    &InspectProposal {
                        proposal_id: "explicit".into(),
                    },
                )
                .unwrap(),
            )
            .unwrap();
        let until = Instant::now() + Duration::from_secs(3);
        let original = loop {
            let ids = handle.retained_operations().unwrap();
            if let Some(id) = ids.first()
                && let ExecutionSnapshot::Terminal(t) = handle.peek(id).unwrap()
            {
                break t;
            }
            assert!(Instant::now() < until);
            thread::sleep(Duration::from_millis(1));
        };
        assert_eq!(original.validate_response().is_err(), kind < 3);
        assert!(
            lease
                .pump()
                .accept(&mut registry.borrow_mut(), &receipt.ticket)
                .is_err()
        );
        assert_eq!(registry.borrow().pending_count(), 1);
        assert_eq!(lease.close().unwrap_err().code, PluginErrorCode::Busy);
        assert_eq!(handle.retained_operations().unwrap().len(), 1);
        assert!(
            lease
                .pump()
                .accept(&mut registry.borrow_mut(), &receipt.ticket)
                .is_err()
        );
        assert_eq!(client.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        let id = handle.retained_operations().unwrap().pop().unwrap();
        let ExecutionSnapshot::Terminal(retained) = handle.peek(&id).unwrap() else {
            panic!("original terminal must remain owned")
        };
        assert!(Arc::ptr_eq(&original.response, &retained.response));
        assert!(Arc::ptr_eq(&original.request, &retained.request));
        // Forced registry loss is an incomplete close, never a successful terminal ACK.
        drop(lease);
        drop(registry);
        let until = Instant::now() + Duration::from_secs(3);
        loop {
            let error = owner.close().unwrap_err();
            if error.details["worker_stopped"] == true {
                break;
            }
            assert!(Instant::now() < until);
            thread::sleep(Duration::from_millis(1));
        }
        assert!(original.response.data.is_some());
    }
}
