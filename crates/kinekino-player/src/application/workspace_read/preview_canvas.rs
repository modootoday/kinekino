use super::{preview::PreviewScenario, *};

pub(super) fn canvas(workspace: WorkspaceId, scenario: PreviewScenario) -> Option<CanvasSnapshot> {
    if workspace != WorkspaceId::A2d || scenario == PreviewScenario::Empty {
        return None;
    }
    let needs_input = scenario == PreviewScenario::NeedsInput;
    let state = if needs_input {
        ReadState::NeedsInput
    } else {
        ReadState::Ready
    };
    Some(CanvasSnapshot {
        width_px: 1672,
        height_px: 941,
        source_asset: Some(asset("forest-v1")),
        layers: vec![
            LayerSnapshot {
                layer_id: "sample-layer-background".into(),
                label: "디자인 샘플 배경".into(),
                asset: Some(asset("forest-plate-v1")),
                visible: true,
                locked: true,
                bounds_px: PixelBounds { x: 0, y: 0, width: 1672, height: 941 },
                pivot_px: PixelPoint { x: 836, y: 470 },
                state: ReadState::Ready,
            },
            LayerSnapshot {
                layer_id: "sample-layer-traveler".into(),
                label: if scenario == PreviewScenario::LongLabels {
                    "디자인 검토용 긴 인물 레이어 이름 — 日本語の長いレイヤー名と English sample subject"
                } else {
                    "디자인 샘플 인물"
                }.into(),
                asset: (!needs_input).then(|| asset("traveler-v2")),
                visible: true,
                locked: false,
                bounds_px: PixelBounds { x: 450, y: 60, width: 560, height: 840 },
                pivot_px: PixelPoint { x: 730, y: 480 },
                state,
            },
        ],
        selected_layer_id: Some("sample-layer-traveler".into()),
        view_mode: CanvasViewMode::Composite,
        zoom_percent: 100,
        origin: ReadOrigin::DesignSample,
        state,
    })
}

fn asset(name: &str) -> AssetReference {
    AssetReference {
        asset_id: format!("demo-anime-{name}"),
        path: format!("embedded://demo/anime/{name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn sample_composition_preserves_source_coordinates_and_selection() {
        for scenario in [
            PreviewScenario::Populated,
            PreviewScenario::LongLabels,
            PreviewScenario::NeedsInput,
        ] {
            let canvas = canvas(WorkspaceId::A2d, scenario).unwrap();
            assert_eq!(canvas.origin, ReadOrigin::DesignSample);
            let ids: BTreeSet<_> = canvas
                .layers
                .iter()
                .map(|layer| layer.layer_id.as_str())
                .collect();
            assert_eq!(ids.len(), canvas.layers.len());
            assert!(ids.contains(canvas.selected_layer_id.as_deref().unwrap()));
            for layer in &canvas.layers {
                let bounds = layer.bounds_px;
                assert!(bounds.width > 0 && bounds.height > 0);
                assert!(bounds.x >= 0 && bounds.y >= 0);
                assert!(
                    i64::from(bounds.x) + i64::from(bounds.width) <= i64::from(canvas.width_px)
                );
                assert!(
                    i64::from(bounds.y) + i64::from(bounds.height) <= i64::from(canvas.height_px)
                );
                assert!(layer.pivot_px.x >= bounds.x && layer.pivot_px.y >= bounds.y);
                assert!(
                    i64::from(layer.pivot_px.x) <= i64::from(bounds.x) + i64::from(bounds.width)
                );
                assert!(
                    i64::from(layer.pivot_px.y) <= i64::from(bounds.y) + i64::from(bounds.height)
                );
            }
            let subject = &canvas.layers[1];
            assert_eq!(
                u64::from(subject.bounds_px.width) * 1536,
                u64::from(subject.bounds_px.height) * 1024
            );
            assert_eq!(
                subject.asset.is_none(),
                scenario == PreviewScenario::NeedsInput
            );
            assert_eq!(subject.state, canvas.state);
        }
    }

    #[test]
    fn missing_canvas_never_fabricates_live_or_empty_composition() {
        for id in WorkspaceId::ALL {
            assert!(LiveWorkspaceProvider.snapshot(id).canvas.is_none());
            assert!(canvas(id, PreviewScenario::Empty).is_none());
            if id != WorkspaceId::A2d {
                assert!(canvas(id, PreviewScenario::Populated).is_none());
            }
        }
    }
}
