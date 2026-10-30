use crate::{
    application::workspace_read::MaskSnapshot,
    presentation::{
        components::{mask_overlay, segmented, toolbar},
        design_tokens::{size, space},
    },
};
use eframe::egui;

#[derive(Clone, Copy, Default)]
enum View {
    Original,
    Mask,
    #[default]
    Composite,
}

pub fn show(ui: &mut egui::Ui, canvas: egui::Rect, mask: Option<&MaskSnapshot>) {
    let id = ui.id().with("subject-preview-mode");
    let view = ui
        .ctx()
        .data_mut(|d| d.get_temp::<View>(id))
        .unwrap_or_default();
    if let Some(mask) = mask {
        match view {
            View::Original => {}
            View::Mask => mask_overlay::paint_mask(ui, canvas, mask),
            View::Composite => mask_overlay::paint(ui, canvas, mask),
        }
    }
    let width = size::TAB_MIN * 3. + space::FIELD * 2. + space::INLINE * 2.;
    let rect = egui::Rect::from_min_size(
        canvas.right_top() + egui::vec2(-width - space::FIELD, space::FIELD),
        egui::vec2(width, size::CONTROL + space::INLINE * 2.),
    );
    let mut overlay = ui.new_child(
        egui::UiBuilder::new()
            .id(id.with("controls"))
            .max_rect(rect),
    );
    overlay.set_clip_rect(canvas.intersect(ui.clip_rect()));
    let mut selected = mask.map(|_| match view {
        View::Original => 0,
        View::Mask => 1,
        View::Composite => 2,
    });
    toolbar::overlay_frame().show(&mut overlay, |ui| {
        ui.add_enabled_ui(mask.is_some(), |ui| {
            if segmented::show(ui, "modes", &["원본", "마스크", "합성"], &mut selected) {
                let next = match selected {
                    Some(0) => View::Original,
                    Some(1) => View::Mask,
                    _ => View::Composite,
                };
                ui.ctx().data_mut(|d| d.insert_temp(id, next));
                ui.ctx().request_repaint();
            }
        })
        .response
        .on_disabled_hover_text("같은 원본 프레임의 마스크 연결이 필요합니다.");
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::workspace_read::{
        WorkspaceId, WorkspaceReadPort,
        preview::{PreviewScenario, PreviewWorkspaceProvider},
    };

    #[test]
    fn focus_survives_switching_between_views_with_different_paint_children() {
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        let snapshot =
            PreviewWorkspaceProvider::new(PreviewScenario::Populated).snapshot(WorkspaceId::Obj);
        let mut state_id = egui::Id::NULL;
        let mut frame = |events| {
            let _ = crate::presentation::test_support::run(
                &ctx,
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    egui::CentralPanel::default().show(ui, |ui| {
                        state_id = ui.id().with("subject-preview-mode");
                        show(
                            ui,
                            egui::Rect::from_min_size(egui::pos2(10., 10.), egui::vec2(600., 340.)),
                            snapshot.mask.as_ref(),
                        );
                    });
                },
            );
        };
        for _ in 0..3 {
            frame(vec![]);
        }
        let pos = egui::pos2(400., 40.);
        for pressed in [true, false] {
            frame(vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ]);
        }
        frame(vec![]);
        assert!(ctx.memory(|m| m.focused()).is_some());
        frame(vec![egui::Event::Key {
            key: egui::Key::ArrowRight,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        frame(vec![]);
        assert!(matches!(
            ctx.data_mut(|d| d.get_temp::<View>(state_id)),
            Some(View::Mask)
        ));
    }
}
