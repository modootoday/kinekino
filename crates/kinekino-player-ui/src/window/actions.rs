//! Window actions (minimize/maximize/fullscreen). Meari's action port goes first. Where it
//! reports `Unsupported` — meari-platform 0.1.0 observes window state only on Windows, so
//! on Linux/X11 every action is refused — the request goes to the window manager as an
//! EWMH client message instead. A failed action is reported, it never ends the player.
use crate::controller::Effect;

/// Why Meari's action port did not take a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortRefusal {
    /// The platform port has no state for this window (no capability).
    Unsupported,
    Other(String),
}

/// The two ways a window action can be carried out.
pub trait WindowOps {
    fn port(&mut self, effect: &Effect) -> Result<(), PortRefusal>;
    /// `None` when this platform has no fallback.
    fn fallback(&mut self, effect: &Effect) -> Option<Result<(), String>>;
}

/// Carries out `effect`; `Err` is a message to report, not a reason to exit.
pub fn perform(ops: &mut dyn WindowOps, effect: &Effect) -> Result<(), String> {
    match ops.port(effect) {
        Ok(()) => Ok(()),
        Err(PortRefusal::Unsupported) => ops
            .fallback(effect)
            .unwrap_or_else(|| Err("window action: Unsupported".into())),
        Err(PortRefusal::Other(e)) => Err(e),
    }
}

/// EWMH requests to the X server for the player's own window (winit's X11 `WindowId` is
/// the X window id). Not used under Wayland.
#[cfg(all(unix, not(target_os = "macos")))]
pub mod x11 {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{
        AtomEnum, ClientMessageEvent, ConnectionExt, EventMask, PropMode,
    };
    use x11rb::rust_connection::RustConnection;
    use x11rb::wrapper::ConnectionExt as _;

    pub struct X11 {
        conn: RustConnection,
        root: u32,
    }

    const REMOVE: u32 = 0;
    const ADD: u32 = 1;

    impl X11 {
        /// The X connection, when this session is X11 (no Wayland display).
        pub fn connect() -> Option<Self> {
            let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some_and(|v| !v.is_empty());
            if wayland || std::env::var_os("DISPLAY").is_none() {
                return None;
            }
            let (conn, screen) = x11rb::connect(None).ok()?;
            let root = conn.setup().roots.get(screen)?.root;
            Some(Self { conn, root })
        }

        fn atom(&self, name: &str) -> Result<u32, String> {
            let reply = self
                .conn
                .intern_atom(false, name.as_bytes())
                .map_err(|e| e.to_string())?
                .reply()
                .map_err(|e| e.to_string())?;
            Ok(reply.atom)
        }

        fn message(&self, window: u32, kind: &str, data: [u32; 5]) -> Result<(), String> {
            let event = ClientMessageEvent::new(32, window, self.atom(kind)?, data);
            let mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
            self.conn
                .send_event(false, self.root, mask, event)
                .map_err(|e| e.to_string())?;
            self.conn.flush().map_err(|e| e.to_string())
        }

        fn state(&self, window: u32, on: bool, a: &str, b: Option<&str>) -> Result<(), String> {
            let a = self.atom(a)?;
            let b = b.map(|b| self.atom(b)).transpose()?.unwrap_or(0);
            let action = if on { ADD } else { REMOVE };
            // Source indication 1: a normal application.
            self.message(window, "_NET_WM_STATE", [action, a, b, 1, 0])
        }

        pub fn set_fullscreen(&self, window: u32, on: bool) -> Result<(), String> {
            self.state(window, on, "_NET_WM_STATE_FULLSCREEN", None)
        }

        pub fn set_maximized(&self, window: u32, on: bool) -> Result<(), String> {
            let (v, h) = (
                "_NET_WM_STATE_MAXIMIZED_VERT",
                "_NET_WM_STATE_MAXIMIZED_HORZ",
            );
            self.state(window, on, v, Some(h))
        }

        /// ICCCM iconify (`WM_CHANGE_STATE` → IconicState).
        pub fn minimize(&self, window: u32) -> Result<(), String> {
            self.message(window, "WM_CHANGE_STATE", [3, 0, 0, 0, 0])
        }

        /// (maximized, fullscreen) as the window manager reports them.
        pub fn observe(&self, window: u32) -> Option<(bool, bool)> {
            let prop = self.atom("_NET_WM_STATE").ok()?;
            let reply = self
                .conn
                .get_property(false, window, prop, AtomEnum::ATOM, 0, 64)
                .ok()?
                .reply()
                .ok()?;
            let states: Vec<u32> = reply.value32()?.collect();
            let has = |name: &str| self.atom(name).is_ok_and(|a| states.contains(&a));
            let maximized =
                has("_NET_WM_STATE_MAXIMIZED_VERT") && has("_NET_WM_STATE_MAXIMIZED_HORZ");
            Some((maximized, has("_NET_WM_STATE_FULLSCREEN")))
        }

        /// The title bar text (`_NET_WM_NAME`, UTF-8).
        pub fn set_title(&self, window: u32, title: &str) -> Result<(), String> {
            let (name, utf8) = (self.atom("_NET_WM_NAME")?, self.atom("UTF8_STRING")?);
            self.conn
                .change_property8(PropMode::REPLACE, window, name, utf8, title.as_bytes())
                .map_err(|e| e.to_string())?;
            self.conn.flush().map_err(|e| e.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        port: Result<(), PortRefusal>,
        fallback: Option<Result<(), String>>,
        fell_back: bool,
    }
    impl WindowOps for Fake {
        fn port(&mut self, _: &Effect) -> Result<(), PortRefusal> {
            self.port.clone()
        }
        fn fallback(&mut self, _: &Effect) -> Option<Result<(), String>> {
            self.fell_back = true;
            self.fallback.clone()
        }
    }
    fn fake(port: Result<(), PortRefusal>, fallback: Option<Result<(), String>>) -> Fake {
        Fake {
            port,
            fallback,
            fell_back: false,
        }
    }

    #[test]
    fn the_port_is_used_when_it_takes_the_request() {
        let mut ops = fake(Ok(()), None);
        assert_eq!(perform(&mut ops, &Effect::Fullscreen(true)), Ok(()));
        assert!(!ops.fell_back);
    }

    #[test]
    fn an_unsupported_port_falls_back_to_the_window_manager() {
        let mut ops = fake(Err(PortRefusal::Unsupported), Some(Ok(())));
        assert_eq!(perform(&mut ops, &Effect::Fullscreen(true)), Ok(()));
        assert!(ops.fell_back);
    }

    #[test]
    fn without_a_fallback_unsupported_is_a_reported_error() {
        let mut ops = fake(Err(PortRefusal::Unsupported), None);
        let result = perform(&mut ops, &Effect::Maximize(true));
        assert_eq!(result, Err("window action: Unsupported".into()));
    }

    #[test]
    fn other_port_errors_do_not_try_the_fallback() {
        let mut ops = fake(Err(PortRefusal::Other("stale".into())), Some(Ok(())));
        assert_eq!(perform(&mut ops, &Effect::Minimize), Err("stale".into()));
        assert!(!ops.fell_back);
    }
}
