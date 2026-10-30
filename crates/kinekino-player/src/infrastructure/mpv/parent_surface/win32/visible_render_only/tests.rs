use super::*;
use meari_native::{
    application::window_lifecycle::*,
    domain::surface_slot::{PhysicalClientRect, SurfaceFocus, SurfaceLogicalFocus},
    infrastructure::native_host::HostStamp,
};
use std::{task::Waker, time::Duration};

struct Clock;
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}
struct NoResources;
impl NativeChild for NoResources {
    type Error = std::io::Error;
    fn parent_lost(&mut self) {
        panic!("metadata fixture has no child")
    }
    fn begin_cleanup(
        &mut self,
        _: CleanupRequest,
        _: CompletionSignal,
    ) -> std::result::Result<CleanupProgress, Self::Error> {
        panic!("metadata fixture has no child")
    }
    fn observe_cleanup(
        &mut self,
        _: CleanupRequest,
    ) -> std::result::Result<CleanupProgress, Self::Error> {
        panic!("metadata fixture has no child")
    }
}
fn snapshot() -> NativeSurfaceSnapshot {
    let window = WindowLifecycle::<NoResources>::new(
        LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
        Clock,
        Waker::noop().clone(),
    )
    .unwrap()
    .ready(ReadyKind::NewWindow)
    .unwrap();
    NativeSurfaceSnapshot {
        frame: meari_native::domain::graphics::FrameTicket { context_id:"visible-metadata".into(),context_generation:1,scene_generation:1 },
        logical_focus: SurfaceLogicalFocus { focused_node:None,surface_focusable:None },
        document: HostStamp { host_id:1,owner:serde_json::from_value(serde_json::json!({"instance":{"document":"visible-metadata","epoch":1,"id":1},"source_generation":1})).unwrap(),definition_revision:1,document_revision:1,turn_sequence:1 },
        window,slot_generation:1,layout_revision:1,host_geometry_revision:1,
        physical_client_rect:Some(PhysicalClientRect{x:10.0,y:20.0,width:100.0,height:50.0}),device_scale:2.0,layout_zoom:1.5,
        visibility:SurfaceVisibility::Visible,clip:SurfaceClip::Rectangular(PhysicalClientRect{x:0.0,y:0.0,width:200.0,height:200.0}),plane:SurfacePlane::RectangularChildWindow,focus:SurfaceFocus::Unknown,
    }
}
#[test]
fn candidate_geometry_never_promotes_unknown_focus_or_partial_clip() {
    let mut value = snapshot();
    assert_eq!(geometry(&value).unwrap(), [10, 20, 100, 50]);
    assert_eq!(value.focus, SurfaceFocus::Unknown);
    assert!(value.require_rectangular_child_window().is_err());
    assert!(crate::infrastructure::mpv::parent_surface::geometry::require_start(&value).is_err());
    value.clip = SurfaceClip::Rectangular(PhysicalClientRect {
        x: 11.0,
        y: 20.0,
        width: 99.0,
        height: 50.0,
    });
    assert!(geometry(&value).is_err());
    value.clip = SurfaceClip::Unknown;
    assert!(geometry(&value).is_err());
}
#[test]
fn new_frame_geometry_or_slot_cannot_replace_the_original_observation() {
    let value = snapshot();
    let mut changed = value.clone();
    changed.layout_revision += 1; // Observation counter is not the frame owner.
    assert!(same_frame(&value, &changed));
    changed.frame.scene_generation += 1;
    assert!(!same_frame(&value, &changed));
    changed = value.clone();
    changed.host_geometry_revision += 1;
    assert!(!same_frame(&value, &changed));
    changed = value.clone();
    changed.slot_generation += 1;
    assert!(!same_frame(&value, &changed));
}
#[test]
fn visible_readback_rejects_each_missing_child_policy_observation() {
    let ready = Readback {
        child_style: true,
        disabled: true,
        visible: true,
        nonactivating: true,
        geometry: true,
    };
    assert!(ready.require().is_ok());
    for invalid in [
        Readback {
            child_style: false,
            ..ready
        },
        Readback {
            disabled: false,
            ..ready
        },
        Readback {
            visible: false,
            ..ready
        },
        Readback {
            nonactivating: false,
            ..ready
        },
        Readback {
            geometry: false,
            ..ready
        },
    ] {
        assert!(invalid.require().is_err());
    }
}
#[test]
fn focus_observation_rejects_child_capture_and_external_focus_change() {
    assert!(unexpected_input(WM_SETFOCUS, WPARAM(0)));
    assert!(unexpected_input(WM_KEYDOWN, WPARAM(0)));
    assert!(!unexpected_input(WM_NCHITTEST, WPARAM(0)));
    assert!(!unexpected_input(WM_ACTIVATE, WPARAM(WA_INACTIVE as usize)));
    let before = Focus {
        keyboard: HWND::default(),
        active: HWND::default(),
        foreground: HWND::default(),
        capture: HWND::default(),
    };
    let child = HWND(7usize as *mut std::ffi::c_void);
    assert!(before.require_unchanged(before, child).is_ok());
    assert!(
        before
            .require_unchanged(
                Focus {
                    capture: child,
                    ..before
                },
                child
            )
            .is_err()
    );
    assert!(
        before
            .require_unchanged(
                Focus {
                    foreground: HWND(9usize as *mut std::ffi::c_void),
                    ..before
                },
                child
            )
            .is_err()
    );
}
#[test]
fn ongoing_focus_allows_each_external_owner_change_without_granting_child_input() {
    let before = Focus {
        keyboard: HWND::default(),
        active: HWND::default(),
        foreground: HWND::default(),
        capture: HWND::default(),
    };
    let child = HWND(7usize as *mut std::ffi::c_void);
    let parent_or_external = HWND(9usize as *mut std::ffi::c_void);
    for current in [
        Focus {
            keyboard: parent_or_external,
            ..before
        },
        Focus {
            active: parent_or_external,
            ..before
        },
        Focus {
            foreground: parent_or_external,
            ..before
        },
        Focus {
            capture: parent_or_external,
            ..before
        },
    ] {
        assert!(current.require_child_unfocused(child).is_ok());
        assert!(matches!(
            before.require_unchanged(current, child),
            Err(VisibleError::FocusChanged)
        ));
    }
}
#[test]
fn ongoing_focus_rejects_child_ownership_in_each_channel_even_when_unchanged() {
    let before = Focus {
        keyboard: HWND::default(),
        active: HWND::default(),
        foreground: HWND::default(),
        capture: HWND::default(),
    };
    let child = HWND(7usize as *mut std::ffi::c_void);
    for current in [
        Focus {
            keyboard: child,
            ..before
        },
        Focus {
            active: child,
            ..before
        },
        Focus {
            foreground: child,
            ..before
        },
        Focus {
            capture: child,
            ..before
        },
    ] {
        assert!(matches!(
            current.require_child_unfocused(child),
            Err(VisibleError::FocusChanged)
        ));
        assert!(matches!(
            current.require_unchanged(current, child),
            Err(VisibleError::FocusChanged)
        ));
    }
}
#[test]
fn ongoing_focus_allows_external_release_and_combined_owner_changes() {
    let child = HWND(7usize as *mut std::ffi::c_void);
    let external = HWND(9usize as *mut std::ffi::c_void);
    let owned_elsewhere = Focus {
        keyboard: external,
        active: external,
        foreground: external,
        capture: external,
    };
    let released = Focus {
        keyboard: HWND::default(),
        active: HWND::default(),
        foreground: HWND::default(),
        capture: HWND::default(),
    };
    assert!(owned_elsewhere.require_child_unfocused(child).is_ok());
    assert!(released.require_child_unfocused(child).is_ok());
    assert!(matches!(
        owned_elsewhere.require_unchanged(released, child),
        Err(VisibleError::FocusChanged)
    ));
    assert!(released.require_unchanged(released, child).is_ok());
}
#[test]
fn cleanup_failure_does_not_replace_the_original_transition_error() {
    let first = RefCell::new(None);
    let original = remember(&first, VisibleError::OriginalFrameChanged);
    let secondary = remember(
        &first,
        VisibleError::Child(ProbeError::ClassFinalizationRequired),
    );
    assert!(Arc::ptr_eq(first.borrow().as_ref().unwrap(), &original));
    assert!(!Arc::ptr_eq(&original, &secondary));
    assert!(matches!(
        &*secondary,
        VisibleError::Child(ProbeError::ClassFinalizationRequired)
    ));
}
