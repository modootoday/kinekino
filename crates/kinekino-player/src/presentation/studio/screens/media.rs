use super::view;
use crate::{
    application::workspace_read::{ReadOrigin, WorkspaceId, WorkspaceSnapshot},
    presentation::{
        components::{
            artboard::{self, Artwork},
            card, data_table,
            filmstrip::{self, Region},
            inspector::{self, unavailable_actions as actions},
            inspector_field::{self, Field as F, Kind as K},
            media_preview, segmented, source_media_card, toolbar,
        },
        control_icon::Icon,
        design_tokens::{size, space},
    },
};
use eframe::egui;

pub fn show(
    ui: &mut egui::Ui,
    snapshot: &WorkspaceSnapshot,
    tab: usize,
    selected: &mut Option<String>,
) {
    match snapshot.workspace_id {
        WorkspaceId::Seg => {
            preview(ui, snapshot, false);
            strip(ui, snapshot);
            ui.add_space(space::PANEL);
            card::show(ui, "감지된 구간 목록", |ui| {
                data_table::show(ui, &snapshot.table, selected);
            });
            toolbar::unavailable(
                ui,
                "segment-edit",
                &[
                    (Icon::Plus, "구간 추가"),
                    (Icon::Segments, "구간 수정"),
                    (Icon::Delete, "삭제"),
                ],
            );
        }
        WorkspaceId::Obj => {
            preview(ui, snapshot, true);
            strip(ui, snapshot);
        }
        WorkspaceId::Sub => {
            preview(ui, snapshot, tab == 2);
            if tab == 4 {
                super::conversion::show(ui, snapshot, selected);
            } else if tab == 2 {
                super::ocr::results(ui, snapshot, selected);
            } else {
                card::show(
                    ui,
                    if tab == 2 {
                        "감지된 텍스트"
                    } else if tab == 1 {
                        "화자 타임라인 · 전용 시안 미제공"
                    } else {
                        "음성 파형"
                    },
                    |ui| {
                        if tab == 0 || tab == 3 {
                            view::waveform(ui, snapshot, selected);
                        } else {
                            super::super::read_view::timeline(ui, snapshot);
                        }
                    },
                );
                toolbar::unavailable_grouped(
                    ui,
                    &[
                        &[
                            (Icon::Clip, "분할"),
                            (Icon::Merge, "병합"),
                            (Icon::Plus, "추가"),
                            (Icon::Delete, "삭제"),
                        ],
                        &[(Icon::Undo, "되돌리기"), (Icon::Redo, "다시 실행")],
                    ],
                );
                data_table::show_with_widths(
                    ui,
                    &data_table::with_row_numbers(&snapshot.table),
                    selected,
                    &[
                        ("display-number", size::TABLE_INDEX),
                        ("start", size::TABLE_TIME),
                        ("end", size::TABLE_TIME),
                        ("speaker", size::TIMELINE_LABEL),
                    ],
                );
            }
        }
        WorkspaceId::Download => {
            let height = artboard::media_height(
                ui.available_width(),
                ui.ctx().content_rect().height(),
                16. / 9.,
                size::ARTBOARD_MIN,
            );
            source_media_card::show(
                ui,
                snapshot
                    .source
                    .as_ref()
                    .map_or("영상 선택", |s| s.title.as_str()),
                &Artwork {
                    asset_id: snapshot
                        .preview
                        .frame
                        .as_ref()
                        .filter(|_| snapshot.origin == ReadOrigin::DesignSample)
                        .map(|asset| asset.asset_id.as_str()),
                    title: "",
                    subtitle: "",
                    cover: false,
                    grid: false,
                },
                height,
            );
            ui.add_space(space::PANEL);
            card::show(ui, "", |ui| {
                crate::presentation::components::track_selection::show_unverified(ui);
            });
        }
        _ => {}
    }
}

fn preview(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot, selection: bool) {
    let asset = snapshot
        .preview
        .frame
        .as_ref()
        .filter(|_| snapshot.origin == ReadOrigin::DesignSample)
        .map(|a| a.asset_id.as_str());
    media_preview::unavailable_with_overlay(
        ui,
        &Artwork {
            asset_id: asset,
            title: "",
            subtitle: "",
            cover: false,
            grid: false,
        },
        16. / 9.,
        "16:9",
        artboard::media_height(
            ui.available_width(),
            ui.ctx().content_rect().height(),
            16. / 9.,
            if snapshot.workspace_id == WorkspaceId::Obj {
                size::PREVIEW_HEIGHT
            } else {
                size::ARTBOARD_MIN
            },
        ),
        |ui, image| {
            if selection && asset.is_some() {
                let bounds = if snapshot.workspace_id == WorkspaceId::Sub {
                    egui::Rect::from_min_max(
                        image.lerp_inside(egui::vec2(0.1, 0.78)),
                        image.lerp_inside(egui::vec2(0.9, 0.94)),
                    )
                } else {
                    egui::Rect::from_min_max(
                        image.lerp_inside(egui::vec2(0.3, 0.12)),
                        image.lerp_inside(egui::vec2(0.55, 0.94)),
                    )
                };
                if snapshot.workspace_id == WorkspaceId::Sub {
                    crate::presentation::components::selection_frame::paint_ocr_region(ui, bounds);
                }
            }
            if snapshot.workspace_id == WorkspaceId::Obj {
                let mask = snapshot.mask.as_ref().filter(|m| {
                    asset.is_some()
                        && m.origin == ReadOrigin::DesignSample
                        && m.source_pts_us == snapshot.preview.position_us
                        && snapshot
                            .preview
                            .frame
                            .as_ref()
                            .is_some_and(|frame| frame.asset_id == m.source_asset.asset_id)
                });
                super::subject_preview::show(ui, image, mask);
                toolbar::canvas_unavailable(
                    ui,
                    image,
                    &[
                        (Icon::SelectionPointer, "선택"),
                        (Icon::Plus, "포함 점"),
                        (Icon::Minus, "제외 점"),
                        (Icon::BrushAdd, "브러시"),
                        (Icon::Eraser, "지우개"),
                    ],
                );
            }
        },
    );
    if snapshot.workspace_id == WorkspaceId::Obj
        && snapshot.mask.is_some()
        && snapshot.origin == ReadOrigin::DesignSample
    {
        ui.weak("수동 마스크 예시 · 미검토 · 실제 분할 결과 아님");
    }
}

fn strip(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot) {
    let regions: Vec<_> = snapshot
        .timeline
        .iter()
        .flat_map(|track| &track.segments)
        .map(|segment| Region {
            label: segment.label.as_str(),
            start: segment.in_us.0,
            end: segment.out_us.0,
        })
        .collect();
    let visible_end = view::visible_end(snapshot);
    if visible_end > 0 {
        ui.small(format!(
            "표시 범위 00:00 – {}",
            crate::presentation::components::data_table::value(
                &crate::application::workspace_read::CellValue::TimeUs(
                    crate::domain::api_values::DecimalU64(visible_end)
                )
            )
        ));
    }
    filmstrip::show(
        ui,
        snapshot
            .preview
            .frame
            .as_ref()
            .filter(|_| snapshot.origin == ReadOrigin::DesignSample)
            .map(|a| a.asset_id.as_str()),
        visible_end,
        snapshot.preview.position_us.0,
        &regions,
    );
}

pub fn inspector(
    ui: &mut egui::Ui,
    snapshot: &WorkspaceSnapshot,
    selected: &Option<String>,
    tab: usize,
) {
    inspector::panel(
        ui,
        &format!("media-inspector-{:?}-{tab}", snapshot.workspace_id),
        |ui| match snapshot.workspace_id {
            WorkspaceId::Seg => {
                ui.strong("구간 건너뛰기 설정");
                inspector_field::unavailable(
                    ui,
                    &[
                        F::new("opening-mode", "오프닝 설정", K::Select),
                        F::new("ending-mode", "엔딩 설정", K::Select),
                        F::new("autoplay-next", "다음 회차 자동 재생", K::Toggle),
                        F::new("episode-confirmation", "회차별 확인", K::Toggle),
                        F::new("after-skip", "건너뛰기 후 동작", K::Select),
                        F::new("advance-notice", "미리 알림", K::Scalar),
                    ],
                );
                inspector::section(ui, "최근 작업 기록");
                ui.weak("작업 기록 연결 필요");
            }
            WorkspaceId::Obj => {
                super::subject_list::show(ui, snapshot);
                inspector::section(ui, "작업 설정");
                ui.label("분석 방식");
                ui.add_enabled_ui(false, |ui| {
                    segmented::show(
                        ui,
                        "subject-analysis-mode",
                        &["탐지", "분할", "추적"],
                        &mut None,
                    );
                });
                ui.weak("분석 방식 연결 필요");
                ui.add_space(space::FIELD);
                inspector_field::unavailable(
                    ui,
                    &[
                        F::new("analysis-range", "선택 구간", K::Readout(None)),
                        F::new("mask-opacity", "마스크 불투명도", K::Scalar),
                        F::new("edge-refinement", "경계 세부 보정", K::Scalar),
                        F::new("model", "모델 선택", K::Select),
                    ],
                );
            }
            WorkspaceId::Sub => {
                ui.strong(
                    [
                        "인식 설정",
                        "화자 분리 설정",
                        "OCR 설정",
                        "자막 교정",
                        "자막 변환",
                    ][tab.min(4)],
                );
                if tab == 3 {
                    super::correction_inspector::show(ui, snapshot, selected);
                }
                super::subtitle_inspector::fields(ui, tab);
                if tab == 2 {
                    super::ocr::text_inspector(ui, snapshot, selected);
                }
            }
            WorkspaceId::Download => {
                super::save_inspector::fields(ui);
            }
            _ => {}
        },
        |ui| {
            ui.separator();
            let (secondary, primary) = match snapshot.workspace_id {
                WorkspaceId::Seg => ("구간 수정", "현재 회차 저장"),
                WorkspaceId::Obj => ("마스크 검토", "인식 시작"),
                WorkspaceId::Sub if tab == 3 => ("제안 기각", "제안 수락"),
                WorkspaceId::Sub if tab == 4 => ("검토", "내보내기"),
                WorkspaceId::Sub => ("검토", "시작"),
                WorkspaceId::Download => ("소스 확인", "대기열에 추가"),
                _ => return,
            };
            actions(ui, secondary, primary);
        },
    );
}
