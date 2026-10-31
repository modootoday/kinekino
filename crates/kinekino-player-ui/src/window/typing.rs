//! Keys while a text field (the browse search) has focus: they type instead of being
//! player shortcuts; Enter searches and Esc still goes back. A new listing also starts
//! at the top of the row list (ADR 20261031103033).
use super::Shared;
use crate::port::Engine;
use crate::session::Command;
use winit::event::KeyEvent;
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

impl<E: Engine> Shared<E> {
    /// A text field has focus, so keys are typing rather than player shortcuts.
    fn typing(&self) -> bool {
        self.handle
            .try_with(|doc| {
                let dom = &doc.renderer.document;
                dom.get_focussed_node_id()
                    .and_then(|id| dom.get_node(id))
                    .and_then(|n| n.element_data())
                    .is_some_and(|e| e.text_input_data().is_some())
            })
            .unwrap_or(false)
    }

    /// A pressed key; true when it went to a text field.
    pub(super) fn key(&mut self, mods: ModifiersState, key: &KeyEvent) -> bool {
        let typing = self.typing();
        if typing {
            if key.physical_key == PhysicalKey::Code(KeyCode::Enter) {
                self.collect();
                let query = self.session.query().to_owned();
                self.run(Command::BrowseSearch(query));
            } else if key.physical_key == PhysicalKey::Code(KeyCode::Escape) {
                self.run(Command::ExitFullscreen);
            }
        } else if let Some(command) = crate::keys::shortcut(mods, key.physical_key, key.repeat) {
            self.run(command);
        }
        typing
    }
}

/// Scrolls the browse row list back to its top, for a newly shown listing.
pub(super) fn to_top(doc: &mut meari_native::infrastructure::document::InteractiveDocument) {
    let dom = &mut doc.renderer.document;
    for id in ["browseLists", "browseRows"] {
        if let Some(node) = dom.get_element_by_id(id) {
            dom.scroll_to(node, 0.0, 0.0, blitz_dom::ScrollBehavior::Instant);
        }
    }
}
