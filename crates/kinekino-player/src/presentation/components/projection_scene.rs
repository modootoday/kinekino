use crate::presentation::{
    control_icon::Icon,
    design_assets,
    design_tokens::{border, color, radius, size, space, text},
};
use eframe::egui::{self, Ui};

pub fn show(ui: &mut Ui, asset_id: Option<&str>, height: f32) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, radius::CONTROL, color::PREVIEW_SURFACE);
    let floor = painter.with_clip_rect(
        egui::Rect::from_min_max(rect.left_center(), rect.right_bottom())
            .intersect(painter.clip_rect()),
    );
    for n in -8..=8 {
        let offset = n as f32 * rect.width() / 16.;
        let mid = rect.center_bottom() + egui::vec2(offset, 0.);
        floor.line_segment(
            [
                mid - egui::vec2(rect.width(), rect.height()),
                mid + egui::vec2(rect.width(), rect.height()),
            ],
            egui::Stroke::new(border::THIN, color::BORDER),
        );
        floor.line_segment(
            [
                mid - egui::vec2(rect.width(), -rect.height()),
                mid + egui::vec2(rect.width(), -rect.height()),
            ],
            egui::Stroke::new(border::THIN, color::BORDER),
        );
    }
    if let Some(asset) = asset_id.and_then(design_assets::find) {
        if let Ok(texture) =
            egui::Image::from_bytes(asset.uri, asset.bytes).load_for_size(ui.ctx(), rect.size())
        {
            if let Some(texture_id) = texture.texture_id() {
                for layer in (0..4).rev() {
                    let step = egui::vec2(rect.width() / 12., -rect.height() / 14.);
                    let base = rect.center() + step * (layer as f32 - 1.5);
                    let aspect = asset.size.x / asset.size.y;
                    let w = (rect.width() * 0.56).min(rect.height() * 0.52 * aspect);
                    let h = w / aspect;
                    let points = [
                        base + egui::vec2(-w / 2., -h / 2. - w / 8.),
                        base + egui::vec2(w / 2., -h / 2. + w / 8.),
                        base + egui::vec2(w / 2., h / 2. + w / 8.),
                        base + egui::vec2(-w / 2., h / 2. - w / 8.),
                    ];
                    let mut mesh = egui::Mesh::with_texture(texture_id);
                    for (pos, uv) in points.into_iter().zip([
                        egui::pos2(0., 0.),
                        egui::pos2(1., 0.),
                        egui::pos2(1., 1.),
                        egui::pos2(0., 1.),
                    ]) {
                        mesh.vertices.push(egui::epaint::Vertex {
                            pos,
                            uv,
                            color: if layer == 0 {
                                color::IMAGE_TINT
                            } else {
                                color::SCENE_BACK
                            },
                        });
                    }
                    mesh.indices = vec![0, 1, 2, 0, 2, 3];
                    painter.add(egui::Shape::mesh(mesh));
                    painter.add(egui::Shape::closed_line(
                        points.to_vec(),
                        egui::Stroke::new(border::THIN, color::SECONDARY),
                    ));
                }
            } else {
                ui.ctx().request_repaint();
            }
        }
    } else {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "투영 입력과 깊이 정보 연결 필요",
            egui::FontId::proportional(text::BODY),
            color::SECONDARY,
        );
    }
    let origin = rect.left_bottom() + egui::vec2(space::SCENE_AXIS_LEFT, -space::SCENE_AXIS_BOTTOM);
    for (label, delta, tint) in [
        (
            "X",
            egui::vec2(size::CONTROL, size::CONTROL / 2.),
            color::AXIS_X,
        ),
        ("Y", egui::vec2(0., -size::CONTROL), color::AXIS_Y),
        (
            "Z",
            egui::vec2(-size::CONTROL, size::CONTROL / 2.),
            color::AXIS_Z,
        ),
    ] {
        painter.arrow(origin, delta, egui::Stroke::new(border::EMPHASIS, tint));
        painter.text(
            origin + delta * 1.25,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(text::SMALL),
            tint,
        );
    }
    super::toolbar::canvas_controls(
        ui,
        rect,
        &[
            (Icon::Hand, "이동"),
            (Icon::MoveView, "시점 이동"),
            (Icon::ZoomIn, "확대"),
            (Icon::Fullscreen, "화면 맞춤"),
        ],
    );
    source_reference(ui, rect, asset_id);
}

fn source_reference(ui: &mut Ui, canvas: egui::Rect, asset_id: Option<&str>) {
    let Some(asset) = asset_id.and_then(design_assets::find) else {
        return;
    };
    let width = size::PACKAGE_TREE.min(canvas.width() - space::PANEL * 2.);
    if width <= space::INLINE * 2. {
        return;
    }
    let toolbar_width = size::CONTROL * 4. + space::TIGHT * 3. + space::INLINE * 2.;
    let top = if width + toolbar_width + space::PANEL * 3. > canvas.width() {
        size::TOOL_RAIL_COMPACT + space::PANEL
    } else {
        space::PANEL
    };
    let origin = canvas.min + egui::vec2(space::PANEL, top);
    let mut overlay = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("projection-source-reference")
            .max_rect(egui::Rect::from_min_size(
                origin,
                egui::vec2(width, canvas.height() - top),
            )),
    );
    overlay.set_clip_rect(canvas.intersect(ui.clip_rect()));
    egui::Frame::new()
        .fill(color::PANEL)
        .stroke(egui::Stroke::new(border::THIN, color::BORDER))
        .corner_radius(radius::CONTROL)
        .inner_margin(space::INLINE_MARGIN)
        .show(&mut overlay, |ui| {
            ui.set_width(width - space::INLINE * 2.);
            ui.small("원본 프레임 · 디자인 예시");
            let image_size = egui::vec2(
                ui.available_width(),
                ui.available_width() * asset.size.y / asset.size.x,
            );
            let (rect, _) = ui.allocate_exact_size(image_size, egui::Sense::hover());
            asset.paint(ui, rect);
        });
}
