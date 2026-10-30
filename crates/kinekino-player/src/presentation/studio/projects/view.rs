use crate::{
    domain::projects::{StoredProjectRevision, WatchingSource, WorkingState},
    presentation::{components::text_document, design_tokens::space},
};
use eframe::egui;

pub(super) fn show(
    ui: &mut egui::Ui,
    stored: &StoredProjectRevision,
    selected: &mut Option<String>,
) {
    let project = &stored.project;
    ui.separator();
    ui.strong(&project.title);
    ui.label(format!(
        "표시 중인 프로젝트: {} · 버전 {}",
        project.project_id, project.revision.0
    ));
    ui.small(format!(
        "조회 시점 최신 버전: {} · SHA {}",
        stored.latest_revision.0, stored.revision_sha256
    ));
    let work = &project.work_state;
    ui.label(format!(
        "수정 필요 {} · 편집 가능 {} · 채택 결과 {} · 입력 필요 {}",
        work.needs_repair, work.editable, work.accepted_output, work.working_needs_input
    ));
    if let Some(watching) = &project.watching {
        ui.label(format!("저장된 시청 선택: {}", watching.label));
        match &watching.source {
            WatchingSource::LocalVideo { path, .. } => {
                ui.label(path);
            }
            WatchingSource::EpisodePage { episode_url, .. } => {
                ui.label(episode_url);
            }
        }
    }
    egui::ScrollArea::vertical()
        .id_salt("project-snapshots")
        .max_height(crate::presentation::design_tokens::size::REVIEW_TEXT_HEIGHT)
        .show(ui, |ui| {
            for snapshot in &project.proposals {
                let document = &snapshot.captured.document;
                if ui
                    .selectable_label(
                        selected.as_ref() == Some(&document.proposal_id),
                        format!("{} · {:?}", document.proposal_id, document.generated.task),
                    )
                    .clicked()
                {
                    *selected = Some(document.proposal_id.clone());
                }
            }
        });
    if let Some(snapshot) = project
        .proposals
        .iter()
        .find(|snapshot| selected.as_ref() == Some(&snapshot.captured.document.proposal_id))
    {
        let document = &snapshot.captured.document;
        ui.add_space(space::FIELD);
        ui.label(format!(
            "원문 {} · 버전 {}",
            document.source.source_id(),
            document.source.source_revision().0
        ));
        ui.small(format!("원문 SHA {}", document.source.source_sha256()));
        ui.label(match snapshot.working_state {
            WorkingState::Absent => "별도 작업본 없음",
            WorkingState::Valid => "별도 작업본 있음",
            WorkingState::NeedsInput => "별도 작업본에 추가 입력 필요",
        });
        if let Some(receipt) = &snapshot.captured.acceptance {
            ui.label(format!(
                "저장된 채택 결과 · 출력 버전 {}",
                receipt.output_revision.0
            ));
        }
        ui.collapsing("원문 · 읽기 전용", |ui| {
            if let Some(document) = document.source.as_document() {
                for text in &document.texts {
                    ui.label(&text.text);
                }
            } else {
                for cue in document.source.cues() {
                    ui.label(&cue.text);
                }
            }
        });
        ui.collapsing("원생성 결과 · 읽기 전용", |ui| {
            text_document::show(ui, "project-generated-raw", &document.generated.raw_text)
        });
        ui.weak("저장본을 현재 제안에 자동 적용하지 않습니다.");
    }
}
