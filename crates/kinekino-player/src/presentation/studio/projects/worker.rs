use crate::application::api::ProjectClient;
use crate::{
    application::{
        api::{ApiError, ApplicationClientHandle, RequestContext},
        projects::InspectProject,
    },
    domain::projects::StoredProjectRevision,
};
use eframe::egui;
use std::sync::mpsc::{self, Receiver};

pub(super) fn inspect(
    client: ApplicationClientHandle,
    input: InspectProject,
    ctx: egui::Context,
) -> Receiver<Result<StoredProjectRevision, ApiError>> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let result = client.inspect_project(
            RequestContext {
                request_id: "ui-project-inspect".into(),
                ..Default::default()
            },
            input,
        );
        let _ = sender.send(result);
        ctx.request_repaint();
    });
    receiver
}
