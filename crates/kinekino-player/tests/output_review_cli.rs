#![cfg(windows)]
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "output_review_cli/fixture.rs"]
mod fixture;
use fixture::{Fixture, record_payload, request};
use serde_json::json;
use std::fs;

#[test]
fn three_tasks_record_exact_targets_and_fresh_readback_without_quality_approval() {
    kinekino_platform::install();
    let f = Fixture::new("normal");
    for (id, task) in [
        ("correction", "correction"),
        ("cards", "cards"),
        ("highlights", "highlights"),
    ] {
        let view = f.seed(id, task);
        assert_eq!(view["target"]["task"], task);
        assert_eq!(view["target"]["kind"], "accepted_output");
        assert_eq!(view["evidence_observation"], "not_historically_bound");
        let before = fs::read(f.revision_file(id, 2)).unwrap();
        let payload = record_payload(&view, "decision-1");
        let stored = f.ok("proposals.record_output_decision", payload, Some("2"));
        let history = f.history(id);
        assert_eq!(history["records"][0], stored);
        assert_eq!(history["current_evidence"], "not_checked");
        assert_eq!(stored["decision"]["input"]["target"], view["target"]);
        assert_eq!(
            stored["decision"]["criteria_document"],
            view["criteria_document"]
        );
        assert_eq!(stored["decision"]["automatic_application_enabled"], false);
        assert_eq!(stored["decision"]["counts_as_model_sample"], false);
        assert_eq!(stored["decision"]["input"]["verdict"], "deferred");
        assert_eq!(fs::read(f.revision_file(id, 2)).unwrap(), before);
    }
}

#[test]
fn same_output_hash_different_proposal_cannot_exchange_selected_identity() {
    kinekino_platform::install();
    let f = Fixture::new("route");
    let p = f.seed("p", "correction");
    let q = f.seed("q", "correction");
    assert_eq!(p["target"]["output_sha256"], q["target"]["output_sha256"]);
    let mut payload = record_payload(&p, "decision-1");
    payload["proposal_id"] = json!("q");
    f.reject(
        &request("proposals.record_output_decision", payload, Some("2")),
        "revision_conflict",
    );
    assert_eq!(f.decision_count("p"), 0);
    assert_eq!(f.decision_count("q"), 0);
}

#[test]
fn edit_clears_acceptance_and_stale_record_is_not_rebound_after_reacceptance() {
    kinekino_platform::install();
    let f = Fixture::new("stale");
    let view = f.seed("p", "correction");
    let payload = record_payload(&view, "decision-1");
    f.ok("proposals.edit",json!({"proposal_id":"p","draft":{"task":"correction","edits":[{"cue_id":"cue-1","text":"Different synthetic output."}]}}),Some("2"));
    f.reject(
        &request(
            "proposals.record_output_decision",
            payload.clone(),
            Some("2"),
        ),
        "revision_conflict",
    );
    f.reject(
        &request(
            "proposals.inspect_output_review_target",
            json!({"proposal_id":"p"}),
            None,
        ),
        "needs_input",
    );
    let mut current_missing = payload.clone();
    current_missing["decision"]["target"]["draft_revision"] = json!("2");
    current_missing["decision"]["target"]["output_revision"] = json!("3");
    f.reject(
        &request(
            "proposals.record_output_decision",
            current_missing,
            Some("3"),
        ),
        "needs_input",
    );
    f.ok(
        "proposals.accept",
        json!({"proposal_id":"p","target_source_revision":"7"}),
        Some("3"),
    );
    f.reject(
        &request("proposals.record_output_decision", payload, Some("4")),
        "revision_conflict",
    );
    assert_eq!(f.decision_count("p"), 0);
}

#[test]
fn caller_criteria_changes_require_representing_trusted_catalog() {
    kinekino_platform::install();
    use kinekino_player::domain::proposals::output_review::CriteriaDocument;
    let f = Fixture::new("criteria");
    let view = f.seed("p", "correction");
    for field in [
        "version",
        "purpose",
        "criteria_id",
        "content_sha256",
        "task",
    ] {
        let mut payload = record_payload(&view, "decision-1");
        payload["decision"]["criteria"][field] = if field == "task" {
            json!("cards")
        } else {
            json!("changed")
        };
        f.reject(
            &request("proposals.record_output_decision", payload, Some("2")),
            "revision_conflict",
        );
    }
    let mut substitute: CriteriaDocument =
        serde_json::from_value(view["criteria_document"].clone()).unwrap();
    substitute.clauses[0].expected.push('!');
    let mut payload = record_payload(&view, "decision-1");
    payload["decision"]["criteria"] = serde_json::to_value(substitute.identity().unwrap()).unwrap();
    f.reject(
        &request("proposals.record_output_decision", payload, Some("2")),
        "revision_conflict",
    );
    assert_eq!(
        f.inspect("p")["criteria_document"],
        view["criteria_document"]
    );
    assert_eq!(f.decision_count("p"), 0);
}

#[test]
fn hash_version_and_canonical_payload_tampering_never_publish_decisions() {
    kinekino_platform::install();
    let f = Fixture::new("hash");
    let view = f.seed("p", "correction");
    let mut payload = record_payload(&view, "decision-1");
    payload["decision"]["target"]["output_sha256"] = json!("b".repeat(64));
    f.reject(
        &request("proposals.record_output_decision", payload, Some("2")),
        "revision_conflict",
    );
    let mut payload = record_payload(&view, "decision-1");
    payload["decision"]["target"]["hash_format"] = json!("future");
    f.reject(
        &request("proposals.record_output_decision", payload, Some("2")),
        "unsupported",
    );
    let path = f.revision_file("p", 2);
    let mut bytes = fs::read(&path).unwrap();
    let index = bytes.iter().position(|byte| *byte == b'S').unwrap();
    bytes[index] = b'X';
    fs::write(&path, bytes).unwrap();
    f.reject(
        &request(
            "proposals.record_output_decision",
            record_payload(&view, "decision-1"),
            Some("2"),
        ),
        "needs_input",
    );
    assert_eq!(f.decision_count("p"), 0);
}

#[test]
fn source_generated_and_companion_exchange_is_rejected() {
    kinekino_platform::install();
    let f = Fixture::new("source");
    let view = f.seed("p", "correction");
    for field in ["source_revision", "source_sha256", "generated_raw_sha256"] {
        let mut payload = record_payload(&view, "decision-1");
        payload["decision"]["target"][field] = if field == "source_revision" {
            json!("8")
        } else {
            json!("b".repeat(64))
        };
        f.reject(
            &request("proposals.record_output_decision", payload, Some("2")),
            "revision_conflict",
        );
    }
    let mut payload = record_payload(&view, "decision-1");
    payload["decision"]["target"]["historical"] = json!({"schema_version":"1","reference":{"sha256":"c".repeat(64),"bytes":17,"item_id":"foreign"}});
    f.reject(
        &request("proposals.record_output_decision", payload, Some("2")),
        "revision_conflict",
    );
    assert_eq!(f.decision_count("p"), 0);
}

#[test]
fn missing_canonical_publication_blocks_new_record_and_preserves_independent_history() {
    kinekino_platform::install();
    let f = Fixture::new("missing");
    let view = f.seed("p", "correction");
    f.ok(
        "proposals.record_output_decision",
        record_payload(&view, "original"),
        Some("2"),
    );
    let original = f.history("p");
    let before = f.record_files("p");
    fs::remove_file(f.revision_file("p", 2)).unwrap();
    f.reject(
        &request(
            "proposals.record_output_decision",
            record_payload(&view, "new"),
            Some("2"),
        ),
        "needs_input",
    );
    assert_eq!(f.history("p"), original);
    assert_eq!(f.record_files("p"), before);
    // Canonical publication loss, not an external historical-file/verifier end-to-end test.
}

#[test]
fn identical_id_request_retry_keeps_original_body_time_and_conflicts_do_not_overwrite() {
    kinekino_platform::install();
    let f = Fixture::new("retry");
    let view = f.seed("p", "correction");
    let payload = record_payload(&view, "decision-1");
    let stored = f.ok(
        "proposals.record_output_decision",
        payload.clone(),
        Some("2"),
    );
    let files = f.record_files("p");
    assert_eq!(
        f.ok(
            "proposals.record_output_decision",
            payload.clone(),
            Some("2")
        ),
        stored
    );
    for field in ["note", "verdict", "claimed_review_unix_ms"] {
        let mut changed = payload.clone();
        changed["decision"][field] = match field {
            "verdict" => json!("needs_changes"),
            "claimed_review_unix_ms" => json!("20"),
            _ => json!("Different synthetic note"),
        };
        f.reject(
            &request("proposals.record_output_decision", changed, Some("2")),
            "revision_conflict",
        );
    }
    let mut changed = payload;
    changed["decision"]["declared_reviewer"]["label"] = json!("Other declared reviewer");
    f.reject(
        &request("proposals.record_output_decision", changed, Some("2")),
        "revision_conflict",
    );
    assert_eq!(f.record_files("p"), files);
    assert_eq!(f.decision_count("p"), 1);
}

#[test]
fn declared_claimed_and_observed_fields_stay_distinct_and_caller_clock_injection_fails() {
    kinekino_platform::install();
    let f = Fixture::new("metadata");
    let view = f.seed("p", "correction");
    let payload = record_payload(&view, "decision-1");
    let stored = f.ok(
        "proposals.record_output_decision",
        payload.clone(),
        Some("2"),
    );
    assert_eq!(stored["decision"]["input"]["claimed_review_unix_ms"], "19");
    assert_eq!(
        stored["decision"]["input"]["declared_reviewer"],
        payload["decision"]["declared_reviewer"]
    );
    assert_eq!(
        stored["decision"]["observed_record_time"]["clock"],
        "local_system_unix_ms"
    );
    assert!(
        stored["decision"]["observed_record_time"]["unix_ms"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .is_ok()
    );
    let mut injected = payload;
    injected["decision"]["observed_record_time"] =
        json!({"unix_ms":"19","clock":"local_system_unix_ms"});
    f.reject(
        &request("proposals.record_output_decision", injected, Some("2")),
        "invalid_argument",
    );
    assert_eq!(f.history("p")["records"][0], stored);
}

#[test]
fn discovery_context_and_legacy_schemas_remain_explicit() {
    kinekino_platform::install();
    use kinekino_player::application::{
        api::{PlayerSdk, Request},
        proposals::schema,
    };
    let f = Fixture::new("discovery");
    let view = f.seed("p", "correction");
    let (_, plain) = f.invoke(&request("capabilities", json!({}), None), false);
    assert_eq!(plain["data"]["output_reviews"]["available"], false);
    let caps = f.ok("capabilities", json!({}), None);
    assert_eq!(caps["output_reviews"]["available"], true);
    for command in [
        "proposals.inspect_output_review_target",
        "proposals.record_output_decision",
        "proposals.output_decisions",
    ] {
        let supported = f.ok("schema", json!({"command":command}), None);
        assert_eq!(supported["status"], "supported");
        let (_, plain) = f.invoke(&request("schema", json!({"command":command}), None), false);
        assert_eq!(plain["data"]["status"], "unsupported");
    }
    for command in [
        "proposals.create",
        "proposals.inspect",
        "proposals.edit",
        "proposals.accept",
    ] {
        assert_eq!(
            f.ok("schema", json!({"command":command}), None),
            schema::describe(&json!({"command":command}), true).unwrap()
        );
    }
    let mut legacy = Request::new("schema");
    legacy.payload = json!({"command":"reviews.record_decision"});
    let expected = PlayerSdk::default().execute(legacy).data.unwrap();
    assert_eq!(
        f.ok("schema", json!({"command":"reviews.record_decision"}), None),
        expected
    );
    let payload = record_payload(&view, "decision-1");
    f.reject(
        &request("proposals.record_output_decision", payload.clone(), None),
        "invalid_argument",
    );
    for field in ["idempotency_key", "timeout_ms"] {
        let mut rejected = request(
            "proposals.record_output_decision",
            payload.clone(),
            Some("2"),
        );
        rejected["context"][field] = if field == "timeout_ms" {
            json!(10)
        } else {
            json!("transport")
        };
        f.reject(&rejected, "unsupported");
    }
    f.reject(
        &request(
            "proposals.output_decisions",
            json!({"proposal_id":"p"}),
            Some("2"),
        ),
        "unsupported",
    );
    assert_eq!(f.decision_count("p"), 0);
}
