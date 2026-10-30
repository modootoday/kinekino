use super::catalog::Workspace;
use crate::application::workspace_read::{ReadState, WorkspaceSnapshot};
use crate::presentation::components;
use crate::presentation::layout;
use eframe::egui;

pub(super) fn show_snapshot(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot) {
    ui.strong("검사기");
    ui.add_space(16.);
    egui_taffy::tui(ui, ui.id().with("snapshot-fields"))
        .reserve_available_width()
        .style(layout::full(layout::column(12.)))
        .show(|tui| {
            for field in &snapshot.inspector {
                components::field(tui, &field.label, |ui| {
                    let value = components::data_table::value(&field.value);
                    let color = if field.state == ReadState::NeedsInput {
                        crate::presentation::player_theme::ACCENT
                    } else {
                        crate::presentation::player_theme::TEXT
                    };
                    egui::Frame::group(ui.style())
                        .inner_margin(8)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.add(
                                egui::Label::new(egui::RichText::new(&value).color(color))
                                    .truncate(),
                            )
                            .on_hover_text(value);
                        });
                });
            }
        });
    ui.add_space(16.);
    ui.add_enabled(
        false,
        components::button::Button::new(
            "디자인 데모 · 실행 안 함",
            components::button::Variant::Secondary,
        ),
    );
    if let Some(notice) = &snapshot.notice {
        ui.weak(notice);
    }
}

pub(super) fn show(ui: &mut egui::Ui, item: &Workspace) {
    ui.strong(match item.id {
        "SEG" => "구간 건너뛰기 설정",
        "SUB" => "인식 설정",
        "OBJ" => "피사체 설정",
        "CLIP" => "숏폼 설정",
        "FRAME" => "이미지 설정",
        "CARD" => "카드 설정",
        "A2D" => "레이어 설정",
        "PROJ" => "투영 설정",
        "MODEL" => "모델 정보",
        "JOB" => "작업 상세",
        "OUT" => "내보내기 설정",
        _ => "저장 설정",
    });
    ui.add_space(16.);
    let fields: &[(&str, &str)] = match item.id {
        "SEG" => &[
            ("오프닝 설정", "수동"),
            ("엔딩 설정", "수동"),
            ("다음 회차 자동 재생", "끔"),
            ("미리 알림", "3초 전"),
            ("되돌리기 표시", "5초"),
        ],
        "SUB" => &[
            ("오디오 트랙", "선택 필요"),
            ("인식 언어", "자동 감지"),
            ("출력 번역", "끔 · 원문"),
            ("분석 범위", "원본 전체"),
            ("모델 선택", "선택 필요"),
            ("내보내기 형식", "SRT · UTF-8"),
        ],
        "OBJ" => &[
            ("분석 작업", "탐지 / 분할 / 추적"),
            ("모델 선택", "선택 필요"),
            ("분석 범위", "선택 필요"),
            ("마스크 표시", "원본과 합성"),
            ("불투명도", "50%"),
            ("경계 부드럽게", "0 px"),
        ],
        "CLIP" => &[
            ("출력 화면비", "9:16"),
            ("길이 제안", "30초"),
            ("화면 맞춤", "원본 유지"),
            ("자막 포함", "선택 필요"),
            ("출력 형식", "지원 확인 필요"),
        ],
        "FRAME" => &[
            ("프레임", "선택 필요"),
            ("출력 크기", "원본 크기"),
            ("자막 포함", "선택 필요"),
            ("출력 형식", "PNG"),
            ("색공간", "sRGB"),
            ("저장 위치", "선택 안 함"),
        ],
        "CARD" => &[
            ("플랫폼", "사용자 지정"),
            ("화면비", "1:1"),
            ("템플릿", "선택 필요"),
            ("글꼴 크기", "32 px"),
            ("출력 형식", "PNG"),
        ],
        "A2D" => &[
            ("표시 방식", "원본 / 알파 / 합성"),
            ("마스크 임계값", "0.5"),
            ("경계 부드럽게", "0 px"),
            ("피벗", "중앙 · 0.5, 0.5"),
            ("크기 배율", "1×"),
            ("출력 형식", "RGBA PNG"),
        ],
        "PROJ" => &[
            ("깊이 모델", "선택 필요"),
            ("깊이 단위", "상대 깊이 · 단위 없음"),
            ("시점 미리보기", "0°"),
            ("카메라 FOV", "45°"),
            ("출력 패키지", "선택 필요"),
        ],
        "MODEL" => &[
            ("저장소", "모델 선택 필요"),
            ("고정 revision", "선택 필요"),
            ("라이선스", "검토 필요"),
            ("정밀도", "지원 확인 필요"),
            ("실행 장치", "지원 확인 필요"),
        ],
        "JOB" => &[
            ("작업 상태", "선택한 작업 없음"),
            ("입력", "확인할 수 없음"),
            ("실행 단계", "확인할 수 없음"),
            ("결과 경로", "선택한 결과 없음"),
        ],
        "OUT" => &[
            ("출력 위치", "선택 안 함"),
            ("파일 충돌", "새 이름으로 저장"),
            ("출력 형식", "선택 필요"),
            ("저장 전 검증", "입력 확인 필요"),
        ],
        _ => &[
            ("소스", "미디어 선택 필요"),
            ("저장 방식", "원본 저장"),
            ("비디오 트랙", "확인 필요"),
            ("오디오 트랙", "확인 필요"),
            ("저장 위치", "선택 안 함"),
        ],
    };
    egui_taffy::tui(ui, ui.id().with((item.id, "inspector-fields")))
        .reserve_available_width()
        .style(layout::full(layout::column(12.)))
        .show(|tui| {
            for (index, (label, value)) in fields.iter().enumerate() {
                if index == 2 {
                    layout::block(tui, |ui| {
                        ui.separator();
                    });
                }
                components::field(tui, label, |ui| {
                    ui.add_enabled_ui(false, |ui| {
                        egui::ComboBox::from_id_salt((item.id, *label))
                            .selected_text(*value)
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                ui.label("기능 준비 중");
                            });
                    })
                    .response
                    .on_disabled_hover_text("현재 버전에서 준비 중인 기능입니다.");
                });
            }
        });
    ui.separator();
    ui.add_space(12.);
    ui.add_enabled(
        false,
        egui::Button::new(item.action).min_size(egui::vec2(ui.available_width(), 40.)),
    )
    .on_disabled_hover_text("실행 기능을 준비하고 있습니다.");
    ui.add_space(8.);
    ui.weak("준비 중 · 원본 파일은 변경되지 않습니다.");
}
