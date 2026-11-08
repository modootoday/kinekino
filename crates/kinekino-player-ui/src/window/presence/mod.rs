//! The player's presence in the desktop shell: the window icon and the tray item, both from
//! the mascot-derived app icon set (ADR 20261031111316, ADR 20261107231529).
pub(crate) mod icon;
mod tray;

use crate::window::actions::x11::X11;
use winit::window::WindowId;

/// The shell presence of one player window, set up the first time the window exists.
#[derive(Default)]
pub struct Presence {
    tray: Option<Box<dyn tray::TrayPresence>>,
    tried: bool,
}

impl Presence {
    /// Once the window exists: hands it the app icon (once per window, X11) and installs
    /// the tray item; a tray click raises the window. A failure is reported, never fatal.
    pub fn step(&mut self, x11: Option<&X11>, window: Option<WindowId>) {
        let Some(id) = window else {
            return;
        };
        if let (Some(x11), Ok(window)) = (x11, u32::try_from(id.into_raw())) {
            icon::send(x11, window);
        }
        if let Some(t) = tray::install_once(&mut self.tried, tray::install) {
            self.tray = Some(t);
        }
        if !self.tray.as_ref().is_some_and(|t| t.take_activation()) {
            return;
        }
        let raised = match (x11, u32::try_from(id.into_raw())) {
            (Some(x11), Ok(window)) => x11.activate(window),
            _ => Err("window raise: Unsupported".into()),
        };
        if let Err(e) = raised {
            eprintln!("{{\"warning\":\"window action failed\",\"detail\":{e:?}}}");
        }
    }
}
