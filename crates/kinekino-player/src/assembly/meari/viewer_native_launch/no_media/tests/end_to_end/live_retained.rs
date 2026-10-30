//! One separately assigned real-window child. No synthetic Some or manual renderer success.
use super::*;
use crate::application::api::ProposalClient;
mod commit_journey;
use crate::{
    application::api::{ApplicationClient, Request, RequestContext, Response},
    assembly::meari::{
        native_cleanup_driver::CleanupDriverOutcome,
        native_execution_startup::NativeExecutionFailure,
        quarantine_surface::QuarantineChild,
        viewer_cleanup::TypedViewerCleanup,
        viewer_native_launch::no_media::{
            NoMediaFailureCleanup, prepare_no_media_live_for_test,
            prepare_no_media_product_for_test,
        },
    },
};
use std::{io, path::Path, sync::Mutex};

use crate::assembly::meari::{
    viewer_host::{TrustedLiveChoices, ViewerHost},
    viewer_native_probe::live_retained::{Case, Mode, ModelProofScope, ProofStage},
};
struct Recorded {
    actual: ApplicationClientHandle,
    calls: Arc<Mutex<Vec<Request>>>,
    replies: Arc<Mutex<Vec<Value>>>,
}
impl ApplicationClient for Recorded {
    fn execute(&self, request: Request) -> Response {
        self.calls.lock().unwrap().push(request.clone());
        let response = self.actual.execute(request);
        self.replies
            .lock()
            .unwrap()
            .push(serde_json::to_value(&response).unwrap());

        response
    }
}
struct Failure {
    stage: &'static str,
    error: String,
    endpoint: Option<LocalHost>,
    execution: Option<Box<NativeExecutionFailure>>,
    setup: Option<NoMediaFailureCleanup>,
    cleanup: Option<CleanupDriverOutcome<TypedViewerCleanup<QuarantineChild>>>,
    diagnostic: Option<io::Error>,
    checkpoint_error: Option<io::Error>,
    model_cancel_error: Option<crate::assembly::meari::viewer_host::ViewerHostError>,
    record: Option<Value>,
    seed: Option<Box<dyn std::any::Any>>,
}
impl std::fmt::Debug for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginalLiveCheckedChildFailure")
            .field("stage", &self.stage)
            .field("seed_error_retained", &self.seed.is_some())
            .field("error", &self.error)
            .field("endpoint_retained", &self.endpoint.is_some())
            .field("startup_retained", &self.execution.is_some())
            .field("setup_retained", &self.setup.is_some())
            .field(
                "cleanup",
                &self.cleanup.as_ref().map(|outcome| &outcome.status),
            )
            .field("diagnostic", &self.diagnostic)
            .field("checkpoint_error", &self.checkpoint_error)
            .field("model_cancel_error", &self.model_cancel_error)
            .field("record", &self.record)
            .finish()
    }
}
impl Drop for Failure {
    fn drop(&mut self) {
        // A failed test's termination must not silently run owner Drop as cleanup evidence.
        if let Some(owner) = self.endpoint.take() {
            std::mem::forget(owner);
        }
        if let Some(owner) = self.execution.take() {
            std::mem::forget(owner);
        }
        if let Some(owner) = self.setup.take() {
            std::mem::forget(owner);
        }
        if self.cleanup.as_ref().is_some_and(|cleanup| {
            cleanup.status != ViewerCleanupStatus::WorkersQuiescent(ViewerNativeState::Closed)
        }) && let Some(owner) = self.cleanup.take()
        {
            std::mem::forget(owner);
        }
    }
}
fn failure(stage: &'static str, error: impl Into<String>) -> Box<Failure> {
    Box::new(Failure {
        stage,
        error: error.into(),
        endpoint: None,
        execution: None,
        setup: None,
        cleanup: None,
        diagnostic: None,
        checkpoint_error: None,
        model_cancel_error: None,
        record: None,
        seed: None,
    })
}
enum CaseFailure {
    Parent(Box<RunFailure>),
    Child(Box<Failure>),
}
impl std::fmt::Debug for CaseFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parent(error) => std::fmt::Debug::fmt(error, f),
            Self::Child(error) => std::fmt::Debug::fmt(error, f),
        }
    }
}
impl Drop for CaseFailure {
    fn drop(&mut self) {
        if let Self::Parent(error) = self
            && let Some(owner) = error.owner.take()
        {
            std::mem::forget(owner);
        }
    }
}
struct ChildRun {
    record: Value,
    cleanup: CleanupDriverOutcome<TypedViewerCleanup<QuarantineChild>>,
}

fn run_child(
    root: &Path,
    case: Case,
    mode: Mode,
    exact: &'static str,
    label: &'static str,
    probe: std::rc::Rc<Probe>,
) -> Result<ChildRun, Box<Failure>> {
    run_child_with_fixture(root, case, mode, exact, label, probe, None)
}
#[derive(Clone, Copy)]
struct PersistedFixture {
    task: &'static str,
    proposal: &'static str,
    revision: u64,
}
#[derive(Clone, Copy)]
enum SourceFixture {
    Persisted(PersistedFixture),
    CurrentInput,
    CommitJourney,
}
const CUSTODY_SHA: &str = "220B57E00F3EB6D1A920F58D812C01E13731DA5EF54DD9C93F9B908EA6FDA811";
const CUSTODY_BYTES: u64 = 14983;
const FIXTURE_STAGE: &str = "runtime/quality-review/persisted-reopen-inputs-v1";
#[derive(serde::Deserialize)]
struct FixtureCustody {
    task: String,
    proposal_id: String,
    leaves: Vec<FixtureLeaf>,
    evidence: Vec<FixtureLog>,
}
#[derive(serde::Deserialize)]
struct FixtureLeaf {
    path: PathBuf,
    bytes: u64,
    sha256: String,
    revision: String,
}
#[derive(serde::Deserialize)]
struct FixtureLog {
    path: PathBuf,
    bytes: u64,
    sha256: String,
}
fn custody_failure<E: std::fmt::Debug + 'static>(stage: &'static str, error: E) -> Box<Failure> {
    let mut retained = failure(stage, format!("{error:?}"));
    retained.seed = Some(Box::new(error));
    retained
}
fn read_custody_file(
    path: &Path,
    expected_bytes: u64,
    expected_sha: &str,
) -> Result<Vec<u8>, Box<Failure>> {
    use crate::infrastructure::immutable_storage_paths as paths;
    use sha2::Digest;
    use std::io::Read;
    if expected_bytes > 4_194_304 {
        return Err(failure(
            "custody input",
            "fixed input exceeds original state bound",
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| failure("custody path", "parent missing"))?;
    let (_parent, _guards) =
        paths::root(parent).map_err(|error| custody_failure("custody directory", error))?;
    let file = paths::file(path).map_err(|error| custody_failure("custody regular file", error))?;
    if file
        .metadata()
        .map_err(|error| custody_failure("custody metadata", error))?
        .len()
        != expected_bytes
    {
        return Err(failure("custody input", "fixed byte length differs"));
    }
    let mut bytes = Vec::new();
    file.take(expected_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| custody_failure("custody read", error))?;
    let actual = format!("{:x}", sha2::Sha256::digest(&bytes));
    if bytes.len() as u64 != expected_bytes || !actual.eq_ignore_ascii_case(expected_sha) {
        return Err(failure("custody input", "fixed source SHA differs"));
    }
    Ok(bytes)
}
fn copy_persisted_fixture(
    store_root: &Path,
    spec: PersistedFixture,
) -> Result<(crate::domain::proposals::StoredProposal, Value), Box<Failure>> {
    use crate::domain::proposals::StoredProposal;
    use crate::infrastructure::immutable_storage_paths as paths;
    use std::io::Write;
    let cwd = std::env::current_dir().map_err(|error| custody_failure("custody cwd", error))?;
    let staging = cwd.join(FIXTURE_STAGE);
    let manifest = read_custody_file(&staging.join("manifest.json"), CUSTODY_BYTES, CUSTODY_SHA)?;
    let rows: Vec<FixtureCustody> = serde_json::from_slice(&manifest)
        .map_err(|error| custody_failure("custody manifest", error))?;
    let matches = rows
        .into_iter()
        .filter(|row| row.task == spec.task && row.proposal_id == spec.proposal)
        .collect::<Vec<_>>();
    let [custody] = matches.as_slice() else {
        return Err(failure("custody manifest", "task identity is not unique"));
    };
    let (_root, _guards) =
        paths::root(store_root).map_err(|error| custody_failure("custody destination", error))?;
    if fs::read_dir(store_root)
        .map_err(|error| custody_failure("custody destination", error))?
        .next()
        .is_some()
    {
        return Err(failure(
            "custody destination",
            "original store destination must be empty",
        ));
    }
    let mut snapshots = std::collections::BTreeMap::<u64, StoredProposal>::new();
    let mut raw = std::collections::BTreeMap::<u64, (String, Vec<u8>)>::new();
    for leaf in &custody.leaves {
        let revision = leaf
            .revision
            .parse::<u64>()
            .map_err(|error| custody_failure("custody revision", error))?;
        if !(1..=3).contains(&revision) || snapshots.contains_key(&revision) {
            return Err(failure(
                "custody revision",
                "fixed revision is duplicate or foreign",
            ));
        }
        let file_name = format!("{}.json", leaf.sha256.to_ascii_lowercase());
        if leaf.path.file_name().and_then(|name| name.to_str()) != Some(file_name.as_str()) {
            return Err(failure("custody leaf", "original SHA filename differs"));
        }
        let source = staging
            .join("leaves")
            .join(spec.task)
            .join(format!("r-{revision}"))
            .join(&file_name);
        let bytes = read_custody_file(&source, leaf.bytes, &leaf.sha256)?;
        let snapshot: StoredProposal = serde_json::from_slice(&bytes)
            .map_err(|error| custody_failure("custody stored snapshot", error))?;
        snapshot
            .validate()
            .map_err(|error| failure("custody canonical validation", error))?;
        if snapshot.document.proposal_id != spec.proposal
            || snapshot.document.draft_revision.0 != revision
        {
            return Err(failure(
                "custody snapshot",
                "fixed proposal identity/revision differs",
            ));
        }
        snapshots.insert(revision, snapshot);
        raw.insert(revision, (file_name, bytes));
    }
    if snapshots.len() != 3 {
        return Err(failure(
            "custody chain",
            "original three revision leaves required",
        ));
    }
    let mut log_values = std::collections::BTreeMap::<String, Value>::new();
    let mut log_hashes = Vec::new();
    for log in &custody.evidence {
        let name = log
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| failure("custody log", "original filename missing"))?;
        let bytes = read_custody_file(
            &staging.join("logs").join(spec.task).join(name),
            log.bytes,
            &log.sha256,
        )?;
        if name.ends_with(".json") || name.ends_with(".stdout") {
            let value: Value = serde_json::from_slice(&bytes)
                .map_err(|error| custody_failure("custody CLI record", error))?;
            log_values.insert(name.to_owned(), value);
        }
        log_hashes.push(json!({"name":name,"sha256":log.sha256,"bytes":log.bytes}));
    }
    let prefix = format!("gemma270-{}", spec.task);
    for (stage, command) in [
        ("technical-edit", "proposals.edit"),
        ("technical-accept", "proposals.accept"),
        ("restart-inspect", "proposals.inspect"),
    ] {
        let request = log_values
            .get(&format!("{prefix}-{stage}.json"))
            .ok_or_else(|| failure("custody CLI record", "request receipt missing"))?;
        let response = log_values
            .get(&format!("{prefix}-{stage}.stdout"))
            .ok_or_else(|| failure("custody CLI record", "original response missing"))?;
        if request["exit_code"] != 0
            || request["request"]["command"] != command
            || request["request"]["payload"]["proposal_id"] != spec.proposal
            || response["ok"] != true
            || !response["error"].is_null()
        {
            return Err(failure(
                "custody CLI record",
                "original successful command/identity differs",
            ));
        }
    }
    let value = |stage: &str| &log_values[&format!("{prefix}-{stage}.stdout")]["data"];
    if value("technical-edit")
        != &serde_json::to_value(&snapshots[&2].document)
            .map_err(|error| custody_failure("custody edit record", error))?
        || value("technical-accept")
            != &serde_json::to_value(&snapshots[&3].acceptance)
                .map_err(|error| custody_failure("custody accept record", error))?
        || value("restart-inspect")
            != &serde_json::to_value(&snapshots[&3])
                .map_err(|error| custody_failure("custody inspect record", error))?
    {
        return Err(failure(
            "custody CLI record",
            "persisted leaves differ from original CLI responses",
        ));
    }
    let proposal_dir = store_root.join(format!("p-{}", spec.proposal));
    fs::create_dir(&proposal_dir)
        .map_err(|error| custody_failure("custody new proposal directory", error))?;
    let _proposal_guard = paths::directory(&proposal_dir)
        .map_err(|error| custody_failure("custody proposal guard", error))?;
    let mut copied = Vec::new();
    for revision in 1..=spec.revision {
        let (name, bytes) = raw
            .get(&revision)
            .ok_or_else(|| failure("custody copy", "fixed revision missing"))?;
        let dir = proposal_dir.join(format!("r-{revision}"));
        fs::create_dir(&dir)
            .map_err(|error| custody_failure("custody new revision directory", error))?;
        let _guard = paths::directory(&dir)
            .map_err(|error| custody_failure("custody revision guard", error))?;
        let destination = dir.join(name);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .map_err(|error| custody_failure("custody new leaf", error))?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| custody_failure("custody copy", error))?;
        copied.push(json!({"revision":revision,"destination":destination,"original_sha_file":name,"bytes":bytes.len()}));
    }
    let expected = snapshots
        .remove(&spec.revision)
        .ok_or_else(|| failure("custody selection", "fixed selected revision missing"))?;
    Ok((
        expected,
        json!({"manifest_sha256":CUSTODY_SHA,"task":spec.task,"proposal_id":spec.proposal,"current_revision":spec.revision,"copied":copied,"original_cli_logs":log_hashes,"mutation_executed":false,"human_quality_verdict":false}),
    ))
}
fn run_child_with_fixture(
    root: &Path,
    case: Case,
    mode: Mode,
    exact: &'static str,
    label: &'static str,
    probe: std::rc::Rc<Probe>,
    fixture: Option<SourceFixture>,
) -> Result<ChildRun, Box<Failure>> {
    probe.checkpoint("setup.before");
    let started = Instant::now();
    let store = root.join("proposals");
    fs::create_dir(&store).map_err(|error| failure("explicit proposal root", error.to_string()))?;
    let input_fixture = matches!(fixture, Some(SourceFixture::CurrentInput));
    let journey_fixture = matches!(fixture, Some(SourceFixture::CommitJourney));
    let persisted = fixture
        .and_then(|fixture| match fixture {
            SourceFixture::Persisted(spec) => Some(spec),
            SourceFixture::CurrentInput | SourceFixture::CommitJourney => None,
        })
        .map(|spec| copy_persisted_fixture(&store, spec))
        .transpose()?;
    let store = crate::infrastructure::proposals::ProposalFiles::open(&store)
        .map_err(|error| failure("original proposal store", format!("{error:?}")))?;
    if let Some((expected, _)) = &persisted {
        use crate::application::proposals::ProposalStore;
        let loaded = store
            .load(&expected.document.proposal_id)
            .map_err(|error| custody_failure("original current fixture load", error))?;
        if serde_json::to_value(&loaded)
            .map_err(|error| custody_failure("current load encoding", error))?
            != serde_json::to_value(expected)
                .map_err(|error| custody_failure("fixture encoding", error))?
        {
            return Err(failure(
                "original current fixture load",
                "current SDK adapter differs from original persisted snapshot",
            ));
        }
        probe
            .configure_persisted_reopen(expected.clone())
            .map_err(|error| failure("persisted observation configuration", error))?;
    }
    let calls = Arc::new(Mutex::new(Vec::new()));
    let replies = Arc::new(Mutex::new(Vec::new()));
    let client: ApplicationClientHandle = Arc::new(Recorded {
        actual: Arc::new(PlayerSdk::default().with_proposal_store(store)),
        calls: calls.clone(),
        replies: replies.clone(),
    });
    let choices = if persisted.is_some() {
        None
    } else if mode == Mode::DefaultChooser {
        seed_default_chooser(&client, case)?;
        if input_fixture || journey_fixture {
            let expected = client
                .inspect_proposal(
                    RequestContext::default(),
                    crate::application::proposals::InspectProposal {
                        proposal_id: if case == Case::Document {
                            "journey-b"
                        } else {
                            "journey-a"
                        }
                        .into(),
                    },
                )
                .map_err(|error| custody_failure("input source setup inspection", error))?;
            if journey_fixture {
                probe
                    .configure_commit_journey(expected, calls.clone(), replies.clone())
                    .map_err(|error| failure("commit journey configuration", error))?;
            } else {
                probe
                    .configure_default_input(expected)
                    .map_err(|error| failure("input source configuration", error))?;
            }
        }
        None
    } else {
        let selected = match ViewerHost::seed_live_fixture(&client, case) {
            Ok(selected) => selected,
            Err(error) => {
                let mut retained = failure("original typed seed", format!("{error:?}"));
                retained.seed = Some(Box::new(error));
                return Err(retained);
            }
        };
        let choices = TrustedLiveChoices::from_trusted_local(
            vec![selected],
            NonZeroU64::new(7).unwrap(),
            NonZeroUsize::new(1).unwrap(),
        )
        .map_err(|error| {
            let mut retained = failure("explicit trusted fixture selection", format!("{error:?}"));
            retained.seed = Some(Box::new(error));
            retained
        })?;
        Some(choices)
    };
    let seed_calls = calls
        .lock()
        .map_err(|_| failure("seed transcript", "poisoned"))?
        .len();
    let endpoint = LocalHost::start(client.clone(), TransportOptions::default())
        .map_err(|error| failure("original endpoint", error.message))?;
    let schedule =
        HostScheduleLimits::new(Duration::from_millis(2), Duration::from_millis(250), 100).unwrap();
    let execution = match prepare_native_execution(
        client.clone(),
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 262144,
            max_total_request_bytes: 8 * 262144,
            max_terminal_bytes: 262144,
            max_total_terminal_bytes: 8 * 262144,
        },
        schedule,
    ) {
        Ok(owner) => owner,
        Err(error) => {
            let mut retained = failure("original Native execution", format!("{error:?}"));
            retained.endpoint = Some(endpoint);
            retained.execution = Some(error);
            return Err(retained);
        }
    };
    let input = ViewerLaunchInput {
        client,
        endpoint,
        execution,
        labels: StartLabels {
            connection: "관측 없음".into(),
            time: "관측 없음".into(),
        },
        document: DocumentOptions {
            namespace: "live-checked-child".into(),
            epoch: 1,
            source_generation: 1,
            viewport: Viewport {
                width: 960,
                height: 640,
                dark: true,
            },
        },
        host: ShellHostConfig {
            options: WindowOptions::default(),
            schedule,
            children: LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
        },
    };
    let _proof_scope = (mode != Mode::DefaultChooser).then(|| {
        ModelProofScope::enter(match mode {
            Mode::EntryProofModel => Some(ProofStage::Entry),
            Mode::ReturnProofModel => Some(ProofStage::Return),
            _ => None,
        })
    });
    let _scope = Scope::enter(probe.clone());
    probe.checkpoint("prepare.before");
    let frame_options = NoMediaFrameOptions {
        request_id: NonZeroU64::new(92).unwrap(),
        expires_at: started + Duration::from_secs(15),
        frame_budget: NonZeroUsize::new(64).unwrap(),
    };
    let preparation = if let Some(choices) = choices {
        prepare_no_media_live_for_test(input, frame_options, choices)
    } else {
        prepare_no_media_product_for_test(input, frame_options)
    };
    let prepared = match preparation {
        Ok(prepared) => prepared,
        Err(error) => {
            let cleanup = (*error).into_cleanup_parts().finish(cleanup_limits());
            let mut retained = failure(
                "original product NoMedia preparation",
                format!("{:?}", cleanup.cleanup.status),
            );
            retained.setup = Some(cleanup);
            return Err(retained);
        }
    };
    probe.checkpoint("prepare.after");
    // Same production blocking startup/run, then original typed cleanup before assertions.
    probe.checkpoint("run.before");
    let owners = prepared.run();
    probe.checkpoint("run.after");
    let (run_ok, same_runtime, closed) = match &owners {
        TypedViewerOwners::Returned(outcome) => (
            outcome.result.is_ok(),
            match (&outcome.runtime, &outcome.controller.runtime) {
                (Some(a), Some(b)) => std::rc::Rc::ptr_eq(a, b),
                _ => false,
            },
            outcome.runtime.as_ref().and_then(|runtime| {
                runtime
                    .try_borrow()
                    .ok()
                    .map(|runtime| runtime.document().lifecycle())
            }) == Some(HostLifecycle::Closed),
        ),
        _ => (false, false, false),
    };
    probe.record(Kind::RunReturned, &(run_ok, same_runtime, closed));
    probe.checkpoint("cleanup.before");
    let cleanup = finish_typed_viewer(owners, cleanup_limits());
    probe.checkpoint("cleanup.after");
    probe.record(Kind::CleanupReturned, &format!("{:?}", cleanup.status));
    let events=probe.inspect(|events|events.iter().flatten().map(|event|json!({"sequence":event.sequence,"kind":format!("{:?}",event.kind),"data":serde_json::from_slice::<Value>(event.payload()).unwrap()})).collect::<Vec<_>>()).unwrap_or_default();
    let calls = match calls.lock() {
        Ok(calls) => calls.clone(),
        Err(_) => {
            let mut retained = failure("request transcript", "request transcript poisoned");
            retained.cleanup = Some(cleanup);
            return Err(retained);
        }
    };
    let replies = match replies.lock() {
        Ok(replies) => replies.clone(),
        Err(_) => {
            let mut retained = failure("response transcript", "response transcript poisoned");
            retained.cleanup = Some(cleanup);
            return Err(retained);
        }
    };
    let notifications = probe.live_notifications();
    let typed_host_return_requested = events
        .iter()
        .any(|event| event["kind"] == "LiveHostReturnRequested");
    let entry_checked_frame_seen = events.iter().any(|event| event["kind"] == "LiveEntryFrame");
    let return_checked_frame_seen = events
        .iter()
        .any(|event| event["kind"] == "LiveViewingFrame");
    let source_return_action = events
        .iter()
        .any(|event| event["kind"] == "LiveSourceNavigationAction")
        && events
            .iter()
            .any(|event| event["kind"] == "LiveSourceNavigationAdmitted")
        && events
            .iter()
            .any(|event| event["kind"] == "LiveSourceReturnRequested");
    let passed = notifications.is_some()
        && run_ok
        && same_runtime
        && closed
        && probe.fault().is_none()
        && cleanup.evidence.error.is_none()
        && cleanup.status == ViewerCleanupStatus::WorkersQuiescent(ViewerNativeState::Closed)
        && if journey_fixture {
            commit_journey::transcript_passes(&calls, &replies, seed_calls)
        } else if input_fixture {
            input_transcript_passes(&calls, &replies, seed_calls)
        } else if let Some((expected, _)) = &persisted {
            persisted_transcript_passes(&calls, &replies, expected)
                && persisted_trace_passes(&events)
        } else if mode == Mode::DefaultChooser {
            chooser_transcript_passes(&calls, &replies, seed_calls, case, &events)
        } else {
            fixture_transcript_passes(&calls, &replies, mode)
        }
        && if journey_fixture {
            commit_journey::trace_passes(&events, case)
        } else if input_fixture {
            input_trace_passes(&events, case)
        } else {
            trace_passes(&events, mode)
        };
    let record = json!({"schema":"viewer-no-media-inner-v1","test":exact,"mode":label,"case":case,"fixture_mode":mode,"scenario_passed":passed,"run_ok":run_ok,"same_runtime":same_runtime,"closed":closed,"probe_fault":format!("{:?}",probe.fault()),"cleanup_status":format!("{:?}",cleanup.status),"events":events,"requests":calls,"responses":replies,"persisted_fixture":persisted.as_ref().map(|(_,report)|report),"persisted_current_load_verified":persisted.is_some(),"seed_request_count":seed_calls,"media_executed":false,"arm_notifications":notifications,"source_return_action":source_return_action,"typed_host_return_requested":typed_host_return_requested,"entry_checked_frame_seen":entry_checked_frame_seen,"return_checked_frame_seen":return_checked_frame_seen,"proof_fault_is_model":matches!(mode,Mode::EntryProofModel|Mode::ReturnProofModel),"default_chooser_mode":mode==Mode::DefaultChooser,"default_product_reachable":false,"default_launcher_reachable":false,"elapsed_ms":started.elapsed().as_millis()});
    Ok(ChildRun { record, cleanup })
}
fn input_transcript_passes(calls: &[Request], replies: &[Value], seed_count: usize) -> bool {
    use crate::domain::proposals::{ProposalDocument, StoredProposal};
    let commands = [
        "proposals.create",
        "proposals.create",
        "proposals.inspect",
        "proposals.list",
        "proposals.inspect",
        "proposals.inspect",
        "proposals.edit",
        "proposals.inspect",
    ];
    if seed_count != 3
        || calls.len() != commands.len()
        || replies.len() != commands.len()
        || !calls
            .iter()
            .zip(commands)
            .all(|(call, command)| call.command == command)
        || !calls.iter().zip(replies).all(|(call, reply)| {
            reply["request_id"] == call.context.request_id
                && reply["ok"] == true
                && reply["error"].is_null()
        })
    {
        return false;
    }
    let Ok(original) = serde_json::from_value::<StoredProposal>(replies[2]["data"].clone()) else {
        return false;
    };
    let Ok(saved) = serde_json::from_value::<StoredProposal>(replies[7]["data"].clone()) else {
        return false;
    };
    let Ok(edited) = serde_json::from_value::<ProposalDocument>(replies[6]["data"].clone()) else {
        return false;
    };
    // Readback observes the original UI save prerequisite and untouched fields, not SDK CAS/publication policy.
    [4, 5]
        .into_iter()
        .all(|index| replies[index]["data"] == replies[2]["data"])
        && [2, 4, 5, 6, 7]
            .into_iter()
            .all(|index| calls[index].payload["proposal_id"] == original.document.proposal_id)
        && original.validate().is_ok()
        && saved.validate().is_ok()
        && original.acceptance.is_none()
        && saved.acceptance.is_none()
        && saved.document.proposal_id == original.document.proposal_id
        && serde_json::to_value(&saved.document.source).ok()
            == serde_json::to_value(&original.document.source).ok()
        && serde_json::to_value(&saved.document.generated).ok()
            == serde_json::to_value(&original.document.generated).ok()
        && serde_json::to_value(&saved.document.draft).ok()
            == serde_json::to_value(&original.document.draft).ok()
        && serde_json::to_value(&saved.document).ok() == serde_json::to_value(&edited).ok()
}
fn input_trace_passes(events: &[Value], case: Case) -> bool {
    let of = |kind: &str| {
        events
            .iter()
            .filter(|event| event["kind"] == kind)
            .collect::<Vec<_>>()
    };
    let field_count = if case == Case::Card { 2 } else { 1 };
    if [
        "LiveEntryCommitted",
        "LiveEntryFrame",
        "LiveActivated",
        "LiveSaveRequested",
        "LiveSaveSettled",
        "LiveReturnCommitted",
        "LiveViewingFrame",
        "LiveOwnersReleased",
        "ChooserGeneration",
        "ChooserActiveIdentity",
        "LiveSourceNavigationAction",
        "LiveSourceNavigationAdmitted",
        "LiveSourceReturnRequested",
    ]
    .into_iter()
    .any(|kind| of(kind).len() != 1)
        || of("ChooserSourceAction").len() != 4
        || !of("LiveFault").is_empty()
        || !of("LiveHostReturnRequested").is_empty()
        || !of("LiveModelProofError").is_empty()
    {
        return false;
    }
    let displayed = of("LiveInputDisplayed");
    let restored = of("LiveInputRestored");
    if displayed.len() != field_count + 1
        || restored.len() != field_count + 1
        || displayed[0]["data"][0] != "input-initial"
        || restored[field_count]["data"][0] != "input-save-displayed"
    {
        return false;
    }
    (0..field_count).all(|index| {
        displayed[index + 1]["data"][0] == "input-changed"
            && displayed[index + 1]["data"][1] == index as u64
            && restored[index]["data"][0] == "input-restored"
            && restored[index]["data"][1] == index as u64
            && displayed[index + 1]["sequence"].as_u64() < restored[index]["sequence"].as_u64()
    }) && restored[field_count]["sequence"].as_u64()
        < of("LiveSourceNavigationAction")[0]["sequence"].as_u64()
}
fn persisted_trace_passes(events: &[Value]) -> bool {
    let displayed = events
        .iter()
        .filter(|event| event["kind"] == "LiveInputDisplayed")
        .collect::<Vec<_>>();
    displayed.len() == 2
        && displayed.iter().enumerate().all(|(cycle, event)| {
            event["data"][0] == "persisted-feedback"
                && event["data"][1] == cycle as u64
                && event["data"][3].is_object()
        })
}
fn persisted_transcript_passes(
    calls: &[Request],
    replies: &[Value],
    expected: &crate::domain::proposals::StoredProposal,
) -> bool {
    let commands = [
        "proposals.list",
        "proposals.inspect",
        "proposals.inspect",
        "proposals.list",
        "proposals.inspect",
        "proposals.inspect",
    ];
    if calls.len() != commands.len()
        || replies.len() != commands.len()
        || !calls
            .iter()
            .zip(commands)
            .all(|(call, name)| call.command == name)
        || !calls.iter().zip(replies).all(|(call, reply)| {
            reply["request_id"] == call.context.request_id
                && reply["ok"] == true
                && reply["error"].is_null()
        })
    {
        return false;
    }
    let Ok(snapshot) = serde_json::to_value(expected) else {
        return false;
    };
    [1, 2, 4, 5].into_iter().all(|index| {
        calls[index].payload["proposal_id"] == expected.document.proposal_id
            && replies[index]["data"] == snapshot
    })
}
fn seed_default_chooser(client: &ApplicationClientHandle, case: Case) -> Result<(), Box<Failure>> {
    use crate::{
        application::proposals::CreateProposal,
        domain::{api_values::DecimalU64, proposals::*},
    };
    use sha2::{Digest, Sha256};
    for (index, id) in ["journey-a", "journey-b"].into_iter().enumerate() {
        let cue = SourceSnapshot::Cue(CueSourceSnapshot {
            source_id: format!("journey-source-{index}"),
            source_revision: DecimalU64(1),
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
        let (source, payload, task) = match case {
            Case::Card => (
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
                ProposalTask::Cards,
            ),
            Case::Highlight => (
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
                ProposalTask::Highlights,
            ),
            _ if index == 0 => (
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
                ProposalTask::Correction,
            ),
            _ => {
                let content = "Edit this ASR draft. Correct punctuation only. Do not guess unclear words. JSON only: {\"proposed\":string,\"uncertain_words\":[string]}. Under100words. /no_think\n[\"첫 원문\",\"둘째 日本語\"]";
                let asset = json!({"messages":[{"role":"user","content":content}]}).to_string();
                let source = DocumentSourceSnapshot::from_asset(
                    "journey-document-b".into(),
                    DecimalU64(1),
                    format!("{:x}", Sha256::digest(asset.as_bytes())),
                    asset,
                )
                .map_err(|error| {
                    let mut retained = failure("document seed", format!("{error:?}"));
                    retained.seed = Some(Box::new(error));
                    retained
                })?;
                let refs = ["/0", "/1"]
                    .into_iter()
                    .map(|pointer| {
                        source.text_reference(pointer).ok_or_else(|| {
                            failure("document seed", "original text reference missing")
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                (
                    SourceSnapshot::Document(source),
                    ProposalPayload::DocumentCorrection {
                        edits: vec![
                            DocumentCorrection {
                                source_ref: refs[0].clone(),
                                text: "첫 수정".into(),
                            },
                            DocumentCorrection {
                                source_ref: refs[1].clone(),
                                text: "둘째 수정".into(),
                            },
                        ],
                    },
                    ProposalTask::Correction,
                )
            }
        };
        let raw_text = serde_json::to_string(&payload).map_err(|error| {
            let mut retained = failure("seed encode", format!("{error:?}"));
            retained.seed = Some(Box::new(error));
            retained
        })?;
        client
            .create_proposal(
                RequestContext::default(),
                CreateProposal {
                    proposal_id: id.into(),
                    source,
                    generated: GeneratedResult { task, raw_text },
                },
            )
            .map_err(|error| {
                let mut retained = failure("original typed chooser seed", format!("{error:?}"));
                retained.seed = Some(Box::new(error));
                retained
            })?;
    }
    Ok(())
}
fn chooser_transcript_passes(
    calls: &[Request],
    replies: &[Value],
    seed_count: usize,
    case: Case,
    events: &[Value],
) -> bool {
    use crate::application::launch::native_proposal::NativeProposalSelection;
    use crate::domain::proposals::{ProposalDocument, ProposalTask, StoredProposal};
    if seed_count != 2
        || calls.len() != replies.len()
        || !calls.iter().zip(replies).all(|(call, reply)| {
            reply["request_id"] == call.context.request_id
                && reply["ok"] == true
                && reply["error"].is_null()
        })
    {
        return false;
    }
    if !calls[..seed_count]
        .iter()
        .all(|call| call.command == "proposals.create")
    {
        return false;
    }
    let journey = &calls[seed_count..];
    let expected = [
        "proposals.list",
        "proposals.inspect",
        "proposals.inspect",
        "proposals.list",
        "proposals.inspect",
        "proposals.inspect",
    ];
    if journey.len() != expected.len()
        || !journey
            .iter()
            .zip(expected)
            .all(|(call, command)| call.command == command)
    {
        return false;
    }
    let task = match case {
        Case::Card => ProposalTask::Cards,
        Case::Highlight => ProposalTask::Highlights,
        _ => ProposalTask::Correction,
    };
    for (cycle, (offset, id)) in [(0, "journey-a"), (3, "journey-b")].into_iter().enumerate() {
        let identity = events.iter().find(|event| {
            event["kind"] == "ChooserActiveIdentity" && event["data"]["cycle"] == cycle
        });
        let Some(identity) = identity else {
            return false;
        };
        let Ok(selected) = serde_json::from_value::<NativeProposalSelection>(
            identity["data"]["input_selection"].clone(),
        ) else {
            return false;
        };
        let Ok(seed) = serde_json::from_value::<ProposalDocument>(replies[cycle]["data"].clone())
        else {
            return false;
        };
        if selected.proposal_id != seed.proposal_id
            || selected.document_revision != seed.draft_revision
            || selected.source_id != seed.source.source_id()
            || selected.source_revision != seed.source.source_revision()
            || selected.source_sha256 != seed.source.source_sha256()
        {
            return false;
        }
        let documents = [seed_count + offset + 1, seed_count + offset + 2]
            .map(|i| serde_json::from_value::<StoredProposal>(replies[i]["data"].clone()));
        let [Ok(prepared), Ok(current)] = documents else {
            return false;
        };
        if prepared.validate().is_err()
            || current.validate().is_err()
            || prepared.document.proposal_id != id
            || current.document.proposal_id != id
            || current.document.generated.task != task
            || prepared.acceptance.is_some()
            || current.acceptance.is_some()
            || serde_json::to_value(&prepared.document).ok()
                != serde_json::to_value(&current.document).ok()
        {
            return false;
        }
    }
    true
}
fn chooser_trace_passes(events: &[Value]) -> bool {
    let of = |name: &str| {
        events
            .iter()
            .filter(|event| event["kind"] == name)
            .collect::<Vec<_>>()
    };
    if of("LiveEntryCommitted").len() != 2
        || of("LiveEntryFrame").len() != 2
        || of("LiveActivated").len() != 2
        || of("LiveReturnCommitted").len() != 1
        || of("LiveViewingFrame").len() != 1
        || of("ChooserJourneyComplete").len() != 1
        || of("LiveSourceNavigationAction").len() != 1
        || of("LiveSourceNavigationAdmitted").len() != 1
        || of("LiveSourceReturnRequested").len() != 1
        || of("ChooserGeneration").len() != 2
        || of("ChooserActiveIdentity").len() != 2
        || of("ChooserSourceAction").len() != 8
        || !of("LiveFault").is_empty()
        || !of("LiveHostReturnRequested").is_empty()
        || !of("LiveSaveRequested").is_empty()
        || !of("LiveModelProofError").is_empty()
    {
        return false;
    }
    let mut expected = vec!["Bind", "ProofObserved"];
    for cycle in 0..2 {
        expected.extend([
            "ChooserSourceAction",
            "ChooserGeneration",
            "ChooserSourceAction",
            "ChooserSourceAction",
            "ChooserSourceAction",
            "LiveEntryCommitted",
            "LiveEntryFrame",
            "LiveActivated",
        ]);
        if cycle == 0 {
            expected.extend([
                "LiveSourceNavigationAction",
                "LiveSourceNavigationAdmitted",
                "LiveSourceReturnRequested",
                "LiveReturnCommitted",
                "LiveViewingFrame",
            ])
        }
    }
    expected.extend([
        "ChooserJourneyComplete",
        "CloseIntent",
        "InnerClose",
        "RunReturned",
        "CleanupReturned",
    ]);
    let mut remaining = expected.into_iter();
    let mut wanted = remaining.next();
    for event in events {
        if wanted.is_some_and(|kind| event["kind"] == kind) {
            wanted = remaining.next();
        }
    }
    wanted.is_none()
}
/// Save is an original navigation prerequisite, not another GUI CAS acceptance test.
fn fixture_transcript_passes(calls: &[Request], replies: &[Value], mode: Mode) -> bool {
    use crate::domain::proposals::{ProposalDocument, StoredProposal};
    let expected: &[&str] = if mode == Mode::EntryProofModel {
        &["proposals.create", "proposals.inspect"]
    } else {
        &[
            "proposals.create",
            "proposals.inspect",
            "proposals.edit",
            "proposals.inspect",
        ]
    };
    if calls.len() != expected.len()
        || replies.len() != expected.len()
        || !calls
            .iter()
            .zip(expected)
            .all(|(call, command)| call.command == *command)
        || !calls.iter().zip(replies).all(|(call, reply)| {
            reply["request_id"] == call.context.request_id
                && reply["ok"] == true
                && reply["error"].is_null()
                && !reply["data"].is_null()
        })
    {
        return false;
    }
    let Ok(initial) = serde_json::from_value::<StoredProposal>(replies[1]["data"].clone()) else {
        return false;
    };
    if initial.validate().is_err() || initial.acceptance.is_some() {
        return false;
    }
    if mode == Mode::EntryProofModel {
        return true;
    }
    let Ok(edited) = serde_json::from_value::<ProposalDocument>(replies[2]["data"].clone()) else {
        return false;
    };
    let Ok(saved) = serde_json::from_value::<StoredProposal>(replies[3]["data"].clone()) else {
        return false;
    };
    saved.validate().is_ok()
        && saved.acceptance.is_none()
        && saved.document.draft_revision.0 > initial.document.draft_revision.0
        && saved.document.proposal_id == initial.document.proposal_id
        && serde_json::to_value(&saved.document.source).ok()
            == serde_json::to_value(&initial.document.source).ok()
        && serde_json::to_value(&saved.document.generated).ok()
            == serde_json::to_value(&initial.document.generated).ok()
        && serde_json::to_value(&saved.document.draft).ok()
            == serde_json::to_value(&initial.document.draft).ok()
        && serde_json::to_value(&saved.document).ok() == serde_json::to_value(&edited).ok()
}
fn trace_passes(events: &[Value], mode: Mode) -> bool {
    if mode == Mode::DefaultChooser {
        return chooser_trace_passes(events);
    }
    let one = |name: &str| {
        let matches: Vec<_> = events
            .iter()
            .filter(|event| event["kind"] == name)
            .collect();
        (matches.len() == 1).then(|| matches[0])
    };
    let model = matches!(mode, Mode::EntryProofModel | Mode::ReturnProofModel);
    let names: &[&str] = if mode == Mode::EntryProofModel {
        &[
            "Bind",
            "ProofObserved",
            "LiveInitialFrame",
            "LiveModelProofError",
            "LiveEntryCommitted",
            "LiveModelRouteCancelled",
            "CloseIntent",
            "InnerClose",
            "RunReturned",
            "CleanupReturned",
        ]
    } else if mode == Mode::ReturnProofModel {
        &[
            "Bind",
            "ProofObserved",
            "LiveInitialFrame",
            "LiveEntryCommitted",
            "LiveEntryFrame",
            "LiveActivated",
            "LiveInputDisplayed",
            "LiveInputRestored",
            "LiveSaveRequested",
            "LiveSaveSettled",
            "LiveHostReturnRequested",
            "LiveModelProofError",
            "LiveReturnCommitted",
            "LiveModelRouteCancelled",
            "CloseIntent",
            "InnerClose",
            "RunReturned",
            "CleanupReturned",
        ]
    } else if mode == Mode::SourceReturn {
        &[
            "Bind",
            "ProofObserved",
            "LiveInitialFrame",
            "LiveEntryCommitted",
            "LiveEntryFrame",
            "LiveActivated",
            "LiveInputDisplayed",
            "LiveInputRestored",
            "LiveSaveRequested",
            "LiveSaveSettled",
            "LiveSourceNavigationRedraw",
            "LiveSourceNavigationAction",
            "LiveSourceNavigationAdmitted",
            "LiveSourceReturnRequested",
            "LiveReturnCommitted",
            "LiveViewingFrame",
            "LiveOwnersReleased",
            "CloseIntent",
            "InnerClose",
            "RunReturned",
            "CleanupReturned",
        ]
    } else {
        &[
            "Bind",
            "ProofObserved",
            "LiveInitialFrame",
            "LiveEntryCommitted",
            "LiveEntryFrame",
            "LiveActivated",
            "LiveInputDisplayed",
            "LiveInputRestored",
            "LiveSaveRequested",
            "LiveSaveSettled",
            "LiveHostReturnRequested",
            "LiveReturnCommitted",
            "LiveViewingFrame",
            "LiveOwnersReleased",
            "CloseIntent",
            "InnerClose",
            "RunReturned",
            "CleanupReturned",
        ]
    };
    let Some(ordered) = names
        .iter()
        .map(|name| one(name))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    if !ordered
        .windows(2)
        .all(|pair| pair[0]["sequence"].as_u64() < pair[1]["sequence"].as_u64())
    {
        return false;
    }
    if events.iter().any(|event| event["kind"] == "LiveFault") {
        return false;
    }
    if mode == Mode::SourceReturn {
        if one("LiveHostReturnRequested").is_some() {
            return false;
        }
    } else if events.iter().any(|event| {
        matches!(
            event["kind"].as_str(),
            Some(
                "LiveSourceNavigationAction"
                    | "LiveSourceNavigationRedraw"
                    | "LiveSourceNavigationAdmitted"
                    | "LiveSourceReturnRequested"
            )
        )
    }) {
        return false;
    }
    if !one("CloseIntent").is_some_and(|event| event["data"]["Ok"].as_bool().is_some()) {
        return false;
    }
    if mode != Mode::EntryProofModel {
        let Some(wake) = one("LiveSaveObservationWake") else {
            return false;
        };
        if !one("LiveSaveRequested")
            .is_some_and(|save| save["sequence"].as_u64() < wake["sequence"].as_u64())
            || !one("LiveSaveSettled")
                .is_some_and(|settled| wake["sequence"].as_u64() < settled["sequence"].as_u64())
        {
            return false;
        }
    } else if events.iter().any(|event| {
        matches!(
            event["kind"].as_str(),
            Some("LiveSaveRequested" | "LiveSaveObservationWake" | "LiveSaveSettled")
        )
    }) {
        return false;
    }
    if model {
        let stage = if mode == Mode::EntryProofModel {
            "Entry"
        } else {
            "Return"
        };
        return one("LiveModelRouteCancelled")
            .is_some_and(|event| event["data"] == json!([stage, true]))
            && one("LiveModelProofError").is_some_and(|event| event["data"][0] == stage)
            && one(if mode == Mode::EntryProofModel {
                "LiveEntryCommitted"
            } else {
                "LiveReturnCommitted"
            })
            .is_some_and(|event| event["data"][3] == false)
            && one("LiveViewingFrame").is_none();
    }
    let Some(wake) = one("LiveWakeCounts") else {
        return false;
    };
    if events.iter().any(|event| {
        matches!(
            event["kind"].as_str(),
            Some("LiveModelRouteCancelled" | "LiveModelProofError")
        )
    }) {
        return false;
    }
    let counts = if mode == Mode::WakeFailure {
        json!([2, 1, 1, 1])
    } else {
        json!([2, 2, 2, 0])
    };
    let retained_lineage = |event: &Value| {
        ["host", "owner", "definition"]
            .iter()
            .all(|field| event["data"][1][*field] == event["data"][2][*field])
            && event["data"][1]["document"]
                .as_u64()
                .zip(event["data"][2]["document"].as_u64())
                .is_some_and(|(committed, submitted)| submitted >= committed)
            && event["data"][4] == true
    };
    wake["data"] == counts
        && (mode != Mode::WakeFailure
            || one("LiveWakeError").is_some_and(|event| {
                event["data"] == json!(["Internal", "fixture original route notifier fail-once"])
            }))
        && (mode != Mode::ArmBusy
            || (one("LiveArmBusy").is_some_and(|event| event["data"] == true)
                && one("LiveArmRecovered").is_some_and(|event| event["data"] == true)))
        && one("LiveEntryFrame").is_some_and(retained_lineage)
        && one("LiveViewingFrame").is_some_and(retained_lineage)
}
fn write_record(root: &Path, record: &Value) -> io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root.join("inner-success.json"))?;
    serde_json::to_writer_pretty(&mut file, record).map_err(io::Error::other)?;
    file.flush()
}
fn run_case(
    case: Case,
    mode: Mode,
    exact: &'static str,
    label: &'static str,
) -> Result<(), Box<CaseFailure>> {
    run_case_with_fixture(case, mode, exact, label, None)
}
fn run_case_with_fixture(
    case: Case,
    mode: Mode,
    exact: &'static str,
    label: &'static str,
    fixture: Option<PersistedFixture>,
) -> Result<(), Box<CaseFailure>> {
    run_case_source(
        case,
        mode,
        exact,
        label,
        fixture.map(SourceFixture::Persisted),
    )
}
fn run_case_source(
    case: Case,
    mode: Mode,
    exact: &'static str,
    label: &'static str,
    fixture: Option<SourceFixture>,
) -> Result<(), Box<CaseFailure>> {
    match std::env::var_os(CHILD_MODE) {
        None => supervision::parent_run(exact, label)
            .map_err(|error| Box::new(CaseFailure::Parent(error))),
        Some(child_mode) if child_mode == label => {
            let root = PathBuf::from(std::env::var_os(CHILD_EVIDENCE).ok_or_else(|| {
                Box::new(CaseFailure::Child(failure(
                    "child evidence",
                    "missing original evidence",
                )))
            })?);
            if !root.is_absolute() || !root.is_dir() {
                return Err(Box::new(CaseFailure::Child(failure(
                    "child evidence",
                    "invalid original evidence root",
                ))));
            }
            let probe = Probe::for_live(case, mode);
            probe.enable_checkpoints();
            let outcome = if fixture.is_some() {
                run_child_with_fixture(&root, case, mode, exact, label, probe.clone(), fixture)
            } else {
                run_child(&root, case, mode, exact, label, probe.clone())
            };
            let checkpoint_error = probe.take_checkpoint_error();
            let model_cancel_error = probe.take_live_cancel_error();
            match outcome {
                Ok(run) => {
                    let mut retained =
                        failure("checked child result", "scenario or diagnostic failed");
                    retained.cleanup = Some(run.cleanup);
                    retained.checkpoint_error = checkpoint_error;
                    retained.model_cancel_error = model_cancel_error;
                    retained.record = Some(run.record);
                    retained.diagnostic =
                        write_record(&root, retained.record.as_ref().unwrap()).err();
                    if retained.diagnostic.is_some()
                        || retained.checkpoint_error.is_some()
                        || retained.model_cancel_error.is_some()
                        || retained.record.as_ref().unwrap()["scenario_passed"] != true
                    {
                        return Err(Box::new(CaseFailure::Child(retained)));
                    }
                    Ok(())
                }
                Err(mut retained) => {
                    retained.checkpoint_error = checkpoint_error;
                    retained.model_cancel_error = model_cancel_error;
                    let record = json!({"schema":"viewer-no-media-inner-v1","test":exact,"mode":label,"scenario_passed":false,"stage":retained.stage,"error":retained.error});
                    retained.diagnostic = write_record(&root, &record).err();
                    retained.record = Some(record);
                    Err(Box::new(CaseFailure::Child(retained)))
                }
            }
        }
        Some(_) => Err(Box::new(CaseFailure::Child(failure(
            "child mode",
            "foreign scenario mode",
        )))),
    }
}

macro_rules! live_case {
    ($name:ident, $case:expr, $mode:expr, $label:literal) => {
        #[test]
        #[ignore = "Assigned original NoMedia supervised checked-window fixture only; no visual acceptance"]
        fn $name() -> Result<(), Box<CaseFailure>> {
            const EXACT: &str = concat!("assembly::meari::viewer_native_launch::no_media::tests::end_to_end::live_retained::", stringify!($name));
            run_case($case, $mode, EXACT, $label)
        }
    };
}
live_case!(
    checked_card_source_entry_input_and_host_return,
    Case::Card,
    Mode::Positive,
    "live-card-v1"
);
live_case!(
    checked_highlight_source_entry_input_and_host_return,
    Case::Highlight,
    Mode::Positive,
    "live-highlight-v1"
);
live_case!(
    checked_cue_source_entry_input_and_host_return,
    Case::Cue,
    Mode::Positive,
    "live-cue-v1"
);
live_case!(
    checked_document_source_entry_input_and_host_return,
    Case::Document,
    Mode::Positive,
    "live-document-v1"
);
live_case!(
    checked_arm_busy_preserves_original_proof_without_recommit,
    Case::Highlight,
    Mode::ArmBusy,
    "live-arm-busy-v1"
);
live_case!(
    checked_route_notify_failure_is_distinct_from_arm_failure,
    Case::Highlight,
    Mode::WakeFailure,
    "live-wake-failure-v1"
);
live_case!(
    model_entry_proof_error_retains_real_commit_and_closes,
    Case::Highlight,
    Mode::EntryProofModel,
    "live-entry-proof-model-v1"
);
live_case!(
    model_return_proof_error_retains_new_viewing_commit_and_closes,
    Case::Highlight,
    Mode::ReturnProofModel,
    "live-return-proof-model-v1"
);
live_case!(
    checked_card_actual_source_navigation_returns_to_viewing,
    Case::Card,
    Mode::SourceReturn,
    "source-return-card-v1"
);
live_case!(
    checked_highlight_actual_source_navigation_returns_to_viewing,
    Case::Highlight,
    Mode::SourceReturn,
    "source-return-highlight-v1"
);
live_case!(
    checked_cue_actual_source_navigation_returns_to_viewing,
    Case::Cue,
    Mode::SourceReturn,
    "source-return-cue-v1"
);
live_case!(
    checked_document_actual_source_navigation_returns_to_viewing,
    Case::Document,
    Mode::SourceReturn,
    "source-return-document-v1"
);
live_case!(
    default_chooser_card_actual_source_return_and_fresh_reopen,
    Case::Card,
    Mode::DefaultChooser,
    "default-chooser-card-d-v1"
);
live_case!(
    default_chooser_highlight_actual_source_return_and_fresh_reopen,
    Case::Highlight,
    Mode::DefaultChooser,
    "default-chooser-highlight-d-v1"
);
live_case!(
    default_chooser_cue_to_document_actual_source_return_and_fresh_reopen,
    Case::Cue,
    Mode::DefaultChooser,
    "default-chooser-correction-d-v1"
);

macro_rules! persisted_case {
    ($name:ident,$case:expr,$task:literal,$proposal:literal,$revision:literal,$label:literal)=>{
        #[test]
        #[ignore="Assigned immutable persisted-result actual source fixture only; no mutation or visual acceptance"]
        fn $name()->Result<(),Box<CaseFailure>> {
            const EXACT:&str=concat!("assembly::meari::viewer_native_launch::no_media::tests::end_to_end::live_retained::",stringify!($name));
            run_case_with_fixture($case,Mode::DefaultChooser,EXACT,$label,Some(PersistedFixture {task:$task,proposal:$proposal,revision:$revision}))
        }
    };
}
persisted_case!(
    persisted_card_saved_result_reopens_same_proposal_with_current_feedback,
    Case::Card,
    "cards",
    "actual-gemma270-cards",
    2,
    "persisted-card-saved-b-v1"
);
persisted_case!(
    persisted_highlight_acceptance_reopens_same_proposal_with_output_revision,
    Case::Highlight,
    "highlights",
    "actual-gemma270-highlights",
    3,
    "persisted-highlight-accepted-b-v1"
);
persisted_case!(
    persisted_document_correction_reopens_same_proposal_with_exact_ref_feedback,
    Case::Document,
    "correction",
    "actual-gemma270-correction",
    3,
    "persisted-document-accepted-b-v1"
);

macro_rules! current_input_case {
    ($name:ident, $case:expr, $label:literal) => {
        #[test]
        #[ignore = "Assigned default chooser current Text/Action source fixture; no physical input or acceptance"]
        fn $name() -> Result<(), Box<CaseFailure>> {
            const EXACT: &str = concat!("assembly::meari::viewer_native_launch::no_media::tests::end_to_end::live_retained::", stringify!($name));
            run_case_source($case, Mode::DefaultChooser, EXACT, $label, Some(SourceFixture::CurrentInput))
        }
    };
}
current_input_case!(
    default_chooser_card_current_front_back_input_preserves_selected_refs,
    Case::Card,
    "default-input-card-i1-v1"
);
current_input_case!(
    default_chooser_highlight_current_title_input_preserves_selected_refs,
    Case::Highlight,
    "default-input-highlight-i2-v1"
);
current_input_case!(
    default_chooser_cue_current_draft_input_preserves_selected_ref,
    Case::Cue,
    "default-input-cue-i3-v1"
);
current_input_case!(
    default_chooser_document_current_draft_input_preserves_exact_ref,
    Case::Document,
    "default-input-document-i4-v1"
);

macro_rules! commit_journey_case {
    ($name:ident, $case:expr, $label:literal) => {
        #[test]
        #[ignore = "Separate allocation: original default chooser Text/Save/Accept/reopen connection; no physical or visual acceptance"]
        fn $name() -> Result<(), Box<CaseFailure>> {
            const EXACT: &str = concat!("assembly::meari::viewer_native_launch::no_media::tests::end_to_end::live_retained::", stringify!($name));
            run_case_source($case, Mode::DefaultChooser, EXACT, $label, Some(SourceFixture::CommitJourney))
        }
    };
}
commit_journey_case!(
    default_chooser_card_edit_save_explicit_accept_reopens_same_item,
    Case::Card,
    "default-commit-card-j1-v1"
);
commit_journey_case!(
    default_chooser_highlight_edit_save_explicit_accept_reopens_same_item,
    Case::Highlight,
    "default-commit-highlight-j2-v1"
);
commit_journey_case!(
    default_chooser_cue_edit_save_explicit_accept_reopens_same_item,
    Case::Cue,
    "default-commit-cue-j3-v1"
);
commit_journey_case!(
    default_chooser_document_edit_save_explicit_accept_reopens_same_item,
    Case::Document,
    "default-commit-document-j4-v1"
);
