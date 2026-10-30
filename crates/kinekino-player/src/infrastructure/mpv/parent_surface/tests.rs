//! Public snapshot metadata tests and compile-time API consumption. No native lease is fabricated.
use super::{NativeSurfaceInput, SurfaceInputError, SurfaceInputFailure, geometry};
use meari_native::{
    application::window_lifecycle::*,
    domain::surface_slot::*,
    infrastructure::{
        native_host::HostStamp,
        window::{NativeParentLease, NativeSurfaceSlot, NativeSurfaceSnapshot},
    },
};
use std::{task::Waker, time::Duration};

#[test]
fn checked_parent_failure_retains_original_error_source() {
    use meari_native::infrastructure::window::NativeParentClientError;
    use std::{error::Error, sync::Arc};
    let original = Arc::new(NativeParentClientError::PlacementMismatch);
    let error = SurfaceInputError::ParentClient(original.clone());
    let retained = error
        .source()
        .unwrap()
        .downcast_ref::<NativeParentClientError>()
        .unwrap();
    assert!(std::ptr::eq(retained, original.as_ref()));
}

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
        panic!("metadata-only fixture has no child")
    }
    fn begin_cleanup(
        &mut self,
        _: CleanupRequest,
        _: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        panic!("metadata-only fixture has no child")
    }
    fn observe_cleanup(&mut self, _: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        panic!("metadata-only fixture has no child")
    }
}
fn window() -> WindowStamp {
    WindowLifecycle::<NoResources>::new(
        LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
        Clock,
        Waker::noop().clone(),
    )
    .unwrap()
    .ready(ReadyKind::NewWindow)
    .unwrap()
}
fn snapshot() -> NativeSurfaceSnapshot {
    // These diagnostic stamps are not attached to a document, reservation or OS parent.
    NativeSurfaceSnapshot {
        frame: meari_native::domain::graphics::FrameTicket {context_id:"metadata-fixture".into(),context_generation:1,scene_generation:1},
        logical_focus:SurfaceLogicalFocus {focused_node:None,surface_focusable:None},
        document:HostStamp {host_id:1,owner:serde_json::from_value(serde_json::json!({"instance":{"document":"metadata-fixture","epoch":1,"id":1},"source_generation":1})).unwrap(),definition_revision:1,document_revision:1,turn_sequence:1},
        window:window(),slot_generation:1,layout_revision:1,host_geometry_revision:1,
        physical_client_rect:Some(PhysicalClientRect{x:10.2,y:20.4,width:100.1,height:50.1}),device_scale:2.0,layout_zoom:1.5,
        visibility:SurfaceVisibility::Visible,clip:SurfaceClip::Unknown,plane:SurfacePlane::Unknown,focus:SurfaceFocus::Unknown,
    }
}

#[test]
fn observed_physical_coordinates_are_checked_without_extra_dpi_or_zoom_multiplication() {
    let value = snapshot();
    let original = value.physical_client_rect;
    geometry::validate_snapshot(&value).unwrap();
    assert_eq!(value.physical_client_rect, original);
    assert_eq!(
        value.physical_client_rect.unwrap().enclosing_i32().unwrap(),
        [10, 20, 101, 51]
    );
    // This numeric check is not proof of the missing parent-client paint origin.
    assert!(matches!(
        geometry::require_start(&value),
        Err(SurfaceInputError::Geometry(SurfaceError::Unsupported(_)))
    ));
}
#[test]
fn nonfinite_negative_and_out_of_range_geometry_is_rejected() {
    for rectangle in [
        PhysicalClientRect {
            x: f64::NAN,
            y: 0.,
            width: 1.,
            height: 1.,
        },
        PhysicalClientRect {
            x: 0.,
            y: f64::INFINITY,
            width: 1.,
            height: 1.,
        },
        PhysicalClientRect {
            x: 0.,
            y: 0.,
            width: -1.,
            height: 1.,
        },
        PhysicalClientRect {
            x: 0.,
            y: 0.,
            width: 1.,
            height: -1.,
        },
        PhysicalClientRect {
            x: f64::MAX,
            y: 0.,
            width: f64::MAX,
            height: 1.,
        },
        PhysicalClientRect {
            x: f64::from(i32::MAX),
            y: 0.,
            width: 1.,
            height: 1.,
        },
    ] {
        let mut value = snapshot();
        value.physical_client_rect = Some(rectangle);
        assert!(matches!(
            geometry::validate_snapshot(&value),
            Err(SurfaceInputError::Geometry(SurfaceError::InvalidGeometry))
        ));
    }
}
#[test]
fn observed_scale_and_clip_are_validated_without_reinterpreting_unknown_as_rectangular() {
    for (scale, zoom) in [(0., 1.), (1., 0.), (f64::NAN, 1.), (1., f64::INFINITY)] {
        let mut value = snapshot();
        value.device_scale = scale;
        value.layout_zoom = zoom;
        assert!(matches!(
            geometry::validate_snapshot(&value),
            Err(SurfaceInputError::Geometry(SurfaceError::InvalidGeometry))
        ));
    }
    let mut value = snapshot();
    value.clip = SurfaceClip::Rectangular(PhysicalClientRect {
        x: 0.,
        y: 0.,
        width: -1.,
        height: 1.,
    });
    assert!(matches!(
        geometry::validate_snapshot(&value),
        Err(SurfaceInputError::Geometry(SurfaceError::InvalidGeometry))
    ));
}
#[test]
fn hidden_unknown_and_zero_area_never_authorize_start() {
    let mut value = snapshot();
    value.visibility = SurfaceVisibility::Unknown;
    assert!(matches!(
        geometry::require_start(&value),
        Err(SurfaceInputError::VisibilityUnknown)
    ));
    value.visibility = SurfaceVisibility::Hidden;
    assert!(matches!(
        geometry::require_start(&value),
        Err(SurfaceInputError::NotVisible)
    ));
    value.visibility = SurfaceVisibility::Visible;
    value.physical_client_rect.as_mut().unwrap().width = 0.;
    assert!(matches!(
        geometry::require_start(&value),
        Err(SurfaceInputError::NotVisible)
    ));
}
#[test]
fn even_freely_constructed_supported_metadata_cannot_attest_parent_client_origin() {
    let mut value = snapshot();
    value.clip = SurfaceClip::Rectangular(value.physical_client_rect.unwrap());
    value.plane = SurfacePlane::RectangularChildWindow;
    value.focus = SurfaceFocus::RenderOnly;
    assert!(matches!(
        geometry::require_start(&value),
        Err(SurfaceInputError::ParentClientOriginUnverified)
    ));
}
#[test]
fn unknown_plane_and_focus_keep_the_original_producer_rejection() {
    let mut value = snapshot();
    value.clip = SurfaceClip::Rectangular(value.physical_client_rect.unwrap());
    assert!(matches!(
        geometry::require_start(&value),
        Err(SurfaceInputError::Geometry(SurfaceError::Unsupported(
            "child-window stacking/overlay plane is not verified"
        )))
    ));
    value.plane = SurfacePlane::RectangularChildWindow;
    assert!(matches!(
        geometry::require_start(&value),
        Err(SurfaceInputError::Geometry(SurfaceError::Unsupported(
            "render-only focus contract is not verified"
        )))
    ));
}
#[test]
fn foreign_window_metadata_is_not_remapped_to_the_expected_owner() {
    let expected = window();
    let foreign = window();
    assert_ne!(expected, foreign);
    assert!(matches!(
        geometry::validate_identity(expected, foreign),
        Err(SurfaceInputError::Identity)
    ));
    geometry::validate_identity(expected, expected).unwrap();
    assert!(matches!(
        geometry::validate_pair(expected, 1, foreign, 1),
        Err(SurfaceInputError::Identity)
    ));
    assert!(matches!(
        geometry::validate_pair(expected, 1, expected, 2),
        Err(SurfaceInputError::Identity)
    ));
    geometry::validate_pair(expected, 1, expected, 1).unwrap();
}
#[test]
fn public_consumer_signatures_retain_original_nonclone_owners_without_minting_them() {
    let _: fn(
        NativeParentLease,
        NativeSurfaceSlot,
    ) -> Result<NativeSurfaceInput, Box<SurfaceInputFailure>> = NativeSurfaceInput::new;
    let _: fn(&NativeSurfaceSlot) -> WindowStamp = NativeSurfaceSlot::window;
    let _: fn(NativeSurfaceInput) -> (NativeParentLease, NativeSurfaceSlot) =
        NativeSurfaceInput::into_owners;
    let _: fn(&NativeSurfaceInput) -> Result<NativeSurfaceSnapshot, SurfaceInputError> =
        NativeSurfaceInput::inspect;
    let _: fn(&NativeSurfaceInput) -> Result<(), SurfaceInputError> =
        NativeSurfaceInput::require_start;
    fn unpack(
        failure: SurfaceInputFailure,
    ) -> (SurfaceInputError, NativeParentLease, NativeSurfaceSlot) {
        (failure.error, failure.parent, failure.slot)
    }
    let _: fn(SurfaceInputFailure) -> (SurfaceInputError, NativeParentLease, NativeSurfaceSlot) =
        unpack;
    // No NativeParentLease/NativeSurfaceSlot constructor, native window or Destroy receipt was used.
}

#[test]
fn missing_paint_rect_is_preserved_without_fabricating_zero_geometry() {
    let mut value = snapshot();
    value.physical_client_rect = None;
    geometry::validate_snapshot(&value).unwrap();
    assert!(value.physical_client_rect.is_none());
    assert!(matches!(
        geometry::require_start(&value),
        Err(SurfaceInputError::Geometry(SurfaceError::NotReady))
    ));
    value.visibility = SurfaceVisibility::Hidden;
    assert!(matches!(
        geometry::require_start(&value),
        Err(SurfaceInputError::NotVisible)
    ));
    value.visibility = SurfaceVisibility::Unknown;
    assert!(matches!(
        geometry::require_start(&value),
        Err(SurfaceInputError::VisibilityUnknown)
    ));
}
