use super::*;
use crate::domain::api_values::DecimalU64;

#[derive(Clone, Copy)]
enum Field {
    Kind,
    Start,
    End,
    Duration,
    Status,
    Speaker,
    Original,
    Correction,
    Subject,
    Range,
    Review,
    Title,
    Name,
    Visible,
    Model,
    Task,
    Precision,
    Install,
    Source,
    Progress,
    Output,
    FileName,
    Format,
    Location,
    Verification,
}
impl Field {
    fn id(self) -> &'static str {
        match self {
            Self::Kind => "kind",
            Self::Start => "start",
            Self::End => "end",
            Self::Duration => "duration",
            Self::Status => "status",
            Self::Speaker => "speaker",
            Self::Original => "original",
            Self::Correction => "correction",
            Self::Subject => "subject",
            Self::Range => "range",
            Self::Review => "review",
            Self::Title => "title",
            Self::Name => "name",
            Self::Visible => "visible",
            Self::Model => "model",
            Self::Task => "task",
            Self::Precision => "precision",
            Self::Install => "install",
            Self::Source => "source",
            Self::Progress => "progress",
            Self::Output => "output",
            Self::FileName => "file-name",
            Self::Format => "format",
            Self::Location => "location",
            Self::Verification => "verification",
        }
    }
    fn kind(self) -> CellKind {
        match self {
            Self::Start | Self::End | Self::Duration => CellKind::Time,
            Self::Visible => CellKind::Boolean,
            Self::Progress => CellKind::Percent,
            _ => CellKind::Text,
        }
    }
}

pub fn table(workspace: WorkspaceId, state: ReadState, empty: bool, long: bool) -> TableSnapshot {
    let specs = columns(workspace);
    let columns = specs
        .iter()
        .map(|(field, label)| TableColumn {
            column_id: field.id().into(),
            label: (*label).into(),
            kind: field.kind(),
        })
        .collect();
    let rows = if empty {
        Vec::new()
    } else {
        (1..=3)
            .map(|index| TableRow {
                row_id: format!("sample-{workspace:?}-row-{index:03}"),
                cells: specs
                    .iter()
                    .map(|(field, _)| TableCell {
                        column_id: field.id().into(),
                        value: value(*field, index, long, state),
                    })
                    .collect(),
                state,
            })
            .collect()
    };
    TableSnapshot { columns, rows }
}
fn columns(workspace: WorkspaceId) -> Vec<(Field, &'static str)> {
    use Field::*;
    match workspace {
        WorkspaceId::Seg => vec![
            (Kind, "유형"),
            (Start, "시작 시간"),
            (End, "종료 시간"),
            (Duration, "길이"),
            (Status, "상태"),
        ],
        WorkspaceId::Sub => vec![
            (Start, "시작"),
            (End, "끝"),
            (Speaker, "화자"),
            (Original, "원문"),
            (Correction, "교정안"),
        ],
        WorkspaceId::Obj => vec![
            (Subject, "피사체"),
            (Range, "구간"),
            (Status, "상태"),
            (Review, "검토"),
        ],
        WorkspaceId::Clip => vec![
            (Title, "제목"),
            (Range, "구간"),
            (Duration, "길이"),
            (Status, "상태"),
        ],
        WorkspaceId::Frame | WorkspaceId::Card | WorkspaceId::A2d => vec![
            (Name, "이름"),
            (Kind, "유형"),
            (Visible, "표시"),
            (Status, "상태"),
        ],
        WorkspaceId::Model => vec![
            (Model, "모델"),
            (Task, "작업 종류"),
            (Precision, "정밀도"),
            (Install, "설치 상태"),
        ],
        WorkspaceId::Job => vec![
            (Task, "작업"),
            (Source, "소스"),
            (Progress, "진행 상태"),
            (Output, "출력"),
        ],
        WorkspaceId::Out => vec![
            (FileName, "파일명"),
            (Format, "형식"),
            (Location, "저장 위치"),
            (Verification, "검증"),
        ],
        WorkspaceId::Proj => vec![
            (Name, "씬"),
            (Kind, "투영 유형"),
            (Range, "구간"),
            (Status, "상태"),
        ],
        WorkspaceId::Download => vec![
            (Source, "소스"),
            (Format, "저장 형식"),
            (Duration, "길이"),
            (Status, "상태"),
        ],
    }
}
fn value(field: Field, index: u64, long: bool, state: ReadState) -> CellValue {
    use Field::*;
    let text = match field {
        Start => return CellValue::TimeUs(DecimalU64((index - 1) * 25_000_000)),
        End => return CellValue::TimeUs(DecimalU64((index - 1) * 25_000_000 + 15_000_000)),
        Duration => return CellValue::TimeUs(DecimalU64(15_000_000)),
        Visible => return CellValue::Boolean(true),
        Progress => {
            return if state == ReadState::NeedsInput {
                CellValue::Missing
            } else {
                CellValue::Percent(25 + index as u8 * 10)
            };
        }
        Kind => ["기본", "강조", "자막"][(index - 1) as usize].into(),
        Status | Review | Verification => {
            if state == ReadState::NeedsInput {
                "입력 필요 · 예시".into()
            } else {
                "검토 대기 · 예시".into()
            }
        }
        Speaker => format!("화자 {index}"),
        Original => [
            "저 산 너머로 길이 이어져 있어요.",
            "準備ができたら、出発しましょう。",
            "우리의 이야기는 여기서 시작됩니다.",
        ][(index - 1) as usize]
            .into(),
        Correction => "교정 제안 예시 · 미적용".into(),
        Subject => format!("피사체 예시 {index}"),
        Range => {
            let start = (index - 1) * 25;
            let end = start + 15;
            format!(
                "{:02}:{:02} – {:02}:{:02}",
                start / 60,
                start % 60,
                end / 60,
                end % 60
            )
        }
        Title | Name => format!("장면 예시 {index}"),
        Model => format!("모델 표시 예시 {index}"),
        Task => ["음성 인식 예시", "프레임 저장 예시", "자산 내보내기 예시"][(index - 1) as usize]
            .into(),
        Precision => "FP16 (디자인 값)".into(),
        Install => "설치 상태 예시 · 실설치 아님".into(),
        Source => format!("디자인 소스 {index}"),
        Output => "산출물 표시 예시".into(),
        FileName => format!("sample-{index:03}.png"),
        Format => "PNG".into(),
        Location => "디자인 예시/출력 폴더".into(),
    };
    CellValue::Text(if long {
        format!("{text} — 아주 긴 한국어・日本語・English 예시 문구로 말줄임과 줄바꿈을 검토합니다")
    } else {
        text
    })
}
