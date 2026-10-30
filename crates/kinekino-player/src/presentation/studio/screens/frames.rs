use super::view;
use crate::{
    application::workspace_read::WorkspaceSnapshot,
    presentation::{
        components::{
            artboard::{self, Artwork},
            frame_thumbnail, keyboard_selection, toolbar,
        },
        control_icon::Icon,
        design_tokens::{size, space},
        layout,
    },
};
use eframe::egui;

const FRAME_ACTIONS: &[(Icon, &str)] = &[
    (Icon::Previous, "이전 프레임"),
    (Icon::Lock, "프레임 고정"),
    (Icon::Next, "다음 프레임"),
    (Icon::Image, "원본 비교"),
];

pub fn inspector(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot, tab: usize) {
    use crate::presentation::components::{
        inspector,
        inspector_field::{self, Field, Kind},
    };
    let source = snapshot.source.as_ref().map(|source| source.title.as_str());
    let pts = source.map(|_| {
        crate::presentation::components::data_table::value(
            &crate::application::workspace_read::CellValue::TimeUs(snapshot.preview.position_us),
        )
    });
    let resolution = snapshot
        .preview
        .frame
        .as_ref()
        .filter(|_| snapshot.origin == crate::application::workspace_read::ReadOrigin::DesignSample)
        .and_then(|frame| crate::presentation::design_assets::find(&frame.asset_id))
        .map(|asset| format!("{} × {}", asset.size.x, asset.size.y));
    inspector::panel(
        ui,
        &format!("frame-inspector-{tab}"),
        |ui| {
            let groups: &[(&str, &[Field<'_>])] = if tab == 0 {
                &[
                    (
                        "프레임 정보",
                        &[
                            Field::new("source", "소스", Kind::Readout(source)),
                            Field::new("source-pts", "sourcePTS", Kind::Readout(pts.as_deref())),
                            Field::new(
                                "source-size",
                                "해상도",
                                Kind::Readout(resolution.as_deref()),
                            ),
                            Field::new("color-space", "색 공간", Kind::Readout(None)),
                        ],
                    ),
                    (
                        "프레임 추출",
                        &[
                            Field::new("output-size", "출력 크기", Kind::Select),
                            Field::new("output-format", "파일 형식", Kind::Select),
                            Field::new("output-path", "저장 위치", Kind::Path),
                            Field::new("output-name", "파일 이름", Kind::Text),
                        ],
                    ),
                ]
            } else {
                &[
                    (
                        "썸네일 설정",
                        &[
                            Field::new("output-size", "출력 크기", Kind::Select),
                            Field::new("crop-ratio", "자르기 비율", Kind::Select),
                            Field::new("background", "배경", Kind::Select),
                        ],
                    ),
                    (
                        "텍스트",
                        &[
                            Field::new("title", "제목", Kind::Text),
                            Field::new("font", "글꼴", Kind::Select),
                            Field::new("font-size", "크기", Kind::Scalar),
                            Field::new("color", "색상", Kind::Color),
                            Field::new("outline", "외곽선", Kind::Toggle),
                            Field::new("shadow", "그림자", Kind::Toggle),
                        ],
                    ),
                    (
                        "출력",
                        &[
                            Field::new("output-format", "파일 형식", Kind::Select),
                            Field::new("output-path", "저장 위치", Kind::Path),
                            Field::new("output-name", "파일 이름", Kind::Text),
                        ],
                    ),
                ]
            };
            for (index, (title, fields)) in groups.iter().enumerate() {
                if index == 0 {
                    inspector::heading(ui, title);
                } else {
                    inspector::section(ui, title);
                }
                inspector_field::unavailable(ui, fields);
            }
        },
        |ui| {
            ui.separator();
            inspector::unavailable_actions(ui, "미리보기", "내보내기");
        },
    );
}

pub fn show(
    ui: &mut egui::Ui,
    snapshot: &WorkspaceSnapshot,
    tab: usize,
    selected: &mut Option<String>,
) {
    let items = view::items(snapshot);
    if selected.is_none() && items.len() > 1 {
        *selected = Some(items[1].id.clone());
    }
    let index = view::selected_index(&items, selected);
    let asset = index.and_then(|index| items[index].asset_id);
    let title = if tab == 1 && index.is_some() {
        "별이 머무는 밤"
    } else {
        ""
    };
    artboard::show(
        ui,
        &Artwork {
            asset_id: asset,
            title,
            subtitle: "",
            cover: tab == 1,
            grid: tab == 1,
        },
        16. / 9.,
        (ui.available_height() - size::SOURCE_REFERENCE_HEIGHT - size::CONTROL - space::PANEL)
            .max(size::ARTBOARD_MIN),
        false,
    );
    ui.add_space(space::PANEL);
    if tab == 1 {
        toolbar::unavailable_grouped(ui, &[FRAME_ACTIONS]);
        ui.add_space(space::FIELD);
    }
    let mut responses = vec![None; items.len()];
    egui_taffy::tui(ui, ui.id().with("frame-strip"))
        .reserve_available_width()
        .style(layout::full(layout::grid(3, space::PANEL)))
        .show(|tui| {
            for (n, item) in items.iter().enumerate() {
                layout::block(tui, |ui| {
                    ui.push_id(&item.id, |ui| {
                        let response = frame_thumbnail::show(
                            ui,
                            item.asset_id,
                            ["이전 프레임", "현재 프레임", "다음 프레임"][n % 3],
                            index == Some(n),
                        );
                        if response.clicked() {
                            *selected = Some(item.id.clone());
                        }
                        responses[n] = Some(response);
                    });
                });
            }
        });
    let responses: Vec<_> = responses.into_iter().flatten().collect();
    if let Some(index) = keyboard_selection::select(ui.ctx(), &responses, true) {
        *selected = Some(items[index].id.clone());
    }
    if tab == 0 {
        ui.add_space(space::FIELD);
        toolbar::unavailable_grouped(ui, &[FRAME_ACTIONS]);
    }
}
