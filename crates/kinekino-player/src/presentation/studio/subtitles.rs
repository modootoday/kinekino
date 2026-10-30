use crate::application::api::ProposalClient;
use crate::presentation::layout;
use crate::{
    application::{
        api::{ApiError, ApplicationClientHandle, ErrorCode, Request},
        subtitles::{
            convert::{ConversionResult, ConvertSubtitles},
            path_picker::SubtitlePathPickerHandle,
        },
    },
    domain::subtitles::{Loss, SpeakerPolicy, SubtitleFormat},
};
use eframe::egui;

use std::sync::mpsc::{self, Receiver, TryRecvError};

pub(super) struct SubtitleConversion {
    client: ApplicationClientHandle,
    paths: SubtitlePathPickerHandle,
    supported: bool,
    input: String,
    output: String,
    input_format: SubtitleFormat,
    output_format: SubtitleFormat,
    speaker: SpeakerPolicy,
    accept_loss: bool,
    pending: Option<Receiver<Result<ConversionResult, ApiError>>>,
    result: Option<Result<ConversionResult, ApiError>>,
}

impl SubtitleConversion {
    pub fn new(
        client: ApplicationClientHandle,
        paths: SubtitlePathPickerHandle,
        supported: bool,
    ) -> Self {
        Self {
            client,
            paths,
            supported,
            input: String::new(),
            output: String::new(),
            input_format: SubtitleFormat::Srt,
            output_format: SubtitleFormat::Vtt,
            speaker: SpeakerPolicy::Omit,
            accept_loss: false,
            pending: None,
            result: None,
        }
    }

    pub(super) fn poll(&mut self) {
        let Some(receiver) = &self.pending else {
            return;
        };
        match receiver.try_recv() {
            Ok(result) => {
                self.result = Some(result);
                self.pending = None;
            }
            Err(TryRecvError::Disconnected) => {
                self.result = Some(Err(ApiError::new(
                    ErrorCode::Internal,
                    "변환 작업의 응답을 받지 못했습니다.",
                )));
                self.pending = None;
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    pub(super) fn activity(&self) -> Option<&'static str> {
        if self.pending.is_some() {
            Some("자막 변환 중")
        } else if let Some(result) = &self.result {
            Some(match result {
                Ok(_) => "자막 변환 완료",
                Err(error) if error.code == ErrorCode::NeedsInput => "자막 변환 · 확인 필요",
                Err(_) => "자막 변환 · 오류",
            })
        } else {
            None
        }
    }

    fn start(&mut self) {
        if !self.supported || self.pending.is_some() {
            return;
        }
        let client = self.client.clone();
        let payload = ConvertSubtitles {
            input_path: self.input.clone(),
            output_path: self.output.clone(),
            input_format: self.input_format,
            output_format: self.output_format,
            allow_loss: self.accept_loss,
            speaker_policy: self.speaker,
        };
        let mut context = Request::new("subtitles.convert").context;
        context.request_id = "gui-subtitle-conversion".into();
        let (sender, receiver) = mpsc::channel();
        self.pending = Some(receiver);
        self.result = None;
        std::thread::spawn(move || {
            let _ = sender.send(client.convert_subtitles(context, payload));
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.poll();
        let running = self.pending.is_some();
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(8.);
            ui.strong("변환 설정");
            ui.small("UTF-8 · 입력 파일 보존 · 기존 출력 파일 덮어쓰기 안 함");
            ui.add_space(16.);
            if !self.supported {
                ui.colored_label(
                    super::super::super::player_theme::ACCENT,
                    "현재 실행 환경에서 자막 변환을 지원하지 않습니다.",
                );
            }
            let mut changed = false;
            ui.add_enabled_ui(!running, |ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(12.)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        let label = ui.strong("입력 자막");
                        changed |= ui
                            .add_sized(
                                [ui.available_width(), 32.],
                                egui::TextEdit::singleline(&mut self.input)
                                    .hint_text("입력 파일의 전체 경로"),
                            )
                            .labelled_by(label.id)
                            .changed();
                        ui.horizontal_wrapped(|ui| {
                            ui.label("입력 형식");
                            changed |= format_choice(ui, "input-format", &mut self.input_format);
                            if ui
                                .add_enabled(self.paths.available(), egui::Button::new("파일 찾기"))
                                .on_disabled_hover_text(
                                    "파일 선택기를 사용할 수 없습니다. 경로를 직접 입력해 주세요.",
                                )
                                .clicked()
                            {
                                match self.paths.open_subtitle() {
                                    Ok(Some(path)) => {
                                        self.input = path;
                                        changed = true;
                                    }
                                    Ok(None) => {}
                                    Err(error) => self.result = Some(Err(error)),
                                }
                            }
                        });
                    });
                ui.add_space(12.);
                egui::Frame::group(ui.style())
                    .inner_margin(12.)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        let label = ui.strong("출력 자막");
                        changed |= ui
                            .add_sized(
                                [ui.available_width(), 32.],
                                egui::TextEdit::singleline(&mut self.output)
                                    .hint_text("새로 만들 출력 파일의 전체 경로"),
                            )
                            .labelled_by(label.id)
                            .changed();
                        ui.horizontal_wrapped(|ui| {
                            ui.label("출력 형식");
                            changed |= format_choice(ui, "output-format", &mut self.output_format);
                            if ui
                                .add_enabled(self.paths.available(), egui::Button::new("저장 위치"))
                                .on_disabled_hover_text(
                                    "파일 선택기를 사용할 수 없습니다. 경로를 직접 입력해 주세요.",
                                )
                                .clicked()
                            {
                                match self.paths.save_subtitle(self.output_format) {
                                    Ok(Some(path)) => {
                                        self.output = path;
                                        changed = true;
                                    }
                                    Ok(None) => {}
                                    Err(error) => self.result = Some(Err(error)),
                                }
                            }
                        });
                    });
                ui.add_space(12.);
                ui.horizontal_wrapped(|ui| {
                    ui.label("WebVTT 화자 표시");
                    changed |= ui
                        .selectable_value(&mut self.speaker, SpeakerPolicy::Omit, "생략")
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut self.speaker,
                            SpeakerPolicy::Prefix,
                            "본문 앞에 이름 표시",
                        )
                        .changed();
                });
                ui.small("화자 구조·스타일·위치 등 보존되지 않는 정보는 저장 전에 확인합니다.");
            });
            if changed {
                self.result = None;
                self.accept_loss = false;
            }
            ui.add_space(16.);
            let needs_review = self.result.as_ref().is_some_and(
                |result| matches!(result, Err(error) if error.code == ErrorCode::NeedsInput),
            );
            if let Some(activity) = self.activity() {
                ui.strong(activity);
            }
            if needs_review {
                ui.checkbox(
                    &mut self.accept_loss,
                    "위 정보 손실을 확인했으며 이 설정으로 저장합니다.",
                );
            }
            ui.add_space(12.);
            ui.horizontal_wrapped(|ui| {
                let ready = self.supported
                    && !running
                    && !self.input.trim().is_empty()
                    && !self.output.trim().is_empty()
                    && (!needs_review || self.accept_loss);
                if ui
                    .add_enabled(
                        ready,
                        egui::Button::new(if needs_review {
                            "손실 수락 후 변환"
                        } else {
                            "새 파일로 변환"
                        })
                        .min_size(egui::vec2(160., 40.)),
                    )
                    .on_disabled_hover_text(if running {
                        "변환 결과를 기다리고 있습니다."
                    } else if !self.supported {
                        "현재 실행 환경에서 지원하지 않습니다."
                    } else if needs_review {
                        "손실 목록을 검토하고 수락해 주세요."
                    } else {
                        "입력 경로와 새 출력 경로를 입력해 주세요."
                    })
                    .clicked()
                {
                    self.start();
                }
                if running {
                    ui.spinner();
                    ui.label("변환 중 · 결과 확인 대기");
                }
            });
            if running {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(50));
            }
        });
    }
}

impl SubtitleConversion {
    pub fn show_preview(&self, ui: &mut egui::Ui, source: &str) {
        egui::ScrollArea::vertical().show(ui,|ui| {
            super::editor::preview(ui,source);
            ui.add_space(12.);
            egui_taffy::tui(ui, ui.id().with("subtitle-comparison")).reserve_available_width().style(layout::full(layout::grid(2, 12.))).show(|tui| {
                for index in 0..2 {
                    layout::block(tui, |column| {
                    egui::Frame::group(column.style()).fill(super::super::super::player_theme::PANEL).inner_margin(12.).show(column,|ui| {
                        ui.set_min_width(ui.available_width());ui.set_min_height(140.);
                        ui.strong(if index==0 {"원본 자막"}else{"출력 자막"});ui.separator();ui.add_space(8.);
                        if index==0 {
                            ui.label(if self.input.is_empty(){"선택한 자막 파일 없음"}else{&self.input});
                            ui.weak("원본 내용과 시각을 보존해 변환합니다.");
                        }else if let Some(Ok(receipt))=&self.result {
                            ui.label(&receipt.output_path);ui.label(format!("자막 {}개 · {} bytes",receipt.cue_count,receipt.output_bytes));ui.weak("UTF-8 · 새 파일로 저장 완료");
                        }else{ui.weak("변환한 결과가 여기에 표시됩니다.");}
                    });
                    });
                }
            });
            ui.add_space(12.);
            if let Some(result @ Err(_))=&self.result {show_result(ui,result);}
            if self.result.is_none(){ui.weak("설정에서 입력·출력 자막과 형식을 선택하세요. 손실이 있으면 저장 전에 확인합니다.");}
        });
    }
}

fn format_choice(ui: &mut egui::Ui, id: &str, value: &mut SubtitleFormat) -> bool {
    ui.push_id(id, |ui| {
        let first = ui
            .selectable_value(value, SubtitleFormat::Srt, "SRT")
            .changed();
        ui.selectable_value(value, SubtitleFormat::Vtt, "WebVTT")
            .changed()
            || first
    })
    .inner
}

fn show_result(ui: &mut egui::Ui, result: &Result<ConversionResult, ApiError>) {
    egui::Frame::group(ui.style())
        .inner_margin(12.)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            match result {
                Ok(receipt) => {
                    ui.strong("변환 완료");
                    ui.label(&receipt.output_path);
                    ui.label(format!(
                        "자막 {}개 · {} bytes · {}",
                        receipt.cue_count, receipt.output_bytes, receipt.encoding
                    ));
                    loss_list(ui, &receipt.losses);
                }
                Err(error) => {
                    ui.colored_label(
                        super::super::super::player_theme::ACCENT,
                        if error.code == ErrorCode::NeedsInput {
                            "정보 손실 확인 필요"
                        } else {
                            "변환하지 못했습니다"
                        },
                    );
                    ui.label(&error.message);
                    if let Some(losses) = error
                        .details
                        .get("losses")
                        .and_then(|value| serde_json::from_value::<Vec<Loss>>(value.clone()).ok())
                    {
                        loss_list(ui, &losses);
                    }
                    if !error.remediation.is_empty() {
                        ui.small(&error.remediation);
                    }
                }
            }
        });
}

fn loss_list(ui: &mut egui::Ui, losses: &[Loss]) {
    for loss in losses {
        let message = match loss.code.as_str() {
            "metadata_removed" => "메모·스타일 정의·영역 메타데이터는 저장되지 않습니다.",
            "cue_identifier_removed" => "자막 항목의 식별자가 다시 만들어지거나 생략됩니다.",
            "position_style_removed" => "자막 위치와 영역 설정은 저장되지 않습니다.",
            "speaker_structure_removed" => {
                "화자 구조는 보존되지 않습니다. 이름 표시는 선택한 방식으로 처리합니다."
            }
            "markup_removed" => "글자 스타일·언어·글자별 시각 표시는 저장되지 않습니다.",
            "overlap_present" => {
                "겹치는 자막은 유지되지만 플레이어에 따라 다르게 보일 수 있습니다."
            }
            _ => &loss.message,
        };
        ui.label(format!("• {message}"))
            .on_hover_text(&loss.message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::{
        api::PlayerSdk,
        subtitles::{path_picker::TextPathOnly, storage::SubtitleStorage},
    };
    use std::sync::{Arc, Mutex};

    struct MemoryStorage(Arc<Mutex<Vec<String>>>);
    impl SubtitleStorage for MemoryStorage {
        fn read_utf8(&self, _: &str) -> Result<String, ApiError> {
            Ok("WEBVTT\n\n00:00:01.000 --> 00:00:02.000\n<v 화자1>안녕하세요</v>\n".into())
        }
        fn publish_new(&self, _: &str, contents: &str) -> Result<(), ApiError> {
            self.0.lock().unwrap().push(contents.into());
            Ok(())
        }
    }

    #[test]
    fn gui_worker_uses_shared_loss_gate_before_publishing() {
        let outputs = Arc::new(Mutex::new(Vec::new()));
        let client = Arc::new(PlayerSdk::new(MemoryStorage(outputs.clone())));
        let mut view = SubtitleConversion::new(client, std::rc::Rc::new(TextPathOnly), true);
        view.input = "input.vtt".into();
        view.output = "output.srt".into();
        view.input_format = SubtitleFormat::Vtt;
        view.output_format = SubtitleFormat::Srt;
        view.speaker = SpeakerPolicy::Prefix;
        view.start();
        let result = view
            .pending
            .take()
            .unwrap()
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        assert_eq!(result.unwrap_err().code, ErrorCode::NeedsInput);
        assert!(outputs.lock().unwrap().is_empty());
        view.accept_loss = true;
        view.start();
        let receipt = view
            .pending
            .take()
            .unwrap()
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap()
            .unwrap();
        assert_eq!(receipt.cue_count, 1);
        assert!(!receipt.losses.is_empty());
        assert!(outputs.lock().unwrap()[0].contains("[화자1] 안녕하세요"));
    }

    #[test]
    fn pending_worker_does_not_block_conversion_ui_render() {
        let client: ApplicationClientHandle = Arc::<PlayerSdk>::new(Default::default());
        let mut view = SubtitleConversion::new(client, std::rc::Rc::new(TextPathOnly), false);
        let (_sender, receiver) = mpsc::channel();
        view.pending = Some(receiver);
        let ctx = egui::Context::default();
        let output = crate::presentation::test_support::run(&ctx, Default::default(), |ui| {
            egui::CentralPanel::default().show(ui, |ui| view.show(ui));
        });
        assert!(!output.shapes.is_empty());
        assert!(view.pending.is_some());
        assert!(view.result.is_none());
    }
}
