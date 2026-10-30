use super::state::ReviewState;
use crate::{
    application::api::ApiError,
    domain::model_review::{
        CheckOutcome, EvidenceAsset, ModelCondition, ReviewItem, ReviewResult, ReviewVerdict,
        valid_sha256,
    },
    presentation::{
        components::{
            accordion, badge,
            button::{Button, Variant},
            card, notice, text_document,
        },
        control_icon::Icon,
        design_tokens::{breakpoint, size, space},
        layout,
    },
};
use eframe::egui;

pub(super) fn show(ui: &mut egui::Ui, state: &mut ReviewState) {
    accordion::show(
        ui,
        "actual-model-review",
        "실제 표본 검토",
        Icon::TextDocument,
        true,
        |ui| {
            ui.small("표본 판정 · 모델 전체 승인 아님 · 자동 적용 꺼짐");
            ui.add_space(space::FIELD);
            if !state.available {
                notice::show(
                    ui,
                    notice::Kind::Info,
                    "검토 자료 연결 필요",
                    "이 실행 환경에는 검토 저장소가 연결되지 않았습니다. 실제 자료를 준비한 실행본에서 열어 주세요.",
                );
            }
            ui.add_enabled_ui(!state.busy(), |ui| {
                let label = ui.label("검토 패키지 SHA256");
                ui.add_sized(
                    [ui.available_width(), size::CONTROL],
                    egui::TextEdit::singleline(&mut state.package_input)
                        .char_limit(64)
                        .hint_text("검토 자료의 64자리 SHA256"),
                )
                .labelled_by(label.id);
            });
            if ui
                .add_enabled(
                    state.available && !state.busy() && valid_sha256(&state.package_input),
                    Button::new("자료 열기", Variant::Secondary).icon(Icon::Folder),
                )
                .clicked()
            {
                state.inspect(ui.ctx());
            }
            if state.busy() {
                ui.spinner();
                ui.label("검토 요청 처리 중 · 중복 실행하지 않습니다.");
            }
            if let Some(error) = &state.error {
                show_error(ui, error);
            }
            let Some(view) = &state.package else {
                return;
            };
            ui.add_space(space::GROUP);
            ui.strong(&view.package.package_id);
            ui.add(egui::Label::new(egui::RichText::new(&view.package_sha256).monospace()).wrap());
            badge::list(
                ui,
                "review-package-state",
                &[
                    if view.evidence_verified {
                        "자료 무결성 확인됨"
                    } else {
                        "자료 무결성 미확인"
                    }
                    .into(),
                    if view.review_ready {
                        "비교 자료 준비됨"
                    } else {
                        "기준선 등 비교 자료 확인 필요"
                    }
                    .into(),
                ],
            );
            let items: Vec<_> = view
                .package
                .items
                .iter()
                .map(|item| {
                    (
                        item.item_id.clone(),
                        format!(
                            "{} · {} · {}",
                            item.task,
                            item.candidate.condition.model_id,
                            item.candidate.condition.precision
                        ),
                    )
                })
                .collect();
            let mut selected = state.selected.clone();
            ui.add_enabled_ui(!state.busy(), |ui| {
                egui::ComboBox::from_id_salt("review-sample")
                    .selected_text(
                        items
                            .iter()
                            .find(|(id, _)| Some(id) == selected.as_ref())
                            .map_or("표본 선택", |(_, label)| label.as_str()),
                    )
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        for (id, label) in &items {
                            ui.selectable_value(&mut selected, Some(id.clone()), label);
                        }
                    });
            });
            if selected != state.selected
                && let Some(id) = selected
            {
                state.select(&id);
            }
            let Some(item) = state.item().cloned() else {
                return;
            };
            ui.add_space(space::FIELD);
            ui.checkbox(&mut state.compare_task, "같은 작업의 다른 모델 조건도 비교");
            let comparison: Vec<_> = state
                .package
                .as_ref()
                .into_iter()
                .flat_map(|view| &view.package.items)
                .filter(|candidate| {
                    candidate.item_id == item.item_id
                        || state.compare_task && candidate.task == item.task
                })
                .cloned()
                .collect();
            for sample in &comparison {
                ui.push_id(&sample.item_id, |ui| sample_view(ui, state, sample));
                ui.add_space(space::GROUP);
            }
            decision_form(ui, state, &item);
        },
    );
}

fn sample_view(ui: &mut egui::Ui, state: &mut ReviewState, item: &ReviewItem) {
    card::show(ui, &format!("{} · {}", item.task, item.item_id), |ui| {
        ui.strong("검토 방법과 기대 결과");
        for criterion in &item.criteria {
            ui.label(format!("검토 방법 · {}", criterion.how_to_review));
            ui.label(format!("기대 결과 · {}", criterion.expected));
        }
        if let Some(range) = &item.range {
            ui.label(format!(
                "표본 범위 · {}–{} µs",
                range.start_us.0, range.end_us.0
            ));
        } else {
            ui.small("비시간 표본 · 임의 재생 구간 없음");
        }
        ui.add_space(space::FIELD);
        ui.collapsing("원본 자료", |ui| asset(ui, state, &item.source));
        let columns = if ui.ctx().content_rect().width() < breakpoint::STACK_EDITOR {
            1
        } else {
            2
        };
        egui_taffy::tui(ui, ui.id().with("baseline-candidate"))
            .reserve_available_width().style(layout::full(layout::grid(columns, space::PANEL)))
            .show(|tui| {
                layout::block(tui, |ui| {
                    if let Some(baseline) = &item.baseline { result(ui, state, "기준선", baseline); }
                    else { notice::show(ui, notice::Kind::Warning, "기준선 없음", "비교 자료가 준비되지 않았습니다. 사용 가능 판정만으로 표본 검토 통과가 되지 않습니다."); }
                });
                layout::block(tui, |ui| result(ui, state, "후보 결과", &item.candidate));
            });
        ui.add_space(space::FIELD);
        ui.strong("알려진 문제");
        if item.known_issues.is_empty() {
            ui.small("등록된 문제 항목 없음 · 품질 통과를 뜻하지 않습니다.");
        }
        for issue in &item.known_issues {
            ui.label(issue);
        }
    });
}

fn result(ui: &mut egui::Ui, state: &mut ReviewState, title: &str, result: &ReviewResult) {
    card::show(ui, title, |ui| {
        condition(ui, &result.condition);
        for evidence in &result.assets {
            ui.push_id(&evidence.asset_id, |ui| asset(ui, state, evidence));
        }
        ui.collapsing("자동 검사 · 사람 판정과 별도", |ui| {
            for check in &result.automated_checks {
                let status = match check.outcome {
                    CheckOutcome::Passed => "통과",
                    CheckOutcome::Failed => "실패",
                    CheckOutcome::NotRun => "미실행",
                };
                ui.label(format!("{} · {status}", check.check_id));
                ui.label(&check.summary);
                for evidence in &check.evidence {
                    ui.push_id((&check.check_id, &evidence.asset_id), |ui| {
                        asset(ui, state, evidence)
                    });
                }
            }
        });
        ui.collapsing("측정 자원 · batch와 request 구분", |ui| {
            if result.measured_resources.is_empty() {
                ui.small("측정값 없음");
            }
            for (name, value) in &result.measured_resources {
                ui.label(format!("{name} · {value}"));
            }
        });
    });
}

fn condition(ui: &mut egui::Ui, condition: &ModelCondition) {
    ui.add(egui::Label::new(egui::RichText::new(&condition.model_id).strong()).wrap());
    ui.label(format!("모델 revision · {}", condition.revision));
    ui.label(format!("정밀도 · {}", condition.precision));
    ui.label(format!("엔진 · {}", condition.engine_id));
    ui.label(format!("엔진 revision · {}", condition.engine_revision));
    ui.collapsing("입력 조건", |ui| {
        for (name, value) in &condition.input_conditions {
            ui.label(format!("{name} · {value}"));
        }
    });
}

fn asset(ui: &mut egui::Ui, state: &mut ReviewState, evidence: &EvidenceAsset) {
    ui.strong(&evidence.asset_id);
    ui.label(format!(
        "{} · {} bytes",
        evidence.media_type, evidence.bytes.0
    ));
    ui.add(egui::Label::new(egui::RichText::new(&evidence.sha256).monospace()).wrap());
    let Some(package) = state.package.as_ref() else {
        return;
    };
    let key = (package.package_sha256.clone(), evidence.sha256.clone());
    if !state.assets.contains_key(&key) {
        if ui
            .add_enabled(
                !state.busy(),
                Button::new("텍스트 자료 읽기", Variant::Secondary).compact(),
            )
            .clicked()
        {
            state.read_asset(&evidence.sha256, ui.ctx());
        }
        ui.small("UTF-8 원문 읽기 · 형식/의미 검증과 별도");
    } else {
        match state.assets.get(&key) {
            Some(Ok(document)) => text_document::show(ui, &evidence.asset_id, &document.utf8),
            Some(Err(error)) => show_error(ui, error),
            None => {}
        }
    }
}

fn decision_form(ui: &mut egui::Ui, state: &mut ReviewState, item: &ReviewItem) {
    card::show(ui, "사용자 표본 판정", |ui| {
        ui.strong(format!(
            "판정 대상 · {} · {}",
            item.item_id, item.candidate.condition.model_id
        ));
        ui.small("선택한 표본에만 기록합니다. 모델 전체 승인이나 자동 적용이 아닙니다.");
        ui.add_enabled_ui(!state.busy(), |ui| {
            ui.horizontal_wrapped(|ui| {
                for (verdict, label) in [
                    (ReviewVerdict::Usable, "사용 가능"),
                    (ReviewVerdict::NeedsChanges, "수정 필요"),
                    (ReviewVerdict::Deferred, "판단 보류"),
                ] {
                    ui.radio_value(&mut state.verdict, Some(verdict), label);
                }
            });
            let label = ui.label("판정 메모 · 필수");
            ui.add(
                egui::TextEdit::multiline(&mut state.note)
                    .desired_width(ui.available_width())
                    .desired_rows(4)
                    .char_limit(8192),
            )
            .labelled_by(label.id);
        });
        if ui
            .add_enabled(
                state.can_record(),
                Button::new("표본 판정 저장", Variant::Primary),
            )
            .clicked()
        {
            state.record(ui.ctx());
        }
        if let Some(saved) = &state.saved {
            match saved {
                Ok(saved) => {
                    ui.label(format!("저장된 판정 · {}", saved.decision.decision_id));
                    ui.label(format!(
                        "표본 검토 통과 · {}",
                        if saved.decision.sample_review_passed {
                            "예"
                        } else {
                            "아니요"
                        }
                    ));
                    ui.add(
                        egui::Label::new(egui::RichText::new(&saved.decision_sha256).monospace())
                            .wrap(),
                    );
                }
                Err(error) => show_error(ui, error),
            }
        }
        ui.add_space(space::GROUP);
        if ui
            .add_enabled(
                !state.busy(),
                Button::new("판정 이력 읽기", Variant::Secondary),
            )
            .clicked()
        {
            state.load_history(ui.ctx());
        }
        if let Some(history) = &state.history {
            match history {
                Ok(history) if history.is_empty() => {
                    ui.label("사용자 표본 판정 없음");
                }
                Ok(history) => {
                    for stored in history {
                        ui.separator();
                        ui.label(format!(
                            "{} · {}",
                            stored.decision.decision_id,
                            verdict_label(stored.decision.verdict)
                        ));
                        ui.label(&stored.decision.note);
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(&stored.decision_sha256).monospace(),
                            )
                            .wrap(),
                        );
                    }
                }
                Err(error) => show_error(ui, error),
            }
        }
    });
}

fn verdict_label(verdict: ReviewVerdict) -> &'static str {
    match verdict {
        ReviewVerdict::Usable => "사용 가능",
        ReviewVerdict::NeedsChanges => "수정 필요",
        ReviewVerdict::Deferred => "판단 보류",
    }
}

fn show_error(ui: &mut egui::Ui, error: &ApiError) {
    notice::show(
        ui,
        notice::Kind::Warning,
        &format!("검토 요청 · {:?}", error.code),
        &error.message,
    );
}
