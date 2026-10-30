//! Display projection only. Callback registration and mounting require a later consumer gate.
use super::output_review_state::{OutputReviewPhase, OutputReviewState};
use super::pending_controller::{OutputReviewAction, OutputReviewField};
use super::{NodeLease, QueueBinding};
use kinekino_player::{
    application::proposals::output_review::{CurrentEvidenceStatus, EvidenceObservation},
    domain::proposals::{ProposalPayload, ProposalTask, SourceSnapshot, output_review::*},
};
use meari::domain::host_callbacks::{CallbackReference, CallbackSignature};
use meari::{domain::Value, runtime::IntoValue};
use std::collections::BTreeMap;

const ACTIONS: [OutputReviewAction; 8] = [
    OutputReviewAction::Inspect,
    OutputReviewAction::Select,
    OutputReviewAction::History,
    OutputReviewAction::Verdict(OutputVerdict::Usable),
    OutputReviewAction::Verdict(OutputVerdict::NeedsChanges),
    OutputReviewAction::Verdict(OutputVerdict::Deferred),
    OutputReviewAction::Record,
    OutputReviewAction::Retry,
];
const FIELDS: [OutputReviewField; 4] = [
    OutputReviewField::Note,
    OutputReviewField::ReviewerLabel,
    OutputReviewField::ReviewerId,
    OutputReviewField::ClaimedTime,
];

/// One common namespace avoids collisions with the editor's own busy/text/action props.
pub fn component_prop_name(name: &str) -> String {
    let mut chars = name.chars();
    format!(
        "outputReview{}{}",
        chars
            .next()
            .map(|first| first.to_ascii_uppercase())
            .unwrap_or('_'),
        chars.as_str()
    )
}
pub fn initial_component_props(proposal_id: &str) -> BTreeMap<String, Value> {
    let state = OutputReviewState::new(proposal_id.into());
    let mut props: BTreeMap<_, _> = output_review_props(&state)
        .into_iter()
        .map(|(name, value)| (component_prop_name(&name), value))
        .collect();
    props.insert(
        component_prop_name("statusLabel"),
        Value::String("이 문서의 품질 판정 연결이 아직 활성화되지 않았습니다.".into()),
    );
    for (name, signature) in ACTIONS
        .into_iter()
        .map(|action| (action.binding(), CallbackSignature::Action))
        .chain(
            FIELDS
                .into_iter()
                .map(|field| (field.binding(), CallbackSignature::Text)),
        )
    {
        let name = component_prop_name(name);
        props.insert(
            name.clone(),
            Value::CallbackReference(CallbackReference {
                binding: name,
                signature,
            }),
        );
    }
    props
}
/// Reuse the editor's existing lease and FIFO; no extra node budget, queue or SDK owner.
pub fn bind_component_callbacks(node: &NodeLease, bindings: &mut BTreeMap<String, QueueBinding>) {
    for action in ACTIONS {
        bindings.insert(
            component_prop_name(action.binding()),
            QueueBinding::OutputReviewAction {
                node: node.clone(),
                action,
            },
        );
    }
    for field in FIELDS {
        bindings.insert(
            component_prop_name(field.binding()),
            QueueBinding::OutputReviewText {
                node: node.clone(),
                field,
            },
        );
    }
}
pub fn project_component_props(
    props: &mut BTreeMap<String, Value>,
    state: Option<&OutputReviewState>,
    transport_pending: bool,
) {
    if let Some(state) = state {
        props.extend(
            output_review_props(state)
                .into_iter()
                .map(|(name, value)| (component_prop_name(&name), value)),
        );
        props.insert(component_prop_name("available"), Value::Bool(true));
    }
    if transport_pending {
        for (name, value) in [
            ("busy", true),
            ("fieldsDisabled", true),
            ("canRecord", false),
            ("canRetry", false),
        ] {
            props.insert(component_prop_name(name), Value::Bool(value));
        }
    }
}

fn task(task: ProposalTask) -> &'static str {
    match task {
        ProposalTask::Correction => "문구 교정",
        ProposalTask::Cards => "학습 카드",
        ProposalTask::Highlights => "하이라이트",
    }
}
fn verdict(verdict: OutputVerdict) -> &'static str {
    match verdict {
        OutputVerdict::Usable => "사용 가능",
        OutputVerdict::NeedsChanges => "수정 필요",
        OutputVerdict::Deferred => "판정 보류",
    }
}
fn target_lines(target: &OutputReviewTarget) -> Vec<String> {
    let mut lines = vec![
        format!(
            "대상: 채택 출력 · 규격 {} · {}",
            target.schema_version,
            task(target.task)
        ),
        format!("제안: {}", target.proposal_id),
        format!(
            "채택한 초안 버전: {} · 출력 버전: {}",
            target.draft_revision.0, target.output_revision.0
        ),
        format!(
            "출력 SHA-256: {} · 형식: {}",
            target.output_sha256, target.hash_format
        ),
        format!(
            "원문: {} · 버전: {} · SHA-256: {}",
            target.source_id, target.source_revision.0, target.source_sha256
        ),
        format!(
            "생성 원문 SHA-256: {} · {} bytes",
            target.generated_raw_sha256, target.generated_raw_bytes.0
        ),
    ];
    lines.push(match &target.historical {
        Some(binding) => format!(
            "저장 결합 참조 규격: {} · companion SHA-256: {} · {} bytes · 항목: {}",
            binding.schema_version,
            binding.reference.sha256,
            binding.reference.bytes,
            binding.reference.item_id
        ),
        None => "저장 결합 참조 없음".into(),
    });
    lines
}
fn criteria_lines(document: &CriteriaDocument, identity: &CriteriaIdentity) -> Vec<String> {
    let mut lines = vec![
        format!(
            "기준: {} · 버전: {} · 규격: {} · {}",
            document.criteria_id,
            document.version,
            document.schema_version,
            task(document.task)
        ),
        format!("검토 목적: {}", document.purpose),
        format!("기준 내용 SHA-256: {}", identity.content_sha256),
    ];
    for (index, clause) in document.clauses.iter().enumerate() {
        lines.push(format!(
            "{}. {}\n검토 방법: {}\n기대 결과: {}",
            index + 1,
            clause.criterion_id,
            clause.how_to_review,
            clause.expected
        ));
    }
    lines
}
fn payload_text(payload: &ProposalPayload) -> String {
    match payload {
        ProposalPayload::Correction { edits } => edits
            .iter()
            .map(|edit| format!("{}: {}", edit.cue_id, edit.text))
            .collect::<Vec<_>>(),
        ProposalPayload::DocumentCorrection { edits } => edits
            .iter()
            .map(|edit| {
                format!(
                    "{} · {} · 원문 자산 SHA-256: {}\n{}",
                    edit.source_ref.outer_pointer,
                    edit.source_ref.inner_pointer,
                    edit.source_ref.source_asset_sha256,
                    edit.text
                )
            })
            .collect(),
        ProposalPayload::Cards { cards } => cards
            .iter()
            .enumerate()
            .map(|(i, card)| {
                format!(
                    "카드 {}\n앞면: {}\n뒷면: {}\n원문 참조: {}",
                    i + 1,
                    card.front,
                    card.back,
                    card.source_refs.join(", ")
                )
            })
            .collect(),
        ProposalPayload::Highlights { highlights } => highlights
            .iter()
            .enumerate()
            .map(|(i, highlight)| {
                format!(
                    "하이라이트 {}: {}\n원문 참조: {}",
                    i + 1,
                    highlight.title,
                    highlight.source_refs.join(", ")
                )
            })
            .collect(),
    }
    .join("\n\n")
}
fn input_text(input: &OutputDecisionInput) -> String {
    let mut lines = target_lines(&input.target);
    lines.push(format!(
        "판정 ID: {} · {}\n검토 메모: {}\n선언 검토자: {} · ID: {}\n주장한 검토 시각(Unix ms): {}",
        input.decision_id,
        verdict(input.verdict),
        input.note,
        input.declared_reviewer.label,
        input
            .declared_reviewer
            .reviewer_id
            .as_deref()
            .unwrap_or("미선언"),
        input
            .claimed_review_unix_ms
            .map(|time| time.0.to_string())
            .unwrap_or_else(|| "미선언".into())
    ));
    lines.push(format!(
        "기준 identity: {} · 버전 {} · {} · 목적: {} · SHA-256: {}",
        input.criteria.criteria_id,
        input.criteria.version,
        task(input.criteria.task),
        input.criteria.purpose,
        input.criteria.content_sha256
    ));
    lines.join("\n")
}
fn phase_label(phase: OutputReviewPhase) -> &'static str {
    match phase {
        OutputReviewPhase::Idle => "판정 대상을 조회하세요.",
        OutputReviewPhase::Inspecting => "현재 채택 출력과 검토 기준을 조회하고 있습니다.",
        OutputReviewPhase::Ready => "대상과 전체 기준을 확인한 뒤 명시적으로 선택하세요.",
        OutputReviewPhase::Recording => "품질 판정을 기록하고 있습니다.",
        OutputReviewPhase::LoadingHistory => "저장된 판정 이력을 조회하고 있습니다.",
        OutputReviewPhase::Recorded => "이 채택 출력에 대한 품질 판정을 기록했습니다.",
        OutputReviewPhase::RefreshRequired => "대상이 변경되었습니다. 재조회 후 다시 선택하세요.",
        OutputReviewPhase::ResultUnknown => {
            "기록 결과가 미확인입니다. 이력을 조회하고 동일 요청 재시도는 재조회·재선택 후 명시적으로 실행하세요."
        }
        OutputReviewPhase::Unavailable => "이 환경에서는 품질 판정 기록을 지원하지 않습니다.",
        OutputReviewPhase::NeedsInput => "검토에 필요한 채택 출력 또는 근거를 확인하세요.",
        OutputReviewPhase::Failed => "요청을 완료하지 못했습니다. 오류를 확인하고 다시 조회하세요.",
    }
}
pub fn output_review_props(state: &OutputReviewState) -> BTreeMap<String, Value> {
    let inspection = state.inspection();
    let fields = state.fields();
    let mut props = BTreeMap::from([
        ("available".into(), Value::Bool(false)),
        ("busy".into(), Value::Bool(state.busy())),
        ("hasInspection".into(), Value::Bool(inspection.is_some())),
        ("selected".into(), Value::Bool(state.selected())),
        ("resultUnknown".into(), Value::Bool(state.result_unknown())),
        (
            "canRecord".into(),
            Value::Bool(
                !state.busy()
                    && !state.result_unknown()
                    && state.selected()
                    && fields.verdict.is_some()
                    && !fields.note.trim().is_empty()
                    && !fields.reviewer_label.trim().is_empty(),
            ),
        ),
        ("canRetry".into(), Value::Bool(state.can_retry())),
        (
            "fieldsDisabled".into(),
            Value::Bool(state.busy() || inspection.is_none() || state.result_unknown()),
        ),
        (
            "statusLabel".into(),
            Value::String(phase_label(state.phase()).into()),
        ),
        (
            "errorLabel".into(),
            Value::String(
                state
                    .error()
                    .map(|error| format!("{:?}: {}", error.code, error.message))
                    .unwrap_or_default(),
            ),
        ),
        (
            "verdictLabel".into(),
            Value::String(
                fields
                    .verdict
                    .map(verdict)
                    .unwrap_or("선택하지 않음")
                    .into(),
            ),
        ),
        ("note".into(), Value::String(fields.note.clone())),
        (
            "reviewerLabel".into(),
            Value::String(fields.reviewer_label.clone()),
        ),
        (
            "reviewerId".into(),
            Value::String(fields.reviewer_id.clone().unwrap_or_default()),
        ),
        (
            "claimedTime".into(),
            Value::String(
                fields
                    .claimed_review_unix_ms
                    .map(|time| time.0.to_string())
                    .unwrap_or_default(),
            ),
        ),
    ]);
    let target = inspection
        .map(|i| target_lines(&i.target))
        .unwrap_or_default();
    let criteria = inspection
        .map(|i| criteria_lines(&i.criteria_document, &i.criteria_identity))
        .unwrap_or_default();
    props.insert("targetLines".into(), target.into_value());
    props.insert("criteriaLines".into(), criteria.into_value());
    let (source, generated, draft, accepted, evidence) = inspection.map(|i| {
        let document = &i.state.document;
        let source = match &document.source {
            SourceSnapshot::Cue(source) => source.cues.iter().map(|cue| format!("{}: {}", cue.cue_id, cue.text)).collect::<Vec<_>>().join("\n"),
            SourceSnapshot::Document(source) => source.texts.iter().map(|text| format!("{}: {}", text.inner_pointer, text.text)).collect::<Vec<_>>().join("\n"),
        };
        let draft = document.draft.as_ref().map(payload_text).unwrap_or_else(|| "저장된 초안 없음".into());
        let accepted = i.state.acceptance.as_ref().map(|receipt| payload_text(&receipt.output)).unwrap_or_else(|| "채택 출력 없음".into());
        let evidence = match i.evidence_observation {
            EvidenceObservation::NotHistoricallyBound => "historical 결합 대상이 아닙니다.",
            EvidenceObservation::HistoricalBindingReverifiedInSession => "이번 대상 조회 세션에서 historical 결합 근거를 재검증했습니다. 이후의 가용성·사람 품질 승인을 의미하지 않습니다.",
        };
        (source, document.generated.raw_text.clone(), format!("저장 초안 버전 {}\n{}", document.draft_revision.0, draft), accepted, evidence)
    }).unwrap_or_default();
    for (name, text) in [
        ("sourceText", source),
        ("generatedRaw", generated),
        ("draftText", draft),
        ("acceptedText", accepted),
        ("evidenceLabel", evidence.into()),
    ] {
        props.insert(name.into(), Value::String(text));
    }
    let mut records = Vec::new();
    if let Some(history) = state.history() {
        for record in &history.records {
            let decision = &record.decision;
            let input = &decision.input;
            let mut lines = vec![format!(
                "판정 {} · {} · 기록 SHA-256: {}",
                input.decision_id,
                verdict(input.verdict),
                record.decision_sha256
            )];
            lines.extend(target_lines(&input.target));
            lines.extend(criteria_lines(&decision.criteria_document, &input.criteria));
            lines.push(format!("검토 메모: {}\n선언 검토자: {} · ID: {}\n주장한 검토 시각(Unix ms): {}\n저장 adapter 관측 시각(로컬 시스템 Unix ms): {}\n범위: 채택 출력만 · 자동 적용: {} · 모델 표본 집계: {}", input.note, input.declared_reviewer.label, input.declared_reviewer.reviewer_id.as_deref().unwrap_or("미선언"), input.claimed_review_unix_ms.map(|time| time.0.to_string()).unwrap_or_else(|| "미선언".into()), decision.observed_record_time.unix_ms.0, decision.automatic_application_enabled, decision.counts_as_model_sample));
            records.push(lines.join("\n"));
        }
    }
    props.insert("historyRecords".into(), records.into_value());
    props.insert(
        "attemptedLabel".into(),
        Value::String(
            state
                .attempted()
                .map(|attempted| input_text(&attempted.decision))
                .unwrap_or_default(),
        ),
    );
    props.insert(
        "historyNotChecked".into(),
        Value::Bool(state.history().is_some_and(|history| {
            matches!(history.current_evidence, CurrentEvidenceStatus::NotChecked)
        })),
    );
    props.insert(
        "confirmedLabel".into(),
        Value::String(
            state
                .confirmed()
                .map(|record| {
                    format!(
                        "확인된 판정: {} · {} · 대상: {} / 출력 버전 {} / 출력 SHA-256 {} · 기록 SHA-256: {} · 저장 관측 시각(로컬 Unix ms): {}",
                        record.decision.input.decision_id,
                        verdict(record.decision.input.verdict),
                        record.decision.input.target.proposal_id,
                        record.decision.input.target.output_revision.0,
                        record.decision.input.target.output_sha256,
                        record.decision_sha256,
                        record.decision.observed_record_time.unix_ms.0
                    )
                })
                .unwrap_or_default(),
        ),
    );
    props
}
