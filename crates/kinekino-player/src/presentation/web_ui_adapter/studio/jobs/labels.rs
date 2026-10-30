//! Shared labels for the first read-only JOB route checkpoint.
use crate::application::jobs::JobState;

pub const NOT_REQUESTED: &str = "작업 목록을 아직 조회하지 않았습니다.";
pub const LOADING: &str = "작업 목록을 조회하고 있습니다.";
pub const RECEIVED: &str = "조회한 작업 목록입니다.";
pub const EMPTY: &str = "조회한 목록에 작업이 없습니다.";
pub const FAILED: &str = "작업 목록 조회에 실패했습니다.";
pub const CLOSING: &str = "원래 조회 결과를 정리하고 있습니다.";
pub const PREVIOUS: &str = "이전 조회값입니다. 최신 상태를 보장하지 않습니다.";
pub const NO_SELECTION: &str = "목록에서 작업을 선택하면 상세 정보를 조회할 수 있습니다.";

pub fn state_label(state: JobState) -> &'static str {
    match state {
        JobState::Queued => "대기",
        JobState::Running => "실행 중",
        JobState::Cancelling => "취소 요청 처리 중",
        JobState::Verifying => "출력 검증 중",
        JobState::Completed => "완료",
        JobState::NeedsInput => "입력 필요",
        JobState::Failed => "실패",
        JobState::Cancelled => "취소됨",
        JobState::Interrupted => "중단됨",
    }
}
