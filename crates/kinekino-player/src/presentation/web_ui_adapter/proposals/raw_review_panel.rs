//! Raw-model review presentation only. SDK owns evidence, decisions and history policy.
use super::raw_review_state::{RawReviewButton, RawReviewField, RawReviewState};
use super::{NodeLease, QueueBinding};
use kinekino_player::domain::model_review::{ReviewRange, ReviewVerdict};
use meari::{
    domain::{
        Value,
        host_callbacks::{CallbackReference, CallbackSignature},
    },
    runtime::IntoValue,
};
use std::collections::BTreeMap;

const BUTTONS: [(&str, RawReviewButton); 12] = [
    (
        "rawReviewOnInspectRetained",
        RawReviewButton::InspectRetained,
    ),
    (
        "rawReviewOnHistoryRetained",
        RawReviewButton::HistoryRetained,
    ),
    (
        "rawReviewOnReleaseRetained",
        RawReviewButton::ReleaseRetained,
    ),
    ("rawReviewOnOpenPackage", RawReviewButton::OpenPackage),
    ("rawReviewOnOpenItem", RawReviewButton::OpenItem),
    ("rawReviewOnReadSource", RawReviewButton::ReadSource),
    ("rawReviewOnReadCandidate", RawReviewButton::ReadCandidate),
    ("rawReviewOnHistory", RawReviewButton::History),
    ("rawReviewOnRecord", RawReviewButton::Record),
    (
        "rawReviewOnUsable",
        RawReviewButton::Verdict(ReviewVerdict::Usable),
    ),
    (
        "rawReviewOnNeedsChanges",
        RawReviewButton::Verdict(ReviewVerdict::NeedsChanges),
    ),
    (
        "rawReviewOnDeferred",
        RawReviewButton::Verdict(ReviewVerdict::Deferred),
    ),
];
const FIELDS: [(&str, RawReviewField); 3] = [
    ("rawReviewOnPackageChange", RawReviewField::Package),
    ("rawReviewOnItemChange", RawReviewField::Item),
    ("rawReviewOnNoteChange", RawReviewField::Note),
];
pub fn bind_callbacks(node: &NodeLease, bindings: &mut BTreeMap<String, QueueBinding>) {
    for (name, action) in BUTTONS {
        bindings.insert(
            name.into(),
            QueueBinding::RawReviewAction {
                node: node.clone(),
                action,
            },
        );
    }
    for (name, field) in FIELDS {
        bindings.insert(
            name.into(),
            QueueBinding::RawReviewText {
                node: node.clone(),
                field,
            },
        );
    }
}
fn label(verdict: ReviewVerdict) -> &'static str {
    match verdict {
        ReviewVerdict::Usable => "사용 가능",
        ReviewVerdict::NeedsChanges => "수정 필요",
        ReviewVerdict::Deferred => "보류",
    }
}
pub fn project_component_props(
    props: &mut BTreeMap<String, Value>,
    state: Option<&RawReviewState>,
    busy: bool,
    retained: Option<&super::pending_controller::RawReviewRetained>,
    recovery: Option<&super::pending_controller::RawRecovery>,
) {
    let recovery_text = recovery
        .map(|value| {
            let history = value
                .history
                .as_ref()
                .map(|entries| {
                    entries
                        .iter()
                        .map(|entry| {
                            format!(
                                "{} · {} · {}",
                                entry.decision.decision_id,
                                entry.decision.note,
                                entry.decision_sha256
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();
            let error = value
                .error
                .as_ref()
                .map(|error| error.message.as_str())
                .unwrap_or_default();
            format!("{}\n{}\n{}", value.summary(), error, history)
        })
        .unwrap_or_default();
    props.insert("rawReviewRecovery".into(), Value::String(recovery_text));
    for (name, enabled) in [
        (
            "rawReviewCanInspectRetained",
            !busy
                && retained.is_some()
                && recovery.is_none_or(|value| value.unmatched.is_none() && !value.released),
        ),
        (
            "rawReviewCanHistoryRetained",
            !busy
                && retained.is_some()
                && recovery.is_some_and(|value| value.can_read_history() && !value.released),
        ),
        (
            "rawReviewCanReleaseRetained",
            !busy && retained.is_some() && recovery.is_some_and(|value| value.can_release()),
        ),
    ] {
        props.insert(name.into(), Value::Bool(enabled));
    }
    let unprojected = retained.is_some();
    let identity = retained
        .map(|carrier| {
            let cancellation = match carrier.transport.as_ref() {
                Some((_, true)) => "요청됨 · 취소 완료는 미확인",
                Some((_, false)) => "요청 없음으로 관측",
                None => "전송 관측 정보 없음",
            };
            format!(
                "{}\n취소 요청: {}",
                carrier.rejected.token.identity_label(),
                cancellation
            )
        })
        .unwrap_or_default();
    let raw = retained
        .map(|carrier| {
            let encoded = match &carrier.rejected.outcome {
                Err(error) => serde_json::to_string_pretty(error),
                Ok(super::pending_host::PendingOutput::RawReviewInspected(value)) => {
                    serde_json::to_string_pretty(value)
                }
                Ok(super::pending_host::PendingOutput::RawReviewAsset(value)) => {
                    serde_json::to_string_pretty(value)
                }
                Ok(super::pending_host::PendingOutput::RawReviewRecorded(value)) => {
                    serde_json::to_string_pretty(value)
                }
                Ok(super::pending_host::PendingOutput::RawReviewHistory(value)) => {
                    serde_json::to_string_pretty(value)
                }
                Ok(_) => {
                    return "원 타입의 응답을 보존했어요. 현재 패널에서 표시할 수 없는 종류예요."
                        .into();
                }
            };
            encoded.unwrap_or_else(|error| format!("응답 표시 오류: {error}"))
        })
        .unwrap_or_default();
    props.insert("rawReviewRetainedIdentity".into(), Value::String(identity));
    props.insert("rawReviewRetainedResponse".into(), Value::String(raw));
    props.insert(
        "rawReviewAttempt".into(),
        Value::String(
            state
                .and_then(|state| state.attempted_record.as_ref())
                .map(|request| {
                    format!(
                        "마지막 명시 기록 요청: {} · {} · {}",
                        request.package_sha256, request.item_id, request.decision_id
                    )
                })
                .unwrap_or_default(),
        ),
    );
    let item = state.and_then(RawReviewState::item);
    props.insert(
        "rawReviewRangeStatus".into(),
        Value::String(range_status(
            item.is_some(),
            item.and_then(|item| item.range.as_ref()),
        )),
    );
    let model = item
        .map(|item| {
            format!(
                "{} · revision {} · {} · {} {}",
                item.candidate.condition.model_id,
                item.candidate.condition.revision,
                item.candidate.condition.precision,
                item.candidate.condition.engine_id,
                item.candidate.condition.engine_revision
            )
        })
        .unwrap_or_default();
    props.insert("rawReviewModel".into(), Value::String(model));
    props.insert(
        "rawReviewSelection".into(),
        Value::String(
            item.map(|item| format!("선택한 후보: {}", item.item_id))
                .unwrap_or_else(|| "자료 안에서 검토할 후보를 선택해 주세요.".into()),
        ),
    );
    props.insert(
        "rawReviewCriteria".into(),
        item.map(|item| {
            item.criteria
                .iter()
                .map(|criterion| format!("{} · {}", criterion.how_to_review, criterion.expected))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
        .into_value(),
    );
    let source = item
        .and_then(|item| state?.assets.get(&item.source.sha256))
        .map(|asset| asset.utf8.as_str())
        .unwrap_or_default();
    let candidate = item
        .and_then(|item| {
            let [asset] = item.candidate.assets.as_slice() else {
                return None;
            };
            state?.assets.get(&asset.sha256)
        })
        .map(|asset| asset.utf8.as_str())
        .unwrap_or_default();
    for (name, value) in [
        (
            "rawReviewPackage",
            state.map(|s| s.package_input.clone()).unwrap_or_default(),
        ),
        (
            "rawReviewItem",
            state.map(|s| s.item_input.clone()).unwrap_or_default(),
        ),
        (
            "rawReviewNote",
            state.map(|s| s.note.clone()).unwrap_or_default(),
        ),
        ("rawReviewSource", source.into()),
        ("rawReviewCandidate", candidate.into()),
        (
            "rawReviewVerdict",
            state
                .and_then(|s| s.verdict)
                .map(label)
                .unwrap_or("판정을 선택해 주세요")
                .into(),
        ),
        (
            "rawReviewError",
            state
                .and_then(|s| s.error.as_ref())
                .map(|error| error.message.clone())
                .unwrap_or_default(),
        ),
        (
            "rawReviewSaved",
            state
                .and_then(|s| s.saved.as_ref())
                .map(|saved| {
                    format!(
                        "{} · {}",
                        label(saved.decision.verdict),
                        saved.decision_sha256
                    )
                })
                .unwrap_or_default(),
        ),
        (
            "rawReviewStatus",
            if unprojected {
                "원 응답을 보존했어요. 현재 선택에 적용하지 않았어요."
            } else if state.is_none() {
                "원결과 검토 연결이 필요해요."
            } else if busy {
                "요청을 처리하고 있어요."
            } else if state.is_some_and(|s| s.record_result_unknown) {
                "기록 결과를 확인하려면 이력을 다시 열어 주세요."
            } else {
                "원 모델 결과에 대한 표본 판정이에요. 수정본 채택과 별도로 기록해요."
            }
            .into(),
        ),
    ] {
        props.insert(name.into(), Value::String(value));
    }
    let items = state
        .and_then(|s| s.package.as_ref())
        .map(|p| {
            p.package
                .items
                .iter()
                .map(|item| item.item_id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let issues = item
        .map(|item| item.known_issues.clone())
        .unwrap_or_default();
    let history = state
        .and_then(|s| s.history.as_ref())
        .map(|history| {
            history
                .iter()
                .map(|saved| {
                    format!(
                        "판정 ID: {} · {} · {} · {}",
                        saved.decision.decision_id,
                        label(saved.decision.verdict),
                        saved.decision.note,
                        saved.decision_sha256
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for (name, values) in [
        ("rawReviewItems", items),
        ("rawReviewIssues", issues),
        ("rawReviewHistory", history),
    ] {
        props.insert(name.into(), values.into_value());
    }
    for (name, value) in [
        (
            "rawReviewFieldsDisabled",
            busy || unprojected || state.is_none(),
        ),
        (
            "rawReviewCanRead",
            !busy
                && !unprojected
                && item.is_some()
                && state.is_some_and(RawReviewState::selection_matches_input),
        ),
        (
            "rawReviewCanRecord",
            !busy
                && !unprojected
                && state.is_some_and(|s| {
                    s.item().is_some()
                        && s.selection_matches_input()
                        && s.verdict.is_some()
                        && !s.note.trim().is_empty()
                        && !s.record_result_unknown
                }),
        ),
    ] {
        props.insert(name.into(), Value::Bool(value));
    }
    for (name, _) in BUTTONS {
        props.insert(
            name.into(),
            Value::CallbackReference(CallbackReference {
                binding: name.into(),
                signature: CallbackSignature::Action,
            }),
        );
    }
    for (name, _) in FIELDS {
        props.insert(
            name.into(),
            Value::CallbackReference(CallbackReference {
                binding: name.into(),
                signature: CallbackSignature::Text,
            }),
        );
    }
}

fn range_status(selected: bool, range: Option<&ReviewRange>) -> String {
    if !selected {
        return "후보를 선택하면 자료의 시간 범위를 확인할 수 있어요.".into();
    }
    match range {
        None => {
            "시간 범위가 제공되지 않았어요. 영상 시간과 장면 정확성은 확인되지 않았어요.".into()
        }
        Some(range) => format!(
            "자료의 시간 범위: {}–{} μs. 영상 PTS 연결과 장면 정확성은 확인되지 않았어요.",
            range.start_us.0, range.end_us.0
        ),
    }
}

#[cfg(test)]
mod display_tests {
    use super::*;
    use kinekino_player::domain::api_values::DecimalU64;

    #[test]
    fn selected_null_range_never_becomes_zero_or_verified_pts() {
        assert!(range_status(false, None).contains("후보를 선택"));
        let missing = range_status(true, None);
        assert!(missing.contains("제공되지"));
        assert!(!missing.contains("0–"));
        let original = ReviewRange {
            start_us: DecimalU64(0),
            end_us: DecimalU64(750_000),
        };
        let provided = range_status(true, Some(&original));
        assert!(provided.contains("0–750000 μs"));
        assert!(provided.contains("영상 PTS 연결과 장면 정확성은 확인되지"));
    }
}
