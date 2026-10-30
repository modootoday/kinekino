use crate::application::workspace_read::{
    AssetReference, CanvasSnapshot, LayerSnapshot, ReadOrigin, WorkspaceSnapshot,
};

/// Presentation projection; it borrows the SDK snapshot without copying policies.
pub(super) struct CanvasInspectorView<'a> {
    pub canvas: Option<&'a CanvasSnapshot>,
    pub layer: Option<&'a LayerSnapshot>,
    pub source_frame: Option<&'a AssetReference>,
    pub position_us: Option<u64>,
    pub sample: bool,
    pub alpha: Option<AlphaExample>,
}

#[derive(Clone, Copy)]
pub(super) struct AlphaExample {
    pub threshold: f32,
    pub feather_px: f32,
    pub edge_refinement: f32,
}

impl<'a> CanvasInspectorView<'a> {
    pub fn new(snapshot: &'a WorkspaceSnapshot, layer: Option<&'a LayerSnapshot>) -> Self {
        let canvas = snapshot.canvas.as_ref();
        let sample = snapshot.origin == ReadOrigin::DesignSample;
        Self {
            canvas,
            layer,
            source_frame: canvas.and_then(|canvas| canvas.source_asset.as_ref()),
            position_us: canvas
                .and(snapshot.source.as_ref())
                .map(|_| snapshot.preview.position_us.0),
            sample,
            alpha: (sample && canvas.is_some()).then_some(AlphaExample {
                threshold: 0.5,
                feather_px: 0.,
                edge_refinement: 0.5,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::workspace_read::{
        LiveWorkspaceProvider, WorkspaceId, WorkspaceReadPort,
        preview::{PreviewScenario, PreviewWorkspaceProvider},
    };

    #[test]
    fn live_missing_canvas_never_acquires_demo_settings_or_pts() {
        let snapshot = LiveWorkspaceProvider.snapshot(WorkspaceId::A2d);
        let view = CanvasInspectorView::new(&snapshot, None);
        assert!(!view.sample);
        assert!(view.alpha.is_none());
        assert!(view.position_us.is_none());
    }

    #[test]
    fn selected_layer_uses_absolute_source_values() {
        let snapshot =
            PreviewWorkspaceProvider::new(PreviewScenario::Populated).snapshot(WorkspaceId::A2d);
        let layer = snapshot.canvas.as_ref().unwrap().layers.last().unwrap();
        let view = CanvasInspectorView::new(&snapshot, Some(layer));
        assert_eq!(view.layer.unwrap().pivot_px.x, 730);
        assert_eq!(view.layer.unwrap().bounds_px.x, 450);
        assert!(view.alpha.is_some());
    }
}
