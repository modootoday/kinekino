//! P6 settings page model, OS-neutral. GUI, CLI (`kinekino settings page|apply`) and MCP
//! render the same JSON: fields with kind, value and default, validation errors, the token
//! *status* (never its value), the ledger and the limit-approach dialog.
use crate::settings;
use kinekino_core::port::credential::CredentialStore;
use kinekino_inference::settings::{TOKEN_ACCOUNT, TOKEN_SERVICE};
use kinekino_inference::{CostGuard, GuardDecision, InferenceSettings};
use serde_json::{Map, Value, json};

/// `(key, label, kind)`; `kind` is `choice:<a|b>`, `number`, `integer`, `text`, `toggle`,
/// `list` or `secret`. Keys are the `kinekino settings` dotted keys.
pub const FIELDS: &[(&str, &str, &str)] = &[
    (
        "inference.mode",
        "추론 모드",
        "choice:cloud|local_first|local_only",
    ),
    ("inference.models.asr", "음성 인식 모델", "text"),
    ("inference.models.ocr", "OCR 모델", "text"),
    ("inference.models.segment", "구간 분할 모델", "text"),
    ("inference.models.translate", "번역 모델", "text"),
    ("inference.models.summarize", "요약 모델", "text"),
    ("inference.budget.total_usd", "총 한도(USD)", "number"),
    (
        "inference.budget.per_call_usd",
        "호출당 한도(USD)",
        "number",
    ),
    ("inference.budget.monthly_usd", "월 한도(USD)", "number"),
    ("inference.budget.confirm_ratio", "확인 비율", "number"),
    (
        "inference.consent.scope",
        "업로드 동의 범위",
        "choice:task|project|always",
    ),
    ("inference.local_packs", "로컬 팩", "list"),
    (
        "inference.network.connect_timeout_s",
        "연결 제한(초)",
        "integer",
    ),
    (
        "inference.network.total_timeout_s",
        "전체 제한(초)",
        "integer",
    ),
    ("inference.network.max_retries", "재시도 횟수", "integer"),
    (
        "inference.diarization.mode",
        "화자 분리",
        "choice:off|local",
    ),
    ("inference.token", "Hugging Face 토큰", "secret"),
    ("mcp.enabled", "MCP 서버 사용", "toggle"),
    ("mcp.allowed_tools", "MCP 허용 도구", "choice:read_only|all"),
    ("mcp.confirm_destructive", "쓰기 도구 확인", "toggle"),
];

fn lookup(doc: &Value, key: &str) -> Value {
    key.split('.')
        .try_fold(doc.clone(), |v, part| v.get(part).cloned())
        .unwrap_or(Value::Null)
}

fn effective(doc: &Value) -> Result<Value, String> {
    let inference = settings::inference(doc).map_err(|e| e.message.clone())?;
    let mcp = settings::mcp(doc).map_err(|e| e.message.clone())?;
    Ok(json!({ "inference": inference, "mcp": mcp }))
}

/// Limit-approach dialog for `month` (`YYYY-MM`): `None` when a zero-cost call is allowed,
/// else `confirm` (near a cap) or `blocked` (a cap is already reached).
pub fn limit_dialog(inference: &InferenceSettings, ledger: &str, month: &str) -> Option<Value> {
    let guard = match CostGuard::from_jsonl(inference.budget.clone(), ledger) {
        Ok(guard) => guard,
        Err(e) => return Some(json!({ "state": "blocked", "reason": format!("ledger: {e}") })),
    };
    match guard.check(month, Some(0.0)) {
        Ok(GuardDecision::Allow) => None,
        Ok(GuardDecision::Confirm(reason)) => Some(json!({
            "state": "confirm", "reason": reason, "actions": ["continue", "cancel"],
        })),
        Err(e) => {
            Some(json!({ "state": "blocked", "reason": e.to_string(), "actions": ["cancel"] }))
        }
    }
}

/// Whole page. Defaults come from an empty document, so the page shows what is overridden.
pub fn page(
    doc: &Value,
    ledger: &str,
    month: &str,
    credentials: &dyn CredentialStore,
) -> Result<Value, String> {
    let current = effective(doc)?;
    let defaults = effective(&json!({}))?;
    // A credential store that cannot answer (e.g. no OS adapter yet) must not hide the
    // rest of the page: the secret row shows the store error instead of a status.
    let token = credentials
        .get(TOKEN_SERVICE, TOKEN_ACCOUNT)
        .map(|t| t.is_some())
        .map_err(|e| e.message.clone());
    let fields: Vec<Value> = FIELDS
        .iter()
        .map(|(key, label, kind)| {
            let (kind, options) = match kind.split_once(':') {
                Some((k, opts)) => (k, opts.split('|').collect::<Vec<_>>()),
                None => (*kind, Vec::new()),
            };
            let mut field = json!({ "key": key, "label": label, "kind": kind });
            if kind == "secret" {
                match &token {
                    Ok(registered) => field["registered"] = (*registered).into(),
                    Err(e) => {
                        field["registered"] = Value::Null;
                        field["store_error"] = e.clone().into();
                    }
                }
            } else {
                field["value"] = lookup(&current, key);
                field["default"] = lookup(&defaults, key);
            }
            if !options.is_empty() {
                field["options"] = options.into();
            }
            field
        })
        .collect();
    let inference = settings::inference(doc).map_err(|e| e.message.clone())?;
    let spent = CostGuard::from_jsonl(inference.budget.clone(), ledger)
        .map(|g| json!({ "total_usd": g.total(), "month_usd": g.month_total(month) }))
        .unwrap_or(Value::Null);
    Ok(json!({
        "fields": fields,
        "errors": inference.validate(),
        "ledger": spent,
        "month": month,
        "limit_dialog": limit_dialog(&inference, ledger, month),
    }))
}

/// Atomic multi-field edit: every `key → value` applies, or none and all errors return.
pub fn apply(doc: &Value, edits: &Map<String, Value>) -> Result<Value, Vec<String>> {
    let mut next = doc.clone();
    let mut errors = Vec::new();
    for (key, value) in edits {
        if !FIELDS
            .iter()
            .any(|(k, _, kind)| k == key && *kind != "secret")
        {
            errors.push(format!("{key}: not an editable settings field"));
            continue;
        }
        let raw = match value {
            Value::String(text) => text.clone(),
            other => other.to_string(),
        };
        if let Err(e) = settings::set_value(&mut next, key, &raw) {
            errors.push(e.message);
        }
    }
    if errors.is_empty() {
        Ok(next)
    } else {
        Err(errors)
    }
}

/// `YYYY-MM` (UTC) of a Unix time, for ledger months.
pub fn month_of(unix_secs: u64) -> String {
    let days = (unix_secs / 86_400) as i64 + 719_468;
    let era = days.div_euclid(146_097);
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_of_known_dates() {
        assert_eq!(month_of(0), "1970-01");
        assert_eq!(month_of(1_793_318_400), "2026-10");
        assert_eq!(month_of(951_782_400), "2000-02");
    }

    #[test]
    fn every_field_key_reads_from_the_effective_document() {
        let current = effective(&json!({})).unwrap();
        for (key, _, kind) in FIELDS {
            if *kind != "secret" {
                assert_ne!(lookup(&current, key), Value::Null, "{key}");
            }
        }
    }
}
