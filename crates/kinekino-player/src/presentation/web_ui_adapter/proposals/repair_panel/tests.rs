//! Projection integration over synthetic original SDK state; no live-route admission.
use super::super::{
    MutationReceipt, ResolvedMutation, ReviewSession,
    review_feedback::{FeedbackLabels, ReviewFeedback},
};
use super::*;
use crate::application::api::{ProposalClient, ReviewClient};
use crate::{
    application::{
        api::{PlayerSdk, RequestContext},
        proposals::{CreateProposal, InspectProposal},
    },
    domain::{api_values::DecimalU64, proposals::*},
};
use meari::{
    application::{Language, plugins::PluginScope, state_program::StateProgram},
    domain::{Value, build::ContentHash},
};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn labels() -> PanelLabels {
    PanelLabels {
        feedback: FeedbackLabels::english(),
        revision: "Document".into(),
        source_identity: "Source".into(),
        quality_unknown: "Not approved".into(),
        selected_item: "Manual input".into(),
        range: "Unverified".into(),
        result: String::new(),
    }
}
fn pins() -> IntentPins {
    IntentPins {
        contract: ContentHash::of(b"synthetic-contract"),
        provider: ContentHash::of(b"synthetic-provider"),
        request: ContentHash::of(b"synthetic-request"),
        reply: ContentHash::of(b"reply"),
    }
}
fn prepare(review: &ReviewSession) -> PreparedPanel {
    prepare_repair_panel(
        review,
        review.scope(),
        labels(),
        PanelBudget {
            entries: 32,
            text_bytes: 262144,
        },
        pins(),
    )
    .unwrap()
}
fn inspect(sdk: &PlayerSdk, id: &str) -> StoredProposal {
    sdk.inspect_proposal(
        RequestContext::default(),
        InspectProposal {
            proposal_id: id.into(),
        },
    )
    .unwrap()
}
fn context(revision: DecimalU64) -> RequestContext {
    RequestContext {
        expected_revision: Some(revision.0.to_string()),
        ..RequestContext::default()
    }
}
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(all(windows, feature = "native-proposal-window"))]
fn strict(props: BTreeMap<String, Value>) {
    use super::super::{
        bootstrap::BootstrapPanel, frame_composition::FrameComposition,
        product_window::product_modules_for_panel,
    };
    let (entry, mut inventory) = product_modules_for_panel(BootstrapPanel::ManualRepair);
    let provider = meari::parse_module_graph(&entry, &inventory).unwrap();
    let (graph, frame) = FrameComposition::new("Contract test".into(), String::new())
        .prepare(provider, &mut inventory)
        .unwrap();
    assert!(!frame.raw_review_enabled());
    // This is the real strict NodeSlot factory, not key filtering or a model of its checks.
    frame
        .mount(
            graph.clone(),
            props.clone(),
            "manual-exact-contract".into(),
            1,
        )
        .unwrap()
        .project()
        .unwrap();
    let mut missing = props.clone();
    missing.remove("sourceChoice");
    assert!(
        frame
            .mount(graph.clone(), missing, "manual-missing".into(), 1)
            .is_err()
    );
    let mut extra = props;
    extra.insert(
        "unknownPrivateExtra".into(),
        Value::String("must reject".into()),
    );
    assert!(frame.mount(graph, extra, "manual-extra".into(), 1).is_err());
}

fn review_states(review: &ReviewSession) {
    use super::super::{
        output_review_state::OutputReviewState, pending_host::PendingOutput,
        raw_review_state::RawReviewState,
    };
    use crate::{
        application::{
            api::{ApiError, ErrorCode},
            model_review::StoredDecision,
            proposals::output_review::{CurrentEvidenceStatus, OutputDecisionHistory},
        },
        domain::model_review::{ModelCondition, ReviewDecision, ReviewVerdict},
    };
    // UI-only synthetic state, never a stored decision or a human quality verdict.
    let mut raw = RawReviewState::new(Box::new(|| Ok("synthetic-ui-only".into())));
    for verdict in [
        ReviewVerdict::Usable,
        ReviewVerdict::NeedsChanges,
        ReviewVerdict::Deferred,
    ] {
        raw.verdict = Some(verdict);
        raw.note = "Synthetic projection sample".into();
        raw.history = Some(vec![StoredDecision {
            decision_sha256: "b".repeat(64),
            decision: ReviewDecision {
                schema_version: "1".into(),
                package_sha256: "c".repeat(64),
                package_id: "synthetic-package".into(),
                task: "synthetic-task".into(),
                item_id: "synthetic-item".into(),
                decision_id: "synthetic-ui-only".into(),
                candidate_condition: ModelCondition {
                    model_id: "synthetic-model".into(),
                    revision: "synthetic-revision".into(),
                    precision: "synthetic".into(),
                    engine_id: "synthetic-engine".into(),
                    engine_revision: "synthetic".into(),
                    input_conditions: BTreeMap::new(),
                },
                verdict,
                note: raw.note.clone(),
                issue_ranges: Vec::new(),
                review_ready: false,
                automated_prerequisites_passed: false,
                sample_review_passed: false,
                automatic_application_enabled: false,
                scope: "Synthetic UI projection only".into(),
            },
        }]);
        for busy in [false, true] {
            let mut props = prepare(review).props;
            super::super::raw_review_panel::project_component_props(
                &mut props,
                Some(&raw),
                busy,
                None,
                None,
            );
            strict(props);
        }
    }
    raw.error = Some(ApiError::new(
        ErrorCode::NeedsInput,
        "Synthetic review validation error",
    ));
    let mut props = prepare(review).props;
    super::super::raw_review_panel::project_component_props(
        &mut props,
        Some(&raw),
        false,
        None,
        None,
    );
    strict(props);
    let mut output = OutputReviewState::new(review.stored().document.proposal_id.clone());
    let (token, _) = output.load_history().unwrap();
    let mut props = prepare(review).props;
    super::super::output_review_panel::project_component_props(&mut props, Some(&output), true);
    strict(props);
    output
        .complete(
            &token,
            Ok(PendingOutput::OutputDecisionsListed(
                OutputDecisionHistory {
                    records: Vec::new(),
                    current_evidence: CurrentEvidenceStatus::NotChecked,
                },
            )),
        )
        .unwrap();
    let mut props = prepare(review).props;
    super::super::output_review_panel::project_component_props(&mut props, Some(&output), false);
    strict(props);
    let (token, _) = output.inspect().unwrap();
    output
        .complete(
            &token,
            Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Synthetic output review validation error",
            )),
        )
        .unwrap();
    let mut props = prepare(review).props;
    super::super::output_review_panel::project_component_props(&mut props, Some(&output), false);
    strict(props);
}

fn sdk_output_review_states(sdk: &PlayerSdk, review: &ReviewSession) {
    use super::super::{
        output_review_state::{DecisionFields, OutputReviewState},
        pending_host::{PendingOutput, PendingRequest},
    };
    use crate::{
        application::proposals::output_review::{InspectOutputReviewTarget, ListOutputDecisions},
        domain::proposals::output_review::OutputVerdict,
    };
    let id = &review.stored().document.proposal_id;
    let mut state = OutputReviewState::new(id.clone());
    let (token, _) = state.inspect().unwrap();
    let inspection = sdk
        .inspect_output_review_target(
            RequestContext::default(),
            InspectOutputReviewTarget {
                proposal_id: id.clone(),
            },
        )
        .unwrap();
    state
        .complete(
            &token,
            Ok(PendingOutput::OutputReviewInspected(Box::new(inspection))),
        )
        .unwrap();
    state.select().unwrap();
    state
        .replace_fields(DecisionFields {
            verdict: Some(OutputVerdict::Deferred),
            note: "Synthetic UI contract regression; no user quality approval".into(),
            reviewer_label: "Synthetic test fixture".into(),
            ..DecisionFields::default()
        })
        .unwrap();
    let mut props = prepare(review).props;
    super::super::output_review_panel::project_component_props(&mut props, Some(&state), false);
    strict(props);
    let (token, request) = state.record(format!("{id}-synthetic-decision")).unwrap();
    let PendingRequest::RecordOutputDecision { revision, input } = request else {
        panic!("Original output review record request required")
    };
    let mut props = prepare(review).props;
    super::super::output_review_panel::project_component_props(&mut props, Some(&state), true);
    strict(props);
    let recorded = sdk
        .record_output_decision(context(revision), *input)
        .unwrap();
    state
        .complete(
            &token,
            Ok(PendingOutput::OutputDecisionRecorded(Box::new(recorded))),
        )
        .unwrap();
    let mut props = prepare(review).props;
    super::super::output_review_panel::project_component_props(&mut props, Some(&state), false);
    strict(props);
    let (token, _) = state.load_history().unwrap();
    let history = sdk
        .output_decisions(
            RequestContext::default(),
            ListOutputDecisions {
                proposal_id: id.clone(),
            },
        )
        .unwrap();
    state
        .complete(&token, Ok(PendingOutput::OutputDecisionsListed(history)))
        .unwrap();
    let mut props = prepare(review).props;
    super::super::output_review_panel::project_component_props(&mut props, Some(&state), false);
    assert!(
        matches!(&props["outputReviewHistoryRecords"], Value::Array(records) if !records.items.is_empty())
    );
    strict(props);
}

#[test]
#[cfg(all(windows, feature = "native-proposal-window"))]
#[ignore = "Assigned original SDK scratch projection slot; no GUI or live-frame acceptance"]
fn manual_repair_state_projections_match_exact_component_contract() {
    let root = std::env::current_dir()
        .unwrap()
        .join("runtime/quality-review/manual-projection")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    let _scratch = Scratch(root.clone());
    std::fs::create_dir_all(&root).unwrap();
    let sdk = PlayerSdk::default()
        .with_proposal_store(crate::infrastructure::proposals::ProposalFiles::open(&root).unwrap());
    for (index, task) in [
        ProposalTask::Cards,
        ProposalTask::Highlights,
        ProposalTask::Correction,
    ]
    .into_iter()
    .enumerate()
    {
        let id = format!("manual-projection-{index}");
        sdk.create_proposal(
            RequestContext::default(),
            CreateProposal {
                proposal_id: id.clone(),
                source: SourceSnapshot::Cue(CueSourceSnapshot {
                    source_id: "synthetic-source".into(),
                    source_revision: DecimalU64(1),
                    source_sha256: "a".repeat(64),
                    cues: vec![SourceCue {
                        cue_id: "cue".into(),
                        text: "Synthetic original 日本語".into(),
                        start_us: None,
                        end_us: None,
                    }],
                }),
                generated: GeneratedResult {
                    task,
                    raw_text: "synthetic invalid schema".into(),
                },
            },
        )
        .unwrap();
        let component = meari::parse(
            "type Props={};export function Scope(props:Props){return <main/>;}",
            Language::Tsx,
        )
        .unwrap();
        let program = StateProgram::mount(component, format!("manual-state-{index}"), 1).unwrap();
        let mut review = ReviewSession::new(
            PluginScope {
                instance: program.snapshot().unwrap().instance,
                source_generation: 1,
            },
            inspect(&sdk, &id),
        )
        .unwrap();
        let scope = review.scope().clone();
        assert!(review.draft().is_none());
        strict(prepare(&review).props);
        assert!(review.choose_manual_source(&scope, "0".into()).is_err());
        let mut invalid_choice = prepare(&review).props;
        apply_feedback(
            &ReviewFeedback::from_review(&review)
                .with_local_error(Some(super::super::ViewError::InvalidSelection)),
            &mut invalid_choice,
            &labels(),
        );
        strict(invalid_choice);
        review.choose_manual_source(&scope, "1".into()).unwrap();
        strict(prepare(&review).props);
        review.begin_manual_repair(&scope).unwrap();
        strict(prepare(&review).props);
        let invalid = review.begin_save(&scope).unwrap();
        strict(prepare(&review).props);
        let error = sdk
            .edit_proposal(context(invalid.expected_revision), invalid.request)
            .unwrap_err();
        review.complete(&invalid.token, Err(error)).unwrap();
        let failed = prepare(&review).props;
        assert!(
            matches!(&failed["resultLabel"], Value::String(text) if text.contains("SDK error"))
        );
        strict(failed);
        for index in 0..review.text_fields().len() {
            let field = review.text_fields().remove(index);
            review
                .change_text(&scope, &field.input, "Human synthetic edit".into())
                .unwrap();
        }
        strict(prepare(&review).props);
        let stale = review.begin_save(&scope).unwrap();
        let error = sdk
            .edit_proposal(
                context(DecimalU64(stale.expected_revision.0 + 10)),
                stale.request,
            )
            .unwrap_err();
        review.complete(&stale.token, Err(error)).unwrap();
        strict(prepare(&review).props);
        let save = review.begin_save(&scope).unwrap();
        strict(prepare(&review).props);
        let saved = sdk
            .edit_proposal(context(save.expected_revision), save.request)
            .unwrap();
        review
            .require_refresh(
                &save.token,
                MutationReceipt::Saved(saved),
                crate::application::api::ApiError::new(
                    crate::application::api::ErrorCode::NeedsInput,
                    "Synthetic follow-up inspect unavailable",
                ),
            )
            .unwrap();
        strict(prepare(&review).props);
        review
            .complete_refresh(&save.token, Ok(inspect(&sdk, &id)))
            .unwrap();
        strict(prepare(&review).props);
        let accept = review.begin_accept(&scope).unwrap();
        strict(prepare(&review).props);
        let accepted = sdk
            .accept_proposal(context(accept.expected_revision), accept.request)
            .unwrap();
        review
            .complete(
                &accept.token,
                Ok(ResolvedMutation {
                    mutation: MutationReceipt::Accepted(accepted),
                    inspected: inspect(&sdk, &id),
                }),
            )
            .unwrap();
        let mut props = prepare(&review).props;
        apply_feedback(
            &ReviewFeedback::from_review(&review)
                .with_local_error(Some(super::super::ViewError::InvalidSelection)),
            &mut props,
            &labels(),
        );
        assert!(!props.contains_key("revisionLabel"));
        strict(props);
        review_states(&review);
        sdk_output_review_states(&sdk, &review);
    }
    drop(sdk);
    std::fs::remove_dir_all(&root).expect("Synthetic SDK scratch must close and release cleanly");
}
