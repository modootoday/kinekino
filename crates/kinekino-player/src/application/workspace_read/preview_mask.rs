use super::{preview::PreviewScenario, *};
use crate::domain::api_values::DecimalU64;

pub fn mask(workspace: WorkspaceId, scenario: PreviewScenario) -> Option<MaskSnapshot> {
    if workspace != WorkspaceId::Obj || scenario == PreviewScenario::Empty {
        return None;
    }
    let points = [
        (488, 187),
        (520, 143),
        (565, 111),
        (609, 93),
        (650, 97),
        (685, 113),
        (704, 153),
        (696, 196),
        (711, 227),
        (686, 254),
        (710, 278),
        (733, 326),
        (747, 411),
        (764, 463),
        (750, 515),
        (747, 552),
        (762, 592),
        (755, 624),
        (739, 637),
        (729, 620),
        (732, 590),
        (720, 548),
        (707, 541),
        (713, 639),
        (719, 775),
        (685, 803),
        (660, 818),
        (624, 810),
        (619, 865),
        (635, 896),
        (638, 941),
        (573, 941),
        (576, 910),
        (548, 901),
        (541, 874),
        (548, 815),
        (513, 812),
        (475, 786),
        (451, 730),
        (477, 644),
        (497, 573),
        (456, 577),
        (427, 553),
        (417, 518),
        (434, 473),
        (447, 455),
        (434, 425),
        (442, 390),
        (470, 368),
        (487, 324),
        (510, 288),
        (555, 268),
        (575, 248),
        (539, 247),
        (506, 234),
    ];
    Some(MaskSnapshot {
        width_px: 1672,
        height_px: 941,
        source_asset: AssetReference {
            asset_id: "demo-anime-forest-v1".into(),
            path: "embedded://demo/anime/forest-v1".into(),
        },
        source_pts_us: DecimalU64(52_000_000),
        subjects: vec![SubjectMask {
            track_id: "sample-subject-001".into(),
            polygon_px: Some(
                points
                    .into_iter()
                    .map(|(x, y)| PixelPoint { x, y })
                    .collect(),
            ),
            alpha_asset: Some(AssetReference {
                asset_id: "demo-forest-subject-mask-v1".into(),
                path: "embedded://demo/mask/forest-subject-v1".into(),
            }),
            review_state: MaskReviewState::Unreviewed,
        }],
        origin: ReadOrigin::DesignSample,
        state: if scenario == PreviewScenario::NeedsInput {
            ReadState::NeedsInput
        } else {
            ReadState::Ready
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mask_asset_points_pts_and_origin_match_the_sample_frame() {
        let snapshot =
            super::super::preview::PreviewWorkspaceProvider::new(PreviewScenario::Populated)
                .snapshot(WorkspaceId::Obj);
        let mask = snapshot.mask.unwrap();
        assert_eq!(mask.origin, ReadOrigin::DesignSample);
        assert_eq!(mask.source_pts_us, snapshot.preview.position_us);
        assert_eq!(
            mask.source_asset.asset_id,
            snapshot.preview.frame.unwrap().asset_id
        );
        let points = mask.subjects[0].polygon_px.as_ref().unwrap();
        assert!(points.iter().all(|p| p.x >= 0
            && p.y >= 0
            && p.x <= mask.width_px as i32
            && p.y <= mask.height_px as i32));
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/sample-subject-mask.json"
        ))
        .unwrap();
        let n = points.len();
        let twice_area: i64 = (0..n)
            .map(|i| {
                let (a, b) = (&points[i], &points[(i + 1) % n]);
                i64::from(a.x) * i64::from(b.y) - i64::from(b.x) * i64::from(a.y)
            })
            .sum();
        let bbox = [
            points.iter().map(|p| p.x).min().unwrap(),
            points.iter().map(|p| p.y).min().unwrap(),
            points.iter().map(|p| p.x).max().unwrap(),
            points.iter().map(|p| p.y).max().unwrap(),
        ];
        let observed = serde_json::json!({
            "width_px": mask.width_px,
            "height_px": mask.height_px,
            "points": n,
            "bbox": bbox,
            "twice_signed_area": twice_area,
            "first": [points[0].x, points[0].y],
            "last": [points[n - 1].x, points[n - 1].y],
        });
        for (key, value) in observed.as_object().unwrap() {
            assert_eq!(&fixture[key], value, "mask fixture field {key}");
        }
        assert!(
            points.windows(2).all(|w| w[0] != w[1]),
            "no repeated vertices"
        );
        assert!(
            super::super::LiveWorkspaceProvider
                .snapshot(WorkspaceId::Obj)
                .mask
                .is_none()
        );
        assert!(mask_for_empty().is_none());
    }
    fn mask_for_empty() -> Option<MaskSnapshot> {
        mask(WorkspaceId::Obj, PreviewScenario::Empty)
    }
}
