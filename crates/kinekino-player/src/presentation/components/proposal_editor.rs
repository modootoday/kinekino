use super::{
    button::{Button, Variant},
    card, input,
};
use crate::{
    domain::proposals::{
        Card, Correction, DocumentCorrection, Highlight, ProposalPayload, SourceSnapshot,
    },
    presentation::design_tokens::space,
};
use eframe::egui;

/// Edits a caller-owned buffer only. Persistence and acceptance belong to the typed SDK.
pub fn show(
    ui: &mut egui::Ui,
    id: &str,
    source: &SourceSnapshot,
    draft: &mut ProposalPayload,
) -> bool {
    ui.push_id(id, |ui| match draft {
        ProposalPayload::DocumentCorrection { edits } => {
            let Some(document) = source.as_document() else {
                ui.weak("문서 원문 연결 필요");
                return false;
            };
            ui.small(format!("문서 원문 · {}", document.source_sha256));
            ui.small(format!(
                "원문 위치 {} · {:?}",
                document.selector.outer_pointer, document.selector.extraction
            ));
            let mut changed = false;
            let mut remove = None;
            for (index, edit) in edits.iter_mut().enumerate() {
                ui.push_id(index, |ui| {
                    card::show(ui, &format!("문서 교정문 {}", index + 1), |ui| {
                        input::select(
                            ui,
                            "document-text",
                            "원문 문장 위치",
                            edit.source_ref.inner_pointer.clone(),
                            |ui| {
                                for text in &document.texts {
                                    if let Some(reference) =
                                        document.text_reference(&text.inner_pointer)
                                    {
                                        changed |= ui
                                            .selectable_value(
                                                &mut edit.source_ref,
                                                reference,
                                                &text.inner_pointer,
                                            )
                                            .changed();
                                    }
                                }
                            },
                        );
                        if let Some(text) = document
                            .texts
                            .iter()
                            .find(|text| text.inner_pointer == edit.source_ref.inner_pointer)
                        {
                            ui.add(egui::Label::new(&text.text).wrap().selectable(true));
                        }
                        changed |= input::multiline(ui, "text", "수정문", &mut edit.text).changed();
                        if remove_button(ui) {
                            remove = Some(index);
                        }
                    })
                });
                ui.add_space(space::FIELD);
            }
            if let Some(index) = remove {
                edits.remove(index);
                changed = true;
            }
            if ui
                .add_enabled(
                    !document.texts.is_empty(),
                    Button::new("문서 교정문 추가", Variant::Secondary),
                )
                .clicked()
                && let Some(reference) = document
                    .texts
                    .first()
                    .and_then(|text| document.text_reference(&text.inner_pointer))
            {
                edits.push(DocumentCorrection {
                    source_ref: reference,
                    text: String::new(),
                });
                changed = true;
            }
            changed
        }
        ProposalPayload::Correction { edits } => {
            let mut changed = false;
            let mut remove = None;
            for (index, edit) in edits.iter_mut().enumerate() {
                ui.push_id(index, |ui| {
                    card::show(ui, &format!("교정문 {}", index + 1), |ui| {
                        changed |= cue(ui, source, &mut edit.cue_id);
                        changed |= input::multiline(ui, "text", "수정문", &mut edit.text).changed();
                        if remove_button(ui) {
                            remove = Some(index);
                        }
                    });
                });
                ui.add_space(space::FIELD);
            }
            if let Some(index) = remove {
                edits.remove(index);
                changed = true;
            }
            if ui
                .add(Button::new("교정문 추가", Variant::Secondary))
                .clicked()
            {
                edits.push(Correction {
                    cue_id: String::new(),
                    text: String::new(),
                });
                changed = true;
            }
            changed
        }
        ProposalPayload::Cards { cards } => {
            let mut changed = false;
            let mut remove = None;
            for (index, item) in cards.iter_mut().enumerate() {
                ui.push_id(index, |ui| {
                    card::show(ui, &format!("카드 {}", index + 1), |ui| {
                        changed |=
                            input::multiline(ui, "front", "앞면 문구", &mut item.front).changed();
                        changed |=
                            input::multiline(ui, "back", "뒷면 문구", &mut item.back).changed();
                        changed |= references(ui, source, &mut item.source_refs);
                        if remove_button(ui) {
                            remove = Some(index);
                        }
                    });
                });
                ui.add_space(space::FIELD);
            }
            if let Some(index) = remove {
                cards.remove(index);
                changed = true;
            }
            if ui
                .add(Button::new("카드 추가", Variant::Secondary))
                .clicked()
            {
                cards.push(Card {
                    front: String::new(),
                    back: String::new(),
                    source_refs: Vec::new(),
                });
                changed = true;
            }
            changed
        }
        ProposalPayload::Highlights { highlights } => {
            let mut changed = false;
            let mut remove = None;
            for (index, item) in highlights.iter_mut().enumerate() {
                ui.push_id(index, |ui| {
                    card::show(ui, &format!("하이라이트 {}", index + 1), |ui| {
                        changed |= input::multiline(ui, "title", "제목", &mut item.title).changed();
                        changed |= references(ui, source, &mut item.source_refs);
                        if remove_button(ui) {
                            remove = Some(index);
                        }
                    });
                });
                ui.add_space(space::FIELD);
            }
            if let Some(index) = remove {
                highlights.remove(index);
                changed = true;
            }
            if ui
                .add(Button::new("하이라이트 추가", Variant::Secondary))
                .clicked()
            {
                highlights.push(Highlight {
                    title: String::new(),
                    source_refs: Vec::new(),
                });
                changed = true;
            }
            changed
        }
    })
    .inner
}

fn remove_button(ui: &mut egui::Ui) -> bool {
    let clicked = ui
        .add(Button::new("수정본에서 항목 제외", Variant::Secondary))
        .clicked();
    if clicked {
        ui.memory_mut(|memory| {
            if let Some(id) = memory.focused() {
                memory.surrender_focus(id);
            }
        });
    }
    clicked
}

fn cue(ui: &mut egui::Ui, source: &SourceSnapshot, selected: &mut String) -> bool {
    let mut changed = false;
    input::select(
        ui,
        "source-cue",
        "원문 자막",
        if selected.is_empty() {
            "원문 선택".to_owned()
        } else {
            selected.clone()
        },
        |ui| {
            for cue in source.cues() {
                changed |= ui
                    .selectable_value(selected, cue.cue_id.clone(), &cue.cue_id)
                    .changed();
            }
        },
    );
    if let Some(cue) = source.cues().iter().find(|cue| cue.cue_id == *selected) {
        ui.add(egui::Label::new(&cue.text).wrap().selectable(true));
    }
    ui.add_space(space::FIELD);
    changed
}

fn references(ui: &mut egui::Ui, source: &SourceSnapshot, selected: &mut Vec<String>) -> bool {
    let mut changed = false;
    ui.add_space(space::FIELD);
    ui.label("원문 근거");
    for cue in source.cues() {
        ui.push_id(&cue.cue_id, |ui| {
            let mut checked = selected.contains(&cue.cue_id);
            if ui.checkbox(&mut checked, &cue.cue_id).changed() {
                selected.retain(|id| id != &cue.cue_id);
                if checked {
                    selected.push(cue.cue_id.clone());
                }
                changed = true;
            }
            ui.add(egui::Label::new(&cue.text).wrap().selectable(true));
        });
    }
    changed
}
