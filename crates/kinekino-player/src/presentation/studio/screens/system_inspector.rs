use super::view;
use crate::{
    application::workspace_read::{ReadOrigin, WorkspaceId, WorkspaceSnapshot},
    presentation::{
        components::{
            badge,
            button::{Button, Variant},
            inspector,
            inspector_field::{self, Field as F, Kind as K},
            notice, stepper,
        },
        control_icon::Icon,
        design_tokens::space,
        layout,
    },
};
use eframe::egui;

pub(super) fn show(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot, selected: &Option<String>) {
    let row = snapshot
        .table
        .rows
        .iter()
        .find(|r| Some(&r.row_id) == selected.as_ref())
        .or(snapshot.table.rows.first());
    inspector::panel(
        ui,
        &format!("system-inspector-{:?}", snapshot.workspace_id),
        |ui| match snapshot.workspace_id {
            WorkspaceId::Model => {
                ui.strong("모델 정보");
                ui.add_space(space::FIELD);
                let Some(row) = row else {
                    notice::show(
                        ui,
                        notice::Kind::Info,
                        "선택된 모델이 없습니다",
                        "목록에서 후보를 선택하면 모델 정보를 확인할 수 있습니다. 목록 확인 상태는 본문 안내를 참조하세요.",
                    );
                    return;
                };
                badge::list(
                    ui,
                    "selected-model-metadata",
                    &["model", "install"].map(|key| view::cell(row, key)),
                );
                ui.add_space(space::FIELD);
                ui.small(if snapshot.origin == ReadOrigin::DesignSample {
                    "디자인 후보 예시 · 실제 선정·설치 상태가 아닙니다."
                } else {
                    "후보 정보 확인과 설치 가능 여부 검토가 필요합니다."
                });
                inspector::section(ui, "후보 모델 · 검토 필요");
                inspector_field::unavailable(
                    ui,
                    &[
                        F::new("repository", "Hugging Face 저장소", K::Text),
                        F::new("revision", "고정 revision", K::Text),
                        F::new("license", "라이선스", K::Readout(None)),
                        F::new("precision", "정밀도", K::Select),
                        F::new("device", "실행 장치", K::Select),
                        F::new("package-size", "패키지 크기", K::Readout(None)),
                    ],
                );
            }
            WorkspaceId::Job => {
                ui.strong(row.map_or_else(|| "작업 선택".into(), |r| view::cell(r, "task")));
                if row.is_none() {
                    ui.add_space(space::FIELD);
                    notice::show(
                        ui,
                        notice::Kind::Info,
                        "선택된 작업이 없습니다",
                        "목록에서 작업을 선택하면 단계·입력·출력 정보를 확인할 수 있습니다. 실제 목록을 확인할 수 있는지는 본문 안내를 참조하세요.",
                    );
                    return;
                }
                ui.add_space(space::GROUP);
                if snapshot.origin == ReadOrigin::DesignSample {
                    ui.small("단계 배치 예시 · 실제 실행 정보 아님");
                    ui.add_space(space::FIELD);
                    stepper::show(ui, &["입력 검사", "대기", "실행", "검증", "완료"], Some(0));
                }
                inspector::section(ui, "현재 단계");
                notice::show(
                    ui,
                    notice::Kind::Info,
                    "작업 단계 연결 필요",
                    "진행률·경과 시간·남은 시간은 실제 작업 정보가 제공될 때 표시합니다.",
                );
                ui.add_space(space::FIELD);
                inspector_field::unavailable(
                    ui,
                    &[
                        F::new("progress", "진행률", K::Readout(None)),
                        F::new("elapsed", "경과 시간", K::Readout(None)),
                        F::new("remaining", "남은 시간", K::Readout(None)),
                    ],
                );
                inspector::section(ui, "작업 입력");
                if let Some(row) = row {
                    let source = view::cell(row, "source");
                    let output = view::cell(row, "output");
                    inspector_field::unavailable(
                        ui,
                        &[
                            F::new("source", "소스", K::Readout(Some(&source))),
                            F::new("output", "출력", K::Readout(Some(&output))),
                        ],
                    );
                }
                ui.collapsing("진단 정보", |ui| {
                    ui.label("실제 작업 진단 연결 필요");
                });
            }
            _ => {}
        },
        |ui| match snapshot.workspace_id {
            WorkspaceId::Model => {
                ui.separator();
                ui.add_enabled(
                    false,
                    Button::new("모델 다운로드", Variant::Primary).icon(Icon::Export),
                )
                .on_disabled_hover_text("실제 모델 카탈로그와 다운로드 명령 연결 필요");
                ui.add_space(space::FIELD);
                ui.small("모델 정보와 다운로드 가능 여부 연결이 필요합니다.");
            }
            WorkspaceId::Job => {
                ui.separator();
                use egui_taffy::taffy::{FlexWrap, Style};
                egui_taffy::tui(ui, ui.id().with("job-actions"))
                    .reserve_available_width()
                    .style(Style {
                        flex_wrap: FlexWrap::Wrap,
                        ..layout::full(layout::row(space::FIELD))
                    })
                    .show(|tui| {
                        for (icon, action) in [
                            (Icon::Undo, "다시 시도"),
                            (Icon::Close, "취소"),
                            (Icon::Folder, "출력 폴더"),
                        ] {
                            layout::widget(tui, |ui| {
                                ui.add_enabled(
                                    false,
                                    Button::new(action, Variant::Secondary).icon(icon).compact(),
                                )
                                .on_disabled_hover_text(
                                    "실제 작업 명령과 실행 가능 여부 연결이 필요합니다.",
                                );
                            });
                        }
                    });
            }
            _ => {}
        },
    );
}
