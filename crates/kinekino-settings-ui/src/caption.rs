//! Settings caption row as the title bar of the undecorated window: drag moves it,
//! a double-click maximizes/restores (same convention as the player).
use meari_native::infrastructure::document::InteractiveDocument;
use std::time::{Duration, Instant};

/// Two caption presses within this time maximize/restore, as on OS title bars.
pub(crate) const DOUBLE_CLICK: Duration = Duration::from_millis(400);

/// What a press at a point of the document lands on.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PressTarget {
    /// The caption row background: drag moves the window.
    Caption,
    Other,
}

pub(crate) fn press_target(doc: &InteractiveDocument, x: f64, y: f64) -> PressTarget {
    use blitz_dom::local_name;
    let base = &*doc.renderer.document;
    let scale = base.viewport().scale_f64().max(f64::MIN_POSITIVE);
    let Some(hit) = base.hit((x / scale) as f32, (y / scale) as f32) else {
        return PressTarget::Other;
    };
    let mut node = base.get_node(hit.node_id);
    while let Some(n) = node {
        if n.element_data()
            .is_some_and(|e| e.name.local == local_name!("button"))
        {
            return PressTarget::Other;
        }
        let class = n.attr(local_name!("class")).unwrap_or("");
        if class.split_whitespace().any(|c| c == "aw-window-chrome") {
            return PressTarget::Caption;
        }
        node = n.parent.and_then(|p| base.get_node(p));
    }
    PressTarget::Other
}

/// Turns the OS frame off once the window exists (Meari creates it decorated).
pub(crate) fn undecorate(doc: &InteractiveDocument) -> bool {
    doc.renderer
        .document
        .shell_provider
        .set_window_decorations(false);
    true
}

/// Left press on caption background: drag, or double-click maximize. `true` when taken.
pub(crate) fn press(doc: &InteractiveDocument, x: f64, y: f64, last: &mut Option<Instant>) -> bool {
    if press_target(doc, x, y) != PressTarget::Caption {
        return false;
    }
    let now = Instant::now();
    let double = last.is_some_and(|t| now.duration_since(t) <= DOUBLE_CLICK);
    if double {
        *last = None;
        let on = !doc.renderer.document.shell_provider.is_window_maximized();
        doc.renderer
            .document
            .shell_provider
            .set_window_maximized(on);
        return true;
    }
    *last = Some(now);
    doc.renderer.document.shell_provider.drag_window();
    true
}
