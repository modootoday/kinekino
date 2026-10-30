use super::{
    CatalogConsumer,
    model::{Intent, Listing},
};
use crate::{
    application::jobs::JobState,
    presentation::web_ui_adapter::player_shell::catalog_browser::{
        self, CatalogBrowserView, CatalogRow,
    },
};
use meari::{
    application::plugins::{PluginError, PluginErrorCode},
    domain::Value,
};
use std::collections::BTreeMap;
impl CatalogConsumer {
    pub(crate) fn props(&self) -> Result<BTreeMap<String, Value>, PluginError> {
        let busy = self.model.rpc.is_some();
        let has_job = self.model.active.is_some();
        let selecting_disabled = !self.automatic_enabled
            || self.transition.blocks_catalog()
            || !self.available
            || busy
            || has_job
            || self
                .model
                .staged
                .as_ref()
                .is_some_and(|s| s.ticket.is_some());
        let state = self.model.active.as_ref().map(|a| a.job.state);
        let completed = state == Some(JobState::Completed);
        let operation = self
            .model
            .active
            .as_ref()
            .map(|a| a.job.input.operation.as_str());
        let status = if busy {
            "요청 결과를 기다리는 중"
        } else {
            match state {
                Some(JobState::Queued) => "브라우저 작업 대기 중",
                Some(JobState::Running) => "작품과 회차 정보를 불러오는 중입니다",
                Some(JobState::Cancelling) => "취소 요청됨 · 종료 결과는 아직 확인되지 않았습니다",
                Some(JobState::Verifying) => "원본 결과를 확인하는 중",
                Some(JobState::Completed) => "선택한 작품과 회차 정보를 표시하는 중입니다",
                Some(JobState::Cancelled) => "원본 작업이 취소되었습니다",
                Some(JobState::Failed | JobState::NeedsInput | JobState::Interrupted) => {
                    "원본 작업의 오류를 확인하세요"
                }
                None if self
                    .model
                    .staged
                    .as_ref()
                    .is_some_and(|s| s.ticket.is_some()) =>
                {
                    "제출 결과가 확인되지 않았습니다 · 새 작업을 중복 제출하지 않습니다"
                }
                None if self.opened.is_some() => {
                    "재생 시작 응답 수신 · 실제 미디어 상태는 엔진 관측에서 확인하세요"
                }
                None if !self.available => "지원 기능을 조회해 라이브 카탈로그 연결을 확인하세요",
                None if self.model.previous.is_some() => {
                    "이전 원본 작업 기록을 보존했습니다. 다른 항목을 선택하세요"
                }
                None => "카테고리에서 작품과 회차를 선택하세요",
            }
        };
        let error = self
            .model
            .diagnostic
            .as_ref()
            .map(|e| format!("{:?}: {}", e.code, e.message))
            .or_else(|| self.model.adapter_error.as_ref().map(|e| e.message.clone()))
            .or_else(|| {
                self.model
                    .active
                    .as_ref()
                    .and_then(|a| a.job.failure.as_ref())
                    .map(|e| format!("{:?}: {} · {}", e.code, e.message, e.remediation))
            })
            .unwrap_or_default();
        let selected_episode = self
            .model
            .resolution
            .as_ref()
            .map(|c| c.selected.key.clone())
            .or_else(|| {
                self.model.active.as_ref().and_then(|a| match &a.intent {
                    Intent::Episode(selected) => Some(selected.key.clone()),
                    _ => None,
                })
            })
            .unwrap_or_default();
        catalog_browser::props(CatalogBrowserView {
            series: rows(self.model.series.as_ref(), selecting_disabled),
            episodes: rows(
                self.model.episodes.as_ref(),
                selecting_disabled
                    || self.opened.is_some()
                    || self.transition.replacement.is_some(),
            ),
            selected_series: self
                .model
                .selected_series
                .as_ref()
                .map(|s| s.key.clone())
                .unwrap_or_default(),
            selected_episode,
            status: status.into(),
            error,
            busy: selecting_disabled,
            can_poll: has_job && !busy,
            can_cancel: has_job && !busy && !self.model.job_terminal(),
            can_read_series: !busy && completed && operation == Some("catalog.search"),
            can_read_episodes: !busy && completed && operation == Some("episodes.list"),
            can_inspect: !busy && completed && operation == Some("episodes.resolve"),
            can_play: !busy && self.resolution().is_ok(),
            can_release: !busy && self.model.job_terminal(),
        })
        .map_err(|message| PluginError::new(PluginErrorCode::InvalidPayload, message))
    }
}
fn rows(listing: Option<&Listing>, disabled: bool) -> Vec<CatalogRow> {
    listing
        .map(|listing| {
            listing
                .result
                .entries
                .iter()
                .enumerate()
                .map(|(index, row)| CatalogRow {
                    key: listing.key(index),
                    title: row.title.clone(),
                    disabled,
                })
                .collect()
        })
        .unwrap_or_default()
}
