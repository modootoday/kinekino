//! The player's tray presence (ADR 20261107231529, port `TrayPresence`): one tray item for
//! the life of the process, showing the mascot-derived app icon (`presence::icon`, nothing
//! derived at run time) and the product name as its tooltip. Activating it brings the player
//! window back to the front. No menu, no close-to-tray, nothing about what is playing.
//!
//! Linux speaks StatusNotifierItem on the session bus; a session without a tray host is
//! `Unsupported`. Other platforms have no adapter yet (`Unsupported`). Either way a failure
//! is reported once and the player carries on, as with the window icon. Registration talks
//! to the bus on its own thread, never on the window loop.
use super::icon::IconImage;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The tray tooltip: the product name, never a title, episode or plugin name.
pub const TOOLTIP: &str = "kinekino";

/// Derived icon sizes offered to the tray host; it scales to its own panel size.
const SIZES: [u32; 4] = [16, 24, 32, 48];

/// Why there is no tray item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayError {
    /// No tray host on this session, or no adapter for this platform.
    Unsupported,
    Failed(String),
}

/// A tray item that is shown while the value lives (dropping it removes the item).
pub trait TrayPresence {
    /// `true` once per activation (click) since the last call.
    fn take_activation(&self) -> bool;
}

/// A click on the tray item, handed from the bus thread to the window loop.
#[derive(Debug, Default, Clone)]
pub struct Activation(Arc<AtomicBool>);

impl Activation {
    pub fn raise(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn take(&self) -> bool {
        self.0.swap(false, Ordering::SeqCst)
    }
}

/// StatusNotifierItem `IconPixmap`: (width, height, ARGB32 in network byte order) for each
/// offered size of the derived app icon.
pub fn pixmaps(images: &[IconImage]) -> Vec<(i32, i32, Vec<u8>)> {
    images
        .iter()
        .filter(|i| SIZES.contains(&i.width))
        .map(|i| {
            let argb = i
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|p| [p[3], p[0], p[1], p[2]])
                .collect();
            (i.width as i32, i.height as i32, argb)
        })
        .collect()
}

/// A bus name for a new tray item. The player window (and its tray item) is rebuilt on
/// remount while the old connection may still be letting go of its name, so each item gets
/// its own instance number.
pub fn item_name() -> String {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    format!("org.kde.StatusNotifierItem-{}-{n}", std::process::id())
}

/// Runs `install` the first time only; a failure is reported once and yields no tray.
pub fn install_once<T>(
    tried: &mut bool,
    install: impl FnOnce() -> Result<T, TrayError>,
) -> Option<T> {
    if std::mem::replace(tried, true) {
        return None;
    }
    install().map_err(|e| eprintln!("{}", warning(&e))).ok()
}

/// The one-line JSON warning for a tray that could not be shown.
pub fn warning(e: &TrayError) -> String {
    let detail = match e {
        TrayError::Unsupported => "Unsupported".to_string(),
        TrayError::Failed(d) => d.clone(),
    };
    format!("{{\"warning\":\"tray icon not shown\",\"detail\":{detail:?}}}")
}

/// Keeps a tray item registered by [`in_background`]; dropping it lets the item go.
pub struct Background {
    _stop: std::sync::mpsc::Sender<()>,
}

/// Runs `register` on its own thread so a slow or silent tray host never holds up the window
/// loop. A failure is reported once from that thread. What `register` returns (the bus
/// connection) is kept alive there until the returned handle is dropped.
pub fn in_background<C: 'static>(
    register: impl FnOnce() -> Result<C, TrayError> + Send + 'static,
) -> Result<Background, TrayError> {
    let (stop, stopped) = std::sync::mpsc::channel::<()>();
    std::thread::Builder::new()
        .name("tray".into())
        .spawn(move || match register() {
            Ok(registered) => {
                // Returns once the handle (the sender) is dropped.
                let _ = stopped.recv();
                drop(registered);
            }
            Err(e) => eprintln!("{}", warning(&e)),
        })
        .map_err(|e| TrayError::Failed(e.to_string()))?;
    Ok(Background { _stop: stop })
}

/// The tray item seen from the window loop: the activation flag plus the registration
/// thread's handle.
struct Tray {
    activation: Activation,
    _registered: Background,
}

impl TrayPresence for Tray {
    fn take_activation(&self) -> bool {
        self.activation.take()
    }
}

/// The platform's tray item. Registration finishes (or is reported as failed) on its own
/// thread; until then, and if it fails, the item simply never activates.
#[cfg(target_os = "linux")]
pub fn install() -> Result<Box<dyn TrayPresence>, TrayError> {
    let icons = super::icon::app_icons().map_err(TrayError::Failed)?;
    let activation = Activation::default();
    let item = activation.clone();
    let registered = in_background(move || sni::install(pixmaps(&icons), item))?;
    Ok(Box::new(Tray {
        activation,
        _registered: registered,
    }))
}
#[cfg(not(target_os = "linux"))]
pub fn install() -> Result<Box<dyn TrayPresence>, TrayError> {
    Err(TrayError::Unsupported)
}

/// StatusNotifierItem (freedesktop/KDE tray protocol) on the session bus.
#[cfg(target_os = "linux")]
mod sni {
    use super::{Activation, TOOLTIP, TrayError};
    use zbus::blocking::Connection;
    use zbus::blocking::connection::Builder;

    type Pixmap = (i32, i32, Vec<u8>);
    const PATH: &str = "/StatusNotifierItem";
    const WATCHER: &str = "org.kde.StatusNotifierWatcher";

    struct Item {
        pixmaps: Vec<Pixmap>,
        activation: Activation,
    }

    #[zbus::interface(name = "org.kde.StatusNotifierItem")]
    impl Item {
        #[zbus(property)]
        fn category(&self) -> &str {
            "ApplicationStatus"
        }
        #[zbus(property)]
        fn id(&self) -> &str {
            TOOLTIP
        }
        #[zbus(property)]
        fn title(&self) -> &str {
            TOOLTIP
        }
        #[zbus(property)]
        fn status(&self) -> &str {
            "Active"
        }
        #[zbus(property)]
        fn icon_name(&self) -> &str {
            ""
        }
        #[zbus(property)]
        fn icon_pixmap(&self) -> Vec<Pixmap> {
            self.pixmaps.clone()
        }
        #[zbus(property)]
        fn tool_tip(&self) -> (String, Vec<Pixmap>, String, String) {
            (String::new(), Vec::new(), TOOLTIP.into(), String::new())
        }
        #[zbus(property)]
        fn item_is_menu(&self) -> bool {
            false
        }
        fn activate(&self, _x: i32, _y: i32) {
            self.activation.raise();
        }
        fn secondary_activate(&self, _x: i32, _y: i32) {
            self.activation.raise();
        }
        fn context_menu(&self, _x: i32, _y: i32) {}
        fn scroll(&self, _delta: i32, _orientation: String) {}
    }

    /// Registers the item with the session's StatusNotifierWatcher. The item lives as long
    /// as the returned connection.
    pub fn install(pixmaps: Vec<Pixmap>, activation: Activation) -> Result<Connection, TrayError> {
        let failed = |e: zbus::Error| TrayError::Failed(e.to_string());
        let item = Item {
            pixmaps,
            activation,
        };
        let name = super::item_name();
        let conn = Builder::session()
            .and_then(|b| b.name(name.as_str()))
            .and_then(|b| b.serve_at(PATH, item))
            .and_then(|b| b.build())
            .map_err(failed)?;
        // No watcher on the bus means no tray host in this session.
        conn.call_method(
            Some(WATCHER),
            "/StatusNotifierWatcher",
            Some(WATCHER),
            "RegisterStatusNotifierItem",
            &(name.as_str(),),
        )
        .map_err(|e| match e {
            zbus::Error::MethodError(n, ..) if n.as_str().ends_with("ServiceUnknown") => {
                TrayError::Unsupported
            }
            e => failed(e),
        })?;
        Ok(conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn tray_pixmaps_are_the_derived_sizes_in_network_order_argb() {
        let icons = crate::window::presence::icon::app_icons().expect("embedded icons decode");
        let pixmaps = pixmaps(&icons);
        let sizes: Vec<(i32, i32)> = pixmaps.iter().map(|p| (p.0, p.1)).collect();
        assert_eq!(sizes, [(16, 16), (24, 24), (32, 32), (48, 48)]);
        for (w, h, bytes) in &pixmaps {
            assert_eq!(bytes.len(), (w * h * 4) as usize);
        }
        // First pixel of the 16 px image, repacked from RGBA to big-endian ARGB.
        let p = &icons[0].rgba[..4];
        assert_eq!(&pixmaps[0].2[..4], &[p[3], p[0], p[1], p[2]]);
    }

    #[test]
    fn the_tooltip_is_the_product_name_only() {
        assert_eq!(TOOLTIP, "kinekino");
    }

    #[test]
    fn install_is_tried_once_and_a_failure_is_not_fatal() {
        let calls = Cell::new(0);
        let mut tried = false;
        let fail = || {
            calls.set(calls.get() + 1);
            Err::<(), _>(TrayError::Unsupported)
        };
        assert!(install_once(&mut tried, fail).is_none());
        assert!(install_once(&mut tried, fail).is_none());
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn a_working_install_is_kept() {
        let mut tried = false;
        assert_eq!(install_once(&mut tried, || Ok(7)), Some(7));
        assert!(tried);
    }

    #[test]
    fn the_warning_names_the_tray_and_the_reason_only() {
        let w = warning(&TrayError::Failed("no watcher".into()));
        assert!(w.contains("tray icon not shown"), "{w}");
        assert!(w.contains("no watcher"), "{w}");
        let u = warning(&TrayError::Unsupported);
        assert!(u.contains("Unsupported"), "{u}");
    }

    #[test]
    fn each_install_asks_for_its_own_bus_name() {
        // The window is rebuilt on remount while the old connection may still hold its
        // name; a second item must not collide with it.
        let (a, b) = (item_name(), item_name());
        assert_ne!(a, b);
        let pid = std::process::id().to_string();
        assert!(
            a.starts_with(&format!("org.kde.StatusNotifierItem-{pid}-")),
            "{a}"
        );
    }

    #[test]
    fn registration_runs_off_the_calling_thread_and_ends_with_the_handle() {
        // A tray host that never answers must not hold up the window loop.
        let (release, wait) = std::sync::mpsc::channel::<()>();
        let (ended_tx, ended) = std::sync::mpsc::channel::<()>();
        let started = std::time::Instant::now();
        let handle = in_background(move || {
            let _ = wait.recv();
            Ok(Ender(ended_tx))
        })
        .expect("thread starts");
        assert!(started.elapsed() < std::time::Duration::from_millis(500));
        release.send(()).unwrap();
        drop(handle);
        // The registered value (the bus connection) is dropped once the handle is.
        assert!(
            ended
                .recv_timeout(std::time::Duration::from_secs(5))
                .is_err_and(|e| e == std::sync::mpsc::RecvTimeoutError::Disconnected)
        );
    }

    struct Ender(#[allow(dead_code)] std::sync::mpsc::Sender<()>);

    #[test]
    fn activation_is_taken_once() {
        let flag = Activation::default();
        assert!(!flag.take());
        flag.raise();
        assert!(flag.take());
        assert!(!flag.take());
    }
}
