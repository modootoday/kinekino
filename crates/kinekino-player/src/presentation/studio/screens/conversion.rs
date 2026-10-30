mod sample;
use crate::{
    application::workspace_read::{ReadOrigin, WorkspaceSnapshot},
    presentation::{
        components::{cue_comparison, notice},
        design_tokens::{breakpoint, space},
        layout,
    },
};
use eframe::egui;

pub(super) fn show(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot, selected: &mut Option<String>) {
    let source = sample::cues(snapshot, false);
    let target = sample::cues(snapshot, true);
    if !source.iter().any(|cue| selected.as_ref() == Some(&cue.id)) {
        *selected = source.first().map(|cue| cue.id.clone());
    }
    let sample = snapshot.origin == ReadOrigin::DesignSample;
    if sample {
        ui.small("디자인 문법 비교 예시 · WebVTT → SRT · 실제 파일 변환 결과 아님");
    }
    let columns = if ui.ctx().content_rect().width() < breakpoint::STACK_EDITOR {
        1
    } else {
        2
    };
    egui_taffy::tui(ui, ui.id().with("subtitle-conversion-comparison"))
        .reserve_available_width()
        .style(layout::full(layout::grid(columns, space::PANEL)))
        .show(|tui| {
            for (id, title, cues) in [
                (
                    "source",
                    if sample {
                        "원본 WebVTT · 예시"
                    } else {
                        "원본 자막"
                    },
                    &source,
                ),
                (
                    "target",
                    if sample {
                        "출력 SRT · 예시"
                    } else {
                        "출력 자막"
                    },
                    &target,
                ),
            ] {
                layout::block(tui, |ui| {
                    cue_comparison::show(
                        ui,
                        id,
                        title,
                        cues,
                        selected,
                        "변환 문서 미리보기 연결 필요",
                    )
                });
            }
        });
    ui.add_space(space::PANEL);
    notice::show(
        ui,
        notice::Kind::Info,
        "정보 손실 검토 연결 필요",
        "실제 입력 형식과 옵션을 분석한 손실 목록이 제공되면 표시합니다. 디자인 예시는 손실 검토나 저장 완료를 의미하지 않습니다.",
    );
}
