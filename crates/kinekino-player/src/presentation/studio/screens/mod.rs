mod card_inspector;
mod cards;
mod clip_timeline;
mod clips;
mod composition_inspector;
mod conversion;
mod correction_inspector;
mod frames;
mod media;
mod ocr;
mod projection;
mod save_inspector;
mod subject_list;
mod subject_preview;
mod subtitle_inspector;
mod system;
mod system_inspector;
mod view;
use crate::application::workspace_read::{WorkspaceId, WorkspaceSnapshot};
use eframe::egui;

pub(super) fn model(
    ui: &mut egui::Ui,
    snapshot: &WorkspaceSnapshot,
    selected: &mut Option<String>,
    reviews: &mut super::model_review::ModelReview,
) {
    egui::ScrollArea::vertical()
        .id_salt("model-review-and-catalog")
        .show(ui, |ui| {
            reviews.show(ui);
            ui.add_space(crate::presentation::design_tokens::space::GROUP);
            system::show(ui, snapshot, selected);
        });
}

pub(super) fn show(
    ui: &mut egui::Ui,
    snapshot: &WorkspaceSnapshot,
    tab: usize,
    selected: &mut Option<String>,
) -> bool {
    if !matches!(
        snapshot.workspace_id,
        WorkspaceId::Seg
            | WorkspaceId::Sub
            | WorkspaceId::Obj
            | WorkspaceId::Download
            | WorkspaceId::Card
            | WorkspaceId::Frame
            | WorkspaceId::Clip
            | WorkspaceId::Model
            | WorkspaceId::Job
            | WorkspaceId::Out
            | WorkspaceId::Proj
    ) {
        return false;
    }
    egui::ScrollArea::vertical()
        .id_salt(format!(
            "dedicated-screen-{:?}-{tab}",
            snapshot.workspace_id
        ))
        .show(ui, |ui| match snapshot.workspace_id {
            WorkspaceId::Card => cards::show(ui, snapshot, selected),
            WorkspaceId::Frame => frames::show(ui, snapshot, tab, selected),
            WorkspaceId::Clip => clips::show(ui, snapshot, tab, selected),
            WorkspaceId::Model | WorkspaceId::Job | WorkspaceId::Out => {
                system::show(ui, snapshot, selected)
            }
            WorkspaceId::Seg | WorkspaceId::Sub | WorkspaceId::Obj | WorkspaceId::Download => {
                media::show(ui, snapshot, tab, selected)
            }
            WorkspaceId::Proj => projection::show(ui, snapshot, tab, selected),
            _ => unreachable!(),
        });
    true
}

pub(super) fn inspector(
    ui: &mut egui::Ui,
    snapshot: &WorkspaceSnapshot,
    selected: &Option<String>,
    tab: usize,
) -> bool {
    if snapshot.workspace_id == WorkspaceId::Out {
        // Export owns its destination and confirmation fields in the main view.
        true
    } else if snapshot.workspace_id == WorkspaceId::Card {
        card_inspector::show(ui, snapshot, selected);
        true
    } else if snapshot.workspace_id == WorkspaceId::Frame {
        frames::inspector(ui, snapshot, tab);
        true
    } else if matches!(
        snapshot.workspace_id,
        WorkspaceId::Model | WorkspaceId::Job | WorkspaceId::Out
    ) {
        system::inspector(ui, snapshot, selected);
        true
    } else if matches!(
        snapshot.workspace_id,
        WorkspaceId::Seg | WorkspaceId::Sub | WorkspaceId::Obj | WorkspaceId::Download
    ) {
        media::inspector(ui, snapshot, selected, tab);
        true
    } else if snapshot.workspace_id == WorkspaceId::Proj {
        projection::inspector(ui, snapshot, tab);
        true
    } else if snapshot.workspace_id == WorkspaceId::Clip {
        composition_inspector::show(ui, snapshot.workspace_id, tab);
        true
    } else {
        false
    }
}
