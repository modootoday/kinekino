mod rejection_observation;
use super::*;
pub(super) fn geometry(snapshot: &NativeSurfaceSnapshot) -> Result<[i32; 4]> {
    crate::infrastructure::mpv::parent_surface::geometry::validate_snapshot(snapshot)
        .map_err(VisibleError::Input)?;
    if snapshot.visibility != SurfaceVisibility::Visible
        || snapshot.plane != SurfacePlane::RectangularChildWindow
    {
        return Err(rejection_observation::reject(
            snapshot,
            "visibility_or_plane",
            None,
        ));
    }
    let rect = snapshot
        .physical_client_rect
        .ok_or_else(|| rejection_observation::reject(snapshot, "missing_rect", None))?;
    let SurfaceClip::Rectangular(clip) = snapshot.clip else {
        return Err(rejection_observation::reject(
            snapshot,
            "nonrectangular_clip",
            None,
        ));
    };
    let bounds = rect
        .enclosing_i32()
        .map_err(|_| rejection_observation::reject(snapshot, "integer_bounds", None))?;
    if bounds[2] <= 0
        || bounds[3] <= 0
        || clip.x > f64::from(bounds[0])
        || clip.y > f64::from(bounds[1])
        || clip.x + clip.width < f64::from(bounds[0]) + f64::from(bounds[2])
        || clip.y + clip.height < f64::from(bounds[1]) + f64::from(bounds[3])
    {
        return Err(rejection_observation::reject(
            snapshot,
            "positive_bounds_or_clip_containment",
            Some(bounds),
        ));
    }
    Ok(bounds)
}
pub(super) fn same_frame(a: &NativeSurfaceSnapshot, b: &NativeSurfaceSnapshot) -> bool {
    a.document == b.document
        && a.window == b.window
        && a.frame == b.frame
        && a.slot_generation == b.slot_generation
        && a.host_geometry_revision == b.host_geometry_revision
        && a.physical_client_rect == b.physical_client_rect
        && a.device_scale == b.device_scale
        && a.layout_zoom == b.layout_zoom
        && a.visibility == b.visibility
        && a.clip == b.clip
        && a.plane == b.plane
        && a.focus == b.focus
        && a.logical_focus == b.logical_focus
}
