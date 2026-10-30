//! Public Viewer entry points, original startup binding, and original owned cleanup.
mod jobs_retained;
mod live_retained;
mod supervision;
use super::super::{NoMediaFrameOptions, prepare_no_media_viewer_launch};
use crate::{
    application::{
        api::{ApplicationClientHandle, PlayerSdk},
        sdk_execution::ExecutionBudget,
    },
    assembly::meari::{
        native_execution_startup::prepare_native_execution,
        player_shell_document::DocumentOptions,
        player_shell_host::ShellHostConfig,
        viewer_cleanup::{
            TypedViewerOwners, ViewerCleanupLimits, ViewerCleanupStatus, ViewerNativeState,
        },
        viewer_native_launch::{ViewerLaunchInput, finish_typed_viewer},
        viewer_native_probe::{Kind, Probe, Scope},
    },
    infrastructure::local_host::{LocalHost, TransportOptions},
    presentation::web_ui_adapter::player_shell::document::StartLabels,
};
use meari_native::{
    application::{WindowOptions, window_lifecycle::LifecycleLimits},
    domain::Viewport,
    infrastructure::native_host::{HostLifecycle, HostScheduleLimits},
};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    num::{NonZeroU64, NonZeroUsize},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use supervision::{CHILD_EVIDENCE, CHILD_MODE, RunFailure};

const NORMAL: &str = "assembly::meari::viewer_native_launch::no_media::tests::end_to_end::public_viewer_consumes_frame_completion_and_closes_children_first";
const CHANGED: &str = "assembly::meari::viewer_native_launch::no_media::tests::end_to_end::changed_pending_document_rejects_original_bind_and_retains_owners";

fn run_inner(changed: bool, exact: &str, mode: &str) -> Value {
    let started = Instant::now();
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
    let input = ViewerLaunchInput {
        client,
        endpoint,
        execution,
        labels: StartLabels {
            connection: "관측 없음".into(),
            time: "관측 없음".into(),
        },
        document: DocumentOptions {
            namespace: "viewer-no-media-e2e".into(),
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
    let probe = Probe::new();
    let _scope = Scope::enter(probe.clone());
    let prepared = match prepare_no_media_viewer_launch(
        input,
        NoMediaFrameOptions {
            request_id: NonZeroU64::new(91).unwrap(),
            expires_at: started + Duration::from_secs(15),
            frame_budget: NonZeroUsize::new(64).unwrap(),
        },
    ) {
        Ok(prepared) => prepared,
        Err(failure) => {
            let cleanup = (*failure).into_cleanup_parts().finish(cleanup_limits());
            panic!(
                "valid Viewer preparation failed; original cleanup status {:?}",
                cleanup.cleanup.status
            );
        }
    };
    let before = prepared
        .controller
        .pending_document_for_test()
        .and_then(|document| document.stamp().ok());
    let Some(before) = before else {
        let cleanup = finish_typed_viewer(prepared.into_unrun(), cleanup_limits());
        return json!({"schema":"viewer-no-media-inner-v1","test":exact,"mode":mode,
            "scenario_passed":false,"stage":"pending stamp unavailable",
            "cleanup_status":format!("{:?}",cleanup.status)});
    };
    let mutation = if changed {
        let document = prepared.controller.pending_document_for_test().unwrap();
        let reprojected =
            document.try_reproject(&before, |_| Ok::<(), std::convert::Infallible>(()));
        let (_, receipt) = match reprojected {
            Ok(receipt) => receipt,
            Err(error) => {
                let error = format!("{error:?}");
                let cleanup = finish_typed_viewer(prepared.into_unrun(), cleanup_limits());
                return json!({"schema":"viewer-no-media-inner-v1","test":exact,"mode":mode,
                    "scenario_passed":false,"stage":"pending reproject failed","error":error,
                    "cleanup_status":format!("{:?}",cleanup.status)});
            }
        };
        Some(
            json!({"before":format!("{:?}",receipt.before()),"after":format!("{:?}",receipt.after())}),
        )
    } else {
        None
    };
    // Production public blocking run, never a substitute controller.
    let owners = prepared.run();
    let (run_ok, same_runtime, lifecycle, bind_failure, retained_stamp, remaining) = match &owners {
        TypedViewerOwners::Returned(outcome) => {
            let same = match (&outcome.runtime, &outcome.controller.runtime) {
                (Some(a), Some(b)) => std::rc::Rc::ptr_eq(a, b),
                _ => false,
            };
            let lifecycle = outcome.runtime.as_ref().and_then(|runtime| {
                runtime
                    .try_borrow()
                    .ok()
                    .map(|runtime| runtime.document().lifecycle())
            });
            let failure = outcome.controller.frame_setup_failure.as_ref();
            let remaining = failure
                .and_then(|failure| failure.preparation.children().report().ok())
                .map(|report| report.remaining);
            let retained_stamp = outcome
                .controller
                .pending_document_for_test()
                .and_then(|document| document.stamp().ok());
            (
                outcome.result.is_ok(),
                same,
                lifecycle,
                failure.is_some(),
                retained_stamp,
                remaining,
            )
        }
        _ => (false, false, None, false, None, None),
    };
    probe.record(
        Kind::RunReturned,
        &(run_ok, same_runtime, format!("{lifecycle:?}")),
    );
    // Always finish original owners before assertions, including instrumentation failures.
    let cleanup = finish_typed_viewer(owners, cleanup_limits());
    probe.record(Kind::CleanupReturned, &format!("{:?}", cleanup.status));
    let events = probe
        .inspect(|events| {
            events
                .iter()
                .flatten()
                .map(|event| {
                    json!({
                        "sequence":event.sequence,"kind":format!("{:?}",event.kind),
                        "data":serde_json::from_slice::<Value>(event.payload()).unwrap(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut passed = probe.fault().is_none() && cleanup.evidence.error.is_none();
    if changed {
        passed &= !run_ok
            && bind_failure
            && remaining == Some(0)
            && retained_stamp
                .as_ref()
                .is_some_and(|stamp| stamp.document_revision == before.document_revision + 1)
            && cleanup.status
                == ViewerCleanupStatus::WorkersQuiescent(ViewerNativeState::Unconfirmed)
            && events
                .iter()
                .filter(|event| {
                    event["kind"] == "Bind"
                        && event["data"][0] == false
                        && event["data"][2] == "original frame request document changed before bind"
                })
                .count()
                == 1
            && !events
                .iter()
                .any(|event| event["kind"] == "ProofObserved" || event["kind"] == "Reflection");
    } else {
        passed &= run_ok
            && same_runtime
            && lifecycle == Some(HostLifecycle::Closed)
            && cleanup.status == ViewerCleanupStatus::WorkersQuiescent(ViewerNativeState::Closed)
            && normal_trace_passes(&events);
    }
    json!({"schema":"viewer-no-media-inner-v1","test":exact,"mode":mode,
        "scenario_passed":passed,"run_ok":run_ok,"same_runtime":same_runtime,
        "lifecycle":format!("{lifecycle:?}"),"bind_failure":bind_failure,
        "retained_stamp":format!("{retained_stamp:?}"),"preparation_remaining":remaining,
        "mutation":mutation,"probe_fault":format!("{:?}",probe.fault()),
        "cleanup_status":format!("{:?}",cleanup.status),"events":events,
        "elapsed_ms":started.elapsed().as_millis(),"media_executed":false})
}
fn cleanup_limits() -> ViewerCleanupLimits {
    ViewerCleanupLimits::new(Duration::from_secs(2), 500, Duration::from_millis(2)).unwrap()
}
fn normal_trace_passes(events: &[Value]) -> bool {
    let of_kind = |kind: &str| {
        events
            .iter()
            .filter(|event| event["kind"] == kind)
            .collect::<Vec<_>>()
    };
    let bind = of_kind("Bind");
    let proof = of_kind("ProofObserved");
    let reflected = of_kind("Reflection")
        .into_iter()
        .filter(|event| event["data"][6] == "observed_disabled")
        .collect::<Vec<_>>();
    let close = of_kind("CloseIntent");
    let inner = of_kind("InnerClose");
    let returned = of_kind("RunReturned");
    let cleaned = of_kind("CleanupReturned");
    if bind.len() != 1
        || proof.len() != 1
        || reflected.len() != 1
        || close.len() != 1
        || inner.len() != 1
        || returned.len() != 1
        || cleaned.len() != 1
    {
        return false;
    }
    let (bind, proof, reflected, close, inner) =
        (bind[0], proof[0], reflected[0], close[0], inner[0]);
    let ordered = [
        bind,
        proof,
        reflected,
        close,
        inner,
        returned[0],
        cleaned[0],
    ]
    .windows(2)
    .all(|pair| pair[0]["sequence"].as_u64() < pair[1]["sequence"].as_u64());
    let incoming = &reflected["data"][1];
    let observed = &reflected["data"][4];
    let current = &reflected["data"][9];
    let committed = &reflected["data"][8];
    let scope_matches = ["host", "owner", "definition", "document"]
        .iter()
        .all(|field| incoming[*field] == observed[*field])
        && incoming["turn"]
            .as_u64()
            .zip(observed["turn"].as_u64())
            .is_some_and(|(incoming, observed)| incoming >= observed);
    let commit_matches = if committed.is_null() {
        current == incoming
    } else {
        committed[0] == *incoming && committed[1] == *current
    };
    ordered
        && scope_matches
        && commit_matches
        && current.is_object()
        && bind["data"][0] == true
        && reflected["data"][0] == 91
        && reflected["data"][2] == "Completion"
        && reflected["data"][3] == 0
        && reflected["data"][4] == proof["data"][0]
        && reflected["data"][5] == proof["data"][1]
        && reflected["data"][7] == proof["data"][2]
        && close_intent_accepted(&close["data"])
        && inner["data"][0] == "Complete"
        && inner["data"][1] == 0
        && close_window_matches(&proof["data"][1], &inner["data"][5])
}
fn close_intent_accepted(result: &Value) -> bool {
    // Both Ok values mean the CLOSE bit was ORed into the original pending wake.
    // false means an event was already pending; final Closed is checked separately.
    result.get("Ok").and_then(Value::as_bool).is_some()
}
fn close_window_matches(observed: &Value, closing: &Value) -> bool {
    // begin_close revokes the ready surface while retaining the same owner/epoch.
    observed.as_array().is_some_and(|parts| parts.len() == 3)
        && closing.as_array().is_some_and(|parts| parts.len() == 3)
        && observed[0] == closing[0]
        && observed[1] == closing[1]
        && observed[2]
            .as_u64()
            .and_then(|surface| surface.checked_add(1))
            .zip(closing[2].as_u64())
            .is_some_and(|(expected, actual)| expected == actual)
}
#[test]
fn close_wake_coalescing_is_accepted_without_treating_failure_as_delivery() {
    assert!(close_intent_accepted(&json!({"Ok":true})));
    assert!(close_intent_accepted(&json!({"Ok":false})));
    for rejected in [json!({"Err":"closed"}), json!({"Ok":null}), json!(false)] {
        assert!(!close_intent_accepted(&rejected));
    }
}
#[test]
fn close_barrier_requires_original_owner_epoch_and_one_surface_revocation() {
    let observed = json!([11, 2, 7]);
    assert!(close_window_matches(&observed, &json!([11, 2, 8])));
    for foreign in [
        json!([12, 2, 8]),
        json!([11, 3, 8]),
        json!([11, 2, 7]),
        json!([11, 2, 9]),
    ] {
        assert!(!close_window_matches(&observed, &foreign));
    }
    assert!(!close_window_matches(
        &json!([11, 2, u64::MAX]),
        &json!([11, 2, 0])
    ));
}
fn dispatch(changed: bool, exact: &'static str, mode: &'static str) -> Result<(), Box<RunFailure>> {
    match std::env::var_os(CHILD_MODE) {
        None => supervision::parent_run(exact, mode),
        Some(actual) if actual == mode => {
            let record = run_inner(changed, exact, mode);
            let root =
                PathBuf::from(std::env::var_os(CHILD_EVIDENCE).expect("owned parent evidence"));
            assert!(root.is_absolute() && root.is_dir());
            let mut file = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(root.join("inner-success.json"))
                .unwrap();
            serde_json::to_writer_pretty(&mut file, &record).unwrap();
            file.flush().unwrap();
            assert_eq!(record["scenario_passed"], true, "{record}");
            Ok(())
        }
        Some(_) => Err(Box::new(RunFailure {
            stage: "child mode",
            error: "wrong Viewer scenario mode".into(),
            evidence: PathBuf::new(),
            owner: None,
            receipt: None,
            secondary: Vec::new(),
        })),
    }
}
#[test]
#[ignore = "requires separately allocated actual Viewer Native/pipe/worker slot; no media or visual claim"]
fn public_viewer_consumes_frame_completion_and_closes_children_first() -> Result<(), Box<RunFailure>>
{
    dispatch(false, NORMAL, "normal-v1")
}
#[test]
#[ignore = "requires separately allocated actual Viewer Native/pipe/worker slot; keeps original failed-bind owners"]
fn changed_pending_document_rejects_original_bind_and_retains_owners() -> Result<(), Box<RunFailure>>
{
    dispatch(true, CHANGED, "changed-v1")
}
