use super::super::fixture::request;
use kinekino_player::infrastructure::content_digest;
use serde_json::{Value, json};
use std::{fs, io::Read};

pub(super) fn expected_requests(endpoint: &str) -> Vec<Value> {
    ["airing", "completed", "movies"].into_iter().flat_map(|category| {
        [
            ("category", "jobs.submit", 0), ("category-get", "jobs.get", 0),
            ("listing", "catalog.search", 0), ("episodes", "jobs.submit", 0),
            ("episodes-get", "jobs.get", 0), ("episode-list", "episodes.list", 0),
            ("resolve", "jobs.submit", 0), ("resolve-get", "jobs.get", 0),
            ("inspect", "episodes.inspect_resolution", 0), ("open", "playback.open", 0),
            ("capture", "production.capture_frame", 0), ("close", "playback.close", 0),
            ("diagnostic", "catalog.read_raw", 0), ("closed-observation", "playback.engine", 4),
        ].into_iter().map(move |(suffix, command, exit)| {
            let label = format!("{category}-{suffix}");
            let request_id = if suffix == "capture" { request(command, json!({})).context.request_id } else { label.clone() };
            json!({"label":label,"command":command,"request_id":request_id,"endpoint":endpoint,"expected_exit":exit})
        })
    }).collect()
}
fn reconcile_request(
    expected: &Value,
    attempt: Option<&Value>,
    settled: Option<&Value>,
    saved_bytes: Option<(u64, &str)>,
) -> Value {
    let linked = attempt.is_some_and(|attempt| {
        attempt["schema_version"] == 1
            && [
                "label",
                "command",
                "request_id",
                "endpoint",
                "expected_exit",
            ]
            .into_iter()
            .all(|key| attempt[key] == expected[key])
            && saved_bytes.is_some_and(|(bytes, sha)| {
                attempt["request_bytes"] == bytes && attempt["request_sha256"] == sha
            })
    });
    let settled_link = attempt.zip(settled).is_some_and(|(attempt, settled)| {
        settled["schema_version"] == 1 && settled["request"] == *attempt
    });
    let returned_expected = settled.is_some_and(|value| {
        value["returned_expected_response"] == true
            && value["actual_exit"] == expected["expected_exit"]
    });
    json!({"expected":expected,"attempt_recorded":attempt.is_some(),"settlement_recorded":settled.is_some(),
        "identity_and_bytes_match":linked,"settlement_link_matches":settled_link,
        "completed_expected_response":linked && settled_link && returned_expected,
        "returned_error":settled.is_some_and(|value|value["returned_expected_response"] == false),
        "actual_exit":settled.map(|value|&value["actual_exit"]),
        "attempt_evidence":format!("logs/owned-requests/{}/attempt.json",expected["label"].as_str().unwrap_or("invalid")),
        "settled_evidence":format!("logs/owned-requests/{}/settled.json",expected["label"].as_str().unwrap_or("invalid"))})
}
pub(super) fn reconcile_requests(root: &std::path::Path, expected: &[Value]) -> Value {
    let rows:Vec<Value> = expected.iter().map(|expected| {
        let label = expected["label"].as_str().expect("Static request label");
        let base = root.join("logs/owned-requests").join(label);
        let load = |name| -> std::io::Result<Option<Value>> {
            match fs::File::open(base.join(name)) {
                Ok(file) => {
                    let mut bytes=Vec::new();file.take(262_145).read_to_end(&mut bytes)?;
                    if bytes.len()>262_144 { return Err(std::io::Error::other("Request link metadata exceeds 256 KiB")); }
                    serde_json::from_slice(&bytes).map(Some).map_err(std::io::Error::other)
                },
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(error),
            }
        };
        let attempt = load("attempt.json"); let settled = load("settled.json");
        let bytes = fs::File::open(root.join(format!("logs/{label}.request.json")))
            .and_then(|mut file|content_digest::sha256(&mut file,kinekino_player::application::local_host::MAX_REQUEST_BYTES as u64)
                .map_err(|error| std::io::Error::other(format!("{error:?}"))));
        let saved = bytes.as_ref().ok().map(|(length,sha)|(*length,sha.as_str()));
        let mut row = reconcile_request(expected,attempt.as_ref().ok().and_then(Option::as_ref),settled.as_ref().ok().and_then(Option::as_ref),saved);
        row["read_errors"] = json!({"attempt":attempt.err().map(|error|error.to_string()),"settled":settled.err().map(|error|error.to_string()),"request_bytes":bytes.err().map(|error|error.to_string())});
        row
    }).collect();
    request_summary(&rows)
}
fn request_summary(rows: &[Value]) -> Value {
    json!({"schema_version":1,"planned_cli_requests":rows.len(),
        "persisted_attempts":rows.iter().filter(|row|row["attempt_recorded"] == true).count(),
        "completed_expected_responses":rows.iter().filter(|row|row["completed_expected_response"] == true).count(),
        "returned_errors":rows.iter().filter(|row|row["returned_error"] == true).count(),
        "unsettled_attempts":rows.iter().filter(|row|row["attempt_recorded"] == true && row["settlement_recorded"] != true).count(),
        "unrecorded_or_unreadable":rows.iter().filter(|row|row["attempt_recorded"] != true).count(),
        "complete":!rows.is_empty() && rows.iter().all(|row|row["completed_expected_response"] == true),
        "scope":"fresh CLI calls only; SDK job/progress polls and ready/EOF excluded; missing evidence does not prove no invocation",
        "rows":rows})
}
pub(super) fn provider_links(
    episode_url: &str,
    player_document_url: &str,
    descriptor_sha: &str,
    raw_artifacts: &impl serde::Serialize,
) -> Value {
    json!({"schema_version":1,"stage":"resolution_before_native_playback","episode_url":episode_url,
        "player_document_url":player_document_url,"source_descriptor_sha256":descriptor_sha,"raw_artifacts":raw_artifacts,
        "admitted_player_document":kinekino_player::domain::player_document::PlayerDocumentLocation::parse(player_document_url).is_ok(),
        "provider_policy":"existing PlayerDocumentLocation only; no host rewrite or new allowlist",
        "meaning":"links original resolver provenance; not decoded media or a claim of current Frame host support"})
}
pub(super) fn gate_outcome(
    business: bool,
    shutdown: bool,
    reconciled: bool,
    ledger_error: Option<&std::io::Error>,
) -> Value {
    json!({"business_chain_succeeded":business,"explicit_shutdown_assertions_succeeded":shutdown,
        "requests_reconciled":reconciled,"request_ledger_saved":ledger_error.is_none(),
        "request_ledger_error":ledger_error.map(ToString::to_string),
        "full_gate_succeeded":business && shutdown && reconciled && ledger_error.is_none(),
        "completion_proof":"the final returned Result only; this file cannot certify its own successful write"})
}
pub(super) fn final_evidence_error(
    ledger_error: Option<std::io::Error>,
    outcome_error: Option<std::io::Error>,
    business: bool,
    reconciled: bool,
) -> Option<std::io::Error> {
    let mismatch = (business && !reconciled)
        .then(|| std::io::Error::other("CLI request evidence did not reconcile to the fixed plan"));
    let errors: Vec<String> = [ledger_error, outcome_error, mismatch]
        .into_iter()
        .flatten()
        .map(|error| error.to_string())
        .collect();
    (!errors.is_empty()).then(|| std::io::Error::other(errors.join("; ")))
}

#[cfg(test)]
mod request_ledger_tests {
    use super::*;
    #[test]
    fn ledger_write_failure_prevents_stored_success_even_when_business_stop_and_reconcile_pass() {
        kinekino_platform::install();
        let error = std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "ledger path is a directory",
        );
        let failed = gate_outcome(true, true, true, Some(&error));
        assert_eq!(failed["full_gate_succeeded"], false);
        assert_eq!(failed["request_ledger_saved"], false);
        assert_eq!(failed["request_ledger_error"], "ledger path is a directory");
        let normal = gate_outcome(true, true, true, None);
        assert_eq!(normal["full_gate_succeeded"], true);
        assert_eq!(normal["request_ledger_saved"], true);
        assert!(normal["request_ledger_error"].is_null());
    }
    #[test]
    fn evidence_error_order_is_preserved_and_outcome_file_is_not_final_completion_proof() {
        kinekino_platform::install();
        let metadata = gate_outcome(false, true, true, None);
        assert_eq!(metadata["business_chain_succeeded"], false);
        assert_eq!(metadata["full_gate_succeeded"], false);
        assert!(final_evidence_error(None, None, false, true).is_none()); // No replacement error for an original business failure.
        let combined = final_evidence_error(
            Some(std::io::Error::other("ledger original")),
            Some(std::io::Error::other("outcome secondary")),
            true,
            false,
        )
        .unwrap()
        .to_string();
        assert!(combined.starts_with("ledger original; outcome secondary; "));
        assert!(combined.ends_with("CLI request evidence did not reconcile to the fixed plan"));
        assert!(final_evidence_error(None, None, true, true).is_none());
        let write_error = final_evidence_error(
            None,
            Some(std::io::Error::other("outcome write failed")),
            true,
            true,
        )
        .unwrap();
        assert_eq!(write_error.to_string(), "outcome write failed");
        assert!(
            gate_outcome(true, true, true, None)["completion_proof"]
                .as_str()
                .unwrap()
                .contains("final returned Result")
        );
    }
    fn observed(expected: &Value, returned: bool) -> (Value, Value, String) {
        let sha = content_digest::sha256(&mut std::io::Cursor::new(b"original bytes"), 14)
            .unwrap()
            .1;
        let mut attempt = expected.clone();
        attempt["schema_version"] = json!(1);
        attempt["request_bytes"] = json!(14);
        attempt["request_sha256"] = json!(sha);
        let settled = json!({"schema_version":1,"request":attempt,"actual_exit":if returned {expected["expected_exit"].clone()}else{Value::Null},"returned_expected_response":returned});
        (attempt, settled, sha)
    }
    #[test]
    fn plan_is_42_distinct_labels_with_nine_jobs_and_three_expected_closed_errors() {
        kinekino_platform::install();
        let plan = expected_requests("one-original-endpoint");
        let labels: std::collections::BTreeSet<_> = plan
            .iter()
            .map(|row| row["label"].as_str().unwrap())
            .collect();
        assert_eq!(plan.len(), 42);
        assert_eq!(labels.len(), 42);
        assert_eq!(
            plan.iter()
                .filter(|row| row["command"] == "jobs.submit")
                .count(),
            9
        );
        assert_eq!(
            plan.iter().filter(|row| row["expected_exit"] == 4).count(),
            3
        );
        assert!(
            plan.iter()
                .all(|row| row["endpoint"] == "one-original-endpoint")
        );
        for row in plan
            .iter()
            .filter(|row| row["command"] == "production.capture_frame")
        {
            assert_eq!(
                row["request_id"],
                request("production.capture_frame", json!({}))
                    .context
                    .request_id
            );
        }
    }
    #[test]
    fn expected_nonzero_exit_counts_as_completed_call_not_as_api_success() {
        kinekino_platform::install();
        let expected = expected_requests("endpoint").pop().unwrap();
        let (attempt, settled, sha) = observed(&expected, true);
        let row = reconcile_request(&expected, Some(&attempt), Some(&settled), Some((14, &sha)));
        assert_eq!(row["actual_exit"], 4);
        assert_eq!(row["completed_expected_response"], true);
        assert_eq!(request_summary(&[row])["complete"], true);
    }
    #[test]
    fn request_identity_hash_nested_link_schema_and_exit_mismatch_cannot_complete() {
        kinekino_platform::install();
        let expected = expected_requests("endpoint").remove(0);
        let (attempt, settled, sha) = observed(&expected, true);
        for key in [
            "label",
            "command",
            "request_id",
            "endpoint",
            "expected_exit",
            "request_bytes",
            "request_sha256",
            "schema_version",
        ] {
            let mut changed = attempt.clone();
            changed[key] = json!("tampered");
            let row =
                reconcile_request(&expected, Some(&changed), Some(&settled), Some((14, &sha)));
            assert_eq!(row["completed_expected_response"], false, "{key}");
        }
        let mut wrong_exit = settled.clone();
        wrong_exit["actual_exit"] = json!(7);
        assert_eq!(
            reconcile_request(
                &expected,
                Some(&attempt),
                Some(&wrong_exit),
                Some((14, &sha))
            )["completed_expected_response"],
            false
        );
        assert_eq!(
            reconcile_request(
                &expected,
                Some(&attempt),
                Some(&settled),
                Some((14, "different raw bytes"))
            )["completed_expected_response"],
            false
        );
        let mut wrong_link = settled;
        wrong_link["request"]["endpoint"] = json!("another endpoint");
        assert_eq!(
            reconcile_request(
                &expected,
                Some(&attempt),
                Some(&wrong_link),
                Some((14, &sha))
            )["completed_expected_response"],
            false
        );
    }
    #[test]
    fn partial_run_reports_persisted_attempts_completed_errors_and_missing_evidence_separately() {
        kinekino_platform::install();
        let plan = expected_requests("endpoint");
        let rows: Vec<Value> = plan
            .iter()
            .enumerate()
            .map(|(index, expected)| {
                let (attempt, settled, sha) = observed(expected, index < 5);
                if index < 5 {
                    reconcile_request(expected, Some(&attempt), Some(&settled), Some((14, &sha)))
                } else if index == 5 {
                    reconcile_request(expected, Some(&attempt), None, Some((14, &sha)))
                } else if index == 6 {
                    reconcile_request(expected, Some(&attempt), Some(&settled), Some((14, &sha)))
                } else {
                    reconcile_request(expected, None, None, None)
                }
            })
            .collect();
        let summary = request_summary(&rows);
        assert_eq!(summary["planned_cli_requests"], 42);
        assert_eq!(summary["persisted_attempts"], 7);
        assert_eq!(summary["completed_expected_responses"], 5);
        assert_eq!(summary["returned_errors"], 1);
        assert_eq!(summary["unsettled_attempts"], 1);
        assert_eq!(summary["unrecorded_or_unreadable"], 35);
        assert_eq!(summary["complete"], false);
    }
    #[test]
    fn provider_provenance_preserves_urls_and_raw_refs_without_extending_admission() {
        kinekino_platform::install();
        let refs = json!([{"artifact_id":"a".repeat(64),"body_sha256":"b".repeat(64)}]);
        let cdn = format!("https://media.example.test/video/{}", "a".repeat(32));
        let admitted = provider_links(
            "https://catalog.example.test/example-episode",
            &cdn,
            "descriptor-sha",
            &refs,
        );
        assert_eq!(admitted["player_document_url"], cdn);
        assert_eq!(admitted["raw_artifacts"], refs);
        assert_eq!(admitted["admitted_player_document"], true);
        let frame_host = format!("https://frame.example.test/video/{}", "a".repeat(32));
        let rejected = provider_links(
            "https://catalog.example.test/example-episode",
            &frame_host,
            "descriptor-sha",
            &refs,
        );
        assert_eq!(rejected["player_document_url"], frame_host);
        assert_eq!(rejected["admitted_player_document"], false);
        assert_eq!(rejected["source_descriptor_sha256"], "descriptor-sha");
    }
}
