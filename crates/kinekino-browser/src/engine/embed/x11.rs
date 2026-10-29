//! X11 window embedding over x11rb: host window, find a client by title, reparent, resize.
//! Linux/X11 (or XWayland) only.
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::engine::port::{EngineError, EngineResult};

fn x<E: std::fmt::Display>(e: E) -> EngineError {
    EngineError::Call(format!("x11: {e}"))
}

/// One X11 connection plus the window foreign clients are embedded into.
pub struct X11Host {
    conn: RustConnection,
    root: Window,
    pub window: Window,
}

impl X11Host {
    /// Wraps an existing host window (the app's own XID).
    pub fn attach(window: u32) -> EngineResult<Self> {
        let (conn, screen_num) = x11rb::connect(None).map_err(x)?;
        let root = conn.setup().roots[screen_num].root;
        Ok(Self { conn, root, window })
    }

    /// Creates and maps a new top-level host window (demos; the app passes its own).
    pub fn open(title: &str, w: u16, h: u16) -> EngineResult<Self> {
        let (conn, screen_num) = x11rb::connect(None).map_err(x)?;
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;
        let window = conn.generate_id().map_err(x)?;
        let aux = CreateWindowAux::new()
            .background_pixel(0x0E0E10)
            .event_mask(EventMask::STRUCTURE_NOTIFY);
        conn.create_window(
            x11rb::COPY_DEPTH_FROM_PARENT,
            window,
            root,
            40,
            40,
            w,
            h,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &aux,
        )
        .map_err(x)?;
        conn.change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            title.as_bytes(),
        )
        .map_err(x)?;
        conn.map_window(window).map_err(x)?;
        conn.flush().map_err(x)?;
        Ok(Self { conn, root, window })
    }

    fn title(&self, w: Window) -> String {
        let net = self
            .conn
            .intern_atom(false, b"_NET_WM_NAME")
            .ok()
            .and_then(|c| c.reply().ok());
        let utf8 = self
            .conn
            .intern_atom(false, b"UTF8_STRING")
            .ok()
            .and_then(|c| c.reply().ok());
        if let (Some(n), Some(u)) = (net, utf8)
            && let Some(p) = self
                .conn
                .get_property(false, w, n.atom, u.atom, 0, 256)
                .ok()
                .and_then(|c| c.reply().ok())
            && !p.value.is_empty()
        {
            return String::from_utf8_lossy(&p.value).into_owned();
        }
        self.conn
            .get_property(false, w, AtomEnum::WM_NAME, AtomEnum::ANY, 0, 256)
            .ok()
            .and_then(|c| c.reply().ok())
            .map(|p| String::from_utf8_lossy(&p.value).into_owned())
            .unwrap_or_default()
    }

    fn walk(&self, w: Window, tag: &str) -> Option<Window> {
        if self.title(w).contains(tag) {
            return Some(w);
        }
        let tree = self.conn.query_tree(w).ok()?.reply().ok()?;
        tree.children.iter().find_map(|c| self.walk(*c, tag))
    }

    pub fn find_by_title(&self, tag: &str) -> Option<Window> {
        for _ in 0..50 {
            if let Some(w) = self.walk(self.root, tag) {
                return Some(w);
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        None
    }

    pub fn parent_of(&self, w: Window) -> Option<Window> {
        Some(self.conn.query_tree(w).ok()?.reply().ok()?.parent)
    }

    /// Reparents `child` into the host at (0,0), sized to the host.
    pub fn embed(&self, child: Window) -> EngineResult<()> {
        self.conn
            .reparent_window(child, self.window, 0, 0)
            .map_err(x)?;
        let g = self
            .conn
            .get_geometry(self.window)
            .map_err(x)?
            .reply()
            .map_err(x)?;
        self.conn
            .configure_window(
                child,
                &ConfigureWindowAux::new()
                    .x(0)
                    .y(0)
                    .width(u32::from(g.width))
                    .height(u32::from(g.height)),
            )
            .map_err(x)?;
        self.conn.map_window(child).map_err(x)?;
        self.conn.flush().map_err(x)?;
        Ok(())
    }

    pub fn resize(&self, w: u32, h: u32) -> EngineResult<()> {
        self.conn
            .configure_window(self.window, &ConfigureWindowAux::new().width(w).height(h))
            .map_err(x)?;
        let tree = self
            .conn
            .query_tree(self.window)
            .map_err(x)?
            .reply()
            .map_err(x)?;
        for c in tree.children {
            self.conn
                .configure_window(c, &ConfigureWindowAux::new().width(w).height(h))
                .map_err(x)?;
        }
        self.conn.flush().map_err(x)?;
        Ok(())
    }

    pub fn focus(&self, w: Window) -> EngineResult<()> {
        self.conn
            .set_input_focus(InputFocus::PARENT, w, x11rb::CURRENT_TIME)
            .map_err(x)?;
        self.conn.flush().map_err(x)?;
        Ok(())
    }

    pub fn height_of(&self, w: Window) -> i32 {
        self.conn
            .get_geometry(w)
            .ok()
            .and_then(|c| c.reply().ok())
            .map_or(0, |g| i32::from(g.height))
    }

    pub fn abs_origin(&self, w: Window) -> (i32, i32) {
        self.conn
            .translate_coordinates(w, self.root, 0, 0)
            .ok()
            .and_then(|c| c.reply().ok())
            .map_or((0, 0), |r| (i32::from(r.dst_x), i32::from(r.dst_y)))
    }
}
