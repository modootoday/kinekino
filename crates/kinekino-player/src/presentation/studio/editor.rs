use super::super::super::{
    control_icon::{self, Icon},
    player_theme,
};
use super::catalog::Workspace;
use crate::application::workspace_read::WorkspaceSnapshot;
use crate::presentation::components::data_table;
use eframe::egui;

pub(super) fn show(
    ui: &mut egui::Ui,
    item: &Workspace,
    source: &str,
    snapshot: &WorkspaceSnapshot,
    selected: &mut Option<String>,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        if !matches!(item.id, "MODEL" | "JOB" | "OUT") {
            super::read_view::preview(ui, source, snapshot);
            ui.add_space(12.);
            if matches!(item.id, "SEG" | "SUB" | "OBJ" | "CLIP") {
                if snapshot.timeline.is_empty() {
                    timeline(ui);
                } else {
                    super::read_view::timeline(ui, snapshot);
                }
                ui.add_space(10.);
            }
            ui.horizontal_wrapped(|ui| {
                for label in match item.id {
                    "SEG" => &["구간 추가", "구간 수정", "삭제"][..],
                    "SUB" => &["분할", "병합", "추가", "삭제", "되돌리기", "다시 실행"],
                    "OBJ" => &["선택", "포함 점", "제외 점", "브러시", "지우개"],
                    "FRAME" | "CARD" | "A2D" => &["선택", "텍스트", "레이어 추가", "원본 비교"],
                    _ => &["추가", "편집", "원본 비교"],
                } {
                    ui.add_enabled(false, egui::Button::new(*label))
                        .on_disabled_hover_text("원본과 편집 도구를 준비하고 있습니다.");
                }
            });
            ui.add_space(10.);
        } else {
            ui.horizontal(|ui| {
                ui.add_enabled(
                    false,
                    egui::TextEdit::singleline(&mut String::new())
                        .hint_text(match item.id {
                            "MODEL" => "모델 검색",
                            "OUT" => "파일 검색",
                            _ => "작업 검색",
                        })
                        .desired_width((ui.available_width() - 140.).max(80.)),
                );
                ui.add_enabled(false, egui::Button::new("필터"));
            });
            ui.add_space(16.);
        }
        if snapshot.table.columns.is_empty() {
            table(ui, item);
        } else {
            data_table::show(ui, &snapshot.table, selected);
        }
    });
}

pub(super) fn preview(ui: &mut egui::Ui, source: &str) {
    let height = (ui.available_height() - 240.).clamp(140., 340.);
    let frame = egui::Frame::new()
        .fill(crate::presentation::design_tokens::color::PREVIEW_SURFACE)
        .stroke(egui::Stroke::new(
            crate::presentation::design_tokens::border::THIN,
            player_theme::BORDER,
        ))
        .corner_radius(crate::presentation::design_tokens::radius::CONTROL);
    frame.show(ui, |ui| {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), height),
            egui::Sense::hover(),
        );
        let c = rect.center();
        Icon::StartMark.paint(
            ui,
            egui::Rect::from_center_size(c - egui::vec2(0., 22.), egui::vec2(44., 44.)),
            player_theme::SECONDARY,
        );
        ui.painter().text(
            c + egui::vec2(0., 20.),
            egui::Align2::CENTER_CENTER,
            if source.is_empty() {
                "원본 영상을 선택해 주세요"
            } else {
                "미리보기 준비 중"
            },
            egui::FontId::proportional(14.),
            player_theme::SECONDARY,
        );
        ui.horizontal(|ui| {
            ui.add_enabled_ui(false, |ui| {
                control_icon::button(ui, Icon::Play, "미리보기 재생", false, 32.)
            });
            ui.monospace("--:-- / --:--");
            let (line, _) = ui.allocate_exact_size(
                egui::vec2((ui.available_width() - 80.).max(20.), 16.),
                egui::Sense::hover(),
            );
            ui.painter().line_segment(
                [line.left_center(), line.right_center()],
                egui::Stroke::new(
                    crate::presentation::design_tokens::border::FOCUS,
                    player_theme::BORDER,
                ),
            );
            ui.add_enabled_ui(false, |ui| {
                control_icon::button(ui, Icon::Volume, "미리보기 볼륨", false, 32.)
            });
        });
    });
}

fn timeline(ui: &mut egui::Ui) {
    egui::Frame::group(ui.style())
        .fill(player_theme::PANEL)
        .inner_margin(8.)
        .show(ui, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), 38.), egui::Sense::hover());
            let p = ui.painter();
            p.line_segment(
                [rect.left_center(), rect.right_center()],
                egui::Stroke::new(
                    crate::presentation::design_tokens::border::THIN,
                    player_theme::BORDER,
                ),
            );
            for index in 0..=20 {
                let x = egui::lerp(rect.x_range(), index as f32 / 20.);
                p.line_segment(
                    [
                        egui::pos2(x, rect.top() + 10.),
                        egui::pos2(x, rect.bottom() - 10.),
                    ],
                    egui::Stroke::new(
                        crate::presentation::design_tokens::border::THIN,
                        player_theme::BORDER,
                    ),
                );
            }
            p.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "타임라인 · 원본 선택 필요",
                egui::FontId::proportional(12.),
                player_theme::SECONDARY,
            );
        });
}

fn table(ui: &mut egui::Ui, item: &Workspace) {
    let (title, columns): (&str, &[&str]) = match item.id {
        "SEG" => (
            "감지된 구간 목록",
            &["유형", "시작 시간", "종료 시간", "길이", "상태"],
        ),
        "SUB" => ("자막 목록", &["시작", "끝", "화자", "원문", "교정안"]),
        "OBJ" => ("피사체 목록", &["피사체", "구간", "상태", "검토"]),
        "CLIP" => ("클립 목록", &["제목", "구간", "길이", "상태"]),
        "FRAME" | "CARD" | "A2D" => ("레이어", &["이름", "유형", "표시", "상태"]),
        "MODEL" => (
            "모델 라이브러리",
            &["모델", "작업 종류", "정밀도", "설치 상태"],
        ),
        "JOB" => ("작업 대기열", &["작업", "소스", "진행 상태", "출력"]),
        "OUT" => ("출력 파일", &["파일명", "형식", "저장 위치", "검증"]),
        _ => ("작업 목록", &["소스", "유형", "형식", "상태"]),
    };
    egui::Frame::group(ui.style())
        .fill(player_theme::PANEL)
        .inner_margin(0.)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.add_space(8.);
            ui.horizontal(|ui| {
                ui.add_space(10.);
                ui.strong(title);
            });
            ui.add_space(6.);
            ui.separator();
            let column_width = ui.available_width() / columns.len() as f32;
            ui.horizontal(|ui| {
                for label in columns {
                    ui.add_sized(
                        [column_width - 6., 28.],
                        egui::Label::new(
                            egui::RichText::new(*label).color(player_theme::SECONDARY),
                        ),
                    );
                }
            });
            ui.separator();
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 90.),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(24.);
                    ui.weak(if matches!(item.id, "MODEL" | "JOB") {
                        "목록 조회 기능을 준비하고 있습니다."
                    } else {
                        "아직 표시할 항목이 없습니다."
                    });
                },
            );
        });
}
