use super::*;
use crate::domain::{api_values::DecimalU64, proposals::*};
use serde_json::json;

fn stored(task: ProposalTask) -> StoredProposal {
    let output = match task {
        ProposalTask::Correction => ProposalPayload::Correction {
            edits: vec![Correction {
                cue_id: "cue-1".into(),
                text: "Synthetic corrected text.".into(),
            }],
        },
        ProposalTask::Cards => ProposalPayload::Cards {
            cards: vec![Card {
                front: "Synthetic question?".into(),
                back: "Synthetic answer.".into(),
                source_refs: vec!["cue-1".into()],
            }],
        },
        ProposalTask::Highlights => ProposalPayload::Highlights {
            highlights: vec![Highlight {
                title: "Synthetic selection".into(),
                source_refs: vec!["cue-1".into()],
            }],
        },
    };
    let source = SourceSnapshot::Cue(CueSourceSnapshot {
        source_id: "source-1".into(),
        source_revision: DecimalU64(3),
        source_sha256: "a".repeat(64),
        cues: vec![SourceCue {
            cue_id: "cue-1".into(),
            text: "Synthetic source.".into(),
            start_us: None,
            end_us: None,
        }],
    });
    StoredProposal {
        document: ProposalDocument {
            schema_version: "1".into(),
            proposal_id: "proposal-1".into(),
            source,
            generated: GeneratedResult {
                task,
                raw_text: serde_json::to_string(&output).unwrap(),
            },
            draft_revision: DecimalU64(2),
            state: DraftState::Editable,
            draft: Some(output.clone()),
            import_error: None,
        },
        acceptance: Some(AcceptanceReceipt {
            proposal_id: "proposal-1".into(),
            draft_revision: DecimalU64(1),
            output_revision: DecimalU64(2),
            source_id: "source-1".into(),
            source_revision: DecimalU64(3),
            source_sha256: "a".repeat(64),
            output,
            automatic_application_enabled: false,
        }),
        historical: None,
    }
}
fn criteria(task: ProposalTask) -> CriteriaDocument {
    CriteriaDocument {
        schema_version: "1".into(),
        criteria_id: "synthetic-criteria".into(),
        version: "fixture-v1".into(),
        task,
        purpose: "Synthetic contract fixture; not approved quality criteria".into(),
        clauses: vec![OutputCriterion {
            criterion_id: "fixture-only".into(),
            how_to_review: "Inspect the synthetic value.".into(),
            expected: "No actual quality approval is asserted.".into(),
        }],
    }
}
fn input(state: &StoredProposal, criteria: &CriteriaDocument) -> OutputDecisionInput {
    OutputDecisionInput {
        decision_id: "decision-1".into(),
        target: OutputReviewTarget::from_stored(state).unwrap(),
        criteria: criteria.identity().unwrap(),
        verdict: OutputVerdict::Deferred,
        note: "Synthetic domain contract only.".into(),
        declared_reviewer: DeclaredReviewer {
            label: "Declared fixture reviewer".into(),
            reviewer_id: None,
        },
        claimed_review_unix_ms: Some(DecimalU64(19)),
    }
}
fn observed(value: u64) -> ObservedRecordTime {
    ObservedRecordTime {
        unix_ms: DecimalU64(value),
        clock: RecordClock::LocalSystemUnixMs,
    }
}

#[test]
fn three_tasks_bind_accepted_output_without_claiming_human_edit_or_model_success() {
    for task in [
        ProposalTask::Correction,
        ProposalTask::Cards,
        ProposalTask::Highlights,
    ] {
        let state = stored(task);
        let criteria = criteria(task);
        let request = input(&state, &criteria);
        let decision = OutputDecision::prepare(request, &state, criteria, observed(23)).unwrap();
        decision.validate_against(&state).unwrap();
        assert_eq!(decision.input.target.kind, OutputTargetKind::AcceptedOutput);
        assert_eq!(decision.scope, OutputDecisionScope::AcceptedOutputOnly);
        assert!(!decision.automatic_application_enabled && !decision.counts_as_model_sample);
        let value = serde_json::to_value(&decision).unwrap();
        assert!(value.get("candidate_condition").is_none());
        assert!(value.get("human_edited").is_none());
    }
}

#[test]
fn hash_has_fixed_envelope_order_and_preserves_string_bytes_and_array_order() {
    let payload = ProposalPayload::Correction {
        edits: vec![Correction {
            cue_id: "c".into(),
            text: "x\r\n한국어".into(),
        }],
    };
    let expected = r#"{"hash_format":"accepted_typed_output_v1","output":{"edits":[{"cue_id":"c","text":"x\r\n한국어"}],"task":"correction"}}"#;
    assert_eq!(
        accepted_output_sha256(&payload).unwrap(),
        super::hash::raw_sha256(expected.as_bytes())
    );
    let first = json!({"b":"2", "a":"1"});
    let second: serde_json::Value = serde_json::from_str(r#"{"a":"1","b":"2"}"#).unwrap();
    assert_eq!(
        super::hash::canonical_sha256(&first).unwrap(),
        super::hash::canonical_sha256(&second).unwrap()
    );
    assert!(super::hash::canonical_sha256(&json!({"n": 1})).is_err());
    assert_ne!(
        super::hash::canonical_sha256(&json!(["a", "b"])).unwrap(),
        super::hash::canonical_sha256(&json!(["b", "a"])).unwrap()
    );
    assert_ne!(
        super::hash::canonical_sha256(&"e\u{301}").unwrap(),
        super::hash::canonical_sha256(&"é").unwrap()
    );
}

#[test]
fn identical_output_hash_cannot_exchange_proposal_source_or_generation() {
    let state = stored(ProposalTask::Correction);
    let target = OutputReviewTarget::from_stored(&state).unwrap();
    let mut changed = state.clone();
    changed.document.proposal_id = "proposal-2".into();
    changed.acceptance.as_mut().unwrap().proposal_id = "proposal-2".into();
    assert_eq!(
        target.output_sha256,
        OutputReviewTarget::from_stored(&changed)
            .unwrap()
            .output_sha256
    );
    assert!(target.validate_against(&changed).is_err());
    changed = state.clone();
    changed.document.generated.raw_text.push(' ');
    assert!(target.validate_against(&changed).is_err());
    changed = state.clone();
    if let SourceSnapshot::Cue(source) = &mut changed.document.source {
        source.source_sha256 = "b".repeat(64);
    }
    changed.acceptance.as_mut().unwrap().source_sha256 = "b".repeat(64);
    assert!(target.validate_against(&changed).is_err());
}

#[test]
fn stale_acceptance_hash_version_payload_and_historical_binding_are_rejected() {
    let state = stored(ProposalTask::Correction);
    let target = OutputReviewTarget::from_stored(&state).unwrap();
    let mut edited = state.clone();
    edited.acceptance = None;
    edited.document.draft_revision = DecimalU64(3);
    assert!(target.validate_against(&edited).is_err());
    let mut exchanged = target.clone();
    exchanged.hash_format = "unknown".into();
    assert!(exchanged.validate_against(&state).is_err());
    exchanged = target.clone();
    exchanged.output_sha256 = "c".repeat(64);
    assert!(exchanged.validate_against(&state).is_err());
    let mut historical = state.clone();
    historical.document.schema_version = HISTORICAL_DOCUMENT_VERSION.into();
    historical.historical = Some(HistoricalBinding::new(CompanionReference {
        sha256: "d".repeat(64),
        bytes: 17,
        item_id: "synthetic-item".into(),
    }));
    assert!(target.validate_against(&historical).is_err());
    let bound = OutputReviewTarget::from_stored(&historical).unwrap();
    historical.historical.as_mut().unwrap().reference.item_id = "other-item".into();
    assert!(bound.validate_against(&historical).is_err());
}

#[test]
fn criteria_identity_covers_full_content_version_task_and_purpose() {
    let criteria = criteria(ProposalTask::Correction);
    let original = criteria.identity().unwrap();
    for field in 0..4 {
        let mut changed = criteria.clone();
        match field {
            0 => changed.clauses[0].expected.push('!'),
            1 => changed.version = "fixture-v2".into(),
            2 => changed.task = ProposalTask::Cards,
            _ => changed.purpose.push('!'),
        }
        assert_ne!(
            original.content_sha256,
            changed.identity().unwrap().content_sha256
        );
    }
    let mut duplicate = criteria.clone();
    duplicate.clauses.push(duplicate.clauses[0].clone());
    assert!(duplicate.validate().is_err());
    let state = stored(ProposalTask::Correction);
    let request = input(&state, &criteria);
    let mut changed = criteria;
    changed.clauses[0].expected.push('!');
    assert!(request.validate(&state, &changed).is_err());
}

#[test]
fn declared_and_observed_times_remain_distinct_and_retry_matches_only_exact_request() {
    let state = stored(ProposalTask::Cards);
    let criteria = criteria(ProposalTask::Cards);
    let request = input(&state, &criteria);
    let decision =
        OutputDecision::prepare(request.clone(), &state, criteria, observed(23)).unwrap();
    assert_eq!(decision.input.claimed_review_unix_ms.unwrap().0, 19);
    assert_eq!(decision.observed_record_time.unix_ms.0, 23);
    assert!(decision.matches_request(&request));
    let mut changed = request.clone();
    changed.note.push('!');
    assert!(!decision.matches_request(&changed));
    changed = request;
    changed.declared_reviewer.reviewer_id = Some("other-reviewer".into());
    assert!(!decision.matches_request(&changed));
    let reopened: OutputDecision =
        serde_json::from_slice(&serde_json::to_vec(&decision).unwrap()).unwrap();
    assert_eq!(reopened, decision);
}

#[test]
fn invalid_metadata_and_model_or_automatic_flags_do_not_validate() {
    let state = stored(ProposalTask::Highlights);
    let criteria = criteria(ProposalTask::Highlights);
    let request = input(&state, &criteria);
    let mut invalid = request.clone();
    invalid.note.clear();
    assert!(invalid.validate(&state, &criteria).is_err());
    invalid = request.clone();
    invalid.declared_reviewer.label = "\0".into();
    assert!(invalid.validate(&state, &criteria).is_err());
    let mut decision = OutputDecision::prepare(request, &state, criteria, observed(23)).unwrap();
    decision.counts_as_model_sample = true;
    assert!(decision.validate_against(&state).is_err());
    decision.counts_as_model_sample = false;
    decision.automatic_application_enabled = true;
    assert!(decision.validate_against(&state).is_err());
}

#[test]
fn creating_domain_decision_preserves_existing_proposal_and_candidate_contract_bytes() {
    let state = stored(ProposalTask::Correction);
    let before = serde_json::to_vec(&state).unwrap();
    let criteria = criteria(ProposalTask::Correction);
    let decision =
        OutputDecision::prepare(input(&state, &criteria), &state, criteria, observed(23)).unwrap();
    assert_eq!(serde_json::to_vec(&state).unwrap(), before);
    let encoded = serde_json::to_value(decision).unwrap();
    assert!(encoded.get("sample_review_passed").is_none());
    assert!(encoded.get("candidate_condition").is_none());
    assert!(encoded["input"]["target"].get("package_sha256").is_none());
}

#[test]
fn stored_readback_validates_without_current_proposal_or_evidence() {
    let bytes = {
        let state = stored(ProposalTask::Correction);
        let criteria = criteria(ProposalTask::Correction);
        let decision =
            OutputDecision::prepare(input(&state, &criteria), &state, criteria, observed(23))
                .unwrap();
        serde_json::to_vec(&decision).unwrap()
    }; // The source state is no longer available; no evidence reader is supplied.
    let reopened: OutputDecision = serde_json::from_slice(&bytes).unwrap();
    reopened.validate_stored().unwrap();
    let mut current = stored(ProposalTask::Correction);
    current.acceptance = None;
    assert!(reopened.validate_against(&current).is_err());
    reopened.validate_stored().unwrap();
    let value = serde_json::to_value(reopened).unwrap();
    assert!(value.get("evidence_verified").is_none());
}

#[test]
fn deserialized_record_tampering_is_rejected_without_current_state() {
    let state = stored(ProposalTask::Correction);
    let criteria = criteria(ProposalTask::Correction);
    let decision =
        OutputDecision::prepare(input(&state, &criteria), &state, criteria, observed(23)).unwrap();
    let original = serde_json::to_value(decision).unwrap();
    for case in 0..9 {
        let mut changed = original.clone();
        match case {
            0 => changed["counts_as_model_sample"] = json!(true),
            1 => changed["automatic_application_enabled"] = json!(true),
            2 => {
                changed["criteria_document"]["clauses"][0]["expected"] =
                    json!("Changed after recording")
            }
            3 => changed["input"]["target"]["output_sha256"] = json!("malformed"),
            4 => changed["input"]["target"]["output_revision"] = json!("9"),
            5 => changed["input"]["target"]["proposal_id"] = json!("../foreign"),
            6 => changed["input"]["declared_reviewer"]["label"] = json!(""),
            7 => changed["input"]["note"] = json!(""),
            _ => changed["schema_version"] = json!("unsupported"),
        }
        let reopened: OutputDecision = serde_json::from_value(changed).unwrap();
        assert!(reopened.validate_stored().is_err(), "tamper case {case}");
    }
    // Closed enums reject a forged scope/clock during deserialization itself.
    let mut changed = original.clone();
    changed["scope"] = json!("model_sample");
    assert!(serde_json::from_value::<OutputDecision>(changed).is_err());
    let mut changed = original;
    changed["observed_record_time"]["clock"] = json!("trusted_external_clock");
    assert!(serde_json::from_value::<OutputDecision>(changed).is_err());
}
