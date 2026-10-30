mod asset_canvas;
mod catalog;
mod editor;
mod inspector;
mod model_review;
mod navigation;
mod projects;
mod proposals;
mod read_view;
mod screens;
mod sdk_status;
mod subtitles;

use super::super::control_icon;
use crate::application::workspace_read::{
    LiveWorkspaceProvider, ReadOrigin, WorkspaceId, WorkspaceReadPort,
};
use crate::presentation::layout;
use catalog::WORKSPACES;
use eframe::egui;
use std::sync::Arc;

pub(in crate::presentation) struct StudioShell {
    pub open: bool,
    selected: usize,
    tabs: Vec<usize>,
    settings: bool,
    restore_settings_focus: bool,
    review_tools: bool,
    sdk: sdk_status::SdkStatus,
    subtitles: subtitles::SubtitleConversion,
    reviews: model_review::ModelReview,
    proposals: proposals::Proposals,
    projects: projects::Projects,
    reader: Arc<dyn WorkspaceReadPort>,
    selected_row: Option<String>,
    asset_canvas: asset_canvas::CanvasEditor,
}

impl Default for StudioShell {
    fn default() -> Self {
        Self::with_client(
            std::sync::Arc::<crate::application::api::PlayerSdk>::new(Default::default()),
            std::rc::Rc::new(crate::application::subtitles::path_picker::TextPathOnly),
        )
    }
}

impl StudioShell {
    pub fn with_client(
        client: crate::application::api::ApplicationClientHandle,
        paths: crate::application::subtitles::path_picker::SubtitlePathPickerHandle,
    ) -> Self {
        let sdk = sdk_status::SdkStatus::query(client.clone());
        let supported = sdk.supports("subtitles.convert");
        let projects = projects::Projects::new(client.clone(), sdk.supports("projects.inspect"));
        let proposals = proposals::Proposals::new(
            client.clone(),
            ["proposals.inspect", "proposals.edit", "proposals.accept"]
                .iter()
                .all(|command| sdk.supports(command)),
        );
        let reviews = model_review::ModelReview::new(
            client.clone(),
            [
                "reviews.inspect",
                "reviews.read_text_asset",
                "reviews.record_decision",
                "reviews.decisions",
            ]
            .iter()
            .all(|command| sdk.supports(command)),
        );
        Self {
            open: false,
            selected: 0,
            tabs: vec![0; WORKSPACES.len()],
            settings: false,
            restore_settings_focus: false,
            review_tools: false,
            sdk,
            reviews,
            proposals,
            projects,
            subtitles: subtitles::SubtitleConversion::new(client, paths, supported),
            reader: Arc::new(LiveWorkspaceProvider),
            selected_row: None,
            asset_canvas: Default::default(),
        }
    }
}

impl StudioShell {
    pub fn with_review_tools(mut self, enabled: bool) -> Self {
        self.review_tools = enabled;
        self
    }

    pub fn set_reader(&mut self, reader: Arc<dyn WorkspaceReadPort>) {
        self.reader = reader;
        self.selected_row = None;
        self.asset_canvas = Default::default();
    }

    fn workspace_id(&self) -> WorkspaceId {
        WorkspaceId::ALL[self.selected]
    }

    pub fn escape(&mut self) {
        if self.settings {
            self.settings = false;
            self.restore_settings_focus = true;
        } else {
            self.open = false;
        }
    }

    fn settings_trigger(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui
            .push_id("studio-settings-trigger", |ui| ui.button("설정"))
            .inner;
        if self.restore_settings_focus {
            response.request_focus();
            self.restore_settings_focus = false;
        }
        response
    }

    pub fn handle_escape(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            if egui::Popup::is_any_open(ctx) {
                egui::Popup::close_all(ctx);
            } else if !self.projects.close() {
                self.escape();
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, source: &str) {
        let ctx = &ui.ctx().clone();
        self.subtitles.poll();
        self.reviews.poll();
        self.proposals.poll();
        self.projects.poll();
        let width = ctx.content_rect().width();
        let mut queue_ui = egui::Panel::bottom("studio-queue")
            .min_size(crate::presentation::design_tokens::size::QUEUE)
            .show(ui, |ui| {
                ui.set_min_height(crate::presentation::design_tokens::size::QUEUE);
                ui.new_child(egui::UiBuilder::new().max_rect(ui.max_rect()))
            })
            .inner;
        egui::Panel::left("studio-navigation")
            .exact_size(if width < crate::presentation::design_tokens::breakpoint::FULL_NAVIGATION {
                crate::presentation::design_tokens::size::NAVIGATION_RAIL
            } else {
                crate::presentation::design_tokens::size::NAVIGATION
            })
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(super::super::player_theme::CHROME)
                    .inner_margin(egui::Margin::same(crate::presentation::design_tokens::space::RAIL_MARGIN)),
            )
            .show(ui, |ui| {
                if control_icon::navigation(
                    ui,
                    super::super::control_icon::Icon::Back,
                    "시청으로 돌아가기",
                    false,
                    width < crate::presentation::design_tokens::breakpoint::FULL_NAVIGATION,
                )
                .clicked()
                {
                    self.open = false;
                }
                ui.separator();
                ui.add_space(8.);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut rows = Vec::new();
                    for (index, item) in WORKSPACES.iter().enumerate() {
                        if index == 9 {
                            ui.separator();
                        }
                        let response = ui
                            .push_id(item.id, |ui| {
                                control_icon::navigation(
                                    ui,
                                    item.icon(),
                                    item.title,
                                    self.selected == index,
                                    width < crate::presentation::design_tokens::breakpoint::FULL_NAVIGATION,
                                )
                            })
                            .inner;
                        if response.clicked() {
                            if self.selected != index {
                                self.selected_row = None;
                            }
                            self.selected = index;
                            response.request_focus();
                        }
                        rows.push(response);
                    }
                    if let Some(index) = navigation::keyboard_selection(ctx, &rows, false) {
                        if self.selected != index {
                            self.selected_row = None;
                        }
                        self.selected = index;
                    }
                });
            });
        let snapshot = self.reader.snapshot(self.workspace_id());
        let has_inspector = snapshot.workspace_id != WorkspaceId::Out;
        if !has_inspector {
            self.settings = false;
        }
        if !has_inspector
            || width >= crate::presentation::design_tokens::breakpoint::FIXED_INSPECTOR
        {
            self.restore_settings_focus = false;
        }
        let mut inspector_ui = if has_inspector
            && width >= crate::presentation::design_tokens::breakpoint::FIXED_INSPECTOR
        {
            Some(
                egui::Panel::right("studio-inspector")
                    .exact_size(crate::presentation::design_tokens::size::INSPECTOR)
                    .resizable(false)
                    .frame(
                        egui::Frame::new()
                            .fill(super::super::player_theme::PANEL)
                            .inner_margin(egui::Margin::same(
                                crate::presentation::design_tokens::space::PANEL_MARGIN,
                            )),
                    )
                    .show(ui, |ui| {
                        ui.new_child(egui::UiBuilder::new().max_rect(ui.max_rect()))
                    })
                    .inner,
            )
        } else {
            None
        };
        if has_inspector
            && width < crate::presentation::design_tokens::breakpoint::FIXED_INSPECTOR
            && self.settings
        {
            let mut open = self.settings;
            crate::presentation::components::inspector::drawer(ctx, &mut open, |ui| {
                self.inspector(ui, false)
            });
            if !open {
                self.restore_settings_focus = true;
            }
            self.settings = open;
        }
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(crate::presentation::player_theme::CHROME)
                    .inner_margin(egui::Margin::same(
                        crate::presentation::design_tokens::space::PANEL_MARGIN,
                    )),
            )
            .show(ui, |ui| {
                let item = &WORKSPACES[self.selected];
                egui_taffy::tui(ui, ui.id().with("workspace-heading"))
                    .reserve_available_width()
                    .style(layout::full(layout::row(
                        crate::presentation::design_tokens::space::PANEL,
                    )))
                    .show(|tui| {
                        layout::block(tui, |ui| {
                            ui.heading(if item.id == "SUB" && self.tabs[self.selected] == 4 {
                                "자막 변환"
                            } else {
                                item.title
                            });
                        });
                        if self.review_tools {
                            layout::widget(tui, crate::presentation::validation::window_size);
                        }
                        if has_inspector
                            && width
                                < crate::presentation::design_tokens::breakpoint::FIXED_INSPECTOR
                            && layout::widget(tui, |ui| self.settings_trigger(ui)).clicked()
                        {
                            self.settings = !self.settings;
                        }
                    });
                ui.add_space(crate::presentation::design_tokens::space::HEADER_TO_DESCRIPTION);
                ui.weak(if item.id == "SUB" && self.tabs[self.selected] == 4 {
                    "원본을 보존하고 SRT · WebVTT 자막을 변환합니다."
                } else {
                    item.description()
                });
                ui.add_space(crate::presentation::design_tokens::space::HEADER_TO_TABS);
                if item.id == "JOB" {
                    self.tabs[self.selected] = 0;
                } else if item.tabs.len() > 1 {
                    ui.horizontal_wrapped(|ui| {
                        let mut rows = Vec::new();
                        for (index, tab) in item.tabs.iter().enumerate() {
                            let response = ui
                                .push_id((item.id, index), |ui| {
                                    navigation::tab(ui, tab, self.tabs[self.selected] == index)
                                })
                                .inner;
                            if response.clicked() {
                                self.tabs[self.selected] = index;
                                response.request_focus();
                            }
                            rows.push(response);
                        }
                        if let Some(index) = navigation::keyboard_selection(ctx, &rows, true) {
                            self.tabs[self.selected] = index;
                        }
                    });
                    ui.separator();
                    ui.add_space(crate::presentation::design_tokens::space::TABS_TO_BODY);
                }
                if item.id == "A2D" {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.asset_canvas.show(ui, snapshot.canvas.as_ref())
                    });
                } else if item.id == "SUB"
                    && self.tabs[self.selected] == 4
                    && snapshot.origin != ReadOrigin::DesignSample
                {
                    self.subtitles.show_preview(ui, source);
                } else if item.id == "MODEL" {
                    screens::model(ui, &snapshot, &mut self.selected_row, &mut self.reviews);
                } else if !screens::show(
                    ui,
                    &snapshot,
                    self.tabs[self.selected],
                    &mut self.selected_row,
                ) {
                    editor::show(ui, item, source, &snapshot, &mut self.selected_row);
                }
            });
        if let Some(ui) = &mut inspector_ui {
            self.inspector(ui, true);
        }
        self.queue(&mut queue_ui);
        self.projects.show(ctx);
    }

    fn queue(&mut self, ui: &mut egui::Ui) {
        let snapshot = self.reader.snapshot(self.workspace_id());
        if snapshot.origin == ReadOrigin::DesignSample {
            ui.horizontal_wrapped(|ui| {
                ui.strong("대기열 · 샘플");
                for item in &snapshot.queue {
                    ui.separator();
                    ui.add(egui::Label::new(&item.title).truncate())
                        .on_hover_text(&item.title);
                    if let Some(progress) = item.progress_percent {
                        ui.add(
                            egui::ProgressBar::new(f32::from(progress) / 100.)
                                .desired_width(100.)
                                .show_percentage(),
                        );
                    }
                }
            });
            return;
        }
        ui.horizontal_wrapped(|ui| {
            self.projects.launcher(ui);
            ui.separator();
            if ui.selectable_label(self.selected == 10, "대기열").clicked() {
                self.selected = 10;
            }
            ui.separator();
            if let Some(activity) = self.subtitles.activity() {
                if ui.button(activity).clicked() {
                    self.selected = 1;
                    self.tabs[1] = 4;
                }
            } else {
                ui.weak("진행 중인 변환 없음");
            }
        });
    }

    fn inspector(&mut self, ui: &mut egui::Ui, _heading: bool) {
        let snapshot = self.reader.snapshot(self.workspace_id());
        if snapshot.origin != ReadOrigin::DesignSample {
            self.sdk.show(ui);
            if let Some(task) =
                proposals::Proposals::task(snapshot.workspace_id, self.tabs[self.selected])
                && self.proposals.inspector_mode(ui, task)
            {
                self.proposals.show(ui, task);
                return;
            }
        }
        if snapshot.workspace_id == WorkspaceId::A2d {
            self.asset_canvas.inspector(ui, &snapshot);
            return;
        }
        if snapshot.origin != ReadOrigin::DesignSample && self.selected == 1 && self.tabs[1] == 4 {
            self.subtitles.show(ui);
            return;
        }
        if screens::inspector(ui, &snapshot, &self.selected_row, self.tabs[self.selected]) {
            return;
        }
        if snapshot.origin == ReadOrigin::DesignSample {
            inspector::show_snapshot(ui, &snapshot);
            return;
        }
        if self.selected == 1 && self.tabs[1] == 4 {
            self.subtitles.show(ui);
            return;
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            inspector::show(ui, &WORKSPACES[self.selected]);
            ui.add_space(16.);
            ui.collapsing("도구 진단", |ui| {
                ui.small(WORKSPACES[self.selected].input);
                ui.small(WORKSPACES[self.selected].requirement);
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_workspaces_render_in_compact_and_full_shells() {
        let ctx = egui::Context::default();
        for width in [760., 980., 1180., 1280., 1440.] {
            for selected in 0..WORKSPACES.len() {
                let mut shell = StudioShell {
                    open: true,
                    selected,
                    settings: true,
                    ..Default::default()
                };
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 760.),
                    )),
                    ..Default::default()
                };
                let output = crate::presentation::test_support::run(&ctx, input, |ui| {
                    shell.show(ui, "한글 긴 제목 · 원본 영상")
                });
                assert!(!output.shapes.is_empty());
            }
        }
    }

    #[test]
    fn every_demo_tab_renders_for_all_fixture_states() {
        use crate::application::workspace_read::preview::{
            PreviewScenario, PreviewWorkspaceProvider,
        };
        for scenario in [
            PreviewScenario::Empty,
            PreviewScenario::Populated,
            PreviewScenario::LongLabels,
            PreviewScenario::NeedsInput,
        ] {
            for width in [760., 1280., 1440.] {
                for (selected, workspace) in WORKSPACES.iter().enumerate() {
                    for tab in 0..workspace.tabs.len().max(1) {
                        let ctx = egui::Context::default();
                        let mut shell = StudioShell {
                            open: true,
                            selected,
                            settings: true,
                            ..Default::default()
                        };
                        shell.set_reader(Arc::new(PreviewWorkspaceProvider::new(scenario)));
                        shell.tabs[selected] = tab;
                        // Taffy measures then settles; exercise both passes.
                        for _ in 0..2 {
                            let output = crate::presentation::test_support::run(
                                &ctx,
                                egui::RawInput {
                                    screen_rect: Some(egui::Rect::from_min_size(
                                        egui::Pos2::ZERO,
                                        egui::vec2(width, 760.),
                                    )),
                                    ..Default::default()
                                },
                                |ui| shell.show(ui, "긴 한국어 · 日本語の題名"),
                            );
                            assert!(
                                !output.shapes.is_empty(),
                                "{} tab {tab} {scenario:?}",
                                workspace.id
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn escape_closes_settings_before_leaving_workspace() {
        let mut shell = StudioShell {
            open: true,
            settings: true,
            ..Default::default()
        };
        shell.escape();
        assert!(shell.open);
        assert!(!shell.settings);
        shell.escape();
        assert!(!shell.open);
    }

    #[test]
    fn settings_escape_returns_focus_to_trigger_for_keyboard_reentry() {
        let ctx = egui::Context::default();
        let mut shell = StudioShell {
            open: true,
            settings: true,
            ..Default::default()
        };
        let mut trigger = None;
        let _ = crate::presentation::test_support::run(&ctx, Default::default(), |ui| {
            trigger = Some(shell.settings_trigger(ui).id);
        });
        for key in [egui::Key::Escape, egui::Key::Enter] {
            let input = egui::RawInput {
                events: vec![egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..Default::default()
            };
            let _ = crate::presentation::test_support::run(&ctx, input, |ui| {
                shell.handle_escape(ui.ctx());
                let response = shell.settings_trigger(ui);
                assert_eq!(Some(response.id), trigger);
                assert_eq!(ui.memory(|memory| memory.focused()), trigger);
                if key == egui::Key::Enter {
                    assert!(response.clicked());
                }
            });
            assert!(shell.open);
            assert!(!shell.settings);
        }
    }

    #[test]
    fn thumbnail_layout_settles_after_tab_transition() {
        use crate::application::workspace_read::preview::{
            PreviewScenario, PreviewWorkspaceProvider,
        };
        let ctx = egui::Context::default();
        let selected = WORKSPACES.iter().position(|w| w.id == "FRAME").unwrap();
        let mut shell = StudioShell {
            open: true,
            selected,
            ..Default::default()
        };
        shell.set_reader(Arc::new(PreviewWorkspaceProvider::new(
            PreviewScenario::Populated,
        )));
        let mut pending = Vec::new();
        for frame in 0..20 {
            if frame == 4 {
                shell.tabs[selected] = 1;
            }
            let output = crate::presentation::test_support::run(
                &ctx,
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1440., 900.),
                    )),
                    time: Some(frame as f64 / 60.),
                    ..Default::default()
                },
                |ui| shell.show(ui, ""),
            );
            pending.push(output.platform_output.request_discard_reasons.len());
        }
        assert!(
            pending[16..].iter().all(|count| *count == 0),
            "layout still discarding frames: {pending:?}"
        );
    }

    #[test]
    fn first_conversion_frame_has_readable_timing() {
        use crate::application::workspace_read::preview::{
            PreviewScenario, PreviewWorkspaceProvider,
        };
        for width in [760., 980., 1180., 1440.] {
            let ctx = egui::Context::default();
            crate::presentation::player_theme::apply(&ctx);
            crate::presentation::typography::install(&ctx).unwrap();
            crate::presentation::layout::configure(&ctx);
            let selected = WORKSPACES
                .iter()
                .position(|workspace| workspace.id == "SUB")
                .unwrap();
            let mut shell = StudioShell {
                open: true,
                selected,
                ..Default::default()
            };
            shell.set_reader(Arc::new(PreviewWorkspaceProvider::new(
                PreviewScenario::Populated,
            )));
            let input = || egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 2000.),
                )),
                ..Default::default()
            };
            for _ in 0..3 {
                crate::presentation::test_support::run(&ctx, input(), |ui| shell.show(ui, ""));
            }
            shell.tabs[selected] = 4;
            let output =
                crate::presentation::test_support::run(&ctx, input(), |ui| shell.show(ui, ""));
            let rows = output
                .shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    egui::epaint::Shape::Text(text)
                        if text.galley.job.text == "00:00:00.000 → 00:00:15.000" =>
                    {
                        Some(text.galley.rows.len())
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(rows, vec![1], "first conversion timing at width {width}");
            assert!(
                output.platform_output.request_discard_reasons.is_empty(),
                "unsettled layout at width {width}"
            );
            let idle =
                crate::presentation::test_support::run(&ctx, input(), |ui| shell.show(ui, ""));
            assert_eq!(
                idle.platform_output.num_completed_passes, 1,
                "idle layout at width {width}"
            );
        }
    }

    #[test]
    fn shared_escape_input_closes_settings_then_workspace() {
        let ctx = egui::Context::default();
        let mut shell = StudioShell {
            open: true,
            settings: true,
            ..Default::default()
        };
        for expected_open in [true, false] {
            let input = egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..Default::default()
            };
            let _ = crate::presentation::test_support::run(&ctx, input, |ui| {
                shell.handle_escape(ui.ctx())
            });
            assert_eq!(shell.open, expected_open);
            assert!(!shell.settings);
        }
    }
}
