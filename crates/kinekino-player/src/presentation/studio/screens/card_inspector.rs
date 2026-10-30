use crate::{
    application::workspace_read::{ReadOrigin, WorkspaceSnapshot},
    presentation::{
        components::{color_swatches, input, inspector, notice, segmented},
        design_tokens::space,
    },
};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot, selected: &Option<String>) {
    let items = super::view::items(snapshot);
    let item = super::view::selected_index(&items, selected).map(|n| &items[n]);
    let sample = snapshot.origin == ReadOrigin::DesignSample && item.is_some();
    inspector::panel(
        ui,
        "card-inspector",
        |ui| {
            ui.strong("업로드 대상");
            let mut target = sample.then_some(0);
            ui.add_enabled_ui(false, |ui| {
                segmented::show(
                    ui,
                    "output-preset",
                    &["YouTube 커뮤니티", "Instagram"],
                    &mut target,
                );
            });
            inspector::section(ui, "가로세로 비율");
            let mut ratio = sample.then_some(0);
            ui.add_enabled_ui(false, |ui| {
                segmented::show(ui, "card-ratio", &["1:1", "4:5", "3:4", "9:16"], &mut ratio);
            });
            inspector::section(ui, "템플릿");
            let mut template = sample.then_some(0);
            ui.add_enabled_ui(false, |ui| {
                segmented::show(
                    ui,
                    "card-template",
                    &["표지", "본문", "마지막"],
                    &mut template,
                );
            });
            inspector::section(ui, "텍스트");
            ui.label("제목 (선택)");
            input::readonly(ui, item.map_or("연결 필요", |item| item.title.as_str()));
            ui.add_space(space::FIELD);
            ui.label("본문 (선택)");
            input::readonly(ui, item.map_or("연결 필요", |item| item.subtitle));
            ui.add_space(space::FIELD);
            notice::show(
                ui,
                notice::Kind::Info,
                "텍스트 자동 맞춤",
                "카드 영역에 맞춰 크기와 줄바꿈을 조정합니다. 넘침 검증 연결 필요",
            );
            inspector::section(ui, "폰트");
            input::readonly(ui, "Noto Sans KR");
            ui.add_space(space::FIELD);
            ui.label("텍스트 색상");
            color_swatches::unavailable(ui);
            inspector::section(ui, "요약 텍스트 초안");
            inspector::unavailable_actions(ui, "요약 초안 생성", "소스 장면");
            ui.add_space(space::GROUP);
            ui.label("대체 텍스트 (alt text)");
            input::readonly(ui, "연결 필요");
        },
        |ui| {
            ui.separator();
            inspector::unavailable_actions(ui, "선택 카드 저장", "전체 저장");
            ui.small("로컬 파일로 저장 · 외부 플랫폼 게시 동작 아님");
        },
    );
}
