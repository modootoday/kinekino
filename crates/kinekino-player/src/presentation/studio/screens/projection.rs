use crate::{
    application::workspace_read::{ReadOrigin, WorkspaceSnapshot},
    presentation::{
        components::{
            artboard::{self, Artwork},
            card, file_row, inspector,
            inspector_field::{self, Field, Kind},
            notice, projection_scene, toolbar,
        },
        control_icon::Icon,
        design_tokens::{breakpoint, size, space, text},
        layout,
    },
};
use eframe::egui;

const SAMPLE_FILES: [&str; 5] = [
    "texture.png",
    "mask.png",
    "depth.png",
    "projection.json",
    "scene.glb",
];

pub fn show(
    ui: &mut egui::Ui,
    snapshot: &WorkspaceSnapshot,
    tab: usize,
    selected: &mut Option<String>,
) {
    let asset = snapshot
        .preview
        .frame
        .as_ref()
        .filter(|_| snapshot.origin == ReadOrigin::DesignSample)
        .map(|a| a.asset_id.as_str());
    let columns = if tab == 1 && ui.ctx().content_rect().width() >= breakpoint::STACK_EDITOR {
        2
    } else {
        1
    };
    let mut file_responses = vec![None; SAMPLE_FILES.len()];
    egui_taffy::tui(ui, ui.id().with("projection-view"))
        .reserve_available_width()
        .style(layout::full(if columns == 2 {
            layout::detail_sidebar(size::PACKAGE_TREE, space::PANEL)
        } else {
            layout::column(space::PANEL)
        }))
        .show(|tui| {
            layout::block(tui, |ui| {
                projection_scene::show(ui, asset, size::PREVIEW_HEIGHT + size::SOURCE_THUMBNAIL);
            });
            if tab == 1 {
                layout::block(tui, |ui| {
                    card::show(ui, "레이어 및 파일", |ui| {
                        egui::collapsing_header::CollapsingState::load_with_default_open(
                            ui.ctx(),
                            ui.id().with("scene-folder"),
                            true,
                        )
                        .show_header(ui, |ui| {
                            ui.add(
                                Icon::Folder
                                    .image()
                                    .fit_to_exact_size(egui::Vec2::splat(size::TOOLBAR_ICON))
                                    .tint(crate::presentation::design_tokens::color::ACCENT),
                            );
                            ui.label("scene");
                        })
                        .body(|ui| {
                            if asset.is_some() {
                                for (index, file) in SAMPLE_FILES.iter().copied().enumerate() {
                                    let icon = match index {
                                        0 => Icon::FileImage,
                                        1 => Icon::FileMask,
                                        2 => Icon::FileDepth,
                                        3 => Icon::FileJson,
                                        _ => Icon::Cube,
                                    };
                                    let response = file_row::show(
                                        ui,
                                        file,
                                        file,
                                        icon,
                                        selected.as_deref() == Some(file),
                                    );
                                    if response.clicked() {
                                        *selected = Some(file.into());
                                        response.request_focus();
                                    }
                                    file_responses[index] = Some(response);
                                }
                            } else {
                                ui.weak("패키지 입력 연결 필요");
                            }
                        });
                        if asset.is_some() {
                            ui.small(selected.as_ref().map_or_else(
                                || "파일을 선택해 주세요".into(),
                                |file| format!("선택: {file} · 파일 미리보기 연결 필요"),
                            ));
                        }
                        ui.add_space(space::FIELD);
                        ui.small("원본 프레임 (참고)");
                        artboard::show(
                            ui,
                            &Artwork {
                                asset_id: asset,
                                title: "",
                                subtitle: "",
                                cover: false,
                                grid: false,
                            },
                            16. / 9.,
                            size::SOURCE_REFERENCE_HEIGHT,
                            false,
                        );
                        ui.small("디자인 패키지 예시 · 실제 파일이 아닙니다.");
                    });
                });
            }
        });
    let file_responses: Vec<_> = file_responses.into_iter().flatten().collect();
    if let Some(index) = crate::presentation::components::keyboard_selection::select(
        ui.ctx(),
        &file_responses,
        false,
    ) {
        *selected = Some(SAMPLE_FILES[index].into());
    }
    if tab == 0 {
        ui.small("RGB / 깊이 / 레이어 / 투영 · 디자인 평면 예시, 추론 결과 아님");
        toolbar::unavailable(
            ui,
            "projection-view",
            &[
                (Icon::FileImage, "원본 프레임"),
                (Icon::Layers, "레이어"),
                (Icon::Cube, "시점 미리보기"),
            ],
        );
    } else {
        card::show(ui, "결과 검증 (패키지 유효성)", |ui| {
            egui::Grid::new("package-validation")
                .num_columns(3)
                .min_col_width((ui.available_width() - space::GROUP * 2.) / 3.)
                .min_row_height(text::BODY + space::TIGHT)
                .striped(true)
                .spacing(egui::vec2(space::GROUP, space::FIELD))
                .show(ui, |ui| {
                    for label in ["항목", "상태", "비고"] {
                        ui.strong(label);
                    }
                    ui.end_row();
                    for (field, detail) in [
                        ("축 (좌표계)", "좌표계 및 축 방향 확인"),
                        ("스케일 (단위)", "단위 일치 여부 확인"),
                        ("UV", "UV 원점 및 방향 확인"),
                        ("알파", "투명 영역 처리 확인"),
                        ("재질", "재질 데이터 포함 여부 확인"),
                        ("sourcePTS", "원본 시점 정보 포함 여부 확인"),
                    ] {
                        ui.label(field);
                        ui.weak("검증 전");
                        ui.label(detail);
                        ui.end_row();
                    }
                });
        });
    }
}
pub fn inspector(ui: &mut egui::Ui, _snapshot: &WorkspaceSnapshot, tab: usize) {
    inspector::panel(
        ui,
        &format!("projection-inspector-{tab}"),
        |ui| {
            if tab == 0 {
                ui.strong("깊이 및 투영 설정");
                ui.add_space(space::FIELD);
                depth_settings(ui);
                inspector::section(ui, "좌표계 및 단위");
            } else {
                ui.strong("좌표계 및 단위");
                ui.add_space(space::FIELD);
            }
            inspector_field::unavailable(
                ui,
                &[
                    Field {
                        id: "coordinate-system",
                        label: "좌표계",
                        kind: Kind::Select,
                    },
                    Field {
                        id: "up-axis",
                        label: "위쪽 축",
                        kind: Kind::Select,
                    },
                    Field {
                        id: "geometry-unit",
                        label: "기하 단위",
                        kind: Kind::Select,
                    },
                ],
            );
            ui.small("단위는 기하에만 적용되며 상대 깊이의 실제 거리를 뜻하지 않습니다.");
            if tab == 1 {
                inspector::section(ui, "깊이 정보");
                depth_settings(ui);
            }
            inspector::section(ui, "행렬 및 UV");
            inspector_field::unavailable(
                ui,
                &[
                    Field {
                        id: "matrix-order",
                        label: "행렬 저장 형식",
                        kind: Kind::Select,
                    },
                    Field {
                        id: "uv-origin",
                        label: "UV 원점",
                        kind: Kind::Select,
                    },
                    Field {
                        id: "alpha-mode",
                        label: "알파 처리",
                        kind: Kind::Select,
                    },
                ],
            );
            inspector::section(ui, "출력 설정");
            inspector_field::unavailable(
                ui,
                &[
                    Field {
                        id: "output-format",
                        label: "출력 형식",
                        kind: Kind::Select,
                    },
                    Field {
                        id: "target-tool",
                        label: "대상 도구",
                        kind: Kind::Select,
                    },
                ],
            );
            notice::show(
                ui,
                notice::Kind::Warning,
                "투영용 패키지입니다",
                "상대 깊이는 실제 거리가 아니며 완전한 3D 복원 결과가 아닙니다. 대상 도구 검증 후 저장할 수 있습니다.",
            );
        },
        |ui| {
            ui.separator();
            inspector::unavailable_actions(ui, "대상 도구 검증", "패키지 저장");
        },
    );
}

fn depth_settings(ui: &mut egui::Ui) {
    inspector_field::unavailable(
        ui,
        &[
            Field::new("depth-model", "깊이 모델", Kind::Select),
            Field::new("depth-range", "상대 깊이 범위", Kind::Scalar),
            Field::new("geometry-kind", "평면 / 깊이 메시", Kind::Select),
        ],
    );
    ui.small("깊이 값은 상대값이며 실제 거리 단위가 아닙니다.");
}
