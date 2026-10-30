use super::super::super::control_icon::Icon;

pub(super) struct Workspace {
    pub id: &'static str,
    pub title: &'static str,
    pub tabs: &'static [&'static str],
    pub input: &'static str,
    pub requirement: &'static str,
    pub action: &'static str,
}

impl Workspace {
    pub fn description(&self) -> &'static str {
        match self.id {
            "SEG" => "오프닝과 엔딩 구간을 감지하고 건너뛰기 설정을 관리합니다.",
            "SUB" => "음성 인식·화자·화면 자막을 편집하고 원하는 형식으로 저장합니다.",
            "OBJ" => "영상 속 피사체를 선택하고 마스크와 추적 구간을 검토합니다.",
            "CLIP" => "필요한 장면을 골라 짧은 영상으로 구성합니다.",
            "FRAME" => "원본 프레임을 저장하거나 썸네일을 편집합니다.",
            "CARD" => "영상과 자막을 활용해 카드뉴스를 구성합니다.",
            "A2D" => "투명 이미지와 레이어·마스크를 편집합니다.",
            "PROJ" => "상대 깊이와 카메라 시점으로 투영 결과를 검토합니다.",
            "MODEL" => "모델·라이선스·고정 버전과 장치 설정을 관리합니다.",
            "JOB" => "실행 중인 작업과 결과·오류를 확인합니다.",
            "OUT" => "출력 파일과 저장 위치·파일 충돌을 확인합니다.",
            _ => "원본 저장·리먹스·변환 방식과 출력 트랙을 확인합니다.",
        }
    }
    pub fn icon(&self) -> Icon {
        match self.id {
            "SEG" => Icon::Segments,
            "SUB" => Icon::Subtitles,
            "OBJ" => Icon::Subject,
            "CLIP" => Icon::Clip,
            "FRAME" => Icon::Image,
            "CARD" => Icon::Cards,
            "A2D" => Icon::Layers,
            "PROJ" => Icon::Cube,
            "MODEL" => Icon::Model,
            "JOB" => Icon::List,
            _ => Icon::Export,
        }
    }
}

pub(super) const WORKSPACES: &[Workspace] = &[
    Workspace {
        id: "SEG",
        title: "구간 감지",
        tabs: &["구간 건너뛰기 설정"],
        input: "원본 영상 · 확인된 시간 구간",
        requirement: "구간 annotation 유스케이스와 감지 모델 연결이 필요합니다.",
        action: "분석 시작",
    },
    Workspace {
        id: "SUB",
        title: "자막",
        tabs: &["음성 인식", "화자 분리", "화면 OCR", "교정", "변환"],
        input: "오디오 트랙 · 자막 문서 · 원본 프레임",
        requirement: "자막 문서 유스케이스와 작업별 모델 연결이 필요합니다.",
        action: "작업에 추가",
    },
    Workspace {
        id: "OBJ",
        title: "피사체",
        tabs: &["피사체 인식"],
        input: "원본 프레임 · PTS · 선택 구간",
        requirement: "탐지·분할·추적 어댑터와 검토된 마스크가 필요합니다.",
        action: "분석 시작",
    },
    Workspace {
        id: "CLIP",
        title: "숏폼",
        tabs: &["수동 편집", "AI 추천", "일괄 작업"],
        input: "원본 영상 · 확인된 시간 구간",
        requirement: "클립 문서와 렌더링 유스케이스 연결이 필요합니다.",
        action: "작업에 추가",
    },
    Workspace {
        id: "FRAME",
        title: "스냅샷",
        tabs: &["원본 프레임", "썸네일 편집"],
        input: "디코딩한 원본 프레임 · 실제 PTS",
        requirement: "원본 프레임 추출과 이미지 저장 어댑터가 필요합니다.",
        action: "별도 파일로 저장",
    },
    Workspace {
        id: "CARD",
        title: "카드뉴스",
        tabs: &["카드 편집"],
        input: "선택 프레임 · 선택 자막 · 빈 문서",
        requirement: "카드 문서와 캔버스 렌더링 유스케이스가 필요합니다.",
        action: "작업에 추가",
    },
    Workspace {
        id: "A2D",
        title: "2D 에셋",
        tabs: &["레이어 · 마스크"],
        input: "고정 프레임 · 검토된 피사체 마스크",
        requirement: "레이어 문서와 알파 출력 검증 어댑터가 필요합니다.",
        action: "작업에 추가",
    },
    Workspace {
        id: "PROJ",
        title: "3D 투영",
        tabs: &["2.5D 깊이", "투영 패키지"],
        input: "원본 프레임 · 상대 깊이 · 마스크",
        requirement: "깊이 모델과 투영 패키지 유스케이스가 필요합니다. 상대 깊이는 실제 거리가 아닙니다.",
        action: "작업에 추가",
    },
    Workspace {
        id: "SAVE",
        title: "영상 저장",
        tabs: &["영상 저장"],
        input: "정상 미디어 세션 · 확인된 트랙",
        requirement: "저장·트랙 확인·출력 검증 어댑터가 필요합니다.",
        action: "작업에 추가",
    },
    Workspace {
        id: "MODEL",
        title: "모델 관리",
        tabs: &["모델 관리"],
        input: "고정 HF revision · 라이선스 · 정밀도",
        requirement: "모델 카탈로그와 설치 상태 SDK 연결이 필요합니다. 설치·준비 여부는 아직 조회되지 않았습니다.",
        action: "다운로드",
    },
    Workspace {
        id: "JOB",
        title: "대기열",
        tabs: &["작업 대기열"],
        input: "공통 SDK의 실제 작업 기록",
        requirement: "작업 조회 SDK가 연결되지 않았습니다. 작업 수와 진행률은 확인되지 않았습니다.",
        action: "작업 시작",
    },
    Workspace {
        id: "OUT",
        title: "내보내기",
        tabs: &["내보내기 확인"],
        input: "고정 프로젝트 revision · 검증할 출력",
        requirement: "출력 경로·충돌 검사와 원자적 저장 유스케이스가 필요합니다.",
        action: "저장 시작",
    },
];
