use super::view;
use crate::{
    application::workspace_read::{ReadOrigin, WorkspaceId, WorkspaceSnapshot},
    presentation::{
        components::{
            accordion,
            artboard::Artwork,
            badge,
            button::{Button, Variant},
            card,
            collection_state::CollectionState,
            data_table,
            described_choice::{self, Choice},
            filter_bar, input, inspector, media_preview, metadata_row, notice, output_summary,
        },
        design_tokens::{breakpoint, size, space},
        layout,
    },
};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot, selected: &mut Option<String>) {
    match snapshot.workspace_id {
        WorkspaceId::Model => {
            egui_taffy::tui(ui, ui.id().with("model-search-filter"))
                .reserve_available_width()
                .style(layout::full(layout::detail_sidebar(
                    size::MODEL_FILTER,
                    space::PANEL,
                )))
                .show(|tui| {
                    layout::block(tui, |ui| {
                        input::unavailable_search(ui, "모델 검색 (이름, 기능, 태그)")
                    });
                    layout::block(tui, |ui| {
                        input::unavailable_select(
                            ui,
                            "필터 연결 필요",
                            "실제 모델 카탈로그 필터 연결이 필요합니다.",
                        );
                    });
                });
            ui.add_space(space::GROUP);
            let collection = CollectionState::from_snapshot(snapshot);
            accordion::show(
                ui,
                "model-candidates",
                &collection.heading("후보 모델"),
                crate::presentation::control_icon::Icon::Model,
                true,
                |ui| {
                    collection.show_notice(ui, "모델");
                    let mut rows = Vec::with_capacity(snapshot.table.rows.len());
                    let active = snapshot
                        .table
                        .rows
                        .iter()
                        .find(|row| selected.as_ref() == Some(&row.row_id))
                        .or(snapshot.table.rows.first())
                        .map(|row| row.row_id.as_str());
                    for row in &snapshot.table.rows {
                        ui.push_id(&row.row_id, |ui| {
                            let label = view::cell(row, "model");
                            let response = card::selectable(
                                ui,
                                &row.row_id,
                                &label,
                                active == Some(row.row_id.as_str()),
                                |ui| {
                                    ui.strong(&label);
                                    ui.add_space(space::FIELD);
                                    badge::list(
                                        ui,
                                        "model-metadata",
                                        &["task", "precision", "install"]
                                            .map(|key| view::cell(row, key)),
                                    );
                                },
                            )
                            .response;
                            if response.clicked() {
                                response.request_focus();
                                *selected = Some(row.row_id.clone());
                            }
                            rows.push(response);
                        });
                        ui.add_space(space::FIELD);
                    }
                    if let Some(index) =
                        super::super::navigation::keyboard_selection(ui.ctx(), &rows, false)
                    {
                        *selected = Some(snapshot.table.rows[index].row_id.clone());
                    }
                },
            );
            use crate::presentation::control_icon::Icon;
            accordion::group(ui, |ui| {
                for (index, (id, feature, icon)) in [
                    ("speech", "음성 인식", Icon::Microphone),
                    ("ocr", "화면 OCR", Icon::ScanText),
                    ("music", "음악 구간", Icon::Music),
                    ("segmentation", "분할·추적", Icon::Subject),
                    ("text", "텍스트", Icon::TextDocument),
                    ("depth", "깊이", Icon::Cube),
                ]
                .into_iter()
                .enumerate()
                {
                    if index > 0 {
                        ui.add(egui::Separator::default().spacing(1.));
                    }
                    accordion::show(ui, id, feature, icon, false, |ui| {
                        ui.weak("기능별 모델 카탈로그 연결 필요");
                    });
                }
            });
        }
        WorkspaceId::Job => {
            let filters = ["전체", "입력 필요", "대기", "실행", "실패", "완료"]
                .into_iter()
                .enumerate()
                .map(|(index, label)| filter_bar::Filter {
                    label,
                    available: index == 0,
                })
                .collect::<Vec<_>>();
            filter_bar::show(
                ui,
                "job-status-filters",
                &filters,
                &mut Some(0),
                "작업 상태 조회 계약 연결 필요 · 현재 전체 목록만 표시합니다.",
            );
            ui.add_space(space::GROUP);
            ui.horizontal_wrapped(|ui| {
                ui.label("동시 실행 수 — · 설정 연결 필요");
                ui.add_enabled(
                    false,
                    Button::new("전체 시작", Variant::Primary)
                        .icon(crate::presentation::control_icon::Icon::Play)
                        .compact(),
                );
            });
            ui.add_space(space::GROUP);
            let mut table = snapshot.table.clone();
            if table.columns.is_empty() {
                use crate::application::workspace_read::{CellKind, TableColumn};
                table.columns = [
                    ("task", "작업", CellKind::Text),
                    ("source", "소스", CellKind::Text),
                    ("progress", "진행 상태", CellKind::Percent),
                    ("output", "출력", CellKind::Text),
                ]
                .map(|(id, label, kind)| TableColumn {
                    column_id: id.into(),
                    label: label.into(),
                    kind,
                })
                .into();
            }
            if !table
                .rows
                .iter()
                .any(|row| selected.as_ref() == Some(&row.row_id))
            {
                *selected = table.rows.first().map(|row| row.row_id.clone());
            }
            if !table
                .columns
                .iter()
                .any(|column| column.column_id == "status")
            {
                table.columns.insert(
                    2.min(table.columns.len()),
                    crate::application::workspace_read::TableColumn {
                        column_id: "status".into(),
                        label: "상태".into(),
                        kind: crate::application::workspace_read::CellKind::Text,
                    },
                );
            }
            ui.weak(
                "상태별 필터 연결 필요 · 전체 목록만 표시합니다. 진행률과 상태는 별도 정보입니다.",
            );
            data_table::show_framed(
                ui,
                &table,
                selected,
                &[
                    ("status", size::QUEUE_STATUS),
                    ("progress", size::QUEUE_PROGRESS),
                ],
                &data_table::Appearance {
                    row_height: size::QUEUE_ROW,
                    leading_icon: Some(crate::presentation::control_icon::Icon::List),
                    whole_row_focus: true,
                },
            );
            CollectionState::from_snapshot(snapshot).show_notice(ui, "작업");
        }
        WorkspaceId::Out => {
            let columns = if ui.ctx().content_rect().width() < breakpoint::STACK_EDITOR {
                1
            } else {
                2
            };
            egui_taffy::tui(ui, ui.id().with("export-summary"))
                .reserve_available_width()
                .style(layout::full(layout::grid(columns, space::PANEL)))
                .show(|tui| {
                    layout::block(tui, |ui| {
                        card::show(ui, "", |ui| {
                            media_preview::unavailable(
                                ui,
                                &Artwork {
                                    asset_id: snapshot
                                        .preview
                                        .frame
                                        .as_ref()
                                        .filter(|_| snapshot.origin == ReadOrigin::DesignSample)
                                        .map(|a| a.asset_id.as_str()),
                                    title: "",
                                    subtitle: "",
                                    cover: false,
                                    grid: false,
                                },
                                16. / 9.,
                                "16:9",
                                size::PREVIEW_HEIGHT,
                            );
                            ui.add_space(space::GROUP);
                            ui.strong("내보내기 요약");
                            ui.add_space(space::FIELD);
                            if let Some(source) = &snapshot.source {
                                ui.label(&source.title);
                            }
                            for row in &snapshot.table.rows {
                                ui.add_space(space::GROUP);
                                let name = view::cell(row, "file-name");
                                let format = view::cell(row, "format");
                                output_summary::show(
                                    ui,
                                    &output_summary::OutputSummary {
                                        id: &row.row_id,
                                        name: &name,
                                        format: &format,
                                        source_artwork: &Artwork {
                                            asset_id: snapshot
                                                .preview
                                                .frame
                                                .as_ref()
                                                .filter(|_| {
                                                    snapshot.origin == ReadOrigin::DesignSample
                                                })
                                                .map(|asset| asset.asset_id.as_str()),
                                            title: "",
                                            subtitle: "",
                                            cover: false,
                                            grid: false,
                                        },
                                    },
                                );
                            }
                        });
                    });
                    layout::block(tui, |ui| destination(ui, snapshot));
                });
        }
        _ => {}
    }
}

pub fn inspector(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot, selected: &Option<String>) {
    super::system_inspector::show(ui, snapshot, selected);
}

fn destination(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot) {
    card::show(ui, "저장 위치", |ui| {
        input::unavailable_path(ui);
        inspector::section(ui, "파일 이름");
        ui.add_enabled_ui(false, |ui| {
            ui.add_sized(
                [ui.available_width(), size::CONTROL],
                egui::TextEdit::singleline(&mut String::new()).hint_text("새 출력 파일 이름"),
            );
        });
        ui.add_space(space::GROUP);
        notice::show(
            ui,
            notice::Kind::Warning,
            "파일 충돌 검증 필요",
            "출력 경로를 선택한 뒤 기존 파일과 비교합니다. 현재 충돌 여부는 확인되지 않았습니다.",
        );
        described_choice::unavailable(
            ui,
            &[
                Choice {
                    id: "export-rename",
                    title: "새 이름으로 저장 (권장)",
                    description: "기존 파일을 보존하고 다른 이름으로 저장합니다. 실제 후보 이름은 경로 검증 후 표시합니다.",
                },
                Choice {
                    id: "export-overwrite",
                    title: "덮어쓰기",
                    description: "기존 파일을 바꿉니다. 대상 파일과 적용 범위를 확인해야 합니다.",
                },
            ],
            "충돌 처리 방식 연결 필요",
        );
        ui.weak("충돌 처리 방식 연결 필요");
        inspector::section(ui, "확인 사항");
        for field in &snapshot.inspector {
            let value = data_table::value(&field.value);
            use crate::presentation::control_icon::Icon;
            let icon = match field.field_id.as_str() {
                "sample-Out-location" => Icon::Folder,
                "sample-Out-collision" => Icon::Warning,
                "sample-Out-format" => Icon::FileOutput,
                _ => Icon::Info,
            };
            metadata_row::show(ui, &field.field_id, icon, &field.label, &value);
        }
        egui_taffy::tui(ui, ui.id().with("export-actions"))
            .reserve_available_width()
            .style(layout::full(layout::grid(3, space::FIELD)))
            .show(|tui| {
                for (label, variant) in [
                    ("취소", Variant::Secondary),
                    ("뒤로", Variant::Secondary),
                    ("저장 시작", Variant::Primary),
                ] {
                    layout::block(tui, |ui| {
                        ui.add_enabled(false, Button::new(label, variant));
                    });
                }
            });
        ui.small("로컬 저장이며 외부 플랫폼 게시가 아닙니다.");
    });
}
