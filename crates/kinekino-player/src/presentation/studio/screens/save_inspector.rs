use crate::presentation::{
    components::{
        described_choice::{self, Choice},
        inspector,
        inspector_field::{self, Field as F, Kind as K},
    },
    design_tokens::{color, space},
};
use eframe::egui;

pub fn fields(ui: &mut egui::Ui) {
    ui.strong("저장 방식");
    ui.add_space(space::FIELD);
    described_choice::unavailable(
        ui,
        &[
            Choice {
                id: "original",
                title: "원본 저장",
                description: "원본 형식과 화질을 유지해 저장합니다.",
            },
            Choice {
                id: "remux",
                title: "컨테이너 변경",
                description: "선택한 트랙을 재인코딩 없이 다른 컨테이너에 담습니다.",
            },
            Choice {
                id: "transcode",
                title: "호환 변환",
                description: "재생 호환성을 위해 영상·오디오를 변환합니다.",
            },
        ],
        "저장 방식 연결 필요",
    );
    ui.add_space(space::FIELD);
    ui.weak("저장 방식 연결 필요 · 선택되지 않음");
    inspector::section(ui, "출력 설정");
    inspector_field::unavailable(
        ui,
        &[
            F::new("output-format", "출력 형식", K::Select),
            F::new("output-path", "저장 위치", K::Path),
            F::new("filename", "파일 이름", K::Text),
        ],
    );
    inspector::section(ui, "예상 정보");
    inspector_field::unavailable(
        ui,
        &[
            F::new("estimated-size", "예상 파일 크기", K::Readout(None)),
            F::new("resume-support", "이어받기 지원", K::Readout(None)),
        ],
    );
    inspector::section(ui, "호환성 확인");
    ui.colored_label(color::ACCENT, "소스와 트랙 확인 필요");
}
