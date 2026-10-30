use crate::application::api::ProposalClient;
use crate::{
    application::{
        api::{ApiError, ApplicationClientHandle, RequestContext},
        proposals::{AcceptProposal, EditProposal, InspectProposal, StoredProposal},
    },
    domain::proposals::{AcceptanceReceipt, ProposalDocument},
};
use eframe::egui;
use std::sync::mpsc::{self, Receiver};
pub(super) enum Action {
    Inspect(InspectProposal),
    Edit(EditProposal),
    Accept(AcceptProposal),
}
pub(super) enum Outcome {
    Inspected(Result<Box<StoredProposal>, ApiError>),
    Document(Result<Box<ProposalDocument>, ApiError>),
    Accepted(Result<Box<AcceptanceReceipt>, ApiError>),
}
pub(super) fn start(
    client: ApplicationClientHandle,
    action: Action,
    expected: Option<String>,
    ctx: egui::Context,
) -> Receiver<Outcome> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let context = RequestContext {
            request_id: "ui-proposal".into(),
            expected_revision: expected,
            ..Default::default()
        };
        let result = match action {
            Action::Inspect(input) => {
                Outcome::Inspected(client.inspect_proposal(context, input).map(Box::new))
            }
            Action::Edit(input) => {
                Outcome::Document(client.edit_proposal(context, input).map(Box::new))
            }
            Action::Accept(input) => {
                Outcome::Accepted(client.accept_proposal(context, input).map(Box::new))
            }
        };
        let _ = sender.send(result);
        ctx.request_repaint();
    });
    receiver
}
