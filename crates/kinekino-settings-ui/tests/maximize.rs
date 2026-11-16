//! M2-V5b follow-up: the caption icon follows maximize changes the window manager makes on
//! its own (keyboard shortcut, tiling, its own menu), not only caption requests.
use kinekino_settings_ui::{MAXIMIZE_SETTLE, MaximizeFollow};
use std::time::{Duration, Instant};

#[test]
fn window_manager_changes_reach_the_caption() {
    let t = Instant::now();
    let mut follow = MaximizeFollow::default();
    assert_eq!(
        follow.observed(true, t),
        Some(true),
        "WM maximized the window"
    );
    assert_eq!(follow.observed(false, t), Some(false), "WM restored it");
}

#[test]
fn a_caption_request_is_not_undone_by_a_late_report() {
    // X11 reports the new state only after the request; resize events in between still
    // carry the old state and must not flip the icon back.
    let t = Instant::now();
    let mut follow = MaximizeFollow::default();
    follow.requested(true, t);
    assert_eq!(
        follow.observed(false, t),
        None,
        "old state while the WM catches up"
    );
    assert_eq!(follow.observed(true, t), Some(true), "the WM applied it");
    // Once applied, later WM changes are followed again.
    assert_eq!(follow.observed(false, t), Some(false));
}

#[test]
fn a_request_the_window_manager_ignores_stops_holding() {
    let t = Instant::now();
    let mut follow = MaximizeFollow::default();
    follow.requested(true, t);
    let late = t + MAXIMIZE_SETTLE + Duration::from_millis(1);
    assert_eq!(
        follow.observed(false, late),
        Some(false),
        "after the settle time the reported state wins"
    );
}
