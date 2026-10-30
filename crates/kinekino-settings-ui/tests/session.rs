//! Headless Meari document checks: rows, edits → typed apply message, reset, status
//! props, raster capture, and that the layout stylesheet only uses design tokens.
use kinekino_settings_ui::Session;
use serde_json::{Value, json};

fn page(dialog: Value) -> Value {
    json!({
        "fields": [
            {"key": "inference.mode", "label": "추론 모드", "kind": "choice", "value": "cloud", "default": "cloud", "options": ["cloud", "local_first", "local_only"]},
            {"key": "inference.models.asr", "label": "음성 인식 모델", "kind": "text", "value": "whisper", "default": "whisper"},
            {"key": "inference.budget.monthly_usd", "label": "월 한도(USD)", "kind": "number", "value": 10, "default": 10},
            {"key": "inference.local_packs", "label": "로컬 팩", "kind": "list", "value": ["a"], "default": []},
            {"key": "network.max_retries", "label": "재시도", "kind": "integer", "value": 2, "default": 2},
            {"key": "inference.token", "label": "토큰", "kind": "secret", "registered": false},
            {"key": "mcp.enabled", "label": "MCP", "kind": "toggle", "value": false, "default": false}
        ],
        "errors": [],
        "ledger": {"total_usd": 1.5, "month_usd": 0.25},
        "month": "2026-10",
        "limit_dialog": dialog
    })
}

fn id_path(doc: &meari_native::infrastructure::document::InteractiveDocument, id: &str) -> String {
    doc.elements()
        .iter()
        .find(|e| e.id == id)
        .unwrap_or_else(|| panic!("{id}"))
        .path
        .clone()
}

#[test]
fn edits_become_one_typed_apply_and_reset_restores_the_page() {
    let (mut session, mut doc) = Session::mount(page(Value::Null)).unwrap();
    assert_eq!(session.rendered(&doc)["rendered"], 7);
    assert_eq!(session.rendered(&doc)["dialog"], false);
    assert!(session.messages(&doc).unwrap().is_empty());

    doc.set_input_value(&id_path(&doc, "field-2"), "12.5")
        .unwrap();
    doc.set_input_value(&id_path(&doc, "field-3"), "a, b")
        .unwrap();
    doc.set_input_value(&id_path(&doc, "field-4"), "3").unwrap();
    for id in ["field-0-2", "field-6", "apply"] {
        let path = id_path(&doc, id);
        doc.dispatch(meari_native::domain::widgets::Action::Activate { path })
            .unwrap();
    }
    let messages = session.messages(&doc).unwrap();
    assert_eq!(messages.len(), 1, "{messages:?}");
    let apply: Value = serde_json::from_str(&messages[0]).unwrap();
    assert_eq!(
        apply,
        json!({"op": "apply", "edits": {
            "inference.mode": "local_only",
            "inference.budget.monthly_usd": 12.5,
            "inference.local_packs": ["a", "b"],
            "network.max_retries": 3,
            "mcp.enabled": true
        }})
    );
    session.reset(&mut doc).unwrap();
    assert!(session.edits(&doc).unwrap().is_empty());
}

#[test]
fn rejected_apply_and_limit_dialog_show_through_props() {
    let dialog =
        json!({"state": "near", "reason": "월 한도의 90%", "actions": ["continue", "cancel"]});
    let (mut session, mut doc) = Session::mount(page(dialog)).unwrap();
    assert_eq!(session.rendered(&doc)["dialog"], true);
    let props = session.update_props(
        None,
        &["inference.budget.confirm_ratio: must be ≤ 1".into()],
    );
    doc.update_module_props(props).unwrap();
    let shown = session.rendered(&doc);
    let errors = shown["errors_shown"].as_str().unwrap();
    assert!(
        errors.contains("전부 또는 없음") && errors.contains("confirm_ratio"),
        "{shown}"
    );
    let path = id_path(&doc, "dialogCancel");
    doc.dispatch(meari_native::domain::widgets::Action::Activate { path })
        .unwrap();
    assert_eq!(
        session.messages(&doc).unwrap(),
        [r#"{"action":"cancel","op":"dialog"}"#]
    );
}

#[test]
fn headless_raster_is_a_painted_png() {
    use meari_native::application::Renderer;
    let (_, mut doc) = Session::mount(page(Value::Null)).unwrap();
    let viewport = doc.viewport.clone();
    let rgba = doc.renderer.rasterize(&viewport).unwrap();
    assert_eq!(rgba.len(), (viewport.width * viewport.height * 4) as usize);
    let distinct: std::collections::BTreeSet<_> =
        rgba.chunks(4).map(|p| [p[0], p[1], p[2]]).collect();
    assert!(
        distinct.len() > 8,
        "flat raster: {} colours",
        distinct.len()
    );
    let out = std::env::var_os("KINEKINO_SETTINGS_SHOT").map(std::path::PathBuf::from);
    if let Some(out) = out {
        kinekino_settings_ui::write_png(&out, viewport.width, viewport.height, &rgba).unwrap();
    }
}

#[test]
fn layout_stylesheet_uses_only_design_tokens() {
    let css = include_str!("../src/settings.css");
    let tokens = include_str!("../../kinekino-player/src/presentation/web_ui/styles/tokens.css");
    let body: String = css
        .lines()
        .filter(|l| !l.trim_start().starts_with("/*"))
        .collect::<Vec<_>>()
        .join("\n");
    for banned in ["#", "rgb(", "hsl(", "px", "font-family"] {
        let hits: Vec<_> = body
            .lines()
            .filter(|l| l.contains(banned) && !l.contains("url("))
            .collect();
        assert!(hits.is_empty(), "{banned}: {hits:?}");
    }
    for var in body.split("var(--").skip(1) {
        let name = var.split(')').next().unwrap();
        assert!(
            tokens.contains(&format!("--{name}:")),
            "unknown token --{name}"
        );
    }
}
