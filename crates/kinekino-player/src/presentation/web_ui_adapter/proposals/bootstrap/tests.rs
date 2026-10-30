use super::*;
use meari::domain::build::ContentHash;
use serde_json::json;

fn labels() -> BootstrapLabels {
    BootstrapLabels {
        panel: PanelLabels {
            feedback: crate::presentation::web_ui_adapter::proposals::review_feedback::FeedbackLabels::korean(),
            revision: "문서 2".into(),
            source_identity: "명시 원본".into(),
            quality_unknown: "품질 미판정".into(),
            selected_item: "선택 2".into(),
            range: "시간 정보 없음".into(),
            result: String::new(),
        },
        source_count_label: "원문 2개".into(),
        unselected_source_text: "원문을 선택하세요".into(),
    }
}
fn cue_source() -> SourceSnapshot {
    SourceSnapshot::Cue(CueSourceSnapshot {
        source_id: "source-a".into(),
        source_revision: DecimalU64(7),
        source_sha256: "a".repeat(64),
        cues: ["a", "b"]
            .into_iter()
            .map(|id| SourceCue {
                cue_id: id.into(),
                text: format!("원문 {id}"),
                start_us: None,
                end_us: None,
            })
            .collect(),
    })
}
fn document_source() -> SourceSnapshot {
    let prefix = "Edit this ASR draft. Correct punctuation only. Do not guess unclear words. JSON only: {\"proposed\":string,\"uncertain_words\":[string]}. Under100words. /no_think\n";
    let asset =
        json!({"messages":[{"role":"user", "content":format!("{prefix}[\"원문 1\",\"原文 2\"]")}]})
            .to_string();
    let hash = ContentHash::of(asset.as_bytes());
    SourceSnapshot::Document(
        DocumentSourceSnapshot::from_asset(
            "source-doc".into(),
            DecimalU64(7),
            hash.as_str().into(),
            asset,
        )
        .unwrap(),
    )
}
// Legacy persisted JSON is decoded by the canonical type; metadata is not mirrored here.
fn stored(
    source: SourceSnapshot,
    task: ProposalTask,
    draft: Option<ProposalPayload>,
) -> StoredProposal {
    let state = if draft.is_some() {
        "editable"
    } else {
        "needs_repair"
    };
    let value = json!({"document": {
        "schema_version": source.schema_version(), "proposal_id":"proposal-a", "source":source,
        "generated":{"task":task,"raw_text":"preserved raw"}, "draft_revision":"2",
        "state":state,"draft":draft,"import_error": if state == "needs_repair" { Some("원본 파싱 오류") } else { None }
    },"acceptance":null});
    serde_json::from_value(value).unwrap()
}
fn selection(stored: &StoredProposal, target: BootstrapTarget) -> BootstrapSelection {
    let source = &stored.document.source;
    BootstrapSelection {
        proposal_id: stored.document.proposal_id.clone(),
        document_revision: stored.document.draft_revision,
        source_id: source.source_id().into(),
        source_revision: source.source_revision(),
        source_sha256: source.source_sha256().into(),
        target,
    }
}
fn cards() -> StoredProposal {
    stored(
        cue_source(),
        ProposalTask::Cards,
        Some(ProposalPayload::Cards {
            cards: vec![
                Card {
                    front: "first".into(),
                    back: "first back".into(),
                    source_refs: vec!["a".into()],
                },
                Card {
                    front: "second".into(),
                    back: "second back".into(),
                    source_refs: vec!["b".into(), "a".into()],
                },
            ],
        }),
    )
}
fn assert_text(props: &BTreeMap<String, Value>, key: &str, expected: &str) {
    assert_eq!(props.get(key), Some(&Value::String(expected.into())));
}
#[test]
fn explicit_second_card_preserves_reference_order_and_entire_snapshot() {
    let stored = cards();
    let before = serde_json::to_value(&stored).unwrap();
    let selected = selection(&stored, BootstrapTarget::Card { index: 1 });
    let output = project_initial(&stored, &selected, labels()).unwrap();
    assert_eq!(output.panel, BootstrapPanel::Card);
    assert_text(&output.props, "front", "second");
    assert_text(&output.props, "back", "second back");
    assert_text(&output.props, "sourceText", "원문 b\n원문 a");
    assert_eq!(
        output.props["sourceRefs"],
        vec!["b".to_owned(), "a".to_owned()].into_value()
    );
    assert!(!output.props.contains_key("selectedItemLabel"));
    assert!(std::ptr::eq(output.stored, &stored));
    assert_eq!(serde_json::to_value(&stored).unwrap(), before);
    assert!(matches!(
        output.props["onFrontChange"],
        Value::CallbackReference(CallbackReference {
            signature: CallbackSignature::Text,
            ..
        })
    ));
}
#[test]
fn stale_identity_is_rejected_in_every_dimension() {
    let stored = cards();
    for dimension in 0..5 {
        let mut selected = selection(&stored, BootstrapTarget::Card { index: 1 });
        match dimension {
            0 => selected.proposal_id.push('x'),
            1 => selected.document_revision.0 += 1,
            2 => selected.source_id.push('x'),
            3 => selected.source_revision.0 += 1,
            _ => selected.source_sha256 = "b".repeat(64),
        }
        assert_eq!(
            project_initial(&stored, &selected, labels()).err(),
            Some(BootstrapError::StaleSelection)
        );
    }
}
#[test]
fn no_hidden_first_item_or_empty_typed_draft() {
    let snapshot = cards();
    let selected = selection(&snapshot, BootstrapTarget::Card { index: 2 });
    assert_eq!(
        project_initial(&snapshot, &selected, labels()).err(),
        Some(BootstrapError::ItemOutOfRange)
    );
    let selected = selection(&snapshot, BootstrapTarget::Highlight { index: 0 });
    assert_eq!(
        project_initial(&snapshot, &selected, labels()).err(),
        Some(BootstrapError::TargetMismatch)
    );
    let repair = stored(cue_source(), ProposalTask::Cards, None);
    let selected = selection(&repair, BootstrapTarget::Card { index: 0 });
    assert_eq!(
        project_initial(&repair, &selected, labels()).err(),
        Some(BootstrapError::NoTypedDraft)
    );
}
#[test]
fn highlight_preserves_missing_pts_and_supplied_range_label() {
    let stored = stored(
        cue_source(),
        ProposalTask::Highlights,
        Some(ProposalPayload::Highlights {
            highlights: vec![
                Highlight {
                    title: "first".into(),
                    source_refs: vec!["a".into()],
                },
                Highlight {
                    title: "second".into(),
                    source_refs: vec!["b".into(), "a".into()],
                },
            ],
        }),
    );
    let selected = selection(&stored, BootstrapTarget::Highlight { index: 1 });
    let output = project_initial(&stored, &selected, labels()).unwrap();
    assert_text(&output.props, "title", "second");
    assert_text(&output.props, "rangeLabel", "시간 정보 없음");
    assert!(
        output
            .stored
            .document
            .source
            .cues()
            .iter()
            .all(|cue| cue.start_us.is_none() && cue.end_us.is_none())
    );
}
#[test]
fn correction_requires_exact_selected_cue() {
    let stored = stored(
        cue_source(),
        ProposalTask::Correction,
        Some(ProposalPayload::Correction {
            edits: vec![Correction {
                cue_id: "b".into(),
                text: "수정문".into(),
            }],
        }),
    );
    let selected = selection(
        &stored,
        BootstrapTarget::CueCorrection {
            index: 0,
            cue_id: "b".into(),
        },
    );
    assert_text(
        &project_initial(&stored, &selected, labels()).unwrap().props,
        "workingText",
        "수정문",
    );
    let selected = selection(
        &stored,
        BootstrapTarget::CueCorrection {
            index: 0,
            cue_id: "a".into(),
        },
    );
    assert_eq!(
        project_initial(&stored, &selected, labels()).err(),
        Some(BootstrapError::SourceReferenceMismatch)
    );
}
#[test]
fn repair_none_never_selects_first_or_creates_draft() {
    let stored = stored(document_source(), ProposalTask::Correction, None);
    let before = serde_json::to_value(&stored).unwrap();
    let selected = selection(
        &stored,
        BootstrapTarget::DocumentRepair { source_ref: None },
    );
    let output = project_initial(&stored, &selected, labels()).unwrap();
    assert_text(&output.props, "sourceChoice", "");
    assert_text(&output.props, "selectedSourceText", "원문을 선택하세요");
    assert_text(&output.props, "generatedRaw", "preserved raw");
    assert_text(&output.props, "importError", "원본 파싱 오류");
    for key in [
        "inputDisabled",
        "saveDisabled",
        "acceptDisabled",
        "startDisabled",
    ] {
        assert_eq!(output.props[key], Value::Bool(true));
    }
    assert!(output.stored.document.draft.is_none());
    assert_eq!(serde_json::to_value(&stored).unwrap(), before);
}
#[test]
fn repair_second_source_requires_full_canonical_reference() {
    let stored = stored(document_source(), ProposalTask::Correction, None);
    let reference = stored
        .document
        .source
        .as_document()
        .unwrap()
        .text_reference("/1")
        .unwrap();
    let selected = selection(
        &stored,
        BootstrapTarget::DocumentRepair {
            source_ref: Some(reference.clone()),
        },
    );
    let output = project_initial(&stored, &selected, labels()).unwrap();
    assert_text(&output.props, "sourceChoice", "2");
    assert_text(&output.props, "selectedSourceText", "原文 2");
    assert_text(&output.props, "workingText", "");
    assert_eq!(output.props["startDisabled"], Value::Bool(false));
    for dimension in 0..3 {
        let mut wrong = reference.clone();
        match dimension {
            0 => wrong.source_asset_sha256 = "b".repeat(64),
            1 => wrong.outer_pointer.push('x'),
            _ => wrong.inner_pointer = "/99".into(),
        }
        let selected = selection(
            &stored,
            BootstrapTarget::DocumentRepair {
                source_ref: Some(wrong),
            },
        );
        assert_eq!(
            project_initial(&stored, &selected, labels()).err(),
            Some(BootstrapError::SourceReferenceMismatch)
        );
    }
}
#[test]
fn document_correction_matches_full_reference_without_mutation() {
    let source = document_source();
    let reference = source.as_document().unwrap().text_reference("/1").unwrap();
    let stored = stored(
        source,
        ProposalTask::Correction,
        Some(ProposalPayload::DocumentCorrection {
            edits: vec![DocumentCorrection {
                source_ref: reference.clone(),
                text: "수정된 원문".into(),
            }],
        }),
    );
    let selected = selection(
        &stored,
        BootstrapTarget::DocumentCorrection {
            index: 0,
            source_ref: reference.clone(),
        },
    );
    assert_text(
        &project_initial(&stored, &selected, labels()).unwrap().props,
        "workingText",
        "수정된 원문",
    );
    let mut wrong = reference;
    wrong.outer_pointer.push('x');
    let selected = selection(
        &stored,
        BootstrapTarget::DocumentCorrection {
            index: 0,
            source_ref: wrong,
        },
    );
    assert_eq!(
        project_initial(&stored, &selected, labels()).err(),
        Some(BootstrapError::SourceReferenceMismatch)
    );
}

fn bound_cards() -> StoredProposal {
    let mut snapshot = cards();
    snapshot.historical = Some(HistoricalBinding::new(CompanionReference {
        sha256: "c".repeat(64),
        bytes: 42,
        item_id: "original-candidate".into(),
    }));
    snapshot.document.schema_version = HISTORICAL_DOCUMENT_VERSION.into();
    snapshot
}

#[test]
fn canonical_v3_binding_survives_projection_and_legacy_none_stays_none() {
    let snapshot = bound_cards();
    let bytes = serde_json::to_vec(&snapshot).unwrap();
    let reopened: StoredProposal = serde_json::from_slice(&bytes).unwrap();
    let selected = selection(&reopened, BootstrapTarget::Card { index: 1 });
    let output = project_initial(&reopened, &selected, labels()).unwrap();
    assert_eq!(output.stored.document.schema_version, "3");
    assert_eq!(output.stored.historical, snapshot.historical);
    assert_eq!(serde_json::to_vec(output.stored).unwrap(), bytes);
    assert!(cards().historical.is_none());
    let view = super::super::project(&reopened).unwrap();
    assert_eq!(
        view.historical_reference.as_ref(),
        Some(&snapshot.historical.as_ref().unwrap().reference)
    );
    assert!(!view.generation_provenance_bound && !view.quality_decision_known);
}

#[test]
fn historical_source_labels_are_shared_without_claiming_verification() {
    let cases = [
        (
            ProposalTask::Cards,
            ProposalPayload::Cards {
                cards: vec![Card {
                    front: "front".into(),
                    back: "back".into(),
                    source_refs: vec!["a".into()],
                }],
            },
        ),
        (
            ProposalTask::Highlights,
            ProposalPayload::Highlights {
                highlights: vec![Highlight {
                    title: "title".into(),
                    source_refs: vec!["a".into()],
                }],
            },
        ),
        (
            ProposalTask::Correction,
            ProposalPayload::Correction {
                edits: vec![Correction {
                    cue_id: "a".into(),
                    text: "text".into(),
                }],
            },
        ),
    ];
    for (task, payload) in cases {
        for bound in [false, true] {
            let mut snapshot = stored(cue_source(), task, Some(payload.clone()));
            if bound {
                snapshot.document.schema_version = HISTORICAL_DOCUMENT_VERSION.into();
                snapshot.historical = Some(HistoricalBinding::new(CompanionReference {
                    sha256: "c".repeat(64),
                    bytes: 42,
                    item_id: "stored-item".into(),
                }));
            }
            let bytes = serde_json::to_vec(&snapshot).unwrap();
            let selected = selection(
                &snapshot,
                match task {
                    ProposalTask::Cards => BootstrapTarget::Card { index: 0 },
                    ProposalTask::Highlights => BootstrapTarget::Highlight { index: 0 },
                    ProposalTask::Correction => BootstrapTarget::CueCorrection {
                        index: 0,
                        cue_id: "a".into(),
                    },
                },
            );
            let output = project_initial(&snapshot, &selected, labels()).unwrap();
            let expected = super::super::historical_source_props(
                snapshot
                    .historical
                    .as_ref()
                    .map(|binding| &binding.reference),
            );
            for (key, value) in expected {
                assert_eq!(output.props[&key], value);
            }
            let program = meari::application::state_program::StateProgram::mount(
                meari::parse(
                    "type Props={};export function Scope(props:Props){return <main/>;}",
                    meari::application::Language::Tsx,
                )
                .unwrap(),
                "historical-source-display".into(),
                1,
            )
            .unwrap();
            let review = super::super::ReviewSession::new(
                meari::application::plugins::PluginScope {
                    instance: program.snapshot().unwrap().instance,
                    source_generation: 1,
                },
                snapshot.clone(),
            )
            .unwrap();
            let pins = super::super::IntentPins {
                contract: ContentHash::of(b"display-contract"),
                provider: ContentHash::of(b"display-provider"),
                request: ContentHash::of(b"display-request"),
                reply: ContentHash::of(b"display-reply"),
            };
            let budget = super::super::panel_model::PanelBudget {
                entries: 8,
                text_bytes: 8192,
            };
            let live = if task == ProposalTask::Correction {
                let selected = super::super::correction_panel::CorrectionSelection::at(
                    &review,
                    review.scope(),
                    0,
                )
                .unwrap();
                super::super::correction_panel::prepare_correction(
                    &review,
                    review.scope(),
                    &selected,
                    super::super::correction_panel::CorrectionLabels {
                        panel: labels().panel,
                        source_count: "원문 2개".into(),
                        unselected_source: "선택 필요".into(),
                    },
                    budget,
                    pins,
                )
                .unwrap()
            } else {
                let selected =
                    super::super::panel_model::PanelSelection::at(&review, review.scope(), 0)
                        .unwrap();
                super::super::panel_model::prepare_panel(
                    &review,
                    review.scope(),
                    &selected,
                    labels().panel,
                    budget,
                    pins,
                )
                .unwrap()
            };
            for key in ["historicalReferenceLabel", "historicalVerificationLabel"] {
                assert_eq!(live.props[key], output.props[key]);
            }
            let view = super::super::project(&snapshot).unwrap();
            assert_eq!(view.historical_reference.is_some(), bound);
            assert!(!view.generation_provenance_bound && !view.quality_decision_known);
            assert_eq!(serde_json::to_vec(&snapshot).unwrap(), bytes);
            assert_text(
                &output.props,
                "historicalVerificationLabel",
                "저장된 참조 표시만 제공합니다. 증거 파일 검증·현재 작업·정밀도·라이선스·사람 품질 승인은 미확인입니다.",
            );
            if bound {
                assert_text(
                    &output.props,
                    "historicalReferenceLabel",
                    &format!("SHA-256: {} · 42 bytes · 항목: stored-item", "c".repeat(64)),
                );
            } else {
                assert_text(
                    &output.props,
                    "historicalReferenceLabel",
                    "저장된 결합 참조 없음",
                );
            }
        }
    }
}

#[test]
fn inconsistent_document_or_binding_version_is_rejected() {
    for dimension in 0..4 {
        let mut snapshot = bound_cards();
        match dimension {
            0 => snapshot.historical = None,
            1 => snapshot.document.schema_version = "1".into(),
            2 => snapshot.historical.as_mut().unwrap().schema_version = "2".into(),
            _ => snapshot
                .historical
                .as_mut()
                .unwrap()
                .reference
                .sha256
                .clear(),
        }
        let selected = selection(&snapshot, BootstrapTarget::Card { index: 1 });
        assert!(matches!(
            project_initial(&snapshot, &selected, labels()),
            Err(BootstrapError::InvalidSnapshot(_))
        ));
    }
}

#[test]
fn acceptance_does_not_turn_original_binding_into_output_quality_approval() {
    let mut snapshot = bound_cards();
    let original_binding = snapshot.historical.clone();
    let original_raw = snapshot.document.generated.raw_text.clone();
    let ProposalPayload::Cards { cards } = snapshot.document.draft.as_mut().unwrap() else {
        unreachable!()
    };
    cards[1].front = "사용자가 수정한 출력".into();
    snapshot.document.draft_revision = DecimalU64(3);
    snapshot.acceptance = Some(AcceptanceReceipt {
        proposal_id: snapshot.document.proposal_id.clone(),
        draft_revision: DecimalU64(2),
        source_id: snapshot.document.source.source_id().into(),
        source_revision: snapshot.document.source.source_revision(),
        source_sha256: snapshot.document.source.source_sha256().into(),
        output_revision: DecimalU64(3),
        output: snapshot.document.draft.clone().unwrap(),
        automatic_application_enabled: false,
    });
    let selected = selection(&snapshot, BootstrapTarget::Card { index: 1 });
    let output = project_initial(&snapshot, &selected, labels()).unwrap();
    assert_text(&output.props, "front", "사용자가 수정한 출력");
    assert_text(&output.props, "qualityLabel", "품질 미판정");
    assert_text(&output.props, "generatedRaw", &original_raw);
    assert_eq!(output.stored.historical, original_binding);
    assert!(
        !output
            .stored
            .acceptance
            .as_ref()
            .unwrap()
            .automatic_application_enabled
    );
    assert!(!output.props.contains_key("qualityApproved"));
}
