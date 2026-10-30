//! One separately assigned real-window child. No synthetic Some or manual renderer success.
use super::*;
use crate::{
    application::api::{ApplicationClient, Request, Response},
    assembly::meari::{
        native_cleanup_driver::CleanupDriverOutcome,
        native_execution_startup::NativeExecutionFailure,
        quarantine_surface::QuarantineChild,
        viewer_cleanup::TypedViewerCleanup,
        viewer_native_launch::no_media::{
            NoMediaFailureCleanup, prepare_no_media_product_for_test,
        },
    },
};
use std::{
    io,
    path::Path,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

const EXACT: &str = "assembly::meari::viewer_native_launch::no_media::tests::end_to_end::jobs_retained::checked_jobs_roundtrip_uses_real_retained_submission_and_owned_cleanup";
const MODE: &str = "jobs-retained-v1";
struct Recorded {
    actual: ApplicationClientHandle,
    calls: Arc<Mutex<Vec<Request>>>,
    replies: Arc<Mutex<Vec<Value>>>,
    listed: Arc<AtomicBool>,
}
impl ApplicationClient for Recorded {
    fn execute(&self, request: Request) -> Response {
        let list = request.command == "jobs.list";
        self.calls.lock().unwrap().push(request.clone());
        let response = self.actual.execute(request);
        self.replies
            .lock()
            .unwrap()
            .push(serde_json::to_value(&response).unwrap());
        if list && response.ok {
            self.listed.store(true, Ordering::Release);
        }
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
    record: Option<Value>,
}
impl std::fmt::Debug for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginalJobsCheckedChildFailure")
            .field("stage", &self.stage)
            .field("error", &self.error)
            .field("endpoint_retained", &self.endpoint.is_some())
            .field("startup_retained", &self.execution.is_some())
            .field("setup_retained", &self.setup.is_some())
            .field(
                "cleanup",
                &self.cleanup.as_ref().map(|outcome| &outcome.status),
            )
            .field("diagnostic", &self.diagnostic)
            .field("record", &self.record)
            .finish()
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
        record: None,
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
struct ChildRun {
    record: Value,
    cleanup: CleanupDriverOutcome<TypedViewerCleanup<QuarantineChild>>,
}

fn run_child(root: &Path) -> Result<ChildRun, Box<Failure>> {
    let started = Instant::now();
    let store = root.join("jobs");
    fs::create_dir(&store).map_err(|error| failure("explicit job root", error.to_string()))?;
    let jobs = crate::infrastructure::jobs::read_only(&store)
        .map_err(|error| failure("original read factory", error.message))?;
    let calls = Arc::new(Mutex::new(Vec::new()));
    let replies = Arc::new(Mutex::new(Vec::new()));
    let listed = Arc::new(AtomicBool::new(false));
    let client: ApplicationClientHandle = Arc::new(Recorded {
        actual: Arc::new(PlayerSdk::default().with_jobs(jobs)),
        calls: calls.clone(),
        replies: replies.clone(),
        listed: listed.clone(),
    });
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
            namespace: "jobs-checked-child".into(),
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
    let probe = Probe::for_jobs(listed);
    let _scope = Scope::enter(probe.clone());
    let prepared = match prepare_no_media_product_for_test(
        input,
        NoMediaFrameOptions {
            request_id: NonZeroU64::new(92).unwrap(),
            expires_at: started + Duration::from_secs(15),
            frame_budget: NonZeroUsize::new(64).unwrap(),
        },
    ) {
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
    // Same production blocking startup/run, then original typed cleanup before assertions.
    let owners = prepared.run();
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
    let cleanup = finish_typed_viewer(owners, cleanup_limits());
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
    let jobs_calls = calls
        .iter()
        .filter(|request| request.command.starts_with("jobs."))
        .collect::<Vec<_>>();
    let passed = run_ok
        && same_runtime
        && closed
        && probe.fault().is_none()
        && cleanup.evidence.error.is_none()
        && cleanup.status == ViewerCleanupStatus::WorkersQuiescent(ViewerNativeState::Closed)
        && jobs_calls.len() == 1
        && jobs_calls[0].command == "jobs.list"
        && trace_passes(&events);
    let record = json!({"schema":"viewer-no-media-inner-v1","test":EXACT,"mode":MODE,"scenario_passed":passed,"run_ok":run_ok,"same_runtime":same_runtime,"closed":closed,"probe_fault":format!("{:?}",probe.fault()),"cleanup_status":format!("{:?}",cleanup.status),"events":events,"requests":calls,"responses":replies,"media_executed":false,"elapsed_ms":started.elapsed().as_millis()});
    Ok(ChildRun { record, cleanup })
}
fn trace_passes(events: &[Value]) -> bool {
    let names = [
        "Bind",
        "ProofObserved",
        "JobsInitialFrame",
        "JobsEntered",
        "JobsReturnCommitted",
        "JobsRetainedFrame",
        "JobsOwnersReleased",
        "JobsRestored",
        "CloseIntent",
        "InnerClose",
        "RunReturned",
        "CleanupReturned",
    ];
    let mut ordered = Vec::new();
    for name in names {
        let matches = events
            .iter()
            .filter(|event| event["kind"] == name)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return false;
        }
        ordered.push(matches[0]);
    }
    if !ordered
        .windows(2)
        .all(|pair| pair[0]["sequence"].as_u64() < pair[1]["sequence"].as_u64())
    {
        return false;
    }
    let reflected = events
        .iter()
        .filter(|event| event["kind"] == "Reflection" && event["data"][6] == "observed_disabled")
        .collect::<Vec<_>>();
    let committed = &ordered[4]["data"][1];
    let submitted = &ordered[5]["data"];
    let same = ["host", "owner", "definition"].iter().all(|field| {
        committed[*field] == submitted[0][*field]
            && committed[*field] == submitted[1][*field]
            && committed[*field] == ordered[2]["data"][0][*field]
            && committed[*field] == ordered[3]["data"][0][*field]
            && committed[*field] == ordered[7]["data"][0][*field]
    });
    reflected.len() == 1
        && ordered[0]["data"][0] == true
        && reflected[0]["sequence"].as_u64() < ordered[2]["sequence"].as_u64()
        && same
        && submitted[0] == *committed
        && submitted[3] == true
        && ordered[3]["data"][2] == true
        && ordered[6]["data"] == true
        && close_intent_accepted(&ordered[8]["data"])
        && ordered[9]["data"][0] == "Complete"
        && ordered[9]["data"][1] == 0
        && close_window_matches(&ordered[1]["data"][1], &ordered[9]["data"][5])
}
fn write_record(root: &Path, record: &Value) -> io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root.join("inner-success.json"))?;
    serde_json::to_writer_pretty(&mut file, record).map_err(io::Error::other)?;
    file.flush()
}
#[test]
#[ignore = "Requires assigned single-heavy actual Native checked-window child; no media/visual acceptance"]
fn checked_jobs_roundtrip_uses_real_retained_submission_and_owned_cleanup()
-> Result<(), Box<CaseFailure>> {
    match std::env::var_os(CHILD_MODE) {
        None => supervision::parent_run(EXACT, MODE)
            .map_err(|error| Box::new(CaseFailure::Parent(error))),
        Some(mode) if mode == MODE => {
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
            match run_child(&root) {
                Ok(run) => {
                    let mut retained =
                        failure("checked child result", "scenario or diagnostic failed");
                    retained.cleanup = Some(run.cleanup);
                    retained.record = Some(run.record);
                    retained.diagnostic =
                        write_record(&root, retained.record.as_ref().unwrap()).err();
                    if retained.diagnostic.is_some()
                        || retained.record.as_ref().unwrap()["scenario_passed"] != true
                    {
                        return Err(Box::new(CaseFailure::Child(retained)));
                    }
                    Ok(())
                }
                Err(mut retained) => {
                    let record = json!({"schema":"viewer-no-media-inner-v1","test":EXACT,"mode":MODE,"scenario_passed":false,"stage":retained.stage,"error":retained.error});
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
