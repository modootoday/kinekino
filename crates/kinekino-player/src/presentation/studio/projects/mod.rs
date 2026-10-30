#[cfg(test)]
mod tests;
mod view;
mod worker;

use crate::{
    application::{
        api::{ApiError, ApplicationClientHandle},
        projects::InspectProject,
    },
    domain::projects::StoredProjectRevision,
};
use eframe::egui;
use std::sync::mpsc::{Receiver, TryRecvError};

pub(super) struct Projects {
    client: ApplicationClientHandle,
    available: bool,
    open: bool,
    input: String,
    selected: Option<String>,
    stored: Option<StoredProjectRevision>,
    error: Option<ApiError>,
    pending: Option<Receiver<Result<StoredProjectRevision, ApiError>>>,
}

impl Projects {
    pub fn new(client: ApplicationClientHandle, available: bool) -> Self {
        Self {
            client,
            available,
            open: false,
            input: String::new(),
            selected: None,
            stored: None,
            error: None,
            pending: None,
        }
    }

    pub fn poll(&mut self) {
        let Some(receiver) = &self.pending else {
            return;
        };
        match receiver.try_recv() {
            Ok(Ok(stored)) => {
                self.stored = Some(stored);
                self.error = None;
                self.pending = None;
            }
            Ok(Err(error)) => {
                self.error = Some(error);
                self.pending = None;
            }
            Err(TryRecvError::Disconnected) => {
                self.error = Some(ApiError::new(
                    crate::application::api::ErrorCode::Internal,
                    "프로젝트 조회 결과를 받지 못했습니다.",
                ));
                self.pending = None;
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    pub fn launcher(&mut self, ui: &mut egui::Ui) {
        use crate::presentation::components::button::{Button, Variant};
        if ui
            .add(Button::new("프로젝트 조회", Variant::Secondary).compact())
            .clicked()
        {
            self.open = true;
        }
    }

    pub fn close(&mut self) -> bool {
        std::mem::take(&mut self.open)
    }

    fn inspect(&mut self, ctx: &egui::Context) {
        let input = InspectProject {
            project_id: self.input.clone(),
            revision: None,
        };
        self.pending = Some(worker::inspect(self.client.clone(), input, ctx.clone()));
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        let mut open = self.open;
        egui::Window::new("저장된 프로젝트 조회")
            .id(egui::Id::new("stored-project-inspector"))
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.label("저장된 문서를 읽습니다. 현재 재생·제안 편집과 별개입니다.");
                if !self.available {
                    ui.weak("프로젝트 저장소 연결 필요");
                }
                ui.add_enabled_ui(self.available && self.pending.is_none(), |ui| {
                    let label = ui.label("프로젝트 ID");
                    ui.text_edit_singleline(&mut self.input)
                        .labelled_by(label.id);
                    if ui
                        .add_enabled(
                            !self.input.trim().is_empty(),
                            crate::presentation::components::button::Button::new(
                                "최신 저장본 조회",
                                crate::presentation::components::button::Variant::Secondary,
                            ),
                        )
                        .clicked()
                    {
                        self.inspect(ui.ctx());
                    }
                });
                if self.pending.is_some() {
                    ui.spinner();
                    ui.label("조회 중");
                }
                if let Some(error) = &self.error {
                    ui.label(format!("{:?}: {}", error.code, error.message));
                }
                if let Some(stored) = &self.stored {
                    view::show(ui, stored, &mut self.selected);
                }
            });
        self.open = open;
    }
}
