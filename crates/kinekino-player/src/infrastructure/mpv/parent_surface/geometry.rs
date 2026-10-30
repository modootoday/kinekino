use super::SurfaceInputError;
use meari_native::{
    application::window_lifecycle::WindowStamp,
    domain::surface_slot::{SurfaceClip, SurfaceError, SurfaceVisibility},
    infrastructure::window::NativeSurfaceSnapshot,
};

pub(super) fn validate_pair(
    expected: WindowStamp,
    expected_child: u64,
    observed: WindowStamp,
    observed_child: u64,
) -> Result<(), SurfaceInputError> {
    validate_identity(expected, observed)?;
    if expected_child != observed_child {
        return Err(SurfaceInputError::Identity);
    }
    Ok(())
}

pub(super) fn validate_identity(
    expected: WindowStamp,
    observed: WindowStamp,
) -> Result<(), SurfaceInputError> {
    if expected == observed {
        Ok(())
    } else {
        Err(SurfaceInputError::Identity)
    }
}
pub(super) fn validate_snapshot(snapshot: &NativeSurfaceSnapshot) -> Result<(), SurfaceInputError> {
    if let Some(rect) = snapshot.physical_client_rect {
        rect.enclosing_i32().map_err(SurfaceInputError::Geometry)?;
    }
    if [snapshot.device_scale, snapshot.layout_zoom]
        .into_iter()
        .any(|value| !value.is_finite() || value <= 0.0)
    {
        return Err(SurfaceInputError::Geometry(SurfaceError::InvalidGeometry));
    }
    if let SurfaceClip::Rectangular(clip) = snapshot.clip {
        clip.enclosing_i32().map_err(SurfaceInputError::Geometry)?;
    }
    Ok(())
}
pub(super) fn require_start(snapshot: &NativeSurfaceSnapshot) -> Result<(), SurfaceInputError> {
    validate_snapshot(snapshot)?;
    if snapshot.visibility == SurfaceVisibility::Unknown {
        return Err(SurfaceInputError::VisibilityUnknown);
    }
    if snapshot.visibility == SurfaceVisibility::Hidden {
        return Err(SurfaceInputError::NotVisible);
    }
    let rect = snapshot
        .physical_client_rect
        .ok_or(SurfaceInputError::Geometry(SurfaceError::NotReady))?;
    if rect.width == 0.0 || rect.height == 0.0 {
        return Err(SurfaceInputError::NotVisible);
    }
    snapshot
        .require_rectangular_child_window()
        .map_err(SurfaceInputError::Geometry)?;
    // a77 measures viewport-relative layout bounds, not an attested parent-client paint origin.
    // Even synthetic "supported" metadata cannot bypass this missing producer observation.
    Err(SurfaceInputError::ParentClientOriginUnverified)
}
