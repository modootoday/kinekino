//! Explicit input roles for unavailable adapters. No guessed values or edit intents.
use crate::presentation::{
    design_tokens::{size, space},
    layout,
};
use eframe::egui;

#[derive(Clone, Copy)]
pub enum Kind<'a> {
    Select,
    Toggle,
    Scalar,
    Text,
    Path,
    Color,
    Readout(Option<&'a str>),
}

pub struct Field<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub kind: Kind<'a>,
}

impl<'a> Field<'a> {
    pub const fn new(id: &'a str, label: &'a str, kind: Kind<'a>) -> Self {
        Self { id, label, kind }
    }
}

pub fn unavailable(ui: &mut egui::Ui, fields: &[Field<'_>]) {
    for field in fields {
        egui_taffy::tui(ui, ui.id().with(field.id))
            .reserve_available_width()
            .style(layout::full(layout::column(space::FIELD)))
            .show(|tui| {
                if matches!(field.kind, Kind::Color) {
                    layout::block(tui, |ui| {
                        ui.label(field.label);
                        super::color_swatches::unavailable(ui);
                    });
                    return;
                }
                super::field(tui, field.label, |ui| {
                    ui.push_id(field.id, |ui| match field.kind {
                        Kind::Select => {
                            super::input::unavailable_select(
                                ui,
                                "연결 필요",
                                "실제 선택 목록과 값 연결 필요",
                            );
                        }
                        Kind::Toggle => {
                            ui.add_enabled(
                                false,
                                egui::Checkbox::new(&mut false, "알 수 없음").indeterminate(true),
                            )
                            .on_disabled_hover_text("설정 값 연결 필요 · 꺼짐 상태가 아닙니다");
                        }
                        Kind::Scalar => {
                            super::input::unavailable_scalar(
                                ui,
                                size::NUMERIC_FIELD,
                                "실제 값 연결 필요 · 0이 아닙니다",
                            );
                        }
                        Kind::Text => {
                            super::input::readonly(ui, "연결 필요");
                        }
                        Kind::Path => super::input::unavailable_path(ui),
                        Kind::Color => super::color_swatches::unavailable(ui),
                        Kind::Readout(value) => {
                            ui.add(egui::Label::new(value.unwrap_or("—")).truncate())
                                .on_hover_text(value.unwrap_or("실제 값 연결 필요"));
                        }
                    });
                });
            });
        ui.add_space(space::FIELD);
    }
}
