use crate::presentation::components::inspector_field::{self, Field as F, Kind as K};
use eframe::egui;

pub(super) fn fields(ui: &mut egui::Ui, tab: usize) {
    let fields: &[F<'_>] = match tab {
        1 => &[
            F::new("speaker-model", "화자 모델", K::Select),
            F::new("speaker-count", "인원 수", K::Scalar),
            F::new("analysis-range", "분석 범위", K::Select),
            F::new("output-labels", "출력 라벨", K::Select),
        ],
        2 => &[
            F::new("subtitle-source", "입력 영상의 자막", K::Select),
            F::new("language", "언어", K::Select),
            F::new("region", "자막 영역 지정", K::Readout(None)),
            F::new("analysis-range", "선택 구간", K::Readout(None)),
            F::new("sample-interval", "표본 간격", K::Scalar),
            F::new("review-confidence", "낮은 신뢰도 검토", K::Toggle),
            F::new("model", "모델 선택", K::Select),
        ],
        3 => &[
            F::new("evidence", "근거", K::Readout(None)),
            F::new("correct-spelling", "맞춤법 교정", K::Toggle),
            F::new("correct-punctuation", "문장부호 교정", K::Toggle),
            F::new("proper-noun-dictionary", "고유명사 사전", K::Text),
            F::new("model", "모델 선택", K::Select),
            F::new("apply-range", "적용 범위", K::Select),
        ],
        4 => &[
            F::new("subtitle-source", "소스 자막", K::Select),
            F::new("language", "언어", K::Select),
            F::new("output-format", "출력 형식", K::Select),
            F::new("encoding", "인코딩", K::Select),
            F::new("conversion-range", "변환 범위", K::Select),
            F::new("time-offset", "시간 오프셋", K::Text),
            F::new("preserve-original", "원본 보존", K::Toggle),
            F::new("output-path", "저장 위치", K::Path),
        ],
        _ => &[
            F::new("audio-track", "오디오 트랙", K::Select),
            F::new("recognition-language", "인식 언어", K::Select),
            F::new("translation-language", "출력 번역", K::Select),
            F::new("analysis-range", "범위", K::Select),
            F::new("model", "모델 선택", K::Select),
            F::new("automatic-punctuation", "문장부호 자동 추가", K::Toggle),
        ],
    };
    inspector_field::unavailable(ui, fields);
}
