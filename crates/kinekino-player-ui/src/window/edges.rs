//! Edge resize of the undecorated window on X11. The press's implicit pointer grab belongs
//! to winit's X connection, so `_NET_WM_MOVERESIZE` sent from the player's own connection
//! never gets the pointer to the window manager (the drag did nothing). The player follows
//! the pointer itself and configures the window; the band is checked before the caption
//! drag, with pointer and window size both read from the X server in physical pixels.
use super::App;
use crate::controller::Engine;
use winit::event::{ElementState, WindowEvent};

/// Width of the invisible resize band along each edge of the undecorated window
/// (logical pixels; multiplied by the window's scale factor).
pub const RESIZE_BAND: f64 = 8.0;

/// The smallest window an edge drag leaves (physical pixels).
pub const MIN_SIZE: (i32, i32) = (480, 270);

/// A window's outer rectangle in root (screen) coordinates, physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// `start` dragged by (`dx`, `dy`) on the edge or corner `direction` (EWMH numbering); the
/// opposite edges stay put and the size never goes below `min`.
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

impl<E: Engine> App<E> {
    /// Starts an edge drag when the press is in the invisible band along the window edge.
    /// Pointer and window size come from the X server in one unit (physical pixels).
    pub(super) fn begin_resize(&mut self) -> bool {
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
        let (Ok(((x, y), root)), Ok(rect)) = (x11.pointer(window), x11.geometry(window)) else {
            return false;
        };
        let band = RESIZE_BAND * scale;
        let (x, y, w, h) = (
            f64::from(x),
            f64::from(y),
            f64::from(rect.w),
            f64::from(rect.h),
        );
        let Some(direction) = resize_direction(x, y, w, h, band) else {
            return false;
        };
        self.resizing = Some((direction, rect, root));
        true
    }
    /// While an edge drag runs, pointer moves size the window and the release ends it; none
    /// of these reach the document. `true` when the event was taken.
    pub(super) fn follow_resize(&mut self, event: &WindowEvent) -> bool {
        let Some((direction, start, from)) = self.resizing else {
            return false;
        };
        match event {
            WindowEvent::PointerMoved { .. } => {
                let (Some(x11), Some(id)) = (self.x11.as_ref(), self.window_id) else {
                    return true;
                };
                let Ok(window) = u32::try_from(id.into_raw()) else {
                    return true;
                };
                if let Ok((_, (x, y))) = x11.pointer(window) {
                    let r = resized(start, direction, x - from.0, y - from.1, MIN_SIZE);
                    if let Err(e) = x11.configure(window, r) {
                        eprintln!("{{\"warning\":\"window resize failed\",\"detail\":{e:?}}}");
                    }
                }
                true
            }
            WindowEvent::PointerButton {
                state: ElementState::Released,
                ..
            } => {
                self.resizing = None;
                true
            }
            WindowEvent::PointerLeft { .. } | WindowEvent::PointerEntered { .. } => true,
            _ => false,
        }
    }
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
