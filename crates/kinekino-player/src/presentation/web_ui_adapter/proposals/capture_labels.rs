//! Presentation defaults; callers can supply translated labels without changing SDK requests.
#[derive(Clone)]
pub struct CaptureLabels {
    pub title: String,
    pub save: String,
    pub accept: String,
    pub consume: String,
    pub discard: String,
    pub retry: String,
    pub revert: String,
    pub return_viewing: String,
    pub choice_label: String,
    pub choice_hint: String,
}
impl CaptureLabels {
    pub fn korean() -> Self {
        Self {
            title: "저장된 카드 문구 편집".into(),
            save: "수정본 저장".into(),
            accept: "명시적 채택".into(),
            consume: "처리 결과 확인".into(),
            discard: "처리 결과 버리기".into(),
            retry: "다시 시도".into(),
            revert: "문구 수정 취소".into(),
            return_viewing: "시청으로 돌아가기".into(),
            choice_label: "저장된 카드 선택".into(),
            choice_hint: "저장된 문구입니다. 편집을 지원하는 카드를 명시적으로 선택하세요.".into(),
        }
    }
}
