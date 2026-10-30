//! Settings page model (JSON from `kinekino-app`) → bounded Meari TSX source.
//! Field rows are fixed per page; values live in source `useState` slots and the
//! status texts are typed props, so an apply result updates props without a remount.
use serde_json::Value;

/// One editable row: its state slot and how its text maps back to a typed JSON value.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub key: String,
    pub kind: String,
    pub slot: String,
    /// The page value as the state slot shows it (for change detection and reset).
    pub shown: StateText,
}

/// Value as held by a source state slot.
#[derive(Debug, Clone, PartialEq)]
pub enum StateText {
    Text(String),
    Bool(bool),
}

/// Source button ids whose click counters the host watches.
pub const BUTTONS: [&str; 5] = ["apply", "reset", "close", "dialogContinue", "dialogCancel"];

fn quote(text: &str) -> String {
    serde_json::to_string(text).expect("string")
}

/// Escape text for a JSX text child.
fn jsx_text(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '{' | '}' | '<' | '>' | '&' => format!("{{{}}}", quote(&c.to_string())),
            c => c.to_string(),
        })
        .collect()
}

pub fn shown(kind: &str, value: &Value) -> StateText {
    match (kind, value) {
        ("toggle", v) => StateText::Bool(v.as_bool().unwrap_or(false)),
        ("list", Value::Array(items)) => StateText::Text(
            items
                .iter()
                .map(|i| i.as_str().map_or_else(|| i.to_string(), str::to_owned))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        (_, Value::String(s)) => StateText::Text(s.clone()),
        (_, Value::Null) => StateText::Text(String::new()),
        (_, v) => StateText::Text(v.to_string()),
    }
}

/// Typed JSON for a slot's current text (the same rules the old page script used).
pub fn typed(kind: &str, state: &StateText) -> Value {
    match state {
        StateText::Bool(b) => Value::Bool(*b),
        StateText::Text(text) => match kind {
            "number" | "integer" => match text.trim().parse::<f64>() {
                Ok(n) if !text.trim().is_empty() => serde_json::Number::from_f64(n)
                    .map(|n| {
                        if kind == "integer" && n.as_f64().is_some_and(|f| f.fract() == 0.0) {
                            Value::from(n.as_f64().unwrap_or_default() as i64)
                        } else {
                            Value::Number(n)
                        }
                    })
                    .unwrap_or_else(|| Value::String(text.clone())),
                _ => Value::String(text.clone()),
            },
            "list" => Value::Array(
                text.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| Value::String(s.into()))
                    .collect(),
            ),
            _ => Value::String(text.clone()),
        },
    }
}

/// Builds the TSX module and its editable rows from one page model.
pub fn build(page: &Value) -> (String, Vec<Row>) {
    let mut hooks = String::new();
    let mut rows = Vec::new();
    let mut body = String::new();
    for (index, field) in page["fields"].as_array().into_iter().flatten().enumerate() {
        let key = field["key"].as_str().unwrap_or_default();
        let kind = field["kind"].as_str().unwrap_or("text");
        let label = jsx_text(field["label"].as_str().unwrap_or(key));
        let slot = format!("f{index}");
        let setter = format!("setF{index}");
        let id = format!("field-{index}");
        let control = match kind {
            "secret" => {
                let status = if let Some(e) = field["store_error"].as_str() {
                    format!("자격 증명 저장소 사용 불가: {e}")
                } else if field["registered"].as_bool() == Some(true) {
                    "등록됨".to_owned()
                } else {
                    "미등록 (보안 입력으로만 등록)".to_owned()
                };
                format!(
                    "<span id={} className=\"aw-settings-hint\">{}</span>",
                    quote(&id),
                    jsx_text(&status)
                )
            }
            _ => {
                let state = shown(kind, &field["value"]);
                let initial = match &state {
                    StateText::Bool(b) => b.to_string(),
                    StateText::Text(t) => quote(t),
                };
                hooks.push_str(&format!(
                    "  const [{slot}, {setter}] = useState({initial});\n"
                ));
                rows.push(Row {
                    key: key.into(),
                    kind: kind.into(),
                    slot: slot.clone(),
                    shown: state,
                });
                match kind {
                    "toggle" => format!(
                        "<input id={} type=\"checkbox\" aria-label={} checked={{{slot}}} onChange={{event => {setter}(event.target.checked)}} />",
                        quote(&id),
                        quote(field["label"].as_str().unwrap_or(key))
                    ),
                    "choice" => {
                        let mut out = String::from("<span className=\"aw-settings-choice\">");
                        for (o, option) in field["options"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(Value::as_str)
                            .enumerate()
                        {
                            out.push_str(&format!(
                                "<button id={} className=\"aw-button aw-button-compact\" onClick={{() => {setter}({})}}>{}</button>",
                                quote(&format!("{id}-{o}")),
                                quote(option),
                                jsx_text(option)
                            ));
                        }
                        out.push_str(&format!(
                            "<span className=\"aw-settings-value\" role=\"status\">{{{slot}}}</span></span>"
                        ));
                        out
                    }
                    _ => format!(
                        "<input id={} className=\"aw-text-input\" aria-label={} value={{{slot}}} onChange={{event => {setter}(event.currentTarget.value)}} />",
                        quote(&id),
                        quote(field["label"].as_str().unwrap_or(key))
                    ),
                }
            }
        };
        body.push_str(&format!(
            "      <div className=\"aw-settings-row\"><label>{label}</label>{control}</div>\n"
        ));
    }
    for button in BUTTONS {
        hooks.push_str(&format!(
            "  const [{button}Count, set{button}Count] = useState(0);\n"
        ));
    }
    let click = |id: &str| format!("onClick={{() => set{id}Count(previous => previous + 1)}}");
    let source = format!(
        r#"import {{ useState }} from "react";
type Props = {{ ledger: string; errors: string; hasErrors: boolean; dialogText: string; dialogOpen: boolean; canContinue: boolean }};
export function SettingsWindow(props: Props) {{
{hooks}  return <main className="aw-settings-root" aria-label="kinekino 설정">
    <header className="aw-settings-header"><span className="aw-icon aw-icon-brand" aria-hidden="true"></span>kinekino 설정</header>
    <section id="dialog" className="aw-settings-dialog" role="alertdialog" hidden={{!props.dialogOpen}}>
      <p><span className="aw-icon aw-icon-warning" aria-hidden="true"></span><span id="dialog-text">{{props.dialogText}}</span></p>
      <button id="dialogContinue" className="aw-button aw-button-compact aw-button-outline-accent" disabled={{!props.canContinue}} {c_continue}>계속</button>
      <button id="dialogCancel" className="aw-button aw-button-compact" {c_cancel}>취소</button>
    </section>
    <section className="aw-settings-body">
      <p id="ledger" className="aw-settings-ledger">{{props.ledger}}</p>
      <p id="errors" className="aw-settings-errors" role="alert" hidden={{!props.hasErrors}}><span className="aw-icon aw-icon-warning" aria-hidden="true"></span><span id="errors-text">{{props.errors}}</span></p>
{body}    </section>
    <footer className="aw-settings-footer">
      <button id="reset" className="aw-button aw-button-compact" {c_reset}><span className="aw-icon aw-icon-undo" aria-hidden="true"></span>되돌리기</button>
      <button id="apply" className="aw-button aw-button-compact aw-button-outline-accent" {c_apply}>적용</button>
      <button id="close" className="aw-button aw-button-compact" {c_close}>닫기</button>
    </footer>
  </main>;
}}
"#,
        c_continue = click("dialogContinue"),
        c_cancel = click("dialogCancel"),
        c_reset = click("reset"),
        c_apply = click("apply"),
        c_close = click("close"),
    );
    (source, rows)
}

/// Status props for one page model plus the last apply outcome.
pub fn props(page: &Value, apply_errors: &[String]) -> Vec<(&'static str, Value)> {
    // `+ 0.0` turns the ledger's empty-sum `-0.0` into `0.00`.
    let ledger = match page["ledger"].as_object() {
        Some(l) => format!(
            "사용액 {}: 이번 달 ${:.2} / 누적 ${:.2}",
            page["month"].as_str().unwrap_or_default(),
            l.get("month_usd")
                .and_then(Value::as_f64)
                .unwrap_or_default()
                + 0.0,
            l.get("total_usd")
                .and_then(Value::as_f64)
                .unwrap_or_default()
                + 0.0
        ),
        None => "원장을 읽지 못했습니다".into(),
    };
    let mut errors: Vec<String> = page["errors"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| e.as_str().map(str::to_owned))
        .collect();
    if !apply_errors.is_empty() {
        errors.insert(0, "적용하지 않음(전부 또는 없음):".into());
        errors.extend(apply_errors.iter().cloned());
    }
    let dialog = &page["limit_dialog"];
    let dialog_text = dialog["reason"].as_str().map_or_else(String::new, |r| {
        let state = if dialog["state"] == "blocked" {
            "한도 도달: "
        } else {
            "한도 근접: "
        };
        format!("{state}{r}")
    });
    let can_continue = dialog["actions"]
        .as_array()
        .is_some_and(|a| a.iter().any(|x| x == "continue"));
    vec![
        ("ledger", Value::String(ledger)),
        ("hasErrors", Value::Bool(!errors.is_empty())),
        ("errors", Value::String(errors.join("\n"))),
        ("dialogText", Value::String(dialog_text)),
        ("dialogOpen", Value::Bool(dialog.is_object())),
        ("canContinue", Value::Bool(can_continue)),
    ]
}
