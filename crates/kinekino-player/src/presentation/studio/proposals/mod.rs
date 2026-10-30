mod actions;
mod body;
mod view;
mod worker;

use crate::{application::api::ApplicationClientHandle, domain::proposals::*};
use eframe::egui;
use std::sync::mpsc::{Receiver, TryRecvError};

pub(super) struct Proposals {
    client: ApplicationClientHandle,
    available: bool,
    states: [State; 3],
}

struct State {
    task: ProposalTask,
    inspector_mode: Option<usize>,
    input: String,
    document: Option<ProposalDocument>,
    draft: Option<ProposalPayload>,
    dirty: bool,
    error: Option<String>,
    receipt: Option<AcceptanceReceipt>,
    pending: Option<Receiver<worker::Outcome>>,
}
impl State {
    fn reset_buffer(&mut self) {
        self.draft = self.document.as_ref().map(|document| {
            document
                .draft
                .clone()
                .unwrap_or_else(|| empty_draft(self.task, &document.source))
        });
        self.dirty = false;
    }
    fn new(task: ProposalTask) -> Self {
        Self {
            task,
            inspector_mode: Some(0),
            input: String::new(),
            document: None,
            draft: None,
            dirty: false,
            error: None,
            receipt: None,
            pending: None,
        }
    }
    fn apply(&mut self, outcome: worker::Outcome) {
        match outcome {
            worker::Outcome::Inspected(Ok(stored)) => {
                if stored.document.generated.task == self.task {
                    self.apply(worker::Outcome::Document(Ok(Box::new(stored.document))));
                    self.receipt = stored.acceptance;
                } else {
                    self.error =
                        Some("이 화면과 다른 종류의 제안입니다. 기존 편집은 유지됩니다.".into());
                }
            }
            worker::Outcome::Document(Ok(document)) if document.generated.task == self.task => {
                self.draft = Some(
                    document
                        .draft
                        .clone()
                        .unwrap_or_else(|| empty_draft(self.task, &document.source)),
                );
                self.document = Some(*document);
                self.dirty = false;
                self.receipt = None;
                self.error = None;
            }
            worker::Outcome::Document(Ok(_)) => {
                self.error =
                    Some("이 화면과 다른 종류의 제안입니다. 기존 편집은 유지됩니다.".into());
            }
            worker::Outcome::Inspected(Err(error))
            | worker::Outcome::Document(Err(error))
            | worker::Outcome::Accepted(Err(error)) => {
                self.error = Some(format!("{:?}: {}", error.code, error.message));
            }
            worker::Outcome::Accepted(Ok(receipt)) => {
                if let Some(document) = self.document.as_mut() {
                    document.draft_revision = receipt.output_revision;
                }
                self.receipt = Some(*receipt);
                self.error = None;
            }
        }
    }
}

fn empty_draft(task: ProposalTask, source: &SourceSnapshot) -> ProposalPayload {
    match task {
        ProposalTask::Correction if source.as_document().is_some() => {
            ProposalPayload::DocumentCorrection { edits: Vec::new() }
        }
        ProposalTask::Correction => ProposalPayload::Correction { edits: Vec::new() },
        ProposalTask::Cards => ProposalPayload::Cards { cards: Vec::new() },
        ProposalTask::Highlights => ProposalPayload::Highlights {
            highlights: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests;
impl Proposals {
    pub fn task(
        workspace: crate::application::workspace_read::WorkspaceId,
        tab: usize,
    ) -> Option<ProposalTask> {
        use crate::application::workspace_read::WorkspaceId;
        match workspace {
            WorkspaceId::Sub if tab == 3 => Some(ProposalTask::Correction),
            WorkspaceId::Card => Some(ProposalTask::Cards),
            WorkspaceId::Clip if tab == 1 => Some(ProposalTask::Highlights),
            _ => None,
        }
    }

    fn state_index(task: ProposalTask) -> usize {
        match task {
            ProposalTask::Correction => 0,
            ProposalTask::Cards => 1,
            ProposalTask::Highlights => 2,
        }
    }

    pub fn inspector_mode(&mut self, ui: &mut egui::Ui, task: ProposalTask) -> bool {
        let state = &mut self.states[Self::state_index(task)];
        crate::presentation::components::segmented::show(
            ui,
            "proposal-inspector-mode",
            &["작업 설정", "제안 검토"],
            &mut state.inspector_mode,
        );
        ui.add_space(crate::presentation::design_tokens::space::FIELD);
        if state.inspector_mode == Some(1) {
            return true;
        }
        if let Some(document) = &state.document {
            ui.small(format!("연결된 제안: {}", document.proposal_id));
            ui.weak(if state.dirty {
                "제안 검토에 저장하지 않은 수정이 있습니다."
            } else {
                "문서 제안은 제안 검토에서 확인합니다. 미디어 편집 설정과 별개입니다."
            });
        }
        false
    }
    pub fn new(client: ApplicationClientHandle, available: bool) -> Self {
        Self {
            client,
            available,
            states: [
                State::new(ProposalTask::Correction),
                State::new(ProposalTask::Cards),
                State::new(ProposalTask::Highlights),
            ],
        }
    }
    pub fn poll(&mut self) {
        for state in &mut self.states {
            let result = state.pending.as_ref().map(|receiver| receiver.try_recv());
            match result {
                Some(Ok(outcome)) => {
                    state.pending = None;
                    state.apply(outcome);
                }
                Some(Err(TryRecvError::Disconnected)) => {
                    state.pending = None;
                    state.error =
                        Some("요청 결과를 확인하지 못했습니다. 편집 내용은 유지됩니다.".into());
                }
                _ => {}
            }
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui, task: ProposalTask) {
        let index = Self::state_index(task);
        ui.push_id(("proposal-task", index), |ui| {
            view::show(ui, &mut self.states[index], self.available, &self.client);
        });
    }
}
