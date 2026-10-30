use crate::{
    application::workspace_read::WorkspaceId,
    presentation::components::{
        inspector,
        inspector_field::{self, Field as F, Kind as K},
    },
};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, workspace: WorkspaceId, tab: usize) {
    inspector::panel(
        ui,
        &format!("composition-inspector-{workspace:?}-{tab}"),
        |ui| {
            let groups: &[(&str, &[F<'_>])] = match workspace {
                WorkspaceId::Clip if tab == 1 => &[
                    (
                        "AI 추천 설정",
                        &[
                            F::new("candidate-count", "후보 수", K::Scalar),
                            F::new("minimum-duration", "최소 길이", K::Scalar),
                            F::new("maximum-duration", "최대 길이", K::Scalar),
                        ],
                    ),
                    (
                        "선정 근거",
                        &[
                            F::new("use-dialogue", "대사", K::Toggle),
                            F::new("use-scenes", "장면 전환", K::Toggle),
                            F::new("use-voice", "음성 활동", K::Toggle),
                            F::new("exclude-duplicates", "중복 후보 제외", K::Toggle),
                        ],
                    ),
                    (
                        "모델",
                        &[
                            F::new("model", "추천 모델", K::Select),
                            F::new("evidence-review", "근거 검토", K::Readout(None)),
                            F::new("generation-state", "생성 상태", K::Readout(None)),
                        ],
                    ),
                ],
                WorkspaceId::Clip if tab == 2 => &[
                    (
                        "일괄 템플릿",
                        &[
                            F::new("ratio", "출력 비율", K::Select),
                            F::new("output-size", "출력 크기", K::Select),
                            F::new("template", "클립 템플릿", K::Select),
                        ],
                    ),
                    (
                        "회차별 설정",
                        &[
                            F::new("episode-candidates", "회차당 후보 수", K::Scalar),
                            F::new("minimum-duration", "최소 길이", K::Scalar),
                            F::new("maximum-duration", "최대 길이", K::Scalar),
                            F::new("exclude-duplicates", "중복 제외", K::Toggle),
                        ],
                    ),
                    (
                        "실행 자원",
                        &[
                            F::new("model", "사용 모델", K::Select),
                            F::new("device", "실행 장치", K::Select),
                            F::new("concurrency", "동시 실행 수", K::Scalar),
                            F::new("retry", "재시도", K::Toggle),
                        ],
                    ),
                    (
                        "출력",
                        &[
                            F::new("output-path", "저장 위치", K::Path),
                            F::new("filename-rule", "파일 이름 규칙", K::Text),
                        ],
                    ),
                ],
                WorkspaceId::Clip => &[
                    (
                        "클립 설정",
                        &[
                            F::new("ratio", "출력 비율", K::Select),
                            F::new("start", "시작 시간", K::Text),
                            F::new("end", "종료 시간", K::Text),
                            F::new("duration", "길이", K::Readout(None)),
                        ],
                    ),
                    (
                        "화면 구성",
                        &[
                            F::new("crop", "자르기", K::Select),
                            F::new("subject-tracking", "피사체 추적", K::Toggle),
                            F::new("background-fill", "배경 채우기", K::Select),
                        ],
                    ),
                    (
                        "자막 및 오디오",
                        &[
                            F::new("subtitle-visible", "자막 표시", K::Toggle),
                            F::new("subtitle-style", "자막 스타일", K::Select),
                            F::new("audio-track", "오디오 트랙", K::Select),
                        ],
                    ),
                    (
                        "출력",
                        &[
                            F::new("resolution", "해상도", K::Select),
                            F::new("frame-rate", "프레임 속도", K::Select),
                            F::new("format", "파일 형식", K::Select),
                            F::new("output-path", "저장 위치", K::Path),
                        ],
                    ),
                ],
                _ => &[],
            };
            for (index, (title, fields)) in groups.iter().enumerate() {
                if index == 0 {
                    inspector::heading(ui, title);
                } else {
                    inspector::section(ui, title);
                }
                inspector_field::unavailable(ui, fields);
            }
        },
        |ui| {
            ui.separator();
            let actions = match (workspace, tab) {
                (WorkspaceId::Clip, 1) => ("후보 검토", "추천 생성"),
                (WorkspaceId::Clip, 2) => ("계획 검토", "일괄 시작"),
                _ => ("미리보기", "내보내기"),
            };
            inspector::unavailable_actions(ui, actions.0, actions.1);
        },
    );
}
