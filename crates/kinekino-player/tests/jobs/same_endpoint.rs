//! Explicit actual gate. No seed host, offline replay or model execution substitutes for this chain.
#[path = "same_endpoint/chain.rs"]
mod chain;
pub(crate) use chain::{ChainBudgets, EndpointInput, run_category};
#[path = "same_endpoint/evidence.rs"]
mod evidence;
#[path = "same_endpoint/routed.rs"]
pub(crate) mod routed;
use evidence::{expected_requests, final_evidence_error, gate_outcome, reconcile_requests};

use super::{
    fixture::Fixture,
    kinekino_process::{CliFailure, ScenarioFailure, close_request},
    playback_owner::BoundedNativeOwner as NativeOwner,
};
use kinekino_player::infrastructure::{catalog::CatalogFiles, content_digest};
use kinekino_process::domain::deadline::Deadline;
use serde_json::{Value, json};
use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    time::{Duration, Instant},
};

struct Budgets {
    execution_ms: u64,
    observation: Duration,
    stop: Duration,
    lifetime: Duration,
    ready: Duration,
    request: Duration,
    cleanup: Duration,
}
fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Explicit {name} is required"))
}
fn milliseconds(name: &str) -> u64 {
    let value = required(name)
        .parse::<u64>()
        .expect("Decimal millisecond budget");
    assert!(value > 0);
    value
}
#[test]
#[ignore = "Requires adopted CLI explicit-join wiring, central Windows browser/network/trusted mpv media slot, fresh evidence and pinned executables; never run by the default suite"]
fn actual_three_categories_to_owned_playback_on_one_endpoint() -> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    let evidence = PathBuf::from(required("JOB_EVIDENCE_DIR"));
    assert!(evidence.is_absolute());
    let cli = PathBuf::from(required("JOB_CLI_EXE"));
    assert!(cli.is_absolute());
    let cli_sha = required("JOB_CLI_SHA256");
    let executable = PathBuf::from(required("NATIVE_MPV_EXE"));
    assert!(executable.is_absolute());
    let engine_sha = required("NATIVE_MPV_SHA256");
    let wiring_revision = required("SAME_ENDPOINT_JOIN_WIRING_REVISION");
    assert_eq!(wiring_revision.len(), 40);
    assert!(wiring_revision.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let budgets = Budgets {
        execution_ms: milliseconds("SAME_ENDPOINT_JOB_EXECUTION_MS"),
        observation: Duration::from_millis(milliseconds("SAME_ENDPOINT_JOB_OBSERVATION_MS")),
        stop: Duration::from_millis(milliseconds("SAME_ENDPOINT_STOP_OBSERVATION_MS")),
        lifetime: Duration::from_millis(milliseconds("SAME_ENDPOINT_TOTAL_OBSERVATION_MS")),
        ready: Duration::from_millis(milliseconds("SAME_ENDPOINT_READY_OBSERVATION_MS")),
        request: Duration::from_millis(milliseconds("SAME_ENDPOINT_REQUEST_OBSERVATION_MS")),
        cleanup: Duration::from_millis(milliseconds("SAME_ENDPOINT_CLEANUP_RESERVE_MS")),
    };
    assert!(
        budgets.stop > Duration::from_secs(30),
        "The supervisor budget must exceed the adopted CLI's 30-second join observation budget; dependency shutdown is not globally bounded"
    );
    let mut fixture = Fixture::named("same-endpoint-three-categories");
    fixture.keep_evidence();
    let catalog = fixture.root.join("catalog");
    fs::create_dir(&catalog).unwrap();
    let original = Deadline::new(Instant::now(), budgets.lifetime, budgets.cleanup)
        .expect("Explicit host lifetime budget");
    let host = NativeOwner::try_start(
        &fixture,
        &catalog,
        &executable,
        &engine_sha,
        original,
        budgets.ready,
    )
    .map_err(ScenarioFailure::host)?;
    let expected = expected_requests(&host.endpoint);
    let mut observations: Vec<Value> = Vec::new();
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), Box<CliFailure>> {
        let files = CatalogFiles::open(&catalog).unwrap();
        let rules = content_digest::sha256(
            &mut std::io::Cursor::new(kinekino_player::infrastructure::adblock::RULES.as_bytes()),
            1024 * 1024,
        )
        .unwrap()
        .1;
        fs::write(fixture.root.join("host.json"),serde_json::to_vec_pretty(&json!({"pid":host.process_id(),"endpoint":host.endpoint,"cli_sha256":cli_sha,"engine_sha256":engine_sha,"join_wiring_revision":wiring_revision,"job_execution_ms":budgets.execution_ms,"job_observation_ms":budgets.observation.as_millis().to_string(),"stop_observation_ms":budgets.stop.as_millis().to_string(),"test_build_embedded_rules_sha256":rules,"gui_local_filter_equivalence_verified":false,"blocked_zero_means":"no blocks counted in that capture; not absence of ads or proof of filter equivalence"})).unwrap()).unwrap();
        fs::write(fixture.root.join("request-plan.json"), serde_json::to_vec_pretty(&json!({"schema_version":1,"planned_cli_requests":expected.len(),"requests":expected,"excludes":"variable SDK job/progress polls and ready/EOF"})).unwrap()).unwrap();
        let endpoint_input = EndpointInput {
            endpoint: &host.endpoint,
            original: host.deadline(),
        };
        let chain_budgets = ChainBudgets {
            execution_ms: budgets.execution_ms,
            observation: budgets.observation,
            request: budgets.request,
        };
        for name in ["airing", "completed", "movies"] {
            let observed = run_category(&fixture, &endpoint_input, &files, name, &chain_budgets)?;
            assert!(
                observations
                    .iter()
                    .all(|prior| prior["engine_instance_id"] != observed["engine_instance_id"]),
                "A closed engine identity was reused by another category"
            );
            observations.push(observed);
            fs::write(
                fixture.root.join("categories.json"),
                serde_json::to_vec_pretty(&observations).unwrap(),
            )
            .unwrap();
        }
        assert_eq!(observations.len(), 3);
        Ok(())
    }));
    let result = close_request(result);
    let business = matches!(&result, Ok(Ok(())));
    let stopped = host.try_stop_chain(9, budgets.stop);
    let ledger = reconcile_requests(&fixture.root, &expected);
    let reconciled = ledger["complete"] == true;
    let ledger_error = fs::write(
        fixture.root.join("request-ledger.json"),
        serde_json::to_vec_pretty(&ledger).expect("Finite evidence JSON"),
    )
    .err();
    let mut outcome = json!({"completed_categories":observations.len(),"request_ledger":"request-ledger.json","schema_version":4,"host_lifetime_ms":budgets.lifetime.as_millis(),"ready_cap_ms":budgets.ready.as_millis(),"request_cap_ms":budgets.request.as_millis(),"cleanup_reserve_ms":budgets.cleanup.as_millis()});
    outcome.as_object_mut().expect("Outcome object").extend(
        gate_outcome(business, stopped.is_ok(), reconciled, ledger_error.as_ref())
            .as_object()
            .expect("Gate outcome object")
            .clone(),
    );
    let bytes = serde_json::to_vec_pretty(&outcome).expect("Scalar outcome JSON");
    let outcome_error = fs::write(fixture.root.join("outcome.json"), bytes).err();
    let evidence_error = final_evidence_error(ledger_error, outcome_error, business, reconciled);
    ScenarioFailure::conclude(result, stopped, evidence_error)
}
