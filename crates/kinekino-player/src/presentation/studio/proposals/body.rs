use super::{
    State,
    worker::{self, Action},
};
use crate::{
    application::{api::ApplicationClientHandle, proposals::InspectProposal},
    domain::proposals::DraftState,
    presentation::{
        components::{
            button::{Button, Variant},
            proposal_editor, text_document,
        },
        design_tokens::{size, space},
    },
};
use eframe::egui;

pub(super) fn show(
    ui: &mut egui::Ui,
    state: &mut State,
    available: bool,
    client: &ApplicationClientHandle,
) {
    ui.strong("제안 비교 · 수정");
    ui.small("원문 보존 · 자동 적용 꺼짐");
    if !available {
        ui.weak("제안 저장소 연결 필요");
    }
    let busy = state.pending.is_some();
    ui.add_enabled_ui(available && !busy && !state.dirty, |ui| {
        let label = ui.label("조회할 제안 ID");
        ui.add_sized(
            [ui.available_width(), size::CONTROL],
            egui::TextEdit::singleline(&mut state.input),
        )
        .labelled_by(label.id);
        if ui
            .add_enabled(
                !state.input.trim().is_empty(),
                Button::new("제안 열기", Variant::Secondary),
            )
            .clicked()
        {
            state.pending = Some(worker::start(
                client.clone(),
                Action::Inspect(InspectProposal {
                    proposal_id: state.input.clone(),
                }),
                None,
                ui.ctx().clone(),
            ));
        }
    });
    if let Some(error) = &state.error {
        ui.label(error);
    }
    if let Some(document) = &state.document {
        ui.add_space(space::FIELD);
        ui.label(format!("표시 중인 제안: {}", document.proposal_id));
        ui.small(format!(
            "원문 {} · 버전 {} / 초안 {}",
            document.source.source_id(),
            document.source.source_revision().0,
            document.draft_revision.0
        ));
        if document.state == DraftState::NeedsRepair {
            ui.strong("수동 수정 필요");
        }
        egui::CollapsingHeader::new("원생성 결과 · 읽기 전용").show(ui, |ui| {
            if let Some(error) = &document.import_error {
                ui.small("원생성 결과 해석 오류 · 수정본 저장 상태와 별개입니다.");
                ui.label(error);
            }
            text_document::show(ui, "generated-raw", &document.generated.raw_text);
        });
        ui.add_enabled_ui(!busy, |ui| {
            if let Some(draft) = state.draft.as_mut() {
                state.dirty |= proposal_editor::show(ui, "draft", &document.source, draft);
            }
        });
        if ui
            .add_enabled(
                !busy && state.dirty,
                Button::new("저장하지 않은 수정 버리기", Variant::Secondary),
            )
            .clicked()
        {
            state.reset_buffer();
        }
    }
}
