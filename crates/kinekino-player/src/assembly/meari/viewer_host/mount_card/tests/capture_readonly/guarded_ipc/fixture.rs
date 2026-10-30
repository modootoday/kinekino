use super::*;
use crate::application::api::{ProjectClient, ProposalClient};
use crate::{
    application::{
        api::{ApplicationClient, Request, Response},
        projects::{CreateProject, ProjectContentInput, ProposalRevisionRef},
        proposals::CreateProposal,
    },
    domain::projects::StoredProjectRevision,
    infrastructure::{
        local_host::{
            EndpointClient, LocalHost, PendingLocalHost, TransportDiagnostics, TransportOptions,
        },
        projects::ProjectFiles,
        proposals::ProposalFiles,
    },
};
use std::{
    path::PathBuf,
    sync::{Mutex, atomic::AtomicU8},
};

struct RecordedSdk {
    inner: ApplicationClientHandle,
    requests: Arc<Mutex<Vec<Request>>>,
    responses: Arc<Mutex<Vec<serde_json::Value>>>,
}
impl ApplicationClient for RecordedSdk {
    fn execute(&self, request: Request) -> Response {
        self.requests.lock().unwrap().push(request.clone());
        let response = self.inner.execute(request);
        self.responses
            .lock()
            .unwrap()
            .push(serde_json::to_value(&response).unwrap());
        response
    }
}
/// Private client-fault pattern from the existing execution tests; never a production injector.
struct EndpointProbe {
    endpoint: EndpointClient,
    fault_after_guard: Arc<AtomicU8>,
}
impl ApplicationClient for EndpointProbe {
    fn execute(&self, request: Request) -> Response {
        let guarded = matches!(
            request.command.as_str(),
            "proposals.edit_capture" | "proposals.accept_capture"
        );
        let mut response = self.endpoint.execute(request);
        if guarded && response.ok {
            match self.fault_after_guard.swap(0, Ordering::SeqCst) {
                1 => panic!("guarded IPC client panic after actual response"),
                2 => response.data = Some(serde_json::json!({"damaged":"after actual response"})),
                _ => {}
            }
        }
        response
    }
}
pub(super) struct StartFailure {
    error: ApiError,
    pending: Option<PendingLocalHost>,
}
impl std::fmt::Debug for StartFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginalGuardedStartFailure")
            .field("error", &self.error)
            .field("original_endpoint_retained", &self.pending.is_some())
            .finish()
    }
}
fn fail(error: ApiError, endpoint: Option<LocalHost>) -> Box<StartFailure> {
    Box::new(StartFailure {
        error,
        pending: endpoint.map(LocalHost::begin_shutdown),
    })
}
pub(super) struct Rig {
    pub(super) host: Option<ViewerHost>,
    pub(super) client: ApplicationClientHandle,
    pub(super) root: PathBuf,
    pub(super) requests: Arc<Mutex<Vec<Request>>>,
    pub(super) responses: Arc<Mutex<Vec<serde_json::Value>>>,
    pub(super) fault_after_guard: Arc<AtomicU8>,
    pub(super) transport: TransportDiagnostics,
}
impl Rig {
    pub(super) fn start() -> Result<Self, Box<StartFailure>> {
        let root = std::env::current_dir()
            .map_err(|e| fail(ApiError::new(ErrorCode::EngineFailed, e.to_string()), None))?
            .join("runtime/quality-review/ui-capture-guarded-stores")
            .join(format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        for leaf in ["proposals", "projects"] {
            std::fs::create_dir_all(root.join(leaf))
                .map_err(|e| fail(ApiError::new(ErrorCode::EngineFailed, e.to_string()), None))?;
        }
        let sdk: ApplicationClientHandle = Arc::new(
            PlayerSdk::default()
                .with_proposal_store(
                    ProposalFiles::open(&root.join("proposals")).map_err(|e| fail(e, None))?,
                )
                .with_project_store(
                    ProjectFiles::open(&root.join("projects")).map_err(|e| fail(e, None))?,
                ),
        );
        let requests = Arc::new(Mutex::new(Vec::new()));
        let responses = Arc::new(Mutex::new(Vec::new()));
        let sdk: ApplicationClientHandle = Arc::new(RecordedSdk {
            inner: sdk,
            requests: requests.clone(),
            responses: responses.clone(),
        });
        let transport = TransportDiagnostics::default();
        let endpoint =
            LocalHost::start_observed(sdk, TransportOptions::default(), transport.clone())
                .map_err(|e| fail(e, None))?;
        let connector = match EndpointClient::new(endpoint.endpoint(), TransportOptions::default())
        {
            Ok(client) => client,
            Err(error) => return Err(fail(error, Some(endpoint))),
        };
        let fault_after_guard = Arc::new(AtomicU8::new(0));
        let client: ApplicationClientHandle = Arc::new(EndpointProbe {
            endpoint: connector,
            fault_after_guard: fault_after_guard.clone(),
        });
        let owner = match ExecutionOwner::start(
            client.clone(),
            ExecutionBudget {
                max_operations: 8,
                max_request_bytes: 262144,
                max_total_request_bytes: 8 * 262144,
                max_terminal_bytes: 262144,
                max_total_terminal_bytes: 8 * 262144,
            },
            Arc::new(Notify),
        ) {
            Ok(owner) => owner,
            Err(error) => return Err(fail(error, Some(endpoint))),
        };
        // Infallible transfer immediately after worker creation. All later setup is inside
        // the parent's original-host panic/close guard, including parser and callback export.
        let host = ViewerHost::new(client.clone(), owner, endpoint);
        Ok(Self {
            host: Some(host),
            client,
            root,
            requests,
            responses,
            fault_after_guard,
            transport,
        })
    }
}
pub(super) fn draft(text: &str) -> ProposalPayload {
    ProposalPayload::Cards {
        cards: vec![Card {
            front: text.into(),
            back: "Answer".into(),
            source_refs: vec!["cue-a".into()],
        }],
    }
}
pub(super) fn seed(client: &ApplicationClientHandle) -> StoredProjectRevision {
    let proposal = client
        .create_proposal(
            RequestContext::default(),
            CreateProposal {
                proposal_id: "guarded-card".into(),
                source: SourceSnapshot::Cue(CueSourceSnapshot {
                    source_id: "guarded-source".into(),
                    source_revision: DecimalU64(3),
                    source_sha256: "a".repeat(64),
                    cues: vec![SourceCue {
                        cue_id: "cue-a".into(),
                        text: "Original caption".into(),
                        start_us: None,
                        end_us: None,
                    }],
                }),
                generated: GeneratedResult {
                    task: ProposalTask::Cards,
                    raw_text: serde_json::to_string(&draft("Generated")).unwrap(),
                },
            },
        )
        .unwrap();
    let stored = client
        .create_project(
            RequestContext::default(),
            CreateProject {
                project_id: "guarded-work".into(),
                title: "Captured work".into(),
                content: ProjectContentInput {
                    watching: None,
                    proposals: vec![ProposalRevisionRef::Live {
                        proposal_id: proposal.proposal_id,
                        draft_revision: proposal.draft_revision,
                        working_draft: None,
                    }],
                },
            },
        )
        .unwrap();
    assert_eq!(stored.project.schema_version, "2");
    assert!(stored.project.proposals[0].proposal_origin.is_some());
    stored
}
pub(super) fn button(
    host: &mut ViewerHost,
    project: StoredProjectRevision,
    binding: &str,
) -> PendingCallback {
    use meari::{
        application::source_modules::ModuleProgram,
        domain::{Value, build::BuildPath},
    };
    use std::collections::BTreeMap;
    host.callbacks()
        .borrow_mut()
        .configure_captured_edit_choices(
            project,
            NonZeroU64::new(1).unwrap(),
            std::num::NonZeroUsize::new(2).unwrap(),
        )
        .unwrap();
    let path: BuildPath = "explicit-capture.tsx".to_owned().try_into().unwrap();
    let graph = meari::parse_module_graph(&path, &BTreeMap::from([(path.clone(),
        "type Props={onOpen:(key:string)=>void;rowKey:string};export function Open(props:Props){return <button onClick={() => props.onOpen(props.rowKey)}>Edit captured card</button>;}".into())])).unwrap();
    let program = ModuleProgram::mount(
        graph,
        BTreeMap::from([
            ("rowKey".into(), Value::String("project-card-1-0-0".into())),
            (
                "onOpen".into(),
                Value::CallbackReference(CallbackReference {
                    binding: binding.into(),
                    signature: CallbackSignature::Text,
                }),
            ),
        ]),
        "explicit-capture".into(),
        1,
    )
    .unwrap();
    let callbacks = host.callbacks();
    let mut document = InteractiveDocument::from_module_program_with_callbacks(
        program,
        "",
        Viewport {
            width: 320,
            height: 160,
            dark: true,
        },
        None,
        1,
        move |scope| callbacks.borrow_mut().export(scope).map_err(|e| e.message),
    )
    .unwrap();
    let button = document
        .elements()
        .iter()
        .find(|e| e.dom_tag == "button")
        .unwrap()
        .path
        .clone();
    document
        .dispatch(Action::Activate { path: button })
        .unwrap();
    let event = document.drain_callbacks().pop().unwrap();
    let document = NativeDocumentHandle::new(document, 1).unwrap();
    host.bind_document(document, BTreeMap::new());
    event
}
pub(super) fn enter(host: &mut ViewerHost, project: StoredProjectRevision) {
    let event = button(host, project, CAPTURED_EDIT_BINDING);
    let stamp = host.document.as_ref().unwrap().stamp().unwrap();
    assert!(matches!(
        host.admit_callback(&stamp, &event),
        IntentAdmission::Accepted
    ));
    let mut state = ShellState::new();
    assert!(
        host.request_explicit_captured_edit(
            &mut state,
            &RouteCatalog::bundled().unwrap(),
            NavigationGuards::default()
        )
        .unwrap()
    );
    assert!(state.observed_route().is_none()); // No synthetic rendered/committed route receipt.
    let end = Instant::now() + Duration::from_secs(3);
    while host.capture_adapter_mut().is_err() {
        host.advance_explicit_captured_entry().unwrap();
        assert!(
            Instant::now() < end,
            "Original read attachment did not release"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(host.captured_entry_route().is_some());
}
pub(super) fn dispatch(host: &ViewerHost, intent: CaptureIntent) {
    assert!(matches!(
        host.capture_adapter_mut()
            .unwrap()
            .dispatch_to_execution(intent, RequestContext::default()),
        Ok(CaptureDispatchProgress::Submitted(_))
    ));
}
pub(super) fn terminal(host: &ViewerHost) {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        if matches!(
            host.capture_adapter_mut()
                .unwrap()
                .poll_execution()
                .unwrap(),
            CaptureExecutionProgress::TerminalReady { .. }
        ) {
            return;
        }
        assert!(
            Instant::now() < end,
            "Original guarded IPC terminal did not arrive"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
pub(super) fn consume_ack(host: &ViewerHost) -> CaptureExecutionAcknowledgement {
    let mut adapter = host.capture_adapter_mut().unwrap();
    adapter.consume_execution_terminal().unwrap();
    adapter.acknowledge_execution_terminal().unwrap()
}
pub(super) fn requests(recorded: &Arc<Mutex<Vec<Request>>>) -> Vec<Request> {
    recorded.lock().unwrap().clone()
}
