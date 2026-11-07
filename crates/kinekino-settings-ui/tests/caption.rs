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

#[test]
fn settings_caption_css_uses_shared_chrome_tokens() {
    // M2-V5: settings caption stays on shared aw-window-chrome tokens (player family).
    // Prefer shared components over mockup-only titlebar hex; lift flex quirks so
    // settings and player caption rows share one chrome contract.
    let settings_css = include_str!("../src/settings.css");
    let components =
        include_str!("../../kinekino-player/src/presentation/web_ui/styles/components.css");
    let chrome_rule = components
        .lines()
        .find(|l| l.contains(".aw-window-chrome {") || l.starts_with(".aw-window-chrome{"))
        .expect(".aw-window-chrome rule in shared components.css");
    assert!(
        chrome_rule.contains("background: var(--aw-surface-chrome)"),
        "chrome fill must use --aw-surface-chrome: {chrome_rule}"
    );
    assert!(
        chrome_rule.contains("height: var(--aw-titlebar)"),
        "chrome height must use --aw-titlebar: {chrome_rule}"
    );
    assert!(
        !settings_css.contains(".aw-window-chrome"),
        "settings.css must not redefine .aw-window-chrome (shared components own it)"
    );
    for banned in [
        "background: #191b1f",
        "background: #1b1e22",
        "background: #202329",
        "background: #111",
        "background: #000",
        "background: rgba(0, 0, 0,",
        "background: rgba(0,0,0,",
    ] {
        assert!(
            !settings_css.contains(banned),
            "mockup-style chrome/panel hex must not appear in settings.css: {banned}"
        );
    }
    // Brand / product name / captions must not shrink; context takes the rest
    // (player.css had this under .kk-player-root — lift into shared components).
    let brand_flex = components.lines().any(|l| {
        l.contains("aw-icon-brand")
            && l.contains("aw-product-name")
            && l.contains("aw-window-captions")
            && l.contains("flex: 0 0 auto")
    });
    assert!(
        brand_flex,
        "shared components.css must pin brand/name/captions to flex: 0 0 auto \
         so settings caption matches the player (not a player.css-only quirk)"
    );
    let context_flex = components.lines().any(|l| {
        l.contains(".aw-window-context") && l.contains("flex: 1 1 0") && l.contains("min-width: 0")
    });
    assert!(
        context_flex,
        "shared components.css must give .aw-window-context flex: 1 1 0; min-width: 0"
    );
    // Mounted markup parity with the player caption row.
    let (_, doc) = Session::mount(page()).unwrap();
    for class in [
        "aw-window-chrome",
        "aw-window-brand",
        "aw-product-name",
        "aw-window-context",
        "aw-window-captions",
        "aw-caption",
        "aw-caption-close",
        "aw-icon-brand",
    ] {
        assert!(
            has_class(&doc, class),
            "mounted settings caption missing .{class}"
        );
    }
    assert!(has_id(&doc, "windowClose"), "caption close id");
    assert!(has_id(&doc, "minimize"), "caption minimize");
    assert!(has_id(&doc, "maximize"), "caption maximize");
    let context = doc
        .elements()
        .iter()
        .find(|e| {
            e.attributes
                .get("class")
                .is_some_and(|c| c.split_whitespace().any(|t| t == "aw-window-context"))
        })
        .expect("aw-window-context");
    let context_text: String = context.text_runs.iter().map(|r| r.text.as_str()).collect();
    assert!(
        context_text.contains("설정"),
        "context label must be 설정: {context_text:?} runs={:?}",
        context.text_runs
    );
}

#[test]
fn maximize_state_updates_the_caption_with_every_declared_prop() {
    // M2-V5b (real X11 window): a caption-only prop update was rejected
    // ("exact declared bindings required"), the icon stayed on maximize and later
    // caption clicks were lost. The session must hand Meari the full prop set.
    let (mut session, mut doc) = Session::mount(page()).unwrap();
    assert!(session.set_maximized(true), "state changed");
    assert!(!session.set_maximized(true), "same state is not a change");
    doc.update_module_props(session.props()).unwrap();
    assert!(
        has_class(&doc, "aw-icon-restore"),
        "restore icon while maximized"
    );
    assert!(!has_class(&doc, "aw-icon-maximize"));
    let label = doc
        .elements()
        .iter()
        .find(|e| e.id == "maximize")
        .and_then(|e| e.attributes.get("aria-label").cloned());
    assert_eq!(label.as_deref(), Some("이전 크기"));
    assert!(session.set_maximized(false));
    doc.update_module_props(session.props()).unwrap();
    assert!(
        has_class(&doc, "aw-icon-maximize"),
        "maximize icon after restore"
    );
    assert!(!has_class(&doc, "aw-icon-restore"));
}

#[test]
fn maximize_update_keeps_apply_errors_and_dismissed_dialog() {
    let mut limit = page();
    limit["limit_dialog"] =
        json!({"state": "near", "reason": "월 한도 80%", "actions": ["continue", "cancel"]});
    let (mut session, mut doc) = Session::mount(limit).unwrap();
    let errors = vec!["inference.budget.confirm_ratio: 0..=1".to_owned()];
    doc.update_module_props(session.update_props(None, &errors))
        .unwrap();
    session.dismiss_dialog();
    session.set_maximized(true);
    doc.update_module_props(session.props()).unwrap();
    let shown = session.rendered(&doc);
    assert_eq!(shown["dialog"], json!(false), "{shown}");
    assert!(
        shown["errors_shown"]
            .as_str()
            .unwrap_or_default()
            .contains("confirm_ratio"),
        "{shown}"
    );
}

#[test]
fn input_focus_ring_uses_shared_focus_token_in_meari() {
    // M2-V5b (real X11 window): Meari never matches `:focus-visible`, so focused inputs
    // showed the engine's blue `input:focus` outline instead of `--aw-focus`.
    let components =
        include_str!("../../kinekino-player/src/presentation/web_ui/styles/components.css");
    let rule = components
        .lines()
        .find(|l| {
            l.split('{')
                .next()
                .unwrap_or_default()
                .split(',')
                .any(|s| s.trim() == "input:focus")
        })
        .expect("shared components.css must style input:focus (Meari has no :focus-visible)");
    assert!(
        rule.contains("outline: var(--aw-border-focus) solid var(--aw-focus)"),
        "input:focus must use the shared focus token: {rule}"
    );
}
