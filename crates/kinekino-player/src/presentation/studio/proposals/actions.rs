use super::{
    State,
    worker::{self, Action},
};
use crate::{
    application::{
        api::ApplicationClientHandle,
        proposals::{AcceptProposal, EditProposal},
    },
    presentation::{
        components::button::{Button, Variant},
        design_tokens::space,
        layout,
    },
};
use eframe::egui;

pub(super) fn show(
    ui: &mut egui::Ui,
    state: &mut State,
    available: bool,
    client: &ApplicationClientHandle,
) {
    ui.separator();
    ui.add_space(space::FIELD);
    let busy = state.pending.is_some();
    let can_save = available && !busy && state.dirty && state.document.is_some();
    let can_accept = available
        && !busy
        && !state.dirty
        && state.receipt.is_none()
        && state
            .document
            .as_ref()
            .is_some_and(|document| document.draft.is_some());
    egui_taffy::tui(ui, ui.id().with("proposal-actions"))
        .reserve_available_width()
        .style(layout::full(layout::grid(2, space::FIELD)))
        .show(|tui| {
            layout::block(tui, |ui| {
                if ui
                    .add_enabled(can_save, Button::new("수정본 저장", Variant::Secondary))
                    .clicked()
                    && let (Some(document), Some(draft)) = (&state.document, state.draft.clone())
                {
                    state.pending = Some(worker::start(
                        client.clone(),
                        Action::Edit(EditProposal {
                            proposal_id: document.proposal_id.clone(),
                            draft,
                        }),
                        Some(document.draft_revision.0.to_string()),
                        ui.ctx().clone(),
                    ));
                }
            });
            layout::block(tui, |ui| {
                if ui
                    .add_enabled(can_accept, Button::new("명시적 채택", Variant::Primary))
                    .clicked()
                    && let Some(document) = &state.document
                {
                    state.pending = Some(worker::start(
                        client.clone(),
                        Action::Accept(AcceptProposal {
                            proposal_id: document.proposal_id.clone(),
                            target_source_revision: document.source.source_revision(),
                        }),
                        Some(document.draft_revision.0.to_string()),
                        ui.ctx().clone(),
                    ));
                }
            });
        });
    if state.pending.is_some() {
        ui.weak("요청 처리 중 · 입력을 보존합니다.");
    } else if state.dirty {
        ui.weak("저장하지 않은 수정 · 채택 전 저장하세요.");
    } else if state.error.is_some() {
        ui.weak("요청 실패 · 본문의 오류를 확인하세요.");
    } else if let Some(receipt) = &state.receipt {
        ui.strong(format!("채택됨 · 출력 버전 {}", receipt.output_revision.0));
        ui.small("문서 버전 생성 · 미디어 렌더링/내보내기 아님");
    } else if state.document.is_some() {
        ui.weak("저장된 수정본만 명시적으로 채택합니다.");
    } else {
        ui.weak("제안 ID로 원문과 수정본을 조회하세요.");
    }
}
