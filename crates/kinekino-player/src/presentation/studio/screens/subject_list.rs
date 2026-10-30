use crate::{
    application::workspace_read::{
        MaskReviewState, MaskSnapshot, ReadOrigin, ReadState, SubjectMask, WorkspaceSnapshot,
    },
    presentation::{
        components::{
            collection_state::CollectionState,
            subject_row::{self, SubjectRow},
        },
        design_assets,
    },
};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot) {
    ui.strong("피사체 목록");
    let Some(mask) = snapshot.mask.as_ref() else {
        let state =
            if snapshot.origin == ReadOrigin::DesignSample && snapshot.state == ReadState::Ready {
                CollectionState::Count(0)
            } else {
                match snapshot.state {
                    ReadState::Unsupported => CollectionState::Unavailable,
                    ReadState::NeedsInput => CollectionState::NeedsInput,
                    _ => CollectionState::Unknown,
                }
            };
        state.show_notice(ui, "피사체");
        return;
    };
    for subject in &mask.subjects {
        let review = match subject.review_state {
            MaskReviewState::Unreviewed => "미검토",
            MaskReviewState::NeedsReview => "검토 필요",
            MaskReviewState::Confirmed => "검토 확인됨",
        };
        let detail = format!("{review} · {}", subject.track_id);
        subject_row::show(
            ui,
            SubjectRow {
                id: &subject.track_id,
                label: "이름 미지정",
                detail: &detail,
                current: mask.subjects.len() == 1,
            },
            |ui, rect| paint_source_crop(ui, rect, mask, subject),
        );
    }
    if mask.subjects.is_empty() {
        CollectionState::Count(0).show_notice(ui, "피사체");
    }
    ui.weak("원본 영역 미리보기 · 이름/선택/메뉴 연결 필요");
}

fn source_bounds(mask: &MaskSnapshot, subject: &SubjectMask) -> Option<egui::Rect> {
    if mask.width_px == 0 || mask.height_px == 0 {
        return None;
    }
    let points = subject.polygon_px.as_ref()?;
    if points.len() < 3
        || points.iter().any(|p| {
            p.x < 0
                || p.y < 0
                || i64::from(p.x) > i64::from(mask.width_px)
                || i64::from(p.y) > i64::from(mask.height_px)
        })
    {
        return None;
    }
    let bounds = egui::Rect::from_points(
        &points
            .iter()
            .map(|p| egui::pos2(p.x as f32, p.y as f32))
            .collect::<Vec<_>>(),
    );
    (bounds.width() > 0. && bounds.height() > 0.).then_some(bounds)
}

fn paint_source_crop(
    ui: &mut egui::Ui,
    slot: egui::Rect,
    mask: &MaskSnapshot,
    subject: &SubjectMask,
) {
    if mask.origin != ReadOrigin::DesignSample {
        return;
    }
    let Some(asset) = design_assets::find(&mask.source_asset.asset_id) else {
        return;
    };
    if asset.size != egui::vec2(mask.width_px as f32, mask.height_px as f32) {
        return;
    }
    let Some(bounds) = source_bounds(mask, subject) else {
        return;
    };
    let scale = (slot.width() / bounds.width()).min(slot.height() / bounds.height());
    let target = egui::Rect::from_center_size(slot.center(), bounds.size() * scale);
    let image = egui::Rect::from_min_size(
        target.min - bounds.min.to_vec2() * scale,
        asset.size * scale,
    );
    asset.paint_in_rect(ui, image, target);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::workspace_read::{
        WorkspaceId, WorkspaceReadPort,
        preview::{PreviewScenario, PreviewWorkspaceProvider},
    };
    #[test]
    fn crop_requires_valid_source_polygon_and_never_uses_unrelated_table_rows() {
        let snapshot =
            PreviewWorkspaceProvider::new(PreviewScenario::Populated).snapshot(WorkspaceId::Obj);
        let mut mask = snapshot.mask.unwrap();
        assert_eq!(mask.subjects.len(), 1);
        assert_eq!(snapshot.table.rows.len(), 3);
        let subject = mask.subjects[0].clone();
        assert!(source_bounds(&mask, &subject).is_some());
        let mut invalid = subject.clone();
        invalid.polygon_px.as_mut().unwrap()[0].x = mask.width_px as i32 + 1;
        assert!(source_bounds(&mask, &invalid).is_none());
        invalid.polygon_px = Some(vec![]);
        assert!(source_bounds(&mask, &invalid).is_none());
        mask.width_px = 0;
        assert!(source_bounds(&mask, &subject).is_none());
    }
}
