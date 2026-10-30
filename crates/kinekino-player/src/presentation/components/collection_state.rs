use crate::application::workspace_read::{ReadOrigin, ReadState, WorkspaceSnapshot};
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionState {
    Unavailable,
    Unknown,
    NeedsInput,
    Count(usize),
}

impl CollectionState {
    pub fn from_snapshot(snapshot: &WorkspaceSnapshot) -> Self {
        if snapshot.origin == ReadOrigin::DesignSample || snapshot.state == ReadState::Ready {
            return Self::Count(snapshot.table.rows.len());
        }
        match snapshot.state {
            ReadState::Unsupported => Self::Unavailable,
            ReadState::Unknown => Self::Unknown,
            ReadState::NeedsInput => Self::NeedsInput,
            ReadState::Ready => Self::Count(snapshot.table.rows.len()),
        }
    }

    pub fn heading(self, label: &str) -> String {
        match self {
            Self::Count(count) => format!("{label} ({count})"),
            Self::Unavailable => format!("{label} · 확인 불가"),
            Self::Unknown => format!("{label} · 확인 필요"),
            Self::NeedsInput => format!("{label} · 입력 확인 필요"),
        }
    }

    pub fn show_notice(self, ui: &mut egui::Ui, subject: &str) {
        let detail = match self {
            Self::Count(0) => {
                ui.weak(format!("등록된 {subject}이 없습니다."));
                return;
            }
            Self::Count(_) => return,
            Self::Unavailable => {
                "목록을 아직 불러올 수 없습니다. 개수와 등록 상태는 확인되지 않았습니다."
            }
            Self::Unknown => {
                "목록을 확인할 정보가 아직 없습니다. 개수와 등록 상태를 확인해야 합니다."
            }
            Self::NeedsInput => {
                "목록을 확인하기 위한 입력이 필요합니다. 개수와 등록 상태는 확인되지 않았습니다."
            }
        };
        super::notice::show(
            ui,
            super::notice::Kind::Info,
            &self.heading(&format!("{subject} 목록")),
            detail,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::workspace_read::{
        LiveWorkspaceProvider, WorkspaceId, WorkspaceReadPort,
        preview::{PreviewScenario, PreviewWorkspaceProvider},
    };

    #[test]
    fn unconfirmed_live_collections_never_become_zero_results() {
        for id in [WorkspaceId::Model, WorkspaceId::Job] {
            let mut snapshot = LiveWorkspaceProvider.snapshot(id);
            for (state, expected) in [
                (ReadState::Unsupported, CollectionState::Unavailable),
                (ReadState::Unknown, CollectionState::Unknown),
                (ReadState::NeedsInput, CollectionState::NeedsInput),
            ] {
                snapshot.state = state;
                assert_eq!(CollectionState::from_snapshot(&snapshot), expected);
                assert!(!expected.heading("목록").contains("(0)"));
            }
            snapshot.state = ReadState::Ready;
            assert_eq!(
                CollectionState::from_snapshot(&snapshot),
                CollectionState::Count(0)
            );
            let sample = PreviewWorkspaceProvider::new(PreviewScenario::Empty).snapshot(id);
            assert_eq!(
                CollectionState::from_snapshot(&sample),
                CollectionState::Count(0)
            );
        }
    }
}
