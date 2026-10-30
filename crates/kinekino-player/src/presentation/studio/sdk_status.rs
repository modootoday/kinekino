use crate::application::api::PlaybackClient;
use crate::application::{
    api::{ApiError, ApplicationClientHandle, ErrorCode, Request},
    playback_status::PlaybackStatus,
};
use eframe::egui;

pub(super) struct SdkStatus {
    client: ApplicationClientHandle,
    playback: Option<Result<PlaybackStatus, ApiError>>,
    version: String,
    commands: Vec<String>,
    error: Option<String>,
}

impl SdkStatus {
    pub fn query(sdk: ApplicationClientHandle) -> Self {
        let version = sdk.execute(Request::new("version"));
        let capabilities = sdk.execute(Request::new("capabilities"));
        let error = version
            .error
            .or(capabilities.error)
            .map(|error| error.message);
        let version = version
            .data
            .and_then(|value| {
                value
                    .get("sdk_version")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "조회 실패".into());
        let commands = capabilities
            .data
            .and_then(|value| value.get("commands").and_then(|v| v.as_array()).cloned())
            .unwrap_or_default()
            .into_iter()
            .filter(|entry| entry.get("status").and_then(|v| v.as_str()) == Some("supported"))
            .filter_map(|entry| {
                entry
                    .get("command")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned)
            })
            .collect();
        Self {
            client: sdk,
            playback: None,
            version,
            commands,
            error,
        }
    }

    pub fn supports(&self, command: &str) -> bool {
        self.commands.iter().any(|item| item == command)
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("SDK 연결 정보", |ui| {
            ui.label(format!("SDK {} · 앱 내부 조회", self.version));
            if let Some(error) = &self.error {
                ui.label(format!("조회 실패: {error}"));
            }
            ui.label(format!("지원 명령: {}", self.commands.join(", ")));
            ui.small("지원 명령은 현재 실행 환경의 실제 응답입니다.");
            ui.separator();
            if ui.button("네이티브 재생 관측 새로고침").clicked() {
                self.playback = Some(self.client.playback_status(Default::default()));
            }
            match &self.playback {
                None => { ui.weak("관측 미조회 · 디자인 데모는 실제 재생 상태를 만들지 않습니다."); }
                Some(Err(error)) => { ui.label(match error.code {
                    ErrorCode::Unsupported => "네이티브 관측 공급자 미연결",
                    ErrorCode::NotFound => "게시된 네이티브 관측 없음",
                    _ => "네이티브 관측 조회 실패",
                }); }
                Some(Ok(status)) => {
                    ui.label("네이티브 관측 · 수동 새로고침 시점의 게시 값");
                    ui.label(format!("연결 {} · 정지 {:?} · idle {:?}", status.connected, status.paused, status.idle));
                    ui.label(format!("재생 위치 {} µs · 길이 {} µs", status.position_us.map_or("미상".into(), |v| v.0.to_string()), status.duration_us.map_or("미상".into(), |v| v.0.to_string())));
                    ui.label(format!("볼륨 {:?}% · 영상 {:?} · 오디오 {:?}", status.volume_percent, status.video_size, status.audio_channels));
                    ui.label(format!("오류 존재 {} · 게시 후 {} ms", status.has_error, status.published_age_ms.0));
                    ui.small("위치는 원본 source PTS가 아닙니다. 게시 경과 시간은 엔진 이벤트의 최신성을 보장하지 않습니다.");
                }
            }
        });
    }
}
