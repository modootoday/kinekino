//! Same original child, explicitly reobserved current frame; no stored snapshot is authority.
use super::super::*;

pub(super) fn refresh(surface: &mut VisibleRenderOnlyChild) -> SharedResult<()> {
    if surface.phase != Phase::Visible || surface.revoked {
        return Err(remember(&surface.first_error, VisibleError::Phase));
    }
    let result = (|| {
        let previous = surface.frame.as_ref().ok_or(VisibleError::Phase)?;
        let previous_geometry = geometry(previous)?;
        let current = surface
            .input
            .with_checked_parent(|checked| {
                checked
                    .with_parent(|parent, snapshot| {
                        surface.require_parent(parent)?;
                        // Validate unchanged original HWND/class/process/thread/style/focus before any move.
                        surface.observe_visible(previous_geometry)?;
                        let next_geometry = geometry(snapshot)?;
                        if next_geometry != previous_geometry {
                            let child = surface.child()?;
                            let [x, y, width, height] = next_geometry;
                            // SAFETY: Original retained child and freshly scoped original parent only.
                            unsafe {
                                SetWindowPos(
                                    child,
                                    None,
                                    x,
                                    y,
                                    width,
                                    height,
                                    SWP_NOACTIVATE | SWP_NOZORDER,
                                )
                                .map_err(VisibleError::Os)?;
                            }
                        }
                        surface.observe_visible(next_geometry)?;
                        Ok(snapshot.clone())
                    })
                    .map_err(VisibleError::Parent)?
            })
            .map_err(VisibleError::Input)??;
        // Reentry/paint/geometry changes during OS work must not silently select another frame.
        surface
            .input
            .with_checked_parent(|checked| {
                checked
                    .with_parent(|parent, snapshot| {
                        if !same_frame(&current, snapshot) {
                            return Err(VisibleError::OriginalFrameChanged);
                        }
                        surface.require_parent(parent)?;
                        surface.observe_visible(geometry(snapshot)?)
                    })
                    .map_err(VisibleError::Parent)?
            })
            .map_err(VisibleError::Input)??;
        Ok(current)
    })();
    match result {
        Ok(current) => {
            surface.frame = Some(current);
            Ok(())
        }
        Err(error) => {
            surface.phase = Phase::Uncertain;
            Err(remember(&surface.first_error, error))
        }
    }
}
