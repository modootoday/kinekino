//! Explicit unknown track rows, pending an application track-read adapter.
use super::{
    input,
    timeline_track_header::{self, TrackKind},
};
use crate::presentation::{
    control_icon::{self, Icon},
    design_tokens::{breakpoint, size, space},
    layout,
};
use eframe::egui;
use egui_taffy::taffy::Style;
use egui_taffy::taffy::prelude::{fr, length, minmax};

fn columns(compact: bool) -> Style {
    let mut widths = vec![length(size::CONTROL), length(size::TIMELINE_LABEL)];
    widths.extend((0..if compact { 1 } else { 3 }).map(|_| minmax(length(0.0_f32), fr(1.0_f32))));
    Style {
        grid_template_columns: widths,
        ..layout::full(layout::column(space::FIELD))
    }
}

pub fn show_unverified(ui: &mut egui::Ui) {
    egui_taffy::tui(ui, ui.id().with("track-heading"))
        .reserve_available_width()
        .style(layout::full(layout::row(space::FIELD)))
        .show(|tui| {
            layout::block(tui, |ui| {
                ui.strong("미디어 트랙 선택");
                ui.small("저장할 영상·오디오·자막 트랙을 확인합니다.");
            });
            layout::widget(tui, |ui| {
                ui.add_enabled_ui(false, |ui| {
                    control_icon::action(
                        ui,
                        Icon::Refresh,
                        "트랙 정보 새로고침",
                        size::TRACK_REFRESH,
                        false,
                    )
                })
                .inner
                .on_disabled_hover_text("트랙 조회 어댑터 연결 필요");
            });
        });
    ui.add_space(space::FIELD);
    let compact = ui.ctx().content_rect().width() < breakpoint::STACK_EDITOR;
    if !compact {
        egui_taffy::tui(ui, ui.id().with("track-columns"))
            .reserve_available_width()
            .style(columns(false))
            .show(|tui| {
                layout::widget(tui, |ui| {
                    ui.allocate_space(egui::vec2(size::CONTROL, size::CONTROL));
                });
                layout::widget(tui, |ui| {
                    ui.add_sized(
                        [size::TIMELINE_LABEL, size::CONTROL],
                        egui::Label::new("구분"),
                    );
                });
                for heading in ["선택", "트랙 정보", "품질 / 언어"] {
                    layout::block(tui, |ui| {
                        ui.label(heading);
                    });
                }
            });
        ui.separator();
    }
    for (id, kind) in [
        ("video", TrackKind::Video),
        ("audio", TrackKind::Audio),
        ("subtitle", TrackKind::Subtitle),
    ] {
        ui.push_id(id, |ui| {
            egui_taffy::tui(ui, ui.id().with("track-row"))
                .reserve_available_width()
                .style(columns(compact))
                .show(|tui| {
                    layout::widget(tui, |ui| {
                        ui.add_enabled_ui(false, |ui| {
                            ui.add_sized(
                                [size::CONTROL, size::CONTROL],
                                egui::Checkbox::new(&mut false, "").indeterminate(true),
                            )
                            .on_disabled_hover_text(
                                "트랙 미확인 · 선택 해제나 트랙 없음이 아닙니다",
                            );
                        });
                    });
                    layout::widget(tui, |ui| {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(size::TIMELINE_LABEL, size::CONTROL),
                            egui::Sense::hover(),
                        );
                        timeline_track_header::paint_media(ui, rect, kind);
                    });
                    layout::block(tui, |ui| {
                        input::unavailable_select(ui, "트랙 확인 필요", "실제 트랙 목록 연결 필요");
                    });
                    if !compact {
                        for _ in 0..2 {
                            layout::block(tui, |ui| {
                                ui.weak("미확인");
                            });
                        }
                    }
                });
            if compact {
                ui.weak("트랙 정보: 미확인 · 품질 / 언어: 미확인");
            }
        });
        ui.separator();
    }
    ui.small("미확인 트랙을 없는 트랙으로 취급하지 않습니다.");
}
