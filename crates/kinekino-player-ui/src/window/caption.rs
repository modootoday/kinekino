//! The app caption row as the title bar of the undecorated window: drag moves the window,
//! a double-click maximizes; the edge resize band is checked first.
use super::App;
use crate::controller::Engine;
use std::time::{Duration, Instant};

/// Two caption presses within this time maximize/restore, as on OS title bars.
const DOUBLE_CLICK: Duration = Duration::from_millis(400);

/// What a press at a point of the document lands on.
#[derive(Debug, PartialEq, Eq)]
enum PressTarget {
    /// The caption row background: drag moves the window.
    Caption,
    Other,
}

fn press_target(
    doc: &meari_native::infrastructure::document::InteractiveDocument,
    x: f64,
    y: f64,
) -> PressTarget {
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

impl<E: Engine> App<E> {
    /// Turns the OS frame off once the window exists (Meari creates it decorated).
    pub(super) fn undecorate(&mut self) {
        let done = self.shared.borrow().handle.try_with(|doc| {
            doc.renderer
                .document
                .shell_provider
                .set_window_decorations(false)
        });
        self.undecorated = done.is_ok();
    }

    /// A left press: edge resize, caption drag or double-click maximize. `true` when the
    /// press was taken here and must not reach the document.
    pub(super) fn press(&mut self, x: f64, y: f64) -> bool {
        let (windowed, maximized) = {
            let view = &self.shared.borrow().controller.view;
            (!view.fullscreen, view.maximized)
        };
        if windowed && !maximized && self.begin_resize() {
            return true;
        }
        let shared = self.shared.borrow();
        let target = shared
            .handle
            .try_with(|doc| press_target(doc, x, y))
            .unwrap_or(PressTarget::Other);
        if !windowed || target != PressTarget::Caption {
            return false;
        }
        let now = Instant::now();
        let double = self
            .caption_press
            .is_some_and(|t| now.duration_since(t) <= DOUBLE_CLICK);
        if double {
            drop(shared);
            self.caption_press = None;
            self.shared
                .borrow_mut()
                .run(crate::session::Command::Maximize);
            return true;
        }
        self.caption_press = Some(now);
        let _ = shared
            .handle
            .try_with(|doc| doc.renderer.document.shell_provider.drag_window());
        true
    }
}
