//! Pointer presses that are about the right-click menu (ADR 20261105161614): a right press
//! opens it at the pointer; a left press outside an open one closes it and goes no further.
use super::App;
use crate::controller::Engine;
use crate::menu::context_anchor;
use crate::session::Command;
use meari_native::infrastructure::document::InteractiveDocument;
use winit::event::{ElementState, MouseButton, WindowEvent};

const MENU_ID: &str = "contextMenu";

/// The document's CSS scale and its viewport size in CSS px.
fn css_viewport(doc: &InteractiveDocument) -> (f64, (f64, f64)) {
    let viewport = doc.renderer.document.viewport();
    let scale = viewport.scale_f64().max(f64::MIN_POSITIVE);
    let (w, h) = viewport.window_size;
    (scale, (f64::from(w) / scale, f64::from(h) / scale))
}

/// Whether the physical point (x, y) lands on the element `#id` or inside it.
fn inside(doc: &InteractiveDocument, x: f64, y: f64, id: &str) -> bool {
    use blitz_dom::local_name;
    let base = &*doc.renderer.document;
    let (scale, _) = css_viewport(doc);
    let Some(hit) = base.hit((x / scale) as f32, (y / scale) as f32) else {
        return false;
    };
    let mut node = base.get_node(hit.node_id);
    while let Some(n) = node {
        if n.attr(local_name!("id")) == Some(id) {
            return true;
        }
        node = n.parent.and_then(|p| base.get_node(p));
    }
    false
}

impl<E: Engine> App<E> {
    /// A pointer press: right opens the menu, left closes an open one when outside it,
    /// then the caption/edge press. `true` when the press must not reach the document.
    pub(super) fn pointer_press(&mut self, event: &WindowEvent) -> bool {
        let WindowEvent::PointerButton {
            state: ElementState::Pressed,
            position,
            primary: true,
            button,
            ..
        } = event
        else {
            return false;
        };
        let (x, y) = (position.x, position.y);
        match button.clone().mouse_button() {
            Some(MouseButton::Right) => self.open_context_menu(x, y),
            Some(MouseButton::Left) => self.dismiss_context_menu(x, y) || self.press(x, y),
            _ => false,
        }
    }

    fn open_context_menu(&mut self, x: f64, y: f64) -> bool {
        let mut shared = self.shared.borrow_mut();
        let Ok((scale, viewport)) = shared.handle.try_with(css_viewport) else {
            return false;
        };
        let (ax, ay) = context_anchor((x / scale, y / scale), viewport);
        shared.run(Command::ContextMenu(ax, ay));
        true
    }

    fn dismiss_context_menu(&mut self, x: f64, y: f64) -> bool {
        let mut shared = self.shared.borrow_mut();
        if shared.controller.view.context_menu.is_none() {
            return false;
        }
        let on_menu = shared
            .handle
            .try_with(|doc| inside(doc, x, y, MENU_ID))
            .unwrap_or(true);
        if on_menu {
            return false;
        }
        shared.run(Command::CloseMenu);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::inside;
    use crate::session::Session;
    use crate::source::View;
    use meari_native::application::Renderer;

    #[test]
    fn a_press_on_the_open_menu_is_inside_and_elsewhere_is_not() {
        let view = View {
            context_menu: Some((100.0, 100.0)),
            ..View::default()
        };
        let (_, mut doc) = Session::mount(&view).unwrap();
        let viewport = doc.viewport.clone();
        doc.renderer.rasterize(&viewport).unwrap();
        let scale = doc.renderer.document.viewport().scale_f64();
        let at = |v: f64| v * scale;
        assert!(inside(&doc, at(120.0), at(120.0), "contextMenu"));
        assert!(!inside(&doc, at(40.0), at(400.0), "contextMenu"));
    }
}
