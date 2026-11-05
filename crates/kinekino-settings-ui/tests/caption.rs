//! Settings window caption: player-style `aw-window-chrome` (M2-6 / UI-5).
use kinekino_settings_ui::Session;
use serde_json::{Value, json};

fn page() -> Value {
    json!({
        "fields": [
            {"key": "inference.mode", "label": "추론 모드", "kind": "choice", "value": "cloud", "default": "cloud", "options": ["cloud", "local_first"]}
        ],
        "errors": [],
        "ledger": {"total_usd": 0.0, "month_usd": 0.0},
        "month": "2026-11",
        "limit_dialog": null
    })
}

fn has_class(
    doc: &meari_native::infrastructure::document::InteractiveDocument,
    class: &str,
) -> bool {
    doc.elements().iter().any(|e| {
        e.attributes
            .get("class")
            .is_some_and(|c| c.split_whitespace().any(|t| t == class))
    })
}

fn has_id(doc: &meari_native::infrastructure::document::InteractiveDocument, id: &str) -> bool {
    doc.elements().iter().any(|e| e.id == id)
}

#[test]
fn mounted_document_has_player_style_window_chrome() {
    let (_, doc) = Session::mount(page()).unwrap();
    assert!(
        has_class(&doc, "aw-window-chrome"),
        "expected .aw-window-chrome caption row"
    );
    assert!(
        has_id(&doc, "windowClose"),
        "caption close id must be windowClose"
    );
    assert!(has_id(&doc, "minimize"), "caption minimize button");
    assert!(has_id(&doc, "maximize"), "caption maximize button");
    // Footer close stays; caption must not reuse that id.
    assert!(has_id(&doc, "close"), "footer #close kept");
    // Product chrome replaces the old settings-only header as the sole title row.
    assert!(
        !has_class(&doc, "aw-settings-header"),
        "aw-settings-header must not remain as the window chrome"
    );
}

#[test]
fn activating_window_close_yields_close_op() {
    let (mut session, mut doc) = Session::mount(page()).unwrap();
    let path = doc
        .elements()
        .iter()
        .find(|e| e.id == "windowClose")
        .expect("windowClose")
        .path
        .clone();
    doc.dispatch(meari_native::domain::widgets::Action::Activate { path })
        .unwrap();
    let messages = session.messages(&doc).unwrap();
    assert_eq!(messages.len(), 1, "{messages:?}");
    let body: Value = serde_json::from_str(&messages[0]).unwrap();
    assert_eq!(body, json!({"op": "close"}));
}

#[test]
fn footer_close_still_yields_close_op() {
    let (mut session, mut doc) = Session::mount(page()).unwrap();
    let path = doc
        .elements()
        .iter()
        .find(|e| e.id == "close")
        .expect("close")
        .path
        .clone();
    doc.dispatch(meari_native::domain::widgets::Action::Activate { path })
        .unwrap();
    let messages = session.messages(&doc).unwrap();
    assert_eq!(messages.len(), 1, "{messages:?}");
    let body: Value = serde_json::from_str(&messages[0]).unwrap();
    assert_eq!(body, json!({"op": "close"}));
}
