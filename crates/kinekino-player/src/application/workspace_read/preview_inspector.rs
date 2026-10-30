use super::*;

pub fn fields(workspace: WorkspaceId, state: ReadState) -> Vec<InspectorField> {
    let entries: &[(&str, &str, &str)] = match workspace {
        WorkspaceId::Seg => &[
            ("opening", "오프닝 설정", "수동"),
            ("ending", "엔딩 설정", "수동"),
            ("auto-next", "다음 회차 자동 재생", "끔"),
            ("pre-notice", "미리 알림", "3초 전"),
            ("undo-notice", "되돌리기 표시", "5초"),
        ],
        WorkspaceId::Sub => &[
            ("audio-track", "오디오 트랙", "선택 필요"),
            ("language", "인식 언어", "자동 감지"),
            ("translation", "출력 번역", "끔 · 원문"),
            ("range", "분석 범위", "원본 전체"),
            ("model", "모델 선택", "선택 필요"),
            ("format", "내보내기 형식", "SRT · UTF-8"),
        ],
        WorkspaceId::Obj => &[
            ("task", "분석 작업", "탐지 / 분할 / 추적"),
            ("model", "모델 선택", "선택 필요"),
            ("range", "분석 범위", "선택 필요"),
            ("mask", "마스크 표시", "원본과 합성"),
            ("opacity", "불투명도", "50%"),
            ("edge", "경계 부드럽게", "0 px"),
        ],
        WorkspaceId::Clip => &[
            ("aspect", "출력 화면비", "9:16"),
            ("duration", "길이 제안", "30초"),
            ("fit", "화면 맞춤", "원본 유지"),
            ("subtitles", "자막 포함", "선택 필요"),
            ("format", "출력 형식", "지원 확인 필요"),
        ],
        WorkspaceId::Frame => &[
            ("frame", "프레임", "선택 필요"),
            ("size", "출력 크기", "원본 크기"),
            ("subtitles", "자막 포함", "선택 필요"),
            ("format", "출력 형식", "PNG"),
            ("color", "색공간", "sRGB (디자인 값)"),
            ("location", "저장 위치", "선택 안 함"),
        ],
        WorkspaceId::Card => &[
            ("platform", "플랫폼", "사용자 지정"),
            ("aspect", "화면비", "1:1"),
            ("template", "템플릿", "선택 필요"),
            ("font-size", "글꼴 크기", "32 px"),
            ("format", "출력 형식", "PNG"),
        ],
        WorkspaceId::A2d => &[
            ("display", "표시 방식", "원본 / 알파 / 합성"),
            ("threshold", "마스크 임계값", "0.5"),
            ("edge", "경계 부드럽게", "0 px"),
            ("pivot", "피벗", "중앙 · 0.5, 0.5"),
            ("scale", "크기 배율", "1×"),
            ("format", "출력 형식", "RGBA PNG"),
        ],
        WorkspaceId::Proj => &[
            ("model", "깊이 모델", "선택 필요"),
            ("unit", "깊이 단위", "상대 깊이 · 단위 없음"),
            ("view", "시점 미리보기", "0°"),
            ("fov", "카메라 FOV", "45°"),
            ("package", "출력 패키지", "선택 필요"),
        ],
        WorkspaceId::Model => &[
            ("repository", "저장소", "모델 선택 필요"),
            ("revision", "고정 revision", "선택 필요"),
            ("license", "라이선스", "검토 필요"),
            ("precision", "정밀도", "지원 확인 필요"),
            ("device", "실행 장치", "지원 확인 필요"),
        ],
        WorkspaceId::Job => &[
            ("state", "작업 상태", "작업 상태 디자인 예시"),
            ("source", "입력", "디자인 예시 소스"),
            ("stage", "실행 단계", "실행하지 않는 화면 예시"),
            ("output", "결과 경로", "샘플 경로 · 실파일 아님"),
        ],
        WorkspaceId::Out => &[
            ("location", "출력 위치", "선택 안 함"),
            ("collision", "파일 충돌", "선택 필요"),
            ("format", "출력 형식", "선택 필요"),
            ("verify", "저장 전 검증", "입력 확인 필요"),
        ],
        WorkspaceId::Download => &[
            ("source", "소스", "미디어 선택 필요"),
            ("mode", "저장 방식", "원본 저장"),
            ("video-track", "비디오 트랙", "확인 필요"),
            ("audio-track", "오디오 트랙", "확인 필요"),
            ("location", "저장 위치", "선택 안 함"),
        ],
    };
    entries
        .iter()
        .map(|(id, label, value)| InspectorField {
            field_id: format!("sample-{workspace:?}-{id}"),
            label: (*label).into(),
            value: if state == ReadState::NeedsInput && value.contains("필요") {
                CellValue::Missing
            } else if *id == "opacity" {
                CellValue::Percent(50)
            } else {
                CellValue::Text((*value).into())
            },
            state,
        })
        .collect()
}
