//! The two popup menus (⋮ above the transport and the right-click menu) share one set of
//! open/close rules (ADR 20261105161614); the controller asks here first.
use crate::session::Command;
use crate::source::View;

/// Box reserved for the right-click menu in CSS px, used to keep it inside the window.
/// A test checks the laid-out menu fits; grow it when items are added.
pub const CONTEXT_MENU_SIZE: (f64, f64) = (280.0, 720.0);

/// Top-left corner for a right-click menu pressed at `press` in a `viewport` (both CSS px):
/// at the pointer, opening left/up when it would overflow, pinned at 0 when it cannot fit.
pub fn context_anchor(press: (f64, f64), viewport: (f64, f64)) -> (f64, f64) {
    let (w, h) = CONTEXT_MENU_SIZE;
    let axis = |at: f64, size: f64, room: f64| {
        let at = at.clamp(0.0, room.max(0.0));
        if at + size <= room {
            at
        } else {
            (at - size).max(0.0)
        }
    };
    (axis(press.0, w, viewport.0), axis(press.1, h, viewport.1))
}

/// Applies the menu rules for `command`; `true` when the command was only about menus
/// and must go no further.
pub(crate) fn route(view: &mut View, command: &Command) -> bool {
    let open = view.menu_open || view.context_menu.is_some();
    match command {
        Command::ContextMenu(x, y) => {
            view.menu_open = false;
            view.context_menu = Some((*x, *y));
            true
        }
        Command::More => {
            view.context_menu = None;
            view.menu_open = !view.menu_open;
            true
        }
        // Esc closes an open menu first; only the next Esc leaves fullscreen.
        Command::ExitFullscreen | Command::CloseMenu if open => {
            close(view);
            true
        }
        Command::CloseMenu => true,
        // Any other command (a menu item, a click elsewhere) closes both menus.
        _ => {
            close(view);
            false
        }
    }
}

fn close(view: &mut View) {
    view.menu_open = false;
    view.context_menu = None;
}
