//! Edge resize of the undecorated window. Every target first asks the OS through meari's
//! `WindowAction::BeginResize` (winit `drag_resize_window`): Windows and Wayland (and X11,
//! verified) hand the drag to the window manager; macOS reports Unsupported. On X11 the
//! player keeps its own fallback (set `KINEKINO_X11_OWN_RESIZE=1` to force it): it follows
//! the pointer and configures the window, reading pointer and size from the X server.
use super::App;
use crate::controller::Engine;
use meari_native::application::window_actions::WindowAction;
use meari_native::domain::window_actions::ResizeDirection;
#[cfg(all(unix, not(target_os = "macos")))]
use winit::event::{ElementState, WindowEvent};

/// Width of the invisible resize band along each edge of the undecorated window
/// (logical pixels; multiplied by the window's scale factor).
pub const RESIZE_BAND: f64 = 8.0;

/// The smallest window an edge drag leaves (physical pixels).
#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
pub const MIN_SIZE: (i32, i32) = (480, 270);

/// A window's outer rectangle in root (screen) coordinates, physical pixels.
#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// `start` dragged by (`dx`, `dy`) on the edge or corner `direction` (EWMH numbering); the
/// opposite edges stay put and the size never goes below `min`.
#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
pub fn resized(start: Rect, direction: u32, dx: i32, dy: i32, min: (i32, i32)) -> Rect {
    let (west, east) = (matches!(direction, 0 | 6 | 7), matches!(direction, 2..=4));
    let (north, south) = (matches!(direction, 0..=2), matches!(direction, 4..=6));
    let mut r = start;
    if east {
        r.w = (start.w + dx).max(min.0);
    }
    if west {
        r.w = (start.w - dx).max(min.0);
        r.x = start.x + start.w - r.w;
    }
    if south {
        r.h = (start.h + dy).max(min.1);
    }
    if north {
        r.h = (start.h - dy).max(min.1);
        r.y = start.y + start.h - r.h;
    }
    r
}

/// The EWMH `_NET_WM_MOVERESIZE` direction for a press at (`x`, `y`) in a `width`×`height`
/// window, or `None` away from the edges. Corners take a wider band so they are easy to hit.
pub fn resize_direction(x: f64, y: f64, width: f64, height: f64, band: f64) -> Option<u32> {
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    let corner = band * 2.0;
    let (left, right) = (x < band, x >= width - band);
    let (top, bottom) = (y < band, y >= height - band);
    let (near_l, near_r) = (x < corner, x >= width - corner);
    let (near_t, near_b) = (y < corner, y >= height - corner);
    Some(match () {
        _ if (top && near_l) || (left && near_t) => 0,
        _ if (top && near_r) || (right && near_t) => 2,
        _ if (bottom && near_r) || (right && near_b) => 4,
        _ if (bottom && near_l) || (left && near_b) => 6,
        _ if top => 1,
        _ if right => 3,
        _ if bottom => 5,
        _ if left => 7,
        _ => return None,
    })
}

/// Meari's resize edge for an EWMH direction code (0 top-left … 7 left).
pub fn os_direction(direction: u32) -> Option<ResizeDirection> {
    Some(match direction {
        0 => ResizeDirection::NorthWest,
        1 => ResizeDirection::North,
        2 => ResizeDirection::NorthEast,
        3 => ResizeDirection::East,
        4 => ResizeDirection::SouthEast,
        5 => ResizeDirection::South,
        6 => ResizeDirection::SouthWest,
        7 => ResizeDirection::West,
        _ => return None,
    })
}

impl<E: Engine> App<E> {
    /// Hands an edge press to the OS through Meari's `BeginResize` (meari 0.2). The band is
    /// measured on the document viewport (physical pixels, same unit as `press`). `true`
    /// when the press was in the band and the OS accepted the request.
    pub(super) fn begin_os_resize(&mut self, press: (f64, f64)) -> bool {
        let Ok((scale, (w, h))) = self.shared.borrow().handle.try_with(|doc| {
            let v = doc.renderer.document.viewport();
            (v.scale_f64(), v.window_size)
        }) else {
            return false;
        };
        let band = RESIZE_BAND * scale;
        let Some(direction) = resize_direction(press.0, press.1, f64::from(w), f64::from(h), band)
            .and_then(os_direction)
        else {
            return false;
        };
        let accepted = self.window(WindowAction::BeginResize(direction)).is_ok();
        if trace() {
            eprintln!(
                "{{\"os_resize_begin\":{{\"direction\":\"{direction:?}\",\"accepted\":{accepted}}}}}"
            );
        }
        accepted
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
impl<E: Engine> App<E> {
    /// Starts an edge drag when the press is in the invisible band along the window edge.
    /// Pointer and window size come from the X server in one unit (physical pixels).
    /// `press` is the button event's own position (window-relative, physical pixels): the
    /// event is handled a frame or more after the press, by when the pointer may already
    /// have moved, so the band and the drag origin must not come from a later pointer query.
    pub(super) fn begin_resize(&mut self, press: (f64, f64)) -> bool {
        if std::env::var_os("KINEKINO_X11_OWN_RESIZE").is_none() && self.begin_os_resize(press) {
            return true;
        }
        let (Some(x11), Some(id)) = (self.x11.as_ref(), self.window_id) else {
            return false;
        };
        let Ok(window) = u32::try_from(id.into_raw()) else {
            return false;
        };
        let scale = self
            .shared
            .borrow()
            .handle
            .try_with(|doc| doc.renderer.document.viewport().scale_f64())
            .unwrap_or(1.0);
        // The window does not move while the button is down, so its geometry read now is
        // the geometry at the press.
        let Ok(rect) = x11.geometry(window) else {
            return false;
        };
        let band = RESIZE_BAND * scale;
        let (x, y) = press;
        let direction = resize_direction(x, y, f64::from(rect.w), f64::from(rect.h), band);
        let root = press_root(rect, press);
        if trace() {
            let now = x11.pointer(window).ok();
            eprintln!(
                "{{\"resize_begin\":{{\"direction\":{direction:?},\"rect\":\"{rect:?}\",\"press\":[{x},{y}],\"press_root\":[{},{}],\"pointer_now\":\"{now:?}\",\"band\":{band}}}}}",
                root.0, root.1
            );
        }
        let Some(direction) = direction else {
            return false;
        };
        self.resizing = Some((direction, rect, root));
        self.resized_to = None;
        true
    }
    /// While an edge drag runs, pointer moves size the window and the release ends it; none
    /// of these reach the document. `true` when the event was taken.
    pub(super) fn follow_resize(&mut self, event: &WindowEvent) -> bool {
        if self.resizing.is_none() {
            return false;
        }
        match event {
            WindowEvent::PointerMoved { .. } => {
                self.step_resize();
                true
            }
            WindowEvent::PointerButton {
                state: ElementState::Released,
                ..
            } => {
                self.step_resize();
                self.resizing = None;
                self.resized_to = None;
                true
            }
            WindowEvent::PointerLeft { .. } | WindowEvent::PointerEntered { .. } => true,
            _ => false,
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
impl<E: Engine> App<E> {
    /// Sizes the window to the pointer's current screen position. Called on every pointer
    /// move and on every tick while the drag runs: winit drops a motion whose position in
    /// the window did not change, and a corner drag can leave it unchanged while the screen
    /// position moves, so events alone are not enough. Ends the drag once the button is up.
    pub(super) fn step_resize(&mut self) {
        let Some((direction, start, from)) = self.resizing else {
            return;
        };
        let (Some(x11), Some(id)) = (self.x11.as_ref(), self.window_id) else {
            return;
        };
        let Ok(window) = u32::try_from(id.into_raw()) else {
            return;
        };
        let Ok((_, (x, y), held)) = x11.pointer_state(window) else {
            return;
        };
        let r = resized(start, direction, x - from.0, y - from.1, MIN_SIZE);
        if trace() {
            eprintln!(
                "{{\"resize\":{{\"direction\":{direction},\"start\":\"{start:?}\",\"pointer\":[{x},{y}],\"from\":[{},{}],\"to\":\"{r:?}\",\"held\":{held}}}}}",
                from.0, from.1
            );
        }
        if self.resized_to != Some(r) {
            if let Err(e) = x11.configure(window, r) {
                eprintln!("{{\"warning\":\"window resize failed\",\"detail\":{e:?}}}");
            }
            self.resized_to = Some(r);
        }
        if !held {
            self.resizing = None;
            self.resized_to = None;
        }
    }
}

/// The screen position of a press at `press` (window-relative) in a window at `rect`.
#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
pub fn press_root(rect: Rect, press: (f64, f64)) -> (i32, i32) {
    (
        rect.x + press.0.floor() as i32,
        rect.y + press.1.floor() as i32,
    )
}

/// `KINEKINO_TRACE_RESIZE=1` prints every edge-drag step (desktop diagnosis).
fn trace() -> bool {
    std::env::var_os("KINEKINO_TRACE_RESIZE").is_some_and(|v| v == "1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_and_corners_map_to_ewmh_directions() {
        let d = |x, y| resize_direction(x, y, 800.0, 600.0, 6.0);
        assert_eq!(
            resize_direction(1.0, 1.0, 1280.0, 720.0, RESIZE_BAND),
            Some(0)
        );
        assert_eq!(
            resize_direction(1273.0, 300.0, 1280.0, 720.0, RESIZE_BAND),
            Some(3)
        );
        assert_eq!(d(400.0, 300.0), None);
        assert_eq!(d(2.0, 2.0), Some(0));
        assert_eq!(d(400.0, 1.0), Some(1));
        assert_eq!(d(798.0, 8.0), Some(2));
        assert_eq!(d(799.0, 300.0), Some(3));
        assert_eq!(d(795.0, 599.0), Some(4));
        assert_eq!(d(400.0, 597.0), Some(5));
        assert_eq!(d(3.0, 590.0), Some(6));
        assert_eq!(d(0.0, 300.0), Some(7));
        assert_eq!(resize_direction(1.0, 1.0, 0.0, 0.0, 6.0), None);
    }

    /// Every corner and edge, dragged by each of the nine sign combinations of (dx, dy):
    /// the dragged edges follow the pointer, the opposite edges never move.
    #[test]
    fn every_corner_and_edge_follows_all_nine_drag_directions() {
        let start = Rect {
            x: 200,
            y: 100,
            w: 1728,
            h: 880,
        };
        for direction in 0..8u32 {
            let west = matches!(direction, 0 | 6 | 7);
            let east = matches!(direction, 2..=4);
            let north = matches!(direction, 0..=2);
            let south = matches!(direction, 4..=6);
            for sx in [-1, 0, 1] {
                for sy in [-1, 0, 1] {
                    let (dx, dy) = (sx * 100, sy * 60);
                    let r = resized(start, direction, dx, dy, MIN_SIZE);
                    let ctx = format!("direction {direction}, d=({dx},{dy}) -> {r:?}");
                    let (l, t) = (r.x, r.y);
                    let (rt, b) = (r.x + r.w, r.y + r.h);
                    let (sl, st, sr, sb) = (start.x, start.y, start.x + start.w, start.y + start.h);
                    assert_eq!(l, if west { sl + dx } else { sl }, "left {ctx}");
                    assert_eq!(rt, if east { sr + dx } else { sr }, "right {ctx}");
                    assert_eq!(t, if north { st + dy } else { st }, "top {ctx}");
                    assert_eq!(b, if south { sb + dy } else { sb }, "bottom {ctx}");
                }
            }
        }
    }

    #[test]
    fn the_reported_bottom_right_shrinks_shrink_both_axes() {
        let start = Rect {
            x: 0,
            y: 0,
            w: 1728,
            h: 880,
        };
        let r = |dx, dy| resized(start, 4, dx, dy, MIN_SIZE);
        assert_eq!(
            r(-200, -120),
            Rect {
                w: 1528,
                h: 760,
                ..start
            }
        );
        assert_eq!(r(-200, 0), Rect { w: 1528, ..start });
        assert_eq!(r(-100, 0), Rect { w: 1628, ..start });
        assert_eq!(
            r(-100, -60),
            Rect {
                w: 1628,
                h: 820,
                ..start
            }
        );
        assert_eq!(r(0, -100), Rect { h: 780, ..start });
    }

    #[test]
    fn left_and_top_shrinks_move_the_edge_and_keep_the_opposite_one() {
        // At the screen's left edge (x = 0), as in the desktop check.
        let start = Rect {
            x: 0,
            y: 0,
            w: 1280,
            h: 720,
        };
        assert_eq!(
            resized(start, 7, 100, 0, MIN_SIZE),
            Rect {
                x: 100,
                w: 1180,
                ..start
            }
        );
        assert_eq!(
            resized(start, 1, 0, 50, MIN_SIZE),
            Rect {
                y: 50,
                h: 670,
                ..start
            }
        );
        assert_eq!(
            resized(start, 0, 100, 50, MIN_SIZE),
            Rect {
                x: 100,
                y: 50,
                w: 1180,
                h: 670
            }
        );
        // Past the minimum the size stops and the far edge still stays put.
        let r = resized(start, 7, 1000, 0, MIN_SIZE);
        assert_eq!((r.x + r.w, r.w), (1280, MIN_SIZE.0));
        let r = resized(start, 1, 0, 1000, MIN_SIZE);
        assert_eq!((r.y + r.h, r.h), (720, MIN_SIZE.1));
    }

    /// The drag origin is the press itself: however late the press is handled (the pointer
    /// already moved on), the window ends where the pointer is.
    #[test]
    fn a_late_handled_press_ends_at_the_same_rect() {
        let rect = Rect {
            x: 100,
            y: 60,
            w: 1400,
            h: 800,
        };
        let press = (1397.0, 797.0); // 3px inside the bottom-right corner
        assert_eq!(
            resize_direction(press.0, press.1, 1400.0, 800.0, RESIZE_BAND),
            Some(4)
        );
        let origin = press_root(rect, press);
        assert_eq!(origin, (1497, 857));
        let end = (1297, 737); // dragged (-200, -120)
        let r = resized(rect, 4, end.0 - origin.0, end.1 - origin.1, MIN_SIZE);
        assert_eq!(
            r,
            Rect {
                w: 1200,
                h: 680,
                ..rect
            }
        );
        // Handled when the pointer had moved 25px left: the old code took this as the
        // origin, lost the corner (bottom edge only) and the first 25px of the drag.
        let late = (1372, 857);
        assert_eq!(
            resize_direction(1372.0 - 100.0, 797.0, 1400.0, 800.0, RESIZE_BAND),
            Some(5)
        );
        assert_ne!(
            resized(rect, 5, end.0 - late.0, end.1 - late.1, MIN_SIZE),
            r
        );
        // Top, top-left and left presses 3px inside are edges, not the caption.
        assert_eq!(
            resize_direction(700.0, 3.0, 1400.0, 800.0, RESIZE_BAND),
            Some(1)
        );
        assert_eq!(
            resize_direction(3.0, 3.0, 1400.0, 800.0, RESIZE_BAND),
            Some(0)
        );
        assert_eq!(
            resize_direction(3.0, 400.0, 1400.0, 800.0, RESIZE_BAND),
            Some(7)
        );
    }

    #[test]
    fn an_edge_drag_keeps_the_opposite_edges() {
        let start = Rect {
            x: 100,
            y: 100,
            w: 1280,
            h: 720,
        };
        let r = |dir, dx, dy| resized(start, dir, dx, dy, MIN_SIZE);
        assert_eq!(r(3, 50, 9), Rect { w: 1330, ..start });
        assert_eq!(
            r(4, 20, 30),
            Rect {
                w: 1300,
                h: 750,
                ..start
            }
        );
        assert_eq!(
            r(0, 10, 20),
            Rect {
                x: 110,
                y: 120,
                w: 1270,
                h: 700
            }
        );
        assert_eq!(
            r(7, 2000, 0),
            Rect {
                x: 900,
                w: 480,
                ..start
            }
        );
        assert_eq!(
            r(1, 0, -40),
            Rect {
                y: 60,
                h: 760,
                ..start
            }
        );
    }
}
