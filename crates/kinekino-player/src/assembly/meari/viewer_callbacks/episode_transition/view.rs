use super::state::{TransitionRequest, TransitionUi};
use crate::{
    application::jobs::JobState,
    presentation::web_ui_adapter::player_shell::episode_transition::TransitionControlsView,
};
impl TransitionUi {
    pub(crate) fn view(&self, can_prepare: bool) -> TransitionControlsView {
        let idle = self.rpc.is_none() && self.staged.is_none() && !self.commit_unconfirmed;
        let active = self.active.as_ref();
        let observed = active.and_then(|active| active.observation.as_ref());
        let job = active.and_then(|active| active.job.as_ref());
        let status = if let Some((request, _)) = &self.staged {
            match request {
                TransitionRequest::Begin(_) => "인접 회차의 준비를 요청했습니다",
                TransitionRequest::Observe => "회차 준비 상태를 확인하는 중입니다",
                TransitionRequest::Commit => "원래 회차 종료와 새 회차 생성을 확인하는 중입니다",
                TransitionRequest::CancelJob => "원래 준비 작업에 취소를 요청하는 중입니다",
                TransitionRequest::ReadJob => "원래 준비 작업의 실제 종료 상태를 확인하는 중입니다",
                TransitionRequest::Release => "준비된 전환을 해제하는 중입니다",
            }
        } else if job.is_some_and(|job| job.state == JobState::Cancelled) {
            "회차 이동 취소를 확인했습니다"
        } else if observed.is_some_and(|observed| observed.job_state == JobState::Completed) {
            "회차 준비를 마치고 이동을 적용하는 중입니다"
        } else if active.is_some() {
            "회차 이동을 준비하는 중입니다"
        } else if self.released.is_some() {
            "준비 전환을 해제했습니다. 실제 작업 결과는 원래 기록에 보존됩니다"
        } else if self.replacement.is_some() {
            "교체 응답을 받았습니다. 실제 재생은 엔진 관측에서 확인하세요"
        } else {
            "원래 재생 관측과 지원 기능을 확인한 뒤 인접 회차를 준비하세요"
        };
        TransitionControlsView {
            prepare: can_prepare && !self.blocks_catalog(),
            observe: idle && active.is_some(),
            commit: idle
                && observed.is_some_and(|observed| observed.job_state == JobState::Completed)
                && job.is_none_or(|job| job.state == JobState::Completed),
            cancel: idle && observed.is_some() && !job.is_some_and(|job| job.state.terminal()),
            read_job: idle && active.is_some(),
            release: idle && job.is_some_and(|job| job.state.terminal()),
            status: status.into(),
            error: self
                .diagnostic
                .as_ref()
                .map(|error| format!("{:?}: {}", error.code, error.message))
                .unwrap_or_default(),
        }
    }
}
