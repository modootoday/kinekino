//! Source-authored real SDK/closed-panel contracts; no OS input or checked-frame claim.
use super::*;
use crate::application::api::ProposalClient;
use crate::{
    application::{
        api::PlayerSdk,
        proposals::{CreateProposal, InspectProposal},
        sdk_execution::{
            CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner,
        },
    },
    domain::{api_values::DecimalU64, proposals::*},
    infrastructure::proposals::ProposalFiles,
};
use meari::application::host_callbacks::HostCallbacks;
use meari_native::{
    domain::{Viewport, widgets::Action},
    infrastructure::document::InteractiveDocument,
};
use sha2::{Digest, Sha256};
use std::{
    any::Any,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy)]
enum Kind {
    Card,
    Highlight,
    Cue,
    Document,
}
pub(crate) struct LiveFixtureSeedError {
    diagnostic: String,
    _original: Box<dyn Any>,
}
impl LiveFixtureSeedError {
    fn new<E: std::fmt::Debug + 'static>(error: E) -> Self {
        Self {
            diagnostic: format!("{error:?}"),
            _original: Box::new(error),
        }
    }
}
impl From<String> for LiveFixtureSeedError {
    fn from(error: String) -> Self {
        Self::new(error)
    }
}
impl std::fmt::Debug for LiveFixtureSeedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveFixtureSeedError")
            .field("original", &self.diagnostic)
            .finish()
    }
}
impl ViewerHost {
    pub(crate) fn seed_live_fixture(
        client: &ApplicationClientHandle,
        case: crate::assembly::meari::viewer_native_probe::live_retained::Case,
    ) -> Result<NativeProposalSelection, LiveFixtureSeedError> {
        use crate::assembly::meari::viewer_native_probe::live_retained::Case;
        seed_live_proposal(
            client,
            match case {
                Case::Card => Kind::Card,
                Case::Highlight => Kind::Highlight,
                Case::Cue => Kind::Cue,
                Case::Document => Kind::Document,
            },
        )
    }
}
fn seed_live_proposal(
    client: &ApplicationClientHandle,
    kind: Kind,
) -> Result<NativeProposalSelection, LiveFixtureSeedError> {
    let cue = SourceSnapshot::Cue(CueSourceSnapshot {
        source_id: "r2-source".into(),
        source_revision: DecimalU64(9_007_199_254_740_993),
        source_sha256: "a".repeat(64),
        cues: vec![
            SourceCue {
                cue_id: "a".into(),
                text: "첫 원문".into(),
                start_us: None,
                end_us: None,
            },
            SourceCue {
                cue_id: "b".into(),
                text: "둘째 日本語".into(),
                start_us: None,
                end_us: None,
            },
        ],
    });
    let (source, payload, target) = match kind {
        Kind::Card => (
            cue,
            ProposalPayload::Cards {
                cards: vec![
                    Card {
                        front: "first".into(),
                        back: "back1".into(),
                        source_refs: vec!["a".into()],
                    },
                    Card {
                        front: "second".into(),
                        back: "back2".into(),
                        source_refs: vec!["b".into()],
                    },
                ],
            },
            NativeProposalTarget::Card { index: 1 },
        ),
        Kind::Highlight => (
            cue,
            ProposalPayload::Highlights {
                highlights: vec![
                    Highlight {
                        title: "first".into(),
                        source_refs: vec!["a".into()],
                    },
                    Highlight {
                        title: "second".into(),
                        source_refs: vec!["b".into()],
                    },
                ],
            },
            NativeProposalTarget::Highlight { index: 1 },
        ),
        Kind::Cue => (
            cue,
            ProposalPayload::Correction {
                edits: vec![
                    Correction {
                        cue_id: "a".into(),
                        text: "첫 수정".into(),
                    },
                    Correction {
                        cue_id: "b".into(),
                        text: "둘째 수정".into(),
                    },
                ],
            },
            NativeProposalTarget::CueCorrection {
                index: 1,
                cue_id: "b".into(),
            },
        ),
        Kind::Document => {
            let content = "Edit this ASR draft. Correct punctuation only. Do not guess unclear words. JSON only: {\"proposed\":string,\"uncertain_words\":[string]}. Under100words. /no_think\n[\"첫 원문\",\"둘째 日本語\"]";
            let asset =
                serde_json::json!({"messages":[{"role":"user","content":content}]}).to_string();
            let document = DocumentSourceSnapshot::from_asset(
                "r2-document".into(),
                DecimalU64(9_007_199_254_740_993),
                format!("{:x}", Sha256::digest(asset.as_bytes())),
                asset,
            )
            .map_err(LiveFixtureSeedError::new)?;
            let first = document
                .text_reference("/0")
                .ok_or_else(|| "Original first document reference missing".to_owned())?;
            let second = document
                .text_reference("/1")
                .ok_or_else(|| "Original second document reference missing".to_owned())?;
            (
                SourceSnapshot::Document(document),
                ProposalPayload::DocumentCorrection {
                    edits: vec![
                        DocumentCorrection {
                            source_ref: first,
                            text: "첫 수정".into(),
                        },
                        DocumentCorrection {
                            source_ref: second.clone(),
                            text: "둘째 수정".into(),
                        },
                    ],
                },
                NativeProposalTarget::DocumentCorrection {
                    index: 1,
                    source_ref: second,
                },
            )
        }
    };
    let task = match kind {
        Kind::Card => ProposalTask::Cards,
        Kind::Highlight => ProposalTask::Highlights,
        _ => ProposalTask::Correction,
    };
    let seed = client
        .create_proposal(
            RequestContext::default(),
            CreateProposal {
                proposal_id: "r2-proposal".into(),
                source,
                generated: GeneratedResult {
                    task,
                    raw_text: serde_json::to_string(&payload).map_err(LiveFixtureSeedError::new)?,
                },
            },
        )
        .map_err(LiveFixtureSeedError::new)?;
    let selected = NativeProposalSelection {
        proposal_id: seed.proposal_id.clone(),
        document_revision: seed.draft_revision,
        source_id: seed.source.source_id().into(),
        source_revision: seed.source.source_revision(),
        source_sha256: seed.source.source_sha256().into(),
        target,
    };
    Ok(selected)
}

struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), crate::application::api::ApiError> {
        Ok(())
    }
}
#[derive(Default)]
struct Rig {
    input: Option<ProductProposalMountInput>,
    runtime: Option<NativeHostRuntime<ActiveCardBridge>>,
    failure: Option<Box<ProductProposalMountFailure>>,
    owner: Option<ExecutionOwner>,
    client: Option<ApplicationClientHandle>,
    handle: Option<NativeDocumentHandle>,
    receipt: Option<HostCommitReceipt>,
    proof: Option<RetainedCommitProof>,
    proof_error: Option<Arc<HostAccessError>>,
    original: Option<StoredProposal>,
    setup_carrier: Option<Box<dyn Any>>,
    cleanup_carrier: Option<Box<dyn Any>>,
}
impl Rig {
    fn setup_error<T: std::fmt::Debug + 'static>(&mut self, error: T) -> String {
        let message = format!("{error:?}");
        self.setup_carrier = Some(Box::new(error));
        message
    }
    fn setup(&mut self, kind: Kind) -> Result<(), String> {
        let root = std::env::current_dir()
            .map_err(|error| self.setup_error(error))?
            .join("runtime/quality-review/ui-live-mount-r2")
            .join(format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|error| self.setup_error(error))?
                    .as_nanos()
            ));
        std::fs::create_dir_all(&root).map_err(|error| self.setup_error(error))?;
        let store = ProposalFiles::open(&root).map_err(|error| self.setup_error(error))?;
        let client: ApplicationClientHandle =
            Arc::new(PlayerSdk::default().with_proposal_store(store));
        self.client = Some(client.clone());
        let selected =
            seed_live_proposal(&client, kind).map_err(|error| self.setup_error(error))?;
        let stored = client
            .inspect_proposal(
                RequestContext::default(),
                InspectProposal {
                    proposal_id: selected.proposal_id.clone(),
                },
            )
            .map_err(|error| self.setup_error(error))?;
        self.original = Some(stored.clone());
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
        .map_err(|error| self.setup_error(error))?;
        let execution = owner.handle();
        self.owner = Some(owner);
        let entry: meari::domain::build::BuildPath = "r2/view.tsx"
            .to_owned()
            .try_into()
            .map_err(|error| self.setup_error(error))?;
        let inventory = BTreeMap::from([(
            entry.clone(),
            "export function View(){return <main/>;}".into(),
        )]);
        let viewing = meari::parse_module_graph(&entry, &inventory)
            .map_err(|error| self.setup_error(error))?;
        let (entry, mut studio_inventory) = product_modules_for_panel(BootstrapPanel::Card);
        let provider = meari::parse_module_graph(&entry, &studio_inventory)
            .map_err(|error| self.setup_error(error))?;
        let (studio, _) = FrameComposition::new(
            crate::domain::product::PRODUCT.display_name.into(),
            String::new(),
        )
        .prepare(provider, &mut studio_inventory)
        .map_err(|error| self.setup_error(error))?;
        let frame = ProductFrameBlueprint::prepare(viewing, inventory, studio, studio_inventory)?;
        let program = frame.mount_viewing(
            BTreeMap::new(),
            "r2-frame".into(),
            1,
            NonZeroU64::new(1).unwrap(),
        )?;
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
            |scope| {
                HostCallbacks::new(scope)
                    .export_bindings()
                    .map_err(|error| error.message)
            },
        )?;
        let handle =
            NativeDocumentHandle::new(document, 1).map_err(|error| self.setup_error(error))?;
        self.handle = Some(handle.clone());
        self.input = Some(ProductProposalMountInput {
            document: handle.clone(),
            turn: HostTurn {
                stamp: handle.stamp().map_err(|error| self.setup_error(error))?,
                reason: HostTurnReason::Input,
                accepted_callbacks: 0,
                rejected_callbacks: 0,
                remaining_callbacks: 0,
            },
            frame,
            selected,
            stored,
            client,
            execution,
            context: RequestContext::default(),
            namespace: "r2-editor".into(),
            generation: NonZeroU64::new(2).unwrap(),
            labels: ProductWindowLabels::korean(),
            queue: None,
            navigation: None,
        });
        Ok(())
    }
    fn mount(&mut self, gate: Option<Rc<Cell<bool>>>) {
        self.mount_observing_proof(gate);
        if self.failure.is_none() {
            assert!(self.proof_error.is_none());
        }
    }
    fn mount_observing_proof(&mut self, gate: Option<Rc<Cell<bool>>>) {
        match mount_current_proposal(self.input.take().unwrap(), gate) {
            Ok(mounted) => {
                self.receipt = Some(mounted.receipt);
                self.proof = mounted.proof;
                self.proof_error = mounted.proof_error;
                self.runtime = Some(NativeHostRuntime::new(
                    self.handle.as_ref().unwrap().clone(),
                    mounted.bridge,
                ));
            }
            Err(failure) => {
                self.failure = Some(failure);
            }
        }
    }
    fn cleanup(&mut self) -> Result<(), String> {
        let end = Instant::now() + Duration::from_secs(5);
        if let Some(failure) = &mut self.failure {
            while !failure.poll_release().map_err(|error| {
                let message = format!("{error:?}");
                self.cleanup_carrier = Some(Box::new(error));
                message
            })? {
                if Instant::now() >= end {
                    return Err("Original partial mount cleanup pending".into());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        if let Some(runtime) = &mut self.runtime {
            loop {
                match runtime.host_mut().unbind_scope() {
                    Ok(()) => break,
                    Err(_) if runtime.host().scope_close_waiting() && Instant::now() < end => {
                        runtime
                            .after_dispatch(HostTurnReason::Completion)
                            .map_err(|error| {
                                let message = format!("{error:?}");
                                self.cleanup_carrier = Some(Box::new(error));
                                message
                            })?;
                    }
                    Err(error) => {
                        let message = format!("{error:?}");
                        self.cleanup_carrier = Some(Box::new(error));
                        return Err(message);
                    }
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        if let Some(owner) = &mut self.owner {
            loop {
                let progress = owner.poll_close().map_err(|error| {
                    let message = format!("{error:?}");
                    self.cleanup_carrier = Some(Box::new(error));
                    message
                })?;
                if let ExecutionCloseProgress::Closed(receipt) = progress {
                    if receipt.worker_stopped && receipt.unacknowledged == 0 {
                        break;
                    }
                    return Err("Original worker close incomplete".into());
                }
                if Instant::now() >= end {
                    return Err("Original worker cleanup pending".into());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        Ok(())
    }
}
struct Failure {
    setup: Option<String>,
    setup_panic: Option<Box<dyn Any + Send>>,
    primary: Option<Box<dyn Any + Send>>,
    cleanup: Option<String>,
    cleanup_panic: Option<Box<dyn Any + Send>>,
    rig: Rig,
}
impl std::fmt::Debug for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("R2OriginalMountFailure")
            .field("setup", &self.setup)
            .field("setup_panic", &self.setup_panic.is_some())
            .field("primary_panic", &self.primary.is_some())
            .field("cleanup", &self.cleanup)
            .field("cleanup_panic", &self.cleanup_panic.is_some())
            .field("setup_carrier", &self.rig.setup_carrier.is_some())
            .field("cleanup_carrier", &self.rig.cleanup_carrier.is_some())
            .finish()
    }
}
fn run(kind: Kind, body: impl FnOnce(&mut Rig)) -> Result<(), Box<Failure>> {
    let mut rig = Rig::default();
    let (setup, setup_panic) =
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| rig.setup(kind))) {
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
    if setup.is_some()
        || setup_panic.is_some()
        || primary.is_some()
        || cleanup.is_some()
        || cleanup_panic.is_some()
    {
        Err(Box::new(Failure {
            setup,
            setup_panic,
            primary,
            cleanup,
            cleanup_panic,
            rig,
        }))
    } else {
        Ok(())
    }
}
fn callback(handle: &NativeDocumentHandle, name: &str) -> String {
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

#[test]
#[ignore = "Original scoped SDK panel slot; actual source local action only, no native frame claim"]
fn current_navigation_source_uses_same_export_and_queues_without_editor_mutation()
-> Result<(), Box<Failure>> {
    for kind in [Kind::Card, Kind::Highlight, Kind::Cue, Kind::Document] {
        run(kind, |rig| {
            let mailbox = Rc::new(RefCell::new(
                super::super::source_navigation::NavigationIntents::default(),
            ));
            rig.input.as_mut().unwrap().navigation = Some(mailbox.clone());
            rig.mount(Some(Rc::new(Cell::new(true))));
            assert!(rig.failure.is_none());
            let handle = rig.handle.as_ref().unwrap().clone();
            let path = callback(&handle, "onNavigate");
            assert!(handle.dispatch_action(Action::Activate { path }).unwrap());
            let turn = rig
                .runtime
                .as_mut()
                .unwrap()
                .after_dispatch(HostTurnReason::Input)
                .unwrap();
            assert_eq!((turn.accepted_callbacks, turn.remaining_callbacks), (1, 0));
            assert_eq!(mailbox.borrow().pending_count(), 1);
            assert!(
                !rig.runtime
                    .as_ref()
                    .unwrap()
                    .host()
                    .navigation_guards()
                    .dirty
            );
        })?;
    }
    Ok(())
}

#[test]
#[ignore = "Original scoped SDK panel slot; entry observation gate is not replaced by source action"]
fn navigation_source_before_entry_observation_does_not_queue_return() -> Result<(), Box<Failure>> {
    run(Kind::Card, |rig| {
        let mailbox = Rc::new(RefCell::new(
            super::super::source_navigation::NavigationIntents::default(),
        ));
        rig.input.as_mut().unwrap().navigation = Some(mailbox.clone());
        rig.mount(Some(Rc::new(Cell::new(false))));
        assert!(rig.failure.is_none());
        let handle = rig.handle.as_ref().unwrap().clone();
        assert!(
            handle
                .dispatch_action(Action::Activate {
                    path: callback(&handle, "onNavigate")
                })
                .unwrap()
        );
        let turn = rig
            .runtime
            .as_mut()
            .unwrap()
            .after_dispatch(HostTurnReason::Input)
            .unwrap();
        assert_eq!(turn.accepted_callbacks, 0);
        assert_eq!(mailbox.borrow().pending_count(), 0);
    })
}

#[test]
fn entry_gate_disables_controls_without_rewriting_original_text_or_identity() {
    let mut props = BTreeMap::from([
        ("sourceText".into(), Value::String("原文".into())),
        ("title".into(), Value::String("working".into())),
        ("canEdit".into(), Value::Bool(true)),
        ("inputDisabled".into(), Value::Bool(false)),
    ]);
    let original = props.clone();
    let original_keys: Vec<_> = props.keys().cloned().collect();
    gate_props(&mut props, false);
    assert_eq!(props.len(), original.len());
    assert_eq!(props.keys().cloned().collect::<Vec<_>>(), original_keys);
    for key in ["sourceText", "title"] {
        assert_eq!(props.get(key), original.get(key));
    }
    assert!(!props.contains_key("saveDisabled"));
    assert!(!props.contains_key("outputReviewCanRecord"));
    assert_eq!(props.get("canEdit"), Some(&Value::Bool(false)));
    assert_eq!(props.get("inputDisabled"), Some(&Value::Bool(true)));
    let before = props.clone();
    gate_props(&mut props, true);
    assert_eq!(
        props, before,
        "Active gating does not invent SDK support or force enabled state"
    );
}

#[test]
#[ignore = "Assigned R2 original SDK/closed-panel slot; no frame or route acceptance"]
fn every_explicit_task_mounts_same_host_and_captures_immediate_proof_before_observer()
-> Result<(), Box<Failure>> {
    for kind in [Kind::Card, Kind::Highlight, Kind::Cue, Kind::Document] {
        run(kind, |rig| {
            let before = rig.handle.as_ref().unwrap().stamp().unwrap();
            let gate = Rc::new(Cell::new(false));
            rig.mount(Some(gate.clone()));
            assert!(rig.failure.is_none());
            let receipt = rig.receipt.as_ref().unwrap();
            assert_eq!(receipt.before(), &before);
            assert_eq!(receipt.after().host_id, before.host_id);
            assert_eq!(receipt.after().owner, before.owner);
            assert_eq!(rig.proof.as_ref().unwrap().receipt(), receipt);
            rig.handle
                .as_ref()
                .unwrap()
                .validate_retained_commit(rig.proof.as_ref().unwrap())
                .unwrap();
            assert!(!gate.get());
            assert_eq!(
                rig.runtime
                    .as_ref()
                    .unwrap()
                    .host()
                    .review()
                    .stored()
                    .document
                    .source
                    .source_sha256(),
                rig.original
                    .as_ref()
                    .unwrap()
                    .document
                    .source
                    .source_sha256()
            );
            assert!(rig.owner.as_ref().unwrap().handle().attach().is_err());
        })?;
    }
    Ok(())
}

#[test]
#[ignore = "Assigned R2 original SDK/closed-panel slot; technical source input only"]
fn common_panel_input_save_and_separate_accept_preserve_original_source_for_all_tasks()
-> Result<(), Box<Failure>> {
    for kind in [Kind::Card, Kind::Highlight, Kind::Cue, Kind::Document] {
        run(kind, |rig| {
            // Isolated mount/controller contract, not Viewer admission or observed-frame gating.
            rig.mount(None);
            assert!(rig.failure.is_none());
            let handle = rig.handle.as_ref().unwrap().clone();
            let binding = match kind {
                Kind::Card => "onFrontChange",
                Kind::Highlight => "onTitleChange",
                _ => "onDraftChange",
            };
            let path = callback(&handle, binding);
            handle
                .try_reproject(&handle.stamp().unwrap(), |document| {
                    document.set_input_value(&path, "explicit human edit")
                })
                .unwrap();
            let runtime = rig.runtime.as_mut().unwrap();
            runtime.after_dispatch(HostTurnReason::Input).unwrap();
            assert!(runtime.host().navigation_guards().dirty);
            for name in ["onSave", "onAccept"] {
                handle
                    .dispatch_action(Action::Activate {
                        path: callback(&handle, name),
                    })
                    .unwrap();
                runtime.after_dispatch(HostTurnReason::Input).unwrap();
                let end = Instant::now() + Duration::from_secs(3);
                while runtime.host().operation_pending() {
                    runtime.after_dispatch(HostTurnReason::Completion).unwrap();
                    assert!(Instant::now() < end);
                    std::thread::sleep(Duration::from_millis(1));
                }
                assert!(runtime.host().review().error().is_none());
                if name == "onSave" {
                    assert!(runtime.host().review().stored().acceptance.is_none());
                }
            }
            let reopened = rig
                .client
                .as_ref()
                .unwrap()
                .inspect_proposal(
                    RequestContext::default(),
                    InspectProposal {
                        proposal_id: "r2-proposal".into(),
                    },
                )
                .unwrap();
            assert!(reopened.acceptance.is_some());
            assert_eq!(
                serde_json::to_value(&reopened.document.source).unwrap(),
                serde_json::to_value(&rig.original.as_ref().unwrap().document.source).unwrap()
            );
            assert_eq!(
                reopened.document.generated.raw_text,
                rig.original.as_ref().unwrap().document.generated.raw_text
            );
        })?;
    }
    Ok(())
}

#[test]
#[ignore = "Assigned R2 original SDK/closed-panel slot"]
fn stale_target_and_missing_closed_provider_keep_original_input_before_registration()
-> Result<(), Box<Failure>> {
    for missing in [false, true] {
        run(Kind::Highlight, |rig| {
            let before = rig.handle.as_ref().unwrap().stamp().unwrap();
            let input = rig.input.as_mut().unwrap();
            if missing {
                let path: meari::domain::build::BuildPath =
                    "studio/clips/highlight-proposal-panel.tsx"
                        .to_owned()
                        .try_into()
                        .unwrap();
                input.frame.inventory.remove(&path);
            } else {
                input.selected.document_revision.0 += 1;
            }
            rig.mount(Some(Rc::new(Cell::new(false))));
            assert!(rig.runtime.is_none());
            let failure = rig.failure.as_ref().unwrap();
            assert_eq!(failure.input.document.stamp().unwrap(), before);
            if missing {
                assert!(failure.partial.is_none());
            } else {
                assert!(
                    matches!(&failure.partial,Some(ProductProposalMountPartial::Session(partial))if partial.parent.is_none())
                );
            }
        })?;
    }
    Ok(())
}

#[test]
#[ignore = "Original SDK closed panel slot; postcommit proof Err is an explicit model fault"]
fn model_proof_error_preserves_real_receipt_and_each_original_scoped_owner()
-> Result<(), Box<Failure>> {
    use crate::assembly::meari::viewer_native_probe::live_retained::{ModelProofScope, ProofStage};
    for kind in [Kind::Card, Kind::Highlight, Kind::Cue, Kind::Document] {
        run(kind, |rig| {
            let _fault = ModelProofScope::enter(Some(ProofStage::Entry));
            let gate = Rc::new(Cell::new(false));
            rig.mount_observing_proof(Some(gate.clone()));
            assert!(rig.failure.is_none());
            assert!(rig.receipt.is_some() && rig.runtime.is_some());
            assert!(rig.proof.is_none());
            assert!(matches!(
                rig.proof_error.as_deref(),
                Some(HostAccessError::Busy)
            ));
            assert!(!gate.get());
            assert!(rig.owner.as_ref().unwrap().handle().attach().is_err());
        })?;
    }
    Ok(())
}
