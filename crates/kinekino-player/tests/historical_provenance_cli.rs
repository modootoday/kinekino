#![cfg(windows)]
use kinekino_player::application::api::ProposalClient;
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "historical_provenance_cli/harness.rs"]
mod harness;
use harness::*;
use serde_json::json;
use std::fs;

#[test]
#[ignore = "explicit actual-nine current CLI and typed source context, no media authority or inference"]
fn actual_nine_source_context_cli_and_typed_client_share_guarded_state() {
    kinekino_platform::install();
    use kinekino_player::application::{
        api::RequestContext,
        proposals::{
            InspectProposal,
            source_context::{COMMAND, MAX_RESPONSE_BYTES},
        },
    };
    let f = Fixture::new();
    let companion: serde_json::Value =
        serde_json::from_slice(&fs::read(f.companion()).unwrap()).unwrap();
    let rows = companion["items"].as_array().unwrap();
    assert_eq!(rows.len(), 9);
    for row in rows {
        let item = row["item_id"].as_str().unwrap();
        f.ok(
            &format!("context-import-{item}"),
            "proposals.import_historical",
            import(&format!("context-{item}"), item),
            None,
        );
    }
    let before = inventory(&f.proposals);
    let projects = inventory(&f.projects);
    let sdk = f.sdk();
    let mut declared = 0;
    let mut untimed = 0;
    for row in rows {
        let item = row["item_id"].as_str().unwrap();
        let id = format!("context-{item}");
        let value = f.ok(
            &format!("source-context-{item}"),
            COMMAND,
            json!({"proposal_id":id}),
            None,
        );
        assert_eq!(value["schema_version"], "proposal-source-context/1");
        assert_eq!(value["state"]["document"]["state"], "needs_repair");
        assert_eq!(value["state"]["document"]["draft_revision"], "1");
        assert_eq!(value["timing_authority"], "declared_request_only");
        assert_eq!(value["media_binding"], "not_verified");
        assert_eq!(value["quality_approved"], false);
        let typed = sdk
            .inspect_proposal_source_context(
                RequestContext::default(),
                InspectProposal {
                    proposal_id: id.clone(),
                },
            )
            .unwrap();
        typed.validate().unwrap();
        assert_eq!(serde_json::to_value(typed).unwrap(), value);
        let count = value["context"]["declared_cue_times"]
            .as_array()
            .unwrap()
            .len();
        if row["task"] == "correction" {
            assert_eq!(count, 0);
            untimed += 1;
        } else {
            assert_eq!(count, 7);
            declared += 1;
            for cue in value["context"]["source"]["cues"].as_array().unwrap() {
                assert!(cue["start_us"].is_null() && cue["end_us"].is_null());
            }
        }
        assert!(serde_json::to_vec(&value).unwrap().len() < MAX_RESPONSE_BYTES);
    }
    assert_eq!((declared, untimed), (6, 3));
    let unavailable = f.invoke(
        "context-no-verifier",
        COMMAND,
        json!({"proposal_id":"context-gemma270-cards"}),
        None,
        false,
    );
    assert_eq!(unavailable.code, 3);
    assert_eq!(unavailable.value["error"]["code"], "unsupported");
    let unknown = f.invoke(
        "context-unknown-field",
        COMMAND,
        json!({"proposal_id":"context-gemma270-cards","source_path":"caller"}),
        None,
        true,
    );
    assert_eq!(unknown.value["error"]["code"], "invalid_argument");
    let revision = f.invoke(
        "context-unsupported-revision",
        COMMAND,
        json!({"proposal_id":"context-gemma270-cards"}),
        Some("1"),
        true,
    );
    assert_eq!(revision.value["error"]["code"], "invalid_argument");
    let context = RequestContext {
        timeout_ms: Some(1),
        ..RequestContext::default()
    };
    assert_eq!(
        sdk.inspect_proposal_source_context(
            context,
            InspectProposal {
                proposal_id: "context-gemma270-cards".into()
            }
        )
        .unwrap_err()
        .code,
        kinekino_player::application::api::ErrorCode::InvalidArgument
    );
    let caps = f.ok("context-capabilities", "capabilities", json!({}), None);
    assert_eq!(caps["commands"].as_array().unwrap().len(), 119);
    assert_eq!(caps["proposals"]["source_context"]["available"], true);
    let schema = f.ok("context-schema", "schema", json!({"command":COMMAND}), None);
    assert_eq!(schema["status"], "supported");
    assert_eq!(schema["response_max_bytes"], MAX_RESPONSE_BYTES);
    let moved = f.companion().with_extension("context-missing");
    fs::rename(f.companion(), &moved).unwrap();
    let missing = f.invoke(
        "context-missing-evidence",
        COMMAND,
        json!({"proposal_id":"context-gemma270-cards"}),
        None,
        true,
    );
    assert_ne!(missing.code, 0);
    fs::rename(moved, f.companion()).unwrap();
    assert_eq!(before, inventory(&f.proposals));
    assert_eq!(projects, inventory(&f.projects));
    f.assert_original_inputs_unchanged();
}

#[test]
#[ignore = "explicit preserved actual-nine source context inspection; no inference/media authority"]
fn actual_nine_declared_times_remain_separate_from_canonical_source_timing() {
    kinekino_platform::install();
    use kinekino_player::{
        application::proposals::source_import::from_request_asset_with_context,
        domain::{
            api_values::DecimalU64,
            proposals::{ProposalTask, SourceSnapshot},
        },
    };
    let f = Fixture::new();
    let companion: serde_json::Value =
        serde_json::from_slice(&fs::read(f.companion()).unwrap()).unwrap();
    let rows = companion["items"].as_array().unwrap();
    assert_eq!(rows.len(), 9);
    let mut declared = 0;
    let mut untimed = 0;
    let mut observations = Vec::new();
    for row in rows {
        let item = row["item_id"].as_str().unwrap();
        let task = match row["task"].as_str().unwrap() {
            "correction" => ProposalTask::Correction,
            "cards" => ProposalTask::Cards,
            "highlights" => ProposalTask::Highlights,
            other => panic!("unexpected task {other}"),
        };
        let digest = row["source"]["sha256"].as_str().unwrap();
        let asset = fs::read_to_string(f.package().join("objects").join(digest)).unwrap();
        let context = from_request_asset_with_context(
            format!("source-{item}"),
            DecimalU64(1),
            digest.into(),
            asset.clone(),
            task,
        )
        .unwrap();
        match &context.source {
            SourceSnapshot::Document(_) => {
                assert_eq!(task, ProposalTask::Correction);
                assert!(context.declared_cue_times.is_empty());
                untimed += 1;
            }
            SourceSnapshot::Cue(source) => {
                assert_eq!(source.cues.len(), 7);
                assert_eq!(context.declared_cue_times.len(), source.cues.len());
                for (cue, timing) in source.cues.iter().zip(&context.declared_cue_times) {
                    assert_eq!(cue.cue_id, timing.cue_id);
                    assert!(
                        cue.start_us.is_none() && cue.end_us.is_none(),
                        "declared request times must not become verified source timing"
                    );
                    assert!(timing.declared_in_us.0 < timing.declared_out_us.0);
                }
                assert_eq!(
                    context.declared_cue_times.first().unwrap().declared_in_us.0,
                    107_250_000
                );
                assert_eq!(
                    context.declared_cue_times.last().unwrap().declared_out_us.0,
                    133_800_000
                );
                declared += 1;
            }
        }
        assert_eq!(
            from_request_asset_with_context(
                format!("source-{item}"),
                DecimalU64(1),
                "0".repeat(64),
                asset,
                task
            )
            .unwrap_err()
            .code,
            kinekino_player::application::api::ErrorCode::InvalidArgument
        );
        observations.push(json!({"item_id":item,"source_sha256":digest,"context":context,"media_binding":"not_verified","playback_admitted":false,"quality_approved":false}));
    }
    assert_eq!((declared, untimed), (6, 3));
    fs::write(f.package().parent().unwrap().join("declared-context.json"),serde_json::to_vec_pretty(&json!({"items":observations,"declared_cue_items":declared,"untimed_document_items":untimed})).unwrap()).unwrap();
    f.assert_original_inputs_unchanged();
}

#[test]
#[ignore = "explicit pinned verifier and preserved offline actual-nine CLI gate"]
fn actual_nine_binding_fresh_cli_accept_retry_and_project_capture() {
    kinekino_platform::install();
    let f = Fixture::new();
    let companion: serde_json::Value =
        serde_json::from_slice(&fs::read(f.companion()).unwrap()).unwrap();
    let rows = companion["items"].as_array().unwrap();
    assert_eq!(rows.len(), 9);
    let unsupported = f.invoke(
        "without-verifier",
        "proposals.import_historical",
        import("unavailable", "gemma270-correction"),
        None,
        false,
    );
    assert_eq!(unsupported.code, 3);
    assert_eq!(unsupported.value["error"]["code"], "unsupported");
    assert!(!f.proposals.join("p-unavailable").exists());
    for row in rows {
        let item = row["item_id"].as_str().unwrap();
        let id = format!("historical-{item}");
        let created = f.ok(
            &format!("import-{item}"),
            "proposals.import_historical",
            import(&id, item),
            None,
        );
        assert_eq!(created["historical"]["reference"]["item_id"], item);
        assert_eq!(created["document"]["draft_revision"], "1");
        assert_eq!(created["document"]["schema_version"], "3");
        assert_eq!(created["document"]["state"], "needs_repair");
        let raw = fs::read_to_string(
            f.package()
                .join("objects")
                .join(row["candidate"]["sha256"].as_str().unwrap()),
        )
        .unwrap();
        assert_eq!(created["document"]["generated"]["raw_text"], raw);
        let reopened = f.ok(
            &format!("fresh-{item}"),
            "proposals.inspect",
            json!({"proposal_id":id}),
            None,
        );
        assert_eq!(created, reopened);
    }
    let id = "historical-gemma270-correction";
    let before = f.ok(
        "before-edit",
        "proposals.inspect",
        json!({"proposal_id":id}),
        None,
    );
    let stored: kinekino_player::domain::proposals::StoredProposal =
        serde_json::from_value(before.clone()).unwrap();
    let source = stored.document.source.as_document().unwrap();
    let draft = json!({"task":"document_correction","edits":[{"source_ref":source.text_reference("/0").unwrap(),"text":"Technical edit fixture only; no human quality approval."}]});
    f.ok(
        "edit",
        "proposals.edit",
        json!({"proposal_id":id,"draft":draft}),
        Some("1"),
    );
    let stale = f.invoke(
        "stale-edit",
        "proposals.edit",
        json!({"proposal_id":id,"draft":draft}),
        Some("1"),
        true,
    );
    assert_eq!(stale.value["error"]["code"], "revision_conflict");
    let accept = json!({"proposal_id":id,"target_source_revision":"1"});
    let accepted = f.ok("accept", "proposals.accept", accept.clone(), Some("2"));
    assert_eq!(accepted["output_revision"], "3");
    assert_eq!(accepted["automatic_application_enabled"], false);
    assert_eq!(
        f.ok(
            "accept-retry",
            "proposals.accept",
            accept.clone(),
            Some("2")
        ),
        accepted
    );
    let after = f.ok(
        "after-accept",
        "proposals.inspect",
        json!({"proposal_id":id}),
        None,
    );
    assert_eq!(after["document"]["source"], before["document"]["source"]);
    assert_eq!(
        after["document"]["generated"],
        before["document"]["generated"]
    );
    assert_eq!(after["historical"], before["historical"]);
    let content = json!({"watching":null,"proposals":[{"origin":"live","proposal_id":id,"draft_revision":"3","working_draft":draft}]});
    let project = f.ok("project-create", "projects.create", json!({"project_id":"historical-work","title":"Technical provenance gate","content":content}), None);
    assert_eq!(project["project"]["proposals"][0]["captured"], after);
    let capture = project["project"]["proposals"][0]["capture_sha256"].clone();
    let saved_content = json!({"watching":null,"proposals":[{"origin":"saved","proposal_id":id,"capture_sha256":capture,"working_draft":null}]});
    let saved = f.ok("project-save", "projects.save_revision", json!({"project_id":"historical-work","title":"Saved technical capture","content":saved_content}), Some("1"));
    let reopened = f.ok(
        "project-fresh",
        "projects.inspect",
        json!({"project_id":"historical-work","revision":null}),
        None,
    );
    assert_eq!(saved, reopened);
    assert_eq!(
        reopened["project"]["proposals"][0]["captured"]["historical"],
        after["historical"]
    );
    assert!(reopened["project"]["proposals"][0]["working_draft"].is_null());
    let stale = f.invoke(
        "project-stale",
        "projects.save_revision",
        json!({"project_id":"historical-work","title":"Stale","content":saved_content}),
        Some("1"),
        true,
    );
    assert_eq!(stale.value["error"]["code"], "revision_conflict");

    let old_locator =
        json!({"kind":"local_video","source_id":"historical-locator","path":"D:/old/sintel.mp4"});
    let new_locator =
        json!({"kind":"local_video","source_id":"historical-locator","path":"C:/media/sintel.mp4"});
    let bound = f.ok("reconnect-project-create", "projects.create", json!({
        "project_id":"historical-reconnect","title":"Historical locator only",
        "content":{"watching":{"label":"Old locator","source":old_locator},
            "proposals":[{"origin":"live","proposal_id":id,"draft_revision":"3","working_draft":null}]}
    }), None);
    let move_request = json!({"project_id":"historical-reconnect","expected_source":old_locator,
        "replacement":{"label":"New locator","source":new_locator}});
    let without_verifier = f.invoke(
        "reconnect-without-verifier",
        "projects.reconnect_source",
        move_request.clone(),
        Some("1"),
        false,
    );
    assert_ne!(without_verifier.code, 0);
    let moved = f.ok(
        "historical-reconnect",
        "projects.reconnect_source",
        move_request,
        Some("1"),
    );
    assert_eq!(moved["state"]["project"]["revision"], "2");
    assert_eq!(
        moved["state"]["project"]["proposals"],
        bound["project"]["proposals"]
    );
    assert_eq!(moved["content_identity_verified"], false);
    assert_eq!(moved["source_opened"], false);
    assert_eq!(moved["playback_rebound"], false);
    assert_eq!(
        f.ok(
            "historical-reconnect-fresh",
            "projects.inspect",
            json!({"project_id":"historical-reconnect"}),
            None
        ),
        moved["state"]
    );
    let old_bound = f.ok(
        "historical-reconnect-old",
        "projects.inspect",
        json!({"project_id":"historical-reconnect","revision":"1"}),
        None,
    );
    assert_eq!(old_bound["project"], bound["project"]);

    verify_cue_task_lifecycles(&f);
    verify_historical_output_decisions(&f);

    // Only the disposable copied workspace is mutated. Every rejected request
    // must leave the actual proposal/project publication bytes unchanged.
    let publications = inventory(&f.proposals);
    let projects = inventory(&f.projects);
    for (name, path) in [
        ("companion", f.companion()),
        (
            "source",
            f.package()
                .join("objects/b282db497133af492690d5d8726549cbafa7dd203517476e64e915c8101b2e27"),
        ),
        (
            "candidate",
            f.package()
                .join("objects/0e201ffaf9cbed7589700edc3e5bec54da74f71c1f81ae34e8d64151ae5f4a28"),
        ),
    ] {
        let bytes = fs::read(&path).unwrap();
        fs::remove_file(&path).unwrap();
        rejected_reads(&f, &format!("missing-{name}"), id, &accept);
        fs::write(&path, &bytes).unwrap();
        let mut corrupt = bytes.clone();
        corrupt[0] ^= 1;
        fs::write(&path, corrupt).unwrap();
        rejected_reads(&f, &format!("tamper-{name}"), id, &accept);
        fs::write(&path, bytes).unwrap();
        assert_eq!(inventory(&f.proposals), publications);
        assert_eq!(inventory(&f.projects), projects);
    }
    let revision = f.proposals.join(format!("p-{id}/r-3"));
    let file = fs::read_dir(&revision)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let original = fs::read(&file).unwrap();
    for (name, change) in [
        ("missing-binding", None),
        ("swapped-item", Some("llama1b-correction")),
    ] {
        let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
        if let Some(item) = change {
            value["historical"]["reference"]["item_id"] = json!(item);
        } else {
            value.as_object_mut().unwrap().remove("historical");
        }
        let bytes = serde_json::to_vec(&value).unwrap();
        fs::remove_file(&file).unwrap();
        let replacement = revision.join(format!("{}.json", sha(&bytes)));
        fs::write(&replacement, &bytes).unwrap();
        let inspect = f.invoke(
            name,
            "proposals.inspect",
            json!({"proposal_id":id}),
            None,
            true,
        );
        assert_ne!(inspect.code, 0);
        let retry = f.invoke(
            &format!("{name}-accept"),
            "proposals.accept",
            accept.clone(),
            Some("2"),
            true,
        );
        assert_ne!(retry.code, 0);
        assert!(!f.proposals.join(format!("p-{id}/r-4")).exists());
        fs::remove_file(replacement).unwrap();
        fs::write(&file, &original).unwrap();
    }
    // A one-revision swap has no earlier proposal revision to compare. The real
    // reader must reject its different source/raw binding independently.
    let other = "historical-qwen06-correction";
    let other_revision = f.proposals.join(format!("p-{other}/r-1"));
    let other_file = fs::read_dir(&other_revision)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let other_original = fs::read(&other_file).unwrap();
    let mut missing: serde_json::Value = serde_json::from_slice(&other_original).unwrap();
    missing.as_object_mut().unwrap().remove("historical");
    let missing_bytes = serde_json::to_vec(&missing).unwrap();
    let missing_file = other_revision.join(format!("{}.json", sha(&missing_bytes)));
    fs::remove_file(&other_file).unwrap();
    fs::write(&missing_file, missing_bytes).unwrap();
    let rejected = f.invoke(
        "single-revision-missing-binding",
        "proposals.inspect",
        json!({"proposal_id":other}),
        None,
        true,
    );
    assert_eq!(rejected.value["error"]["code"], "needs_input");
    fs::remove_file(missing_file).unwrap();
    fs::write(&other_file, &other_original).unwrap();
    let mut swapped: serde_json::Value = serde_json::from_slice(&other_original).unwrap();
    swapped["historical"]["reference"]["item_id"] = json!("gemma270-correction");
    let bytes = serde_json::to_vec(&swapped).unwrap();
    let replacement = other_revision.join(format!("{}.json", sha(&bytes)));
    fs::remove_file(&other_file).unwrap();
    fs::write(&replacement, bytes).unwrap();
    let rejected = f.invoke(
        "single-revision-swapped-item",
        "proposals.inspect",
        json!({"proposal_id":other}),
        None,
        true,
    );
    assert_eq!(rejected.value["error"]["code"], "invalid_argument");
    fs::remove_file(replacement).unwrap();
    fs::write(other_file, other_original).unwrap();
    let partial = f.proposals.join(format!("p-{id}/r-4"));
    fs::create_dir(&partial).unwrap();
    assert_ne!(
        f.invoke(
            "partial-publication",
            "proposals.inspect",
            json!({"proposal_id":id}),
            None,
            true
        )
        .code,
        0
    );
    fs::remove_dir(&partial).unwrap();
    assert_eq!(inventory(&f.proposals), publications);
    assert_eq!(inventory(&f.projects), projects);
    f.ok(
        "restored",
        "proposals.inspect",
        json!({"proposal_id":id}),
        None,
    );
    f.assert_original_inputs_unchanged();
}

fn historical_decision(view: &serde_json::Value, id: &str) -> serde_json::Value {
    json!({"proposal_id":view["target"]["proposal_id"],"decision":{
        "decision_id":id,"target":view["target"],"criteria":view["criteria_identity"],
        "verdict":"deferred","note":"Automated persistence fixture only; no human quality review.",
        "declared_reviewer":{"label":"Automated test fixture","reviewer_id":"fixture"},
        "claimed_review_unix_ms":"19"}})
}

fn verify_historical_output_decisions(f: &Fixture) {
    let mut correction = None;
    for task in ["correction", "cards", "highlights"] {
        let id = format!("historical-gemma270-{task}");
        let view = f.ok(
            &format!("output-{task}-target"),
            "proposals.inspect_output_review_target",
            json!({"proposal_id":id}),
            None,
        );
        assert_eq!(view["target"]["task"], task);
        assert_eq!(
            view["evidence_observation"],
            "historical_binding_reverified_in_session"
        );
        assert!(view["state"]["historical"].is_object());
        let payload = historical_decision(&view, "fixture-deferred");
        let stored = f.ok(
            &format!("output-{task}-record"),
            "proposals.record_output_decision",
            payload.clone(),
            Some("3"),
        );
        assert_eq!(stored["decision"]["input"]["verdict"], "deferred");
        assert_eq!(stored["decision"]["counts_as_model_sample"], false);
        assert_eq!(stored["decision"]["automatic_application_enabled"], false);
        assert_eq!(stored["decision"]["input"]["target"], view["target"]);
        assert_eq!(
            f.ok(
                &format!("output-{task}-retry"),
                "proposals.record_output_decision",
                payload,
                Some("3")
            ),
            stored
        );
        let history = f.ok(
            &format!("output-{task}-history"),
            "proposals.output_decisions",
            json!({"proposal_id":id}),
            None,
        );
        assert_eq!(history["records"], json!([stored]));
        assert_eq!(history["current_evidence"], "not_checked");
        assert_eq!(
            f.ok(
                &format!("output-{task}-proposal"),
                "proposals.inspect",
                json!({"proposal_id":id}),
                None
            ),
            view["state"]
        );
        if task == "correction" {
            correction = Some((view, history));
        }
    }
    let (view, history) = correction.unwrap();
    let companion: serde_json::Value =
        serde_json::from_slice(&fs::read(f.companion()).unwrap()).unwrap();
    let row = companion["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["item_id"] == "gemma270-correction")
        .unwrap();
    let candidate = f
        .package()
        .join("objects")
        .join(row["candidate"]["sha256"].as_str().unwrap());
    let original = fs::read(&candidate).unwrap();
    let before = inventory(&f.proposals);
    fs::remove_file(&candidate).unwrap();
    let rejected = f.invoke(
        "output-missing-candidate-target",
        "proposals.inspect_output_review_target",
        json!({"proposal_id":"historical-gemma270-correction"}),
        None,
        true,
    );
    assert_ne!(rejected.code, 0);
    assert_eq!(rejected.value["ok"], false);
    let rejected = f.invoke(
        "output-missing-candidate-record",
        "proposals.record_output_decision",
        historical_decision(&view, "must-not-publish"),
        Some("3"),
        true,
    );
    assert_ne!(rejected.code, 0);
    assert_eq!(rejected.value["ok"], false);
    assert_eq!(
        f.ok(
            "output-missing-candidate-history",
            "proposals.output_decisions",
            json!({"proposal_id":"historical-gemma270-correction"}),
            None
        ),
        history
    );
    assert_eq!(inventory(&f.proposals), before);
    fs::write(&candidate, original).unwrap();
    assert_eq!(
        f.ok(
            "output-restored-target",
            "proposals.inspect_output_review_target",
            json!({"proposal_id":"historical-gemma270-correction"}),
            None
        ),
        view
    );
}

fn import(id: &str, item: &str) -> serde_json::Value {
    json!({"proposal_id":id,"source_id":format!("source-{item}"),"source_revision":"1","reference":{"sha256":"7a7cd05eafa6294e846c8080c976600ea135aa91e9da315ac9f770e45f624b10","bytes":16566,"item_id":item}})
}
fn rejected_reads(f: &Fixture, prefix: &str, id: &str, accept: &serde_json::Value) {
    for (suffix, command, payload, expected) in [
        (
            "inspect",
            "proposals.inspect",
            json!({"proposal_id":id}),
            None,
        ),
        (
            "accept-retry",
            "proposals.accept",
            accept.clone(),
            Some("2"),
        ),
        (
            "project",
            "projects.inspect",
            json!({"project_id":"historical-work","revision":null}),
            None,
        ),
        (
            "reconnect",
            "projects.reconnect_source",
            json!({"project_id":"historical-reconnect",
                "expected_source":{"kind":"local_video","source_id":"historical-locator","path":"C:/media/sintel.mp4"},
                "replacement":{"label":"Must not publish","source":{"kind":"local_video","source_id":"historical-locator","path":"C:/other/sintel.mp4"}}}),
            Some("2"),
        ),
    ] {
        let result = f.invoke(
            &format!("{prefix}-{suffix}"),
            command,
            payload,
            expected,
            true,
        );
        assert_ne!(result.code, 0);
        assert_eq!(result.value["ok"], false);
    }
}

fn verify_cue_task_lifecycles(f: &Fixture) {
    for task in ["cards", "highlights"] {
        let id = format!("historical-gemma270-{task}");
        let before = f.ok(
            &format!("{task}-before"),
            "proposals.inspect",
            json!({"proposal_id":id}),
            None,
        );
        let cue = before["document"]["source"]["cues"][0]["cue_id"]
            .as_str()
            .unwrap();
        let draft = if task == "cards" {
            json!({"task":"cards","cards":[{"front":"Technical fixture question","back":"Technical fixture answer; no quality verdict","source_refs":[cue]}]})
        } else {
            json!({"task":"highlights","highlights":[{"title":"Technical fixture selection; no PTS or quality claim","source_refs":[cue]}]})
        };
        let mut foreign = draft.clone();
        foreign[task][0]["source_refs"] = json!(["not-in-canonical-source"]);
        let rejected = f.invoke(
            &format!("{task}-foreign-reference"),
            "proposals.edit",
            json!({"proposal_id":id,"draft":foreign}),
            Some("1"),
            true,
        );
        assert_eq!(rejected.value["error"]["code"], "invalid_argument");
        f.ok(
            &format!("{task}-edit"),
            "proposals.edit",
            json!({"proposal_id":id,"draft":draft}),
            Some("1"),
        );
        let stale = f.invoke(
            &format!("{task}-stale"),
            "proposals.edit",
            json!({"proposal_id":id,"draft":draft}),
            Some("1"),
            true,
        );
        assert_eq!(stale.value["error"]["code"], "revision_conflict");
        let accept = json!({"proposal_id":id,"target_source_revision":"1"});
        let receipt = f.ok(
            &format!("{task}-accept"),
            "proposals.accept",
            accept.clone(),
            Some("2"),
        );
        assert_eq!(receipt["automatic_application_enabled"], false);
        assert_eq!(receipt["output_revision"], "3");
        assert_eq!(
            f.ok(
                &format!("{task}-accept-retry"),
                "proposals.accept",
                accept,
                Some("2")
            ),
            receipt
        );
        let after = f.ok(
            &format!("{task}-after"),
            "proposals.inspect",
            json!({"proposal_id":id}),
            None,
        );
        assert_eq!(after["document"]["source"], before["document"]["source"]);
        assert_eq!(
            after["document"]["generated"],
            before["document"]["generated"]
        );
        assert_eq!(after["historical"], before["historical"]);
        let project_id = format!("historical-work-{task}");
        let content = json!({"watching":null,"proposals":[{"origin":"live","proposal_id":id,"draft_revision":"3","working_draft":draft}]});
        let created = f.ok(
            &format!("{task}-project-create"),
            "projects.create",
            json!({"project_id":project_id,"title":"Technical task capture","content":content}),
            None,
        );
        let reopened = f.ok(
            &format!("{task}-project-reopen"),
            "projects.inspect",
            json!({"project_id":project_id,"revision":null}),
            None,
        );
        assert_eq!(created, reopened);
        assert_eq!(reopened["project"]["proposals"][0]["captured"], after);
    }
}
